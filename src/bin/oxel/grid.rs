use glm::{vec3, Vec3};
use nalgebra_glm as glm;
use rayon::prelude::*;
use std::collections::{HashMap, VecDeque};

pub struct VoxelGrid {
    pub size: i32,

    pub bb_min: Vec3,
    pub bb_max: Vec3,
    pub bb_diff: Vec3,
    pub voxel_size: Vec3,

    pub voxels: Vec<bool>,
    pub sdf: Vec<glm::TVec3<i32>>,
    pub labels: Vec<i32>,
    pub label: i32,
    pub label_inner: HashMap<i32, bool>,

    pub occluders: Vec<(glm::TVec3<i32>, glm::TVec3<i32>)>,
}
#[derive(Copy, Clone)]
pub enum Round {
    Floor,
    Ceil,
}

impl VoxelGrid {
    pub fn new(atoms: &mut Vec<glm::Vec4>) -> Self {
        // Find bounding box of the entire structure
        let mut bb_max = vec3(std::f32::NEG_INFINITY, std::f32::NEG_INFINITY, std::f32::NEG_INFINITY);
        let mut bb_min = vec3(std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY);
        for atom in atoms.iter() {
            let atom_position = atom.xyz();
            let atom_radius = atom[3];
            bb_max = glm::max2(&bb_max, &(atom_position + vec3(atom_radius, atom_radius, atom_radius)));
            bb_min = glm::min2(&bb_min, &(atom_position - vec3(atom_radius, atom_radius, atom_radius)));
        }

        // Center the molecules (+their bounding box)
        let bb_center = (bb_max + bb_min) / 2.0;
        bb_max = bb_max - bb_center;
        bb_min = bb_min - bb_center;
        for atom in atoms.iter_mut() {
            atom.x -= bb_center.x;
            atom.y -= bb_center.y;
            atom.z -= bb_center.z;
        }

        let bb_diff = bb_max - bb_min;

        // Create voxel grid
        let grid_dimension = 256i32;
        let grid_size = vec3(grid_dimension, grid_dimension, grid_dimension);

        let voxel_size = vec3(
            bb_diff.x / grid_size.x as f32,
            bb_diff.y / grid_size.y as f32,
            bb_diff.z / grid_size.z as f32,
        );
        let voxel_diameter = glm::distance(&voxel_size, &glm::vec3(0.0, 0.0, 0.0));
        let voxel_halfsize = voxel_size.apply_into(|e| e * 0.5);

        let grid_vec_size = (grid_dimension * grid_dimension * grid_dimension) as usize;
        let mut voxels = vec![false; grid_vec_size];

        // Helper functions
        // World position inside BB -> voxel space
        let snap_to_grid = |input: Vec3, round: Round| -> glm::TVec3<i32> {
            let grid_position = input - bb_min;

            let grid_position = vec3(
                grid_position.x / voxel_size.x,
                grid_position.y / voxel_size.y,
                grid_position.z / voxel_size.z,
            );

            let grid_position = match round {
                Round::Floor => grid_position.apply_into(|e| e.floor()),
                Round::Ceil => grid_position.apply_into(|e| e.ceil()),
            };

            vec3(grid_position.x as i32, grid_position.y as i32, grid_position.z as i32)
        };

        // Voxel space -> middle of voxel in world space
        let grid_to_position = |input: glm::TVec3<i32>| -> Vec3 {
            let input_f32 = vec3(input.x as f32, input.y as f32, input.z as f32);
            let voxel_center = vec3(
                input_f32.x * voxel_size.x + voxel_size.x / 2.0,
                input_f32.y * voxel_size.y + voxel_size.y / 2.0,
                input_f32.z * voxel_size.z + voxel_size.z / 2.0,
            );

            voxel_center + bb_min
        };

        let grid_3d_to_1d = |input: glm::TVec3<i32>| -> usize {
            let width = grid_size.x as usize;
            let height = grid_size.y as usize;
            let x = input.x as usize;
            let y = input.y as usize;
            let z = input.z as usize;

            (width * height * z) + (width * y) + x
        };

        let grid_1d_to_3d = |input: usize| -> glm::TVec3<i32> {
            let width = grid_size.x as usize;
            let height = grid_size.y as usize;
            let slice = width * height;

            let z = input / slice;
            let y = (input % slice) / width;
            let x = (input % slice) % width;

            vec3(x as i32, y as i32, z as i32)
        };

        let offsets: [Vec3; 8] = [
            vec3(0.5, 0.5, 0.5),
            vec3(-0.5, 0.5, 0.5),
            vec3(-0.5, -0.5, 0.5),
            vec3(0.5, -0.5, 0.5),
            vec3(0.5, 0.5, -0.5),
            vec3(-0.5, 0.5, -0.5),
            vec3(-0.5, -0.5, -0.5),
            vec3(0.5, -0.5, -0.5),
        ];

        // Voxelize atoms
        let start = std::time::Instant::now();
        println!("Voxelization start: {}", start.elapsed().as_secs_f64());
        for atom in atoms.iter() {
            let atom_position = atom.xyz();
            let atom_radius = atom[3];

            // 1. Find the bounding box of the atom
            let atom_bb_max = atom_position + vec3(atom_radius, atom_radius, atom_radius);
            let atom_bb_min = atom_position - vec3(atom_radius, atom_radius, atom_radius);

            let atom_bb_max = snap_to_grid(atom_bb_max, Round::Ceil);
            let atom_bb_min = snap_to_grid(atom_bb_min, Round::Floor);

            // 2. Iterate over all the voxels inside the atom's bounding box
            for x in atom_bb_min.x..atom_bb_max.x {
                for y in atom_bb_min.y..atom_bb_max.y {
                    for z in atom_bb_min.z..atom_bb_max.z {
                        let grid_position = vec3(x, y, z);

                        if grid_position < glm::zero()
                            || grid_position.x >= grid_dimension
                            || grid_position.y >= grid_dimension
                            || grid_position.z >= grid_dimension
                        {
                            continue;
                        }

                        let world_position = grid_to_position(grid_position);
                        let mut inside = true;
                        for offset in &offsets {
                            let world_position = vec3(
                                world_position.x + offset.x * voxel_size.x,
                                world_position.y + offset.y * voxel_size.y,
                                world_position.z + offset.z * voxel_size.z,
                            );

                            if glm::distance(&atom_position, &world_position) > atom_radius {
                                inside = false;
                                break;
                            }
                        }

                        if inside {
                            voxels[grid_3d_to_1d(grid_position)] = true;
                        }
                    }
                }
            }
        }
        println!("Voxelization: {}", start.elapsed().as_secs_f64());

        // Compute inner voxels
        let start = std::time::Instant::now();

        // Floodfill
        let offsets: [glm::TVec3<i32>; 8] = [
            vec3(1, 1, 1),
            vec3(-1, 1, 1),
            vec3(-1, -1, 1),
            vec3(1, -1, 1),
            vec3(1, 1, -1),
            vec3(-1, 1, -1),
            vec3(-1, -1, -1),
            vec3(1, -1, -1),
        ];
        let mut queue = VecDeque::new();

        let mut label = 0i32;
        let mut labels = vec![0i32; grid_vec_size];
        let mut label_inner: HashMap<i32, bool> = HashMap::new();
        for voxel_index in 0..grid_vec_size {
            if voxels[voxel_index] || labels[voxel_index] != 0 {
                continue;
            }

            // Create new label for new volume
            label += 1;

            // Set the label as inner, unless found otherwise
            label_inner.insert(label, true);

            //
            queue.push_back(grid_1d_to_3d(voxel_index));

            while !queue.is_empty() {
                let grid_position = queue.pop_front().unwrap();
                let grid_index = grid_3d_to_1d(grid_position);

                if grid_position.x < 0
                    || grid_position.y < 0
                    || grid_position.z < 0
                    || grid_position.x >= grid_dimension
                    || grid_position.y >= grid_dimension
                    || grid_position.z >= grid_dimension
                {
                    *label_inner.get_mut(&label).unwrap() = false;
                    continue;
                }

                if voxels[grid_index] || labels[grid_index] != 0 {
                    continue;
                }

                labels[grid_index] = label;

                for offset in &offsets {
                    queue.push_back(grid_position + offset);
                }
            }
        }

        for voxel_index in 0..grid_vec_size {
            let label = labels[voxel_index];
            if !voxels[voxel_index] && label_inner[&label] {
                voxels[voxel_index] = true;
            }
        }

        println!("Computation of inner voxels: {}", start.elapsed().as_secs_f64());

        // Compute positive distance field
        let mut sdf: Vec<glm::TVec3<i32>> = vec![glm::zero(); grid_vec_size];
        let start = std::time::Instant::now();

        for x in 0..grid_dimension {
            for y in 0..grid_dimension {
                let mut count: i32 = 1;

                for z in (0..grid_dimension).rev() {
                    if voxels[grid_3d_to_1d(glm::vec3(x, y, z))] {
                        sdf[grid_3d_to_1d(glm::vec3(x, y, z))].z = count;
                        count += 1;
                    } else {
                        count = 1;
                    }
                }
            }
        }

        for y in 0..grid_dimension {
            for z in 0..grid_dimension {
                let mut count: i32 = 1;

                for x in (0..grid_dimension).rev() {
                    if voxels[grid_3d_to_1d(glm::vec3(x, y, z))] {
                        sdf[grid_3d_to_1d(glm::vec3(x, y, z))].x = count;
                        count += 1;
                    } else {
                        count = 1;
                    }
                }
            }
        }

        for z in 0..grid_dimension {
            for x in 0..grid_dimension {
                let mut count: i32 = 1;

                for y in (0..grid_dimension).rev() {
                    if voxels[grid_3d_to_1d(glm::vec3(x, y, z))] {
                        sdf[grid_3d_to_1d(glm::vec3(x, y, z))].y = count;
                        count += 1;
                    } else {
                        count = 1;
                    }
                }
            }
        }

        println!("SDF computation time: {}", start.elapsed().as_secs_f64());

        // Compute largest bounding box        
        let occluders_limit = 16;
        let mut occluders = Vec::new();

        for _ in 0..occluders_limit {
            let start = std::time::Instant::now();

            let mut max_volume = 0;
            let mut max_position = glm::vec3(0, 0, 0);
            let mut max_extent = glm::vec3(0, 0, 0);

            for voxel_index in 0..grid_vec_size {
                use std::cmp::min;

                let voxel = voxels[voxel_index];

                if !voxel {
                    continue;
                }

                let position = grid_1d_to_3d(voxel_index);
                let distance = sdf[voxel_index];

                let mut max_aabb_extents = Vec::new();

                for z in position.z..position.z + distance.z {
                    let z_slice_position = glm::vec3(position.x, position.y, z);
                    let z_slice_index = grid_3d_to_1d(z_slice_position);
                    let sample_min_distance = sdf[z_slice_index];

                    let mut local_max_extent = glm::vec2(sample_min_distance.x, sample_min_distance.y);

                    let mut x = z_slice_position.x + 1;
                    let mut y = z_slice_position.y + 1;
                    let mut i = 1;

                    while x < z_slice_position.x + sample_min_distance.x && y < z_slice_position.y + sample_min_distance.y {
                        let index = grid_3d_to_1d(glm::vec3(x, y, z));

                        if voxels[index] {
                            let distance = sdf[index];
                            local_max_extent.x = min(distance.x + i, local_max_extent.x);
                            local_max_extent.y = min(distance.y + i, local_max_extent.y);
                        } else {
                            local_max_extent.x = i;
                            local_max_extent.y = i;
                            break;
                        }

                        x += 1;
                        y += 1;
                        i += 1;
                    }

                    max_aabb_extents.push(local_max_extent);
                }

                let mut min_extent = glm::vec2(std::i32::MAX, std::i32::MAX);
                let mut z_slice = 1;
                let mut volume = 0;

                assert!(max_aabb_extents.len() > 0);

                for extent in max_aabb_extents {
                    min_extent.x = min(extent.x, min_extent.x);
                    min_extent.y = min(extent.y, min_extent.y);

                    let new_volume = min_extent.x * min_extent.y * z_slice;
                    if new_volume > volume {
                        volume = new_volume;
                    }

                    z_slice += 1;
                }

                let extent = glm::vec3(min_extent.x, min_extent.y, z_slice - 1);
                let volume = extent.x * extent.y * extent.z;

                if volume > max_volume {
                    max_volume = volume;
                    max_extent = extent;
                    max_position = position;
                }

                break;
            }
            
            // Clean the inner voxels 
            for x in max_position.x..=max_position.x + max_extent.x {
                for y in max_position.y..=max_position.y + max_extent.y {
                    for z in max_position.z..=max_position.z + max_extent.z {
                        voxels[grid_3d_to_1d(vec3(x, y, z))] = false;
                    }
                }
            }

            // Recompute SDF
            for x in 0..grid_dimension {
                for y in 0..grid_dimension {
                    let mut count: i32 = 1;
    
                    for z in (0..grid_dimension).rev() {
                        if voxels[grid_3d_to_1d(glm::vec3(x, y, z))] {
                            sdf[grid_3d_to_1d(glm::vec3(x, y, z))].z = count;
                            count += 1;
                        } else {
                            count = 1;
                        }
                    }
                }
            }
    
            for y in 0..grid_dimension {
                for z in 0..grid_dimension {
                    let mut count: i32 = 1;
    
                    for x in (0..grid_dimension).rev() {
                        if voxels[grid_3d_to_1d(glm::vec3(x, y, z))] {
                            sdf[grid_3d_to_1d(glm::vec3(x, y, z))].x = count;
                            count += 1;
                        } else {
                            count = 1;
                        }
                    }
                }
            }
    
            for z in 0..grid_dimension {
                for x in 0..grid_dimension {
                    let mut count: i32 = 1;
    
                    for y in (0..grid_dimension).rev() {
                        if voxels[grid_3d_to_1d(glm::vec3(x, y, z))] {
                            sdf[grid_3d_to_1d(glm::vec3(x, y, z))].y = count;
                            count += 1;
                        } else {
                            count = 1;
                        }
                    }
                }
            }

            occluders.push((max_position, max_position + max_extent));

            println!("Occluder: {:?} {:?}", max_position, max_extent);
            println!("Occluder time: {}", start.elapsed().as_secs_f64());
        }

        Self {
            size: grid_dimension,

            bb_min,
            bb_max,
            bb_diff,
            voxel_size,

            voxels,
            sdf,
            labels,
            label,
            label_inner,

            occluders,
        }
    }
}
