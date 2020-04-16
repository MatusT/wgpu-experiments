use glm::{vec3, Vec3};
use image;
use nalgebra_glm as glm;
use safe_transmute::*;
use std::collections::{HashMap, VecDeque};
use wgpu_experiments::camera::CameraUbo;
use wgpu_experiments::pipelines::boxes::ClippedGridPipeline;

static T: [(i8, i8); 8] = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)];
static O_VERTEX: [(i8, i8); 7] = [(-1, 0), (0, 0), (-1, -1), (0, 0), (0, -1), (0, 0), (0, 0)]; // Vertex coordinates for the outlines (bottom left) according to the orientation
static H_VERTEX: [(i8, i8); 7] = [(0, 0), (0, 0), (-1, 0), (0, 0), (-1, -1), (0, 0), (0, -1)]; // Vertex coordinates for the holes (bottom right) according to the orientation
static O_VALUE: [i8; 7] = [1, 0, 2, 0, 4, 0, 8]; // Value to add into the array of contours for the outlines
static H_VALUE: [i8; 7] = [-4, 0, -8, 0, -1, 0, -2]; // Value to add into the array of contours for the holes

pub fn bits_to_paths(width: usize, height: usize, bits: &[i8], closepaths: bool) -> String {
    let rows: usize = height;
    let cols: usize = width;

    let mut contours = vec![vec![0i8; cols + 2]; rows + 2]; // The array of contours needs a border of 1 bit
    for y in 0..=rows - 1 as usize {
        for x in 0..=cols - 1 as usize {
            contours[y + 1][x + 1] = bits[y * width + x];
        }
    }

    let mut paths = String::new();
    let mut ol: usize;
    let mut hl: usize;
    for y in 1..=rows as usize {
        ol = 1;
        hl = 1;
        for x in 1..=cols as usize {
            if ol == hl && contours[y][x] == 1 && contours[y][x - 1] <= 0 && contours[y - 1][x] <= 0 {
                trace(
                    true,
                    x,
                    y,
                    [2, 3, 4, 5, 6, 7, 0, 1],
                    2,
                    (7, 1, 0),
                    O_VERTEX,
                    O_VALUE,
                    &mut contours,
                    &mut paths,
                    closepaths,
                );
            }
            if contours[y][x] == 2 || contours[y][x] == 4 || contours[y][x] == 10 || contours[y][x] == 12 {
                ol += 1;
            }
            if contours[y][x] == 5 || contours[y][x] == 7 || contours[y][x] == 13 || contours[y][x] == 15 {
                ol -= 1;
            }
            if ol > hl && contours[y][x] == 0 && contours[y][x - 1] > 0 && contours[y - 1][x] > 0 {
                trace(
                    false,
                    x,
                    y,
                    [4, 5, 6, 7, 0, 1, 2, 3],
                    -2,
                    (1, 7, 6),
                    H_VERTEX,
                    H_VALUE,
                    &mut contours,
                    &mut paths,
                    closepaths,
                );
            }
            if contours[y][x] == -1 || contours[y][x] == -3 || contours[y][x] == -9 || contours[y][x] == -11 {
                hl += 1;
            }
            if contours[y][x] == -4 || contours[y][x] == -6 || contours[y][x] == -12 || contours[y][x] == -14 {
                hl -= 1;
            }
        }
    }
    paths
}

