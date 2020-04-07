use nalgebra_glm as glm;
use wgpu;

pub struct VoxelGrid {
    pub size: u32,

    pub bb_min: glm::Vec3,
    pub bb_max: glm::Vec3,
    pub bb_diff: glm::Vec3,

    pub voxels: Vec<Voxel>,
}
#[derive(Copy, Clone)]
pub struct Voxel {
    pub filled: bool,

    /// Distance of filled voxel to shell in each positive direction
    pub distance: glm::TVec3<u32>,
}

impl Default for Voxel {
    fn default() -> Self {
        Voxel {
            filled: false,
            distance: glm::vec3(0, 0, 0),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Round {
    Floor,
    Ceil,
}

impl VoxelGrid {
    pub fn new(device: &wgpu::Device, mut atoms: Vec<glm::Vec4>) -> Self {
        // Find bounding box of the entire structure
        let mut bb_max = glm::vec3(std::f32::NEG_INFINITY, std::f32::NEG_INFINITY, std::f32::NEG_INFINITY);
        let mut bb_min = glm::vec3(std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY);
        for atom in atoms.iter() {
            let atom_position = atom.xyz();
            let atom_radius = atom[4];
            bb_max = glm::max2(&bb_max, &(atom_position + glm::vec3(atom_radius, atom_radius, atom_radius)));
            bb_min = glm::min2(&bb_min, &(atom_position - glm::vec3(atom_radius, atom_radius, atom_radius)));
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
        let grid_dimension = 128u32;
        let grid_size = glm::vec3(grid_dimension, grid_dimension, grid_dimension);
        let voxel_size = glm::vec3(
            bb_diff.x / grid_size.x as f32,
            bb_diff.y / grid_size.y as f32,
            bb_diff.z / grid_size.z as f32,
        );
        let voxel_halfsize = voxel_size.apply_into(|e| e * 0.5);
        let mut voxels = vec![Voxel::default(); (grid_dimension * grid_dimension * grid_dimension) as usize];

        // Voxelize atoms
        let snap_to_grid = |input: glm::Vec3, round: Round| -> glm::TVec3<u32> {
            let grid_position = match round {
                Round::Floor => input.apply_into(|e| e.floor()),
                Round::Ceil => input.apply_into(|e| e.ceil()),
            };
            let grid_position = grid_position - bb_min;
            let grid_position = glm::vec3(
                grid_position.x / voxel_size.x,
                grid_position.y / voxel_size.y,
                grid_position.z / voxel_size.z,
            );

            glm::vec3(grid_position.x as u32, grid_position.y as u32, grid_position.z as u32)
        };

        let grid_to_position = |input: glm::TVec3<u32>| -> glm::Vec3 {
            let input_f32 = glm::vec3(input.x as f32, input.y as f32, input.z as f32);
            let voxel_center = glm::vec3(
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

        // let grid_1d_to_3d: = input|input: usize| -> glm::TVec3<u32> {

        // }

        for atom in atoms.iter() {
            let atom_position = atom.xyz();
            let atom_radius = atom[4];

            // 1. Find the bounding box of the atom
            let atom_bb_max = atom_position + glm::vec3(atom_radius, atom_radius, atom_radius);
            let atom_bb_min = atom_position - glm::vec3(atom_radius, atom_radius, atom_radius);

            let atom_bb_max = snap_to_grid(atom_bb_max, Round::Ceil);
            let atom_bb_min = snap_to_grid(atom_bb_min, Round::Floor);

            let offsets: [glm::Vec3; 8] = [
                glm::vec3(1.0, 1.0, 1.0),
                glm::vec3(-1.0, 1.0, 1.0),
                glm::vec3(-1.0, -1.0, 1.0),
                glm::vec3(1.0, -1.0, 1.0),
                glm::vec3(1.0, 1.0, -1.0),
                glm::vec3(-1.0, 1.0, -1.0),
                glm::vec3(-1.0, -1.0, -1.0),
                glm::vec3(1.0, -1.0, -1.0),
            ];
            // 2. Iterate over all the voxels inside the atom's bounding box
            for x in atom_bb_min.x..atom_bb_max.x {
                for y in atom_bb_min.y..atom_bb_max.y {
                    for z in atom_bb_min.z..atom_bb_max.z {
                        let grid_position = glm::vec3(x, y, z);
                        let world_position = grid_to_position(grid_position);

                        // 3. Mark the voxels as `filled` only when all corners of a voxel are inside the radius of the atom
                        let mut inside_sphere = true;
                        for offset in offsets.iter() {
                            let offset = glm::vec3(
                                voxel_halfsize.x * offset.x,
                                voxel_halfsize.y * offset.y,
                                voxel_halfsize.z * offset.z,
                            );
                            let world_position = world_position + offset;

                            if glm::distance(&atom_position, &world_position) > atom_radius {
                                inside_sphere = false;
                            }
                        }

                        if inside_sphere {
                            voxels[grid_3d_to_1d(grid_position)].filled = true;
                        }
                    }
                }
            }
        }

        // Compute in the same loop:
        // - positive distance field
        // - inner voxels
        for global_x in 0..grid_size.x {
            for global_y in 0..grid_size.y {
                for global_z in 0..grid_size.z {
                    let mut distance = glm::vec3(0, 0, 0);
                    let mut inner = [false; 6];

                    for x in 0..grid_size.x {
                        if x < global_x && voxels[grid_3d_to_1d(glm::vec3(x, global_y, global_z))].filled {
                            inner[0] = true;
                        }

                        if x > global_x {
                            if voxels[grid_3d_to_1d(glm::vec3(x, global_y, global_z))].filled {
                                distance.x += 1;
                            } else {
                                break;
                            }
                        }
                    }

                    for y in 0..grid_size.y {
                        if y < global_y && voxels[grid_3d_to_1d(glm::vec3(global_x, y, global_z))].filled {
                            inner[0] = true;
                        }

                        if y > global_y {
                            if voxels[grid_3d_to_1d(glm::vec3(global_x, y, global_z))].filled {
                                distance.y += 1;
                            } else {
                                break;
                            }
                        }
                    }

                    for z in 0..grid_size.z {
                        if z < global_z && voxels[grid_3d_to_1d(glm::vec3(global_x, global_y, z))].filled {
                            inner[0] = true;
                        }

                        if z > global_z {
                            if voxels[grid_3d_to_1d(glm::vec3(global_x, global_y, z))].filled {
                                distance.z += 1;
                            } else {
                                break;
                            }
                        }
                    }

                    if inner == [true; 6] {
                        voxels[grid_3d_to_1d(glm::vec3(global_x, global_y, global_z))].filled;
                    }
                }
            }
        }

        Self {
            size : grid_dimension,
            
            bb_min,
            bb_max,
            bb_diff,

            voxels,
        }
    }
}
