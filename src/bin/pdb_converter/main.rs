use ron::ser::{to_string_pretty, PrettyConfig};
use wgpu_experiments::kmeans;
use wgpu_experiments::pdb_loader;
use wgpu_experiments::rpdb;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let in_file_path: &str = &args[1];

    let name = std::path::Path::new(in_file_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .trim_end_matches(".pdb")
        .to_ascii_uppercase();
    let in_atoms = pdb_loader::load_molecule(std::path::Path::new(in_file_path));
    let bounding_box = pdb_loader::bounding_box(&in_atoms);

    let mut lods = vec![rpdb::MoleculeLod::new(in_atoms)];
    loop {
        let last_lod_index = lods.len() - 1;
        let last_lod = &lods[last_lod_index];

        let new_centroids_num = last_lod.atoms().len() / 4;

        if new_centroids_num <= 1 {
            break;
        }

        let new_atoms = kmeans::kmeans_spheres(last_lod.atoms(), new_centroids_num);
        lods.push(rpdb::MoleculeLod::new(new_atoms));
    }

    let molecule = rpdb::Molecule {
        name: name.to_string(),
        bounding_box,
        lods,
    };

    // Convert the molecule to a new RON format
    let pretty = PrettyConfig {
        depth_limit: 8,
        new_line: "\n".to_string(),
        indentor: " ".to_string(),
        separate_tuple_members: true,
        enumerate_arrays: false,
    };
    let s = to_string_pretty(&molecule, pretty).expect("Serialization failed");

    let out_file_path = if args.len() >= 3 {
        args[2].clone()
    } else {
        std::path::Path::new(in_file_path).parent().unwrap().to_str().unwrap().to_owned() + "\\" + &name + ".ron"
    };
    println!("Writing to: {}", out_file_path);

    std::fs::write(out_file_path, s).expect("Unable to write file");
}