fn trace(
    hole: bool,
    x: usize,
    y: usize,
    mut o: [usize; 8],
    rot: i8,
    viv: (usize, usize, usize),
    c_vertex: [(i8, i8); 7],
    c_value: [i8; 7],
    contours: &mut Vec<Vec<i8>>,
    paths: &mut String,
    closepaths: bool,
) {
    let mut cx = x; // Current x
    let mut cy = y; // Current y
    let mut v: usize = 1; // Number of vertices
    paths.push_str(&format!(
        "M{} {}",
        cx.wrapping_add(c_vertex[o[0]].0 as usize),
        cy.wrapping_add(c_vertex[o[0]].1 as usize)
    ));
    let mut rn: u8;
    loop {
        let neighbors: [i8; 8] = [
            contours[cy - 1][cx],
            contours[cy - 1][cx + 1],
            contours[cy][cx + 1],
            contours[cy + 1][cx + 1],
            contours[cy + 1][cx],
            contours[cy + 1][cx - 1],
            contours[cy][cx - 1],
            contours[cy - 1][cx - 1],
        ];
        if hole {
            if neighbors[o[7]] > 0 && neighbors[o[0]] > 0 {
                rn = 1;
            } else if neighbors[o[0]] > 0 {
                rn = 2;
            } else if neighbors[o[1]] > 0 && neighbors[o[2]] > 0 {
                rn = 3;
            } else {
                rn = 0;
            }
        } else {
            if neighbors[o[1]] <= 0 && neighbors[o[0]] <= 0 {
                rn = 1;
            } else if neighbors[o[0]] <= 0 {
                rn = 2;
            } else if neighbors[o[7]] <= 0 && neighbors[o[6]] <= 0 {
                rn = 3;
            } else {
                rn = 0;
            }
        }
        if rn == 1 {
            contours[cy][cx] += c_value[o[0]];
            cx = cx.wrapping_add(T[o[viv.0]].0 as usize);
            cy = cy.wrapping_add(T[o[viv.0]].1 as usize);
            o.rotate_right(rot.rem_euclid(8) as usize); // Rotate 90 degrees, counterclockwise for the outlines (rot = 2) or clockwise for the holes (rot = -2)
            v += 1;
            if o[0] == 0 || o[0] == 4 {
                paths.push_str(&format!("H{}", cx.wrapping_add(c_vertex[o[0]].0 as usize)));
            } else {
                paths.push_str(&format!("V{}", cy.wrapping_add(c_vertex[o[0]].1 as usize)));
            }
        } else if rn == 2 {
            contours[cy][cx] += c_value[o[0]];
            cx = cx.wrapping_add(T[o[0]].0 as usize);
            cy = cy.wrapping_add(T[o[0]].1 as usize);
        } else if rn == 3 {
            contours[cy][cx] += c_value[o[0]];
            o.rotate_left(rot.rem_euclid(8) as usize); // Rotate 90 degrees, clockwise for the outlines (rot = 2) or counterclockwise for the holes (rot = -2)
            contours[cy][cx] += c_value[o[0]];
            v += 1;
            if o[0] == 0 || o[0] == 4 {
                paths.push_str(&format!("H{}", cx.wrapping_add(c_vertex[o[0]].0 as usize)));
            } else {
                paths.push_str(&format!("V{}", cy.wrapping_add(c_vertex[o[0]].1 as usize)));
            }
            o.rotate_right(rot.rem_euclid(8) as usize);
            cx = cx.wrapping_add(T[o[viv.1]].0 as usize);
            cy = cy.wrapping_add(T[o[viv.1]].1 as usize);
            v += 1;
            if o[0] == 0 || o[0] == 4 {
                paths.push_str(&format!("H{}", cx.wrapping_add(c_vertex[o[0]].0 as usize)));
            } else {
                paths.push_str(&format!("V{}", cy.wrapping_add(c_vertex[o[0]].1 as usize)));
            }
        } else {
            contours[cy][cx] += c_value[o[0]];
            o.rotate_left(rot.rem_euclid(8) as usize);
            v += 1;
            if o[0] == 0 || o[0] == 4 {
                paths.push_str(&format!("H{}", cx.wrapping_add(c_vertex[o[0]].0 as usize)));
            } else {
                paths.push_str(&format!("V{}", cy.wrapping_add(c_vertex[o[0]].1 as usize)));
            }
        }
        if cx == x && cy == y && v > 2 {
            break;
        }
    }
    loop {
        contours[cy][cx] += c_value[o[0]];
        if o[0] == viv.2 {
            break;
        }
        o.rotate_left(rot.rem_euclid(8) as usize);
        v += 1;
        if o[0] == 0 || o[0] == 4 {
            paths.push_str(&format!("H{}", cx.wrapping_add(c_vertex[o[0]].0 as usize)));
        } else {
            paths.push_str(&format!("V{}", cy.wrapping_add(c_vertex[o[0]].1 as usize)));
        }
    }
    if closepaths {
        paths.push_str("Z");
    }
}
pub struct VoxelGrid {
    //
    pub size: i32,

