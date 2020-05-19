use crate::rpdb;
use glm::*;
use lib3dmol::structures::{atom::AtomType, GetAtom};
use nalgebra_glm as glm;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn bounding_box(atoms: &[Vec4]) -> rpdb::BoundingBox {
    // Find bounding box of the entire structure
    let mut bb_max = vec3(std::f32::NEG_INFINITY, std::f32::NEG_INFINITY, std::f32::NEG_INFINITY);
    let mut bb_min = vec3(std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY);
    for atom in atoms.iter() {
        let atom_position = atom.xyz();
        let atom_radius = atom[3];
        bb_max = glm::max2(&bb_max, &(atom_position + vec3(atom_radius, atom_radius, atom_radius)));
        bb_min = glm::min2(&bb_min, &(atom_position - vec3(atom_radius, atom_radius, atom_radius)));
    }

    rpdb::BoundingBox { min: bb_min, max: bb_max }
}

pub fn center_atoms(mut atoms: Vec<Vec4>) -> Vec<Vec4> {
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
    for atom in atoms.iter_mut() {
        atom.x -= bb_center.x;
        atom.y -= bb_center.y;
        atom.z -= bb_center.z;
    }

    atoms
}

pub fn load_molecule(path: &Path) -> Vec<Vec4> {
    let mut atoms = Vec::new();

    let molecule_structure = lib3dmol::parser::read_pdb(path.to_str().unwrap(), "");
    for atom in molecule_structure.get_atom() {
        let radius = match atom.a_type {
            AtomType::Carbon => 1.548,
            AtomType::Hydrogen => 1.100,
            AtomType::Nitrogen => 1.400,
            AtomType::Oxygen => 1.348,
            AtomType::Phosphorus => 1.880,
            AtomType::Sulfur => 1.880,
            _ => 1.0, // 'A': 1.5
        };
        atoms.push(glm::vec4(atom.coord[0], atom.coord[1], atom.coord[2], radius));
    }

    center_atoms(atoms)
}

pub fn load_molecules(path: &Path) -> Vec<Vec4> {
    let mut atoms = Vec::new();
    let mut molecules = HashMap::new();

    let directory = path.parent().expect("File must be in a directory.");
    for entry in fs::read_dir(directory).expect("read_dir call failed") {
        let entry = entry.unwrap();
        let molecule_path = entry.path();

        let molecule_path_str = molecule_path.to_str().unwrap();
        if molecule_path_str.ends_with(".pdb") {
            let pdb_name = molecule_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .trim_end_matches(".pdb")
                .to_ascii_uppercase();
            let pdb_molecule = load_molecule(&molecule_path);

            molecules.insert(pdb_name, pdb_molecule);
        }
    }

    let structure_file = File::open(path).expect("Could not open structure file.");
    let structure_reader = BufReader::new(&structure_file);
    for line in structure_reader.lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.split(' ').collect();

            if parts.len() == 9 {
                let molecule_name = parts[0];

                let molecule_position = vec3(
                    parts[1].parse::<f32>().unwrap(),
                    parts[2].parse::<f32>().unwrap(),
                    parts[3].parse::<f32>().unwrap(),
                );
                let molecule_quaternion = quat(
                    -parts[7].parse::<f32>().unwrap(),
                    parts[4].parse::<f32>().unwrap(),
                    parts[5].parse::<f32>().unwrap(),
                    -parts[6].parse::<f32>().unwrap(),
                );

                let translation = translation(&(3333.33 * molecule_position));
                let rotation = quat_to_mat4(&molecule_quaternion);

                let molecule_atoms: Vec<Vec4> = molecules[molecule_name]
                    .clone()
                    .iter()
                    .map(|v| translation * rotation * v)
                    .collect();
                atoms.extend(molecule_atoms);
            }
        }
    }

    atoms
}
