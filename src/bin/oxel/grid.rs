use glm::{vec3, Vec3};
use nalgebra_glm as glm;
use rayon::prelude::*;

pub struct VoxelGrid {
    pub size: u32,

    pub bb_min: Vec3,
    pub bb_max: Vec3,
    pub bb_diff: Vec3,
    pub voxel_size: Vec3,

    pub voxels: Vec<bool>,
    pub sdf: Vec<glm::TVec3<u32>>,

    pub occluders: Vec<(glm::TVec3<u32>, glm::TVec3<u32>)>,
}
// #[derive(Copy, Clone, Debug)]
// pub struct Voxel {
//     pub filled: bool,

//     /// Distance of filled voxel to shell in each positive direction
//     pub distance: glm::TVec3<u32>,
// }

// impl Default for Voxel {
//     fn default() -> Self {
//         Voxel {
//             filled: false,
//             distance: vec3(0, 0, 0),
//         }
//     }
// }
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
        let grid_dimension = 256u32;
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
        let snap_to_grid = |input: Vec3, round: Round| -> glm::TVec3<u32> {
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

            vec3(grid_position.x as u32, grid_position.y as u32, grid_position.z as u32)
        };

        // Voxel space -> middle of voxel in world space
        let grid_to_position = |input: glm::TVec3<u32>| -> Vec3 {
            let input_f32 = vec3(input.x as f32, input.y as f32, input.z as f32);
            let voxel_center = vec3(
                input_f32.x * voxel_size.x + voxel_size.x / 2.0,
                input_f32.y * voxel_size.y + voxel_size.y / 2.0,
                input_f32.z * voxel_size.z + voxel_size.z / 2.0,
            );

            voxel_center + bb_min
        };

        let grid_3d_to_1d = |input: glm::TVec3<u32>| -> usize {
            let width = grid_size.x as usize;
            let height = grid_size.y as usize;
            let x = input.x as usize;
            let y = input.y as usize;
            let z = input.z as usize;

            (width * height * z) + (width * y) + x
        };

        let grid_1d_to_3d = |input: usize| -> glm::TVec3<u32> {
            let width = grid_size.x as usize;
            let height = grid_size.y as usize;
            let slice = width * height;

            let z = input / slice;
            let y = (input % slice) / width;
            let x = (input % slice) % width;

            vec3(x as u32, y as u32, z as u32)
        };

        let offsets: [Vec3; 8] = [
            vec3(1.0, 1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(-1.0, -1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, -1.0),
            vec3(-1.0, 1.0, -1.0),
            vec3(-1.0, -1.0, -1.0),
            vec3(1.0, -1.0, -1.0),
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

            let atom_radius = atom[3] - voxel_diameter;

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
                        if glm::distance(&atom_position, &world_position) < atom_radius {
                            voxels[grid_3d_to_1d(grid_position)] = true;
                        }
                    }
                }
            }
        }
        println!("Voxelization: {}", start.elapsed().as_secs_f64());

        // Compute inner voxels

        let start = std::time::Instant::now();
        let mut inner = vec![true; grid_vec_size];

        /*
        //
        for x in 0..grid_dimension {
            for y in 0..grid_dimension {
                let mut first_voxel = None;
                let mut last_voxel = None;

                for z in 0..grid_dimension {
                    if voxels[grid_3d_to_1d(glm::vec3(x, y, z))] {
                        if first_voxel == None {
                            first_voxel = Some(z);
                        } else {
                            last_voxel = Some(z);
                        }
                    }
                }

                if first_voxel == None || last_voxel == None {
                    for z in 0..grid_dimension {
                        inner[grid_3d_to_1d(glm::vec3(x, y, z))] = false;
                    }
                } else {
                    let start = first_voxel.unwrap();
                    let end = last_voxel.unwrap();

                    for z in 0..start {
                        inner[grid_3d_to_1d(glm::vec3(x, y, z))] = false;
                    }

                    for z in end + 1..grid_dimension {
                        inner[grid_3d_to_1d(glm::vec3(x, y, z))] = false;
                    }
                }
            }
        }

        for y in 0..grid_dimension {
            for z in 0..grid_dimension {
                let mut first_voxel = None;
                let mut last_voxel = None;

                for x in 0..grid_dimension {
                    if voxels[grid_3d_to_1d(glm::vec3(x, y, z))] {
                        if first_voxel == None {
                            first_voxel = Some(x);
                        } else {
                            last_voxel = Some(x);
                        }
                    }
                }

                if first_voxel == None || last_voxel == None {
                    for x in 0..grid_dimension {
                        inner[grid_3d_to_1d(glm::vec3(x, y, z))] = false;
                    }
                } else {
                    let start = first_voxel.unwrap();
                    let end = last_voxel.unwrap();

                    for x in 0..start {
                        inner[grid_3d_to_1d(glm::vec3(x, y, z))] = false;
                    }

                    for x in end + 1..grid_dimension {
                        inner[grid_3d_to_1d(glm::vec3(x, y, z))] = false;
                    }
                }
            }
        }

        for x in 0..grid_dimension {
            for z in 0..grid_dimension {
                let mut first_voxel = None;
                let mut last_voxel = None;

                for y in 0..grid_dimension {
                    if voxels[grid_3d_to_1d(glm::vec3(x, y, z))] {
                        if first_voxel == None {
                            first_voxel = Some(y);
                        } else {
                            last_voxel = Some(y);
                        }
                    }
                }

                if first_voxel == None || last_voxel == None {
                    for y in 0..grid_dimension {
                        inner[grid_3d_to_1d(glm::vec3(x, y, z))] = false;
                    }
                } else {
                    let start = first_voxel.unwrap();
                    let end = last_voxel.unwrap();

                    for y in 0..start {
                        inner[grid_3d_to_1d(glm::vec3(x, y, z))] = false;
                    }

                    for y in end + 1..grid_dimension {
                        inner[grid_3d_to_1d(glm::vec3(x, y, z))] = false;
                    }
                }
            }
        }

        for index in 0..grid_vec_size {
            if inner[index] {
                voxels[index] = true;
            }
        }
        */

        println!("Computation of inner voxels: {}", start.elapsed().as_secs_f64());

        // Compute positive distance field
        let mut sdf: Vec<glm::TVec3<u32>> = vec![glm::zero(); grid_vec_size];
        let start = std::time::Instant::now();

        for x in 0..grid_dimension {
            for y in 0..grid_dimension {
                let mut count: u32 = 1;

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
                let mut count: u32 = 1;

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
                let mut count: u32 = 1;

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
        // let voxels_return = voxels.clone();
        //
        // Make a vector to hold the children which are spawned.
        let start = std::time::Instant::now();
        let occluders_limit = 1;
        let mut occluders = Vec::new();
        /*
        let num_threads = 24;
        let part = grid_vec_size / num_threads;
        let g_max_position = std::sync::Arc::new(std::sync::RwLock::new(glm::vec3(0, 0, 0)));
        let g_max_area = std::sync::Arc::new(std::sync::RwLock::new(0));

        crossbeam_utils::thread::scope(|s| {
            let sdf = &sdf;
            let voxels = &voxels;

            for i in 0..num_threads {
                let g_max_position = g_max_position.clone();
                let g_max_area = g_max_area.clone();
                s.spawn(move |_| {
                    let mut max_position = glm::vec3(0, 0, 0);
                    let mut max_area = 0;
                    for voxel_index in i * part..(i + 1) * part {
                        use std::cmp::min;

                        let voxel = voxels[voxel_index];

                        if !voxel {
                            continue;
                        }

                        let position = grid_1d_to_3d(voxel_index);
                        let distance = sdf[voxel_index];

                        let mut min_x = distance.x;
                        for d_y in 0..distance.y {
                            min_x = min(min_x, sdf[grid_3d_to_1d(position + vec3(0, d_y, 0))].x);

                            let mut min_z = distance.z;
                            for x in 0..min_x {
                                for y in 0..d_y {
                                    min_z = min(min_z, sdf[grid_3d_to_1d(position + vec3(x, y, 0))].z);

                                    let area = min_x * d_y * min_z;

                                    if area > max_area {
                                        max_area = area;
                                        max_position = position;
                                    }
                                }
                            }
                        }
                    }

                    let mut g_max_area = g_max_area.write().unwrap();

                    if max_area > *g_max_area {
                        *g_max_area = max_area;

                        let mut g_max_position = g_max_position.write().unwrap();
                        *g_max_position = max_position;
                    }
                });
            }
        })
        .unwrap();
        */

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
                /*
                let z_slice_position = glm::vec3(position.x, position.y, z);
                let z_slice_index = grid_3d_to_1d(z_slice_position);
                let sample_min_distance = sdf[z_slice_index];

                let mut local_max_extent = glm::vec2(1, 1);
                let mut min_x = sample_min_distance.x;
                for y in position.y..position.y + distance.y {
                    min_x = min(min_x, sdf[grid_3d_to_1d(vec3(position.x, y, z))].x);
                    
                    let area = y * min_x;
                    if area > local_max_extent.x * local_max_extent.y {
                        println!("{:?}", local_max_extent);
                        local_max_extent = glm::vec2(min_x, y);
                    }
                }
                */
                
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

            let mut min_extent = glm::vec2(std::u32::MAX, std::u32::MAX);
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

        println!("{:?} {:?}", max_position, max_extent);
        occluders.push((max_position, max_position + max_extent));

        println!("Occluder: {}", start.elapsed().as_secs_f64());

        Self {
            size: grid_dimension,

            bb_min,
            bb_max,
            bb_diff,
            voxel_size,

            voxels,
            sdf,

            occluders,
        }
    }
}