    // Bounding box of the grid in world space
    pub bb_min: Vec3,
    pub bb_max: Vec3,
    pub bb_diff: Vec3,

    // Sizes related to single voxel
    pub voxel_size: Vec3,
    pub voxel_halfsize: Vec3,
    pub voxel_diameter: f32,

    pub voxels: Vec<bool>,
}
#[derive(Copy, Clone)]
pub enum Round {
    Floor,
    Ceil,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ClipPlane {
    pub point: [f32; 4],
    pub normal: [f32; 4],
}

impl Default for ClipPlane {
    fn default() -> Self {
        Self {
            point: [0.0; 4],
            normal: [0.0; 4],
        }
    }
}

unsafe impl TriviallyTransmutable for ClipPlane {}

impl VoxelGrid {
    // World position inside BB -> voxel space
    pub fn snap(&self, input: Vec3, round: Round) -> glm::TVec3<i32> {
        let grid_position = input - self.bb_min;

        let grid_position = vec3(
            grid_position.x / self.voxel_size.x,
            grid_position.y / self.voxel_size.y,
            grid_position.z / self.voxel_size.z,
        );

        let grid_position = match round {
            Round::Floor => grid_position.apply_into(|e| e.floor()),
            Round::Ceil => grid_position.apply_into(|e| e.ceil()),
        };

        vec3(grid_position.x as i32, grid_position.y as i32, grid_position.z as i32)
    }

    // Voxel space -> middle of voxel in world space
    pub fn to_ws(&self, input: glm::TVec3<i32>) -> Vec3 {
        let input_f32 = vec3(input.x as f32, input.y as f32, input.z as f32);
        let voxel_center = vec3(
            input_f32.x * self.voxel_size.x + self.voxel_halfsize.x,
            input_f32.y * self.voxel_size.y + self.voxel_halfsize.y,
            input_f32.z * self.voxel_size.z + self.voxel_halfsize.z,
        );

        voxel_center + self.bb_min
    }

    pub fn to_1d(&self, input: glm::TVec3<i32>) -> usize {
        let width = self.size as usize;
        let height = self.size as usize;
        let x = input.x as usize;
        let y = input.y as usize;
        let z = input.z as usize;

        (width * height * z) + (width * y) + x
    }

    pub fn to_3d(&self, input: usize) -> glm::TVec3<i32> {
        let width = self.size as usize;
        let height = self.size as usize;
        let slice = width * height;

        let z = input / slice;
        let y = (input % slice) / width;
        let x = (input % slice) % width;

        vec3(x as i32, y as i32, z as i32)
    }

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
        let size = 256i32;

        let voxel_size = vec3(bb_diff.x / size as f32, bb_diff.y / size as f32, bb_diff.z / size as f32);
        let voxel_halfsize = voxel_size.apply_into(|e| e * 0.5);
        let voxel_diameter = glm::distance(&voxel_size, &glm::vec3(0.0, 0.0, 0.0));

        let voxels = vec![false; (size * size * size) as usize];

