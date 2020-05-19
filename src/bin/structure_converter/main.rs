use glm::{quat, quat_to_mat4, translation, vec3};
use nalgebra_glm as glm;
use ron::ser::{to_string, PrettyConfig};
use std::io::*;
use wgpu_experiments::rpdb;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let in_file_path: &str = &args[1];
    let out_file_path: &str = &args[2];

    let structure_file = std::fs::File::open(in_file_path).expect("Could not open structure file.");
    let structure_reader = std::io::BufReader::new(&structure_file);
    let mut molecule_names = Vec::new();
    let mut molecule_model_matrices = Vec::new();
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

                molecule_names.push(molecule_name.to_string());

                let translation = translation(&(3333.33 * molecule_position));
                let rotation = quat_to_mat4(&molecule_quaternion);
                let model_matrix = translation * rotation;

                molecule_model_matrices.push(model_matrix);
            }
        }
    }

    let structure = rpdb::Structure {
        names: molecule_names,
        model_matrices: molecule_model_matrices,
    };

    // Convert the molecule to a new RON format
    let pretty = PrettyConfig {
        depth_limit: 8,
        new_line: "\n".to_string(),
        indentor: " ".to_string(),
        separate_tuple_members: true,
        enumerate_arrays: false,
    };
    let s = to_string(&structure).expect("Serialization failed");
    std::fs::write(out_file_path, s).expect("Unable to write file");
}
