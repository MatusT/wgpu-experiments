/*
 * Contour tracing library (Rust)
 * https://github.com/STPR/contour_tracing
 *
 * Copyright (c) 2020, STPR - https://github.com/STPR
 *
 * SPDX-License-Identifier: EUPL-1.2
 */

//! A 2D library to trace contours.
//!
//! # Features
//! Core features:
//! - Trace contours using the Theo Pavlidis' algorithm (connectivity: 4-connected)
//! - Trace **outlines** in **clockwise direction**
//! - Trace **holes** in **counterclockwise direction**
//! - Input format: a 2D array of bits
//! - Output format: a string of SVG Path commands
//!
//! Manual parameters:
//! - User can specify to close or not the paths (with the SVG Path **Z** command)
//!
//! # Examples
//! For examples, have a look at the **bits_to_paths** function below.

use lyon::math::Point;
use lyon::svg::path_utils::*;
use lyon::tessellation::*;
use nalgebra_glm as glm;

static T: [(i8, i8); 8] = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)];
static O_VERTEX: [(i8, i8); 7] = [(-1, 0), (0, 0), (-1, -1), (0, 0), (0, -1), (0, 0), (0, 0)]; // Vertex coordinates for the outlines (bottom left) according to the orientation
static H_VERTEX: [(i8, i8); 7] = [(0, 0), (0, 0), (-1, 0), (0, 0), (-1, -1), (0, 0), (0, -1)]; // Vertex coordinates for the holes (bottom right) according to the orientation
static O_VALUE: [i8; 7] = [1, 0, 2, 0, 4, 0, 8]; // Value to add into the array of contours for the outlines
static H_VALUE: [i8; 7] = [-4, 0, -8, 0, -1, 0, -2]; // Value to add into the array of contours for the holes

pub fn bits_to_paths(bits: Vec<Vec<i8>>, closepaths: bool) -> String {
    let rows: usize = bits.len();
    let cols: usize = bits[0].len();

    let mut contours = vec![vec![0i8; cols + 2]; rows + 2]; // The array of contours needs a border of 1 bit
    for y in 0..=rows - 1 as usize {
        for x in 0..=cols - 1 as usize {
            contours[y + 1][x + 1] = bits[y][x];
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

fn main() {
    let bits = vec![
        vec![0, 0, 0, 1, 0, 0],
        vec![0, 0, 1, 1, 1, 0],
        vec![0, 1, 1, 0, 1, 1],
        vec![1, 1, 1, 1, 1, 0],
        // vec![0, 0, 0, 0, 0, 0],
        // vec![0, 0, 1, 0, 0, 0],
        // vec![0, 0, 0, 0, 0, 0],
        // vec![0, 0, 0, 0, 0, 0],
    ];

    let svg_path = bits_to_paths(bits, true);
    println!("{}", svg_path);

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
    println!("{:?}", geometry.indices);
    for i in 0..geometry.indices.len() / 3 {
        let p1 = geometry.vertices[geometry.indices[i * 3] as usize];
        let p2 = geometry.vertices[geometry.indices[i * 3 + 1] as usize];
        let p3 = geometry.vertices[geometry.indices[i * 3 + 2] as usize];

        let a = glm::distance(&p1, &p2);
        let b = glm::distance(&p2, &p3);
        let c = glm::distance(&p3, &p1);

        let s = (a + b + c) / 2.0;

        let area = s * (s - a) * (s - b) * (s - c);

        if area == 0.0 {
            geometry.indices[i * 3] = -1;
            geometry.indices[i * 3 + 1] = -1;
            geometry.indices[i * 3 + 2] = -1;
        }
    }

    println!("{:?}", geometry.indices);

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
}