        let mut grid = Self {
            size,

            bb_min,
            bb_max,
            bb_diff,

            voxel_size,
            voxel_halfsize,
            voxel_diameter,

            voxels,
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
        for atom in atoms.iter() {
            let atom_position = atom.xyz();
            let atom_radius = atom[3];

            // 1. Find the bounding box of the atom
            let atom_bb_max = atom_position + vec3(atom_radius, atom_radius, atom_radius);
            let atom_bb_min = atom_position - vec3(atom_radius, atom_radius, atom_radius);

            let atom_bb_max = grid.snap(atom_bb_max, Round::Ceil);
            let atom_bb_min = grid.snap(atom_bb_min, Round::Floor);

            // 2. Iterate over all the voxels inside the atom's bounding box
            for x in atom_bb_min.x..atom_bb_max.x {
                for y in atom_bb_min.y..atom_bb_max.y {
                    for z in atom_bb_min.z..atom_bb_max.z {
                        let grid_position = vec3(x, y, z);

                        if grid_position < glm::zero()
                            || grid_position.x >= grid.size
                            || grid_position.y >= grid.size
                            || grid_position.z >= grid.size
                        {
                            continue;
                        }

                        let world_position = grid.to_ws(grid_position);
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
                            let pos = grid.to_1d(grid_position);
                            grid.voxels[pos] = true;
                        }
                    }
                }
            }
        }

        // Flood fill
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
        let mut labels = vec![0i32; grid.voxels.len()];
        let mut label_inner: HashMap<i32, bool> = HashMap::new();
        for voxel_index in 0..grid.voxels.len() {
            if grid.voxels[voxel_index] || labels[voxel_index] != 0 {
                continue;
            }

            // Create new label for new volume
            label += 1;

            // Set the label as inner, unless found otherwise
            label_inner.insert(label, true);

            //
            queue.push_back(grid.to_3d(voxel_index));

            while !queue.is_empty() {
                let grid_position = queue.pop_front().unwrap();
                let grid_index = grid.to_1d(grid_position);

                if grid_position.x < 0
                    || grid_position.y < 0
                    || grid_position.z < 0
                    || grid_position.x >= grid.size
                    || grid_position.y >= grid.size
                    || grid_position.z >= grid.size
                {
                    *label_inner.get_mut(&label).unwrap() = false;
                    continue;
                }

                if grid.voxels[grid_index] || labels[grid_index] != 0 {
                    continue;
                }

                labels[grid_index] = label;

                for offset in &offsets {
                    queue.push_back(grid_position + offset);
                }
            }
        }

        for voxel_index in 0..grid.voxels.len() {
            let label = labels[voxel_index];
            if !grid.voxels[voxel_index] && label_inner[&label] {
                grid.voxels[voxel_index] = true;
            }
        }

        grid
    }

    pub fn get_box_occluders(&mut self, limit: usize) -> Vec<(glm::TVec3<i32>, glm::TVec3<i32>)> {
        // Compute positive distance field
        let mut sdf: Vec<glm::TVec3<i32>> = vec![glm::zero(); self.voxels.len()];

        for x in 0..self.size {
            for y in 0..self.size {
                let mut count: i32 = 1;

                for z in (0..self.size).rev() {
                    if self.voxels[self.to_1d(glm::vec3(x, y, z))] {
                        sdf[self.to_1d(glm::vec3(x, y, z))].z = count;
                        count += 1;
                    } else {
                        count = 1;
                    }
                }
            }
        }

        for y in 0..self.size {
            for z in 0..self.size {
                let mut count: i32 = 1;

                for x in (0..self.size).rev() {
                    if self.voxels[self.to_1d(glm::vec3(x, y, z))] {
                        sdf[self.to_1d(glm::vec3(x, y, z))].x = count;
                        count += 1;
                    } else {
                        count = 1;
                    }
                }
            }
        }

        for z in 0..self.size {
            for x in 0..self.size {
                let mut count: i32 = 1;

                for y in (0..self.size).rev() {
                    if self.voxels[self.to_1d(glm::vec3(x, y, z))] {
                        sdf[self.to_1d(glm::vec3(x, y, z))].y = count;
                        count += 1;
                    } else {
                        count = 1;
                    }
                }
            }
        }

        // Compute largest bounding box
        let mut occluders = Vec::new();

        for _ in 0..limit {
            let mut max_volume = 0;
            let mut max_position = glm::vec3(0, 0, 0);
            let mut max_extent = glm::vec3(0, 0, 0);

            for voxel_index in 0..self.voxels.len() {
                use std::cmp::min;

                let voxel = self.voxels[voxel_index];

                if !voxel {
                    continue;
                }

                let position = self.to_3d(voxel_index);
                let distance = sdf[voxel_index];

                let mut max_aabb_extents = Vec::new();

                for z in position.z..position.z + distance.z {
                    let z_slice_position = glm::vec3(position.x, position.y, z);
                    let z_slice_index = self.to_1d(z_slice_position);
                    let sample_min_distance = sdf[z_slice_index];

                    let mut local_max_extent = glm::vec2(sample_min_distance.x, sample_min_distance.y);

                    let mut x = z_slice_position.x + 1;
                    let mut y = z_slice_position.y + 1;
                    let mut i = 1;

                    while x < z_slice_position.x + sample_min_distance.x && y < z_slice_position.y + sample_min_distance.y {
                        let index = self.to_1d(glm::vec3(x, y, z));

                        if self.voxels[index] {
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

            for voxel_index in 0..self.voxels.len() {
                use std::cmp::min;

                let voxel = self.voxels[voxel_index];

                if !voxel {
                    continue;
                }

                let position = self.to_3d(voxel_index);
                let distance = sdf[voxel_index];

                let mut min_x = std::i32::MAX;
                for d_y in 0..distance.y {
                    min_x = min(min_x, sdf[self.to_1d(position + vec3(0, d_y, 0))].x);

                    let mut min_z = std::i32::MAX;
                    for y in 0..d_y {
                        for x in 0..min_x {
                            min_z = min(min_z, sdf[self.to_1d(position + vec3(x, y, 0))].z);
                        }
                    }

                    let volume = min_x * d_y * min_z;

                    if volume > max_volume {
                        max_volume = volume;
                        max_position = position;
                        max_extent = vec3(min_x, d_y, min_z);
                    }
                }
            }

            // Clean the inner voxels
            for x in max_position.x..=max_position.x + max_extent.x {
                for y in max_position.y..=max_position.y + max_extent.y {
                    for z in max_position.z..=max_position.z + max_extent.z {
                        let position = self.to_1d(vec3(x, y, z));
                        self.voxels[position] = false;
                    }
                }
            }

            // Recompute SDF
            for x in 0..self.size {
                for y in 0..self.size {
                    let mut count: i32 = 1;

                    for z in (0..self.size).rev() {
                        if self.voxels[self.to_1d(glm::vec3(x, y, z))] {
                            sdf[self.to_1d(glm::vec3(x, y, z))].z = count;
                            count += 1;
                        } else {
                            count = 1;
                        }
                    }
                }
            }

            for y in 0..self.size {
                for z in 0..self.size {
                    let mut count: i32 = 1;

                    for x in (0..self.size).rev() {
                        if self.voxels[self.to_1d(glm::vec3(x, y, z))] {
                            sdf[self.to_1d(glm::vec3(x, y, z))].x = count;
                            count += 1;
                        } else {
                            count = 1;
                        }
                    }
                }
            }

            for z in 0..self.size {
                for x in 0..self.size {
                    let mut count: i32 = 1;

                    for y in (0..self.size).rev() {
                        if self.voxels[self.to_1d(glm::vec3(x, y, z))] {
                            sdf[self.to_1d(glm::vec3(x, y, z))].y = count;
                            count += 1;
                        } else {
                            count = 1;
                        }
                    }
                }
            }

            occluders.push((max_position, max_position + max_extent));
        }

        occluders
    }
    pub fn get_planar_occluders(&mut self, device: &wgpu::Device, queue: &mut wgpu::Queue, limit: usize) -> Vec<Vec<Vec3>> {
        use lyon::math::Point;
        use lyon::svg::path_utils::*;
        use lyon::tessellation::*;

        let radius = glm::length(&self.bb_diff) / 2.0;

        let no_cuts = 9;
        let cut_step = glm::length(&self.bb_diff) / no_cuts as f32;

        let views = [vec3(0.0, 0.0, 1.0)];

        let mut planes_triangles: Vec<Vec<Vec3>> = Vec::new();
        for view in &views {
            let triangles = Vec::new();
            let mut max_plane = ClipPlane::default(); // Plane with maximum area (optional: after erosion)
            let mut i = 0;

            let mut max_area = 0;
            let mut max_img = Vec::new();
            for cut in -no_cuts / 2..=no_cuts / 2 {
                let mut save_img = image::ImageBuffer::new(self.size as u32, self.size as u32);
                let mut img = Vec::new();
                let mut area: u64 = 0;
                let step = glm::length(&self.bb_diff) / self.size as f32;
                for y in -self.size / 2..self.size / 2 {
                    for x in -self.size / 2..self.size / 2 {
                        // World-space coordinates
                        let x_ws = x as f32 * step + step * 0.5;
                        let y_ws = y as f32 * step + step * 0.5;
                        let z_ws = cut as f32 * cut_step;

                        let occupied;
                        if x_ws <= self.bb_min.x
                            || x_ws >= self.bb_max.x
                            || y_ws <= self.bb_min.y
                            || y_ws >= self.bb_max.y
                            || z_ws <= self.bb_min.z
                            || z_ws >= self.bb_max.z
                        {
                            occupied = 0;
                        } else {
                            occupied = if self.voxels[self.to_1d(self.snap(glm::vec3(x_ws, y_ws, z_ws), Round::Floor))] { 1 } else { 0 } as i8;
                            // println!("{}", occupied);
                            area += occupied as u64;
                        }

                        save_img.put_pixel((x + self.size / 2) as u32, (y + self.size / 2) as u32, image::Rgb([occupied as u8 * 255, occupied as u8 * 255, occupied as u8 * 255]));
                        img.push(occupied);
                    }
                }

                if area > max_area {
                    max_area = area;
                    max_img = img;
                }

                let name = String::from("slice_") + &i.to_string() + "_" + &area.to_string() + ".png";
                save_img.save_with_format(&name, image::ImageFormat::Png).unwrap();
                // Find edge loops

                i += 1;
            }

            let svg_path = bits_to_paths(self.size as usize, self.size as usize, &max_img, true);
            println!("{}", svg_path);

            // Triangulate
            // Create a simple path.
            let svg_builder = lyon::svg::path::Path::builder().with_svg();
            let path = build_path(svg_builder, &svg_path).unwrap();

            // Will contain the result of the tessellation.
            let mut geometry: VertexBuffers<glm::Vec2, i32> = VertexBuffers::new();
            let mut tessellator = FillTessellator::new();
            {
                // Compute the tessellation.
                tessellator
                    .tessellate_path(
                        &path,
                        &FillOptions::default(),
                        &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: FillAttributes| glm::vec2(pos.x, pos.y)),
                    )
                    .unwrap();
            }

            // Kill degenerate triangles
            for i in 0..geometry.indices.len() / 3 {
                let p1 = geometry.vertices[geometry.indices[i * 3] as usize];
                let p2 = geometry.vertices[geometry.indices[i * 3 + 1] as usize];
                let p3 = geometry.vertices[geometry.indices[i * 3 + 2] as usize];

                let a = glm::distance(&p1, &p2);
                let b = glm::distance(&p2, &p3);
                let c = glm::distance(&p3, &p1);

                let s = (a + b + c) / 2.0;

                let area = s * (s - a) * (s - b) * (s - c);

                if area <= 2.0 {
                    geometry.indices[i * 3] = -1;
                    geometry.indices[i * 3 + 1] = -1;
                    geometry.indices[i * 3 + 2] = -1;
                }
            }

/*            
            for i in 0..geometry.indices.len() {
                let index = geometry.indices[i];
                if index == -1 {
                    continue;
                }

                if i % 3 == 0 {
                    print!("<polygon points=\"");
                }

                print!("{},{} ", geometry.vertices[index as usize].x, geometry.vertices[index as usize].y);

                if i % 3 == 2 {
                    println!("\"/>");
                }
            }
            */
            

            // Remove degenerate triangles, deindex, and sort them by area

            //

            planes_triangles.push(triangles);
        }

        // Decimate triangles based on limit
        // TODO: based on occlusion

        planes_triangles
    }
}
