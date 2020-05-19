use glm::{Mat4, Vec3, Vec4};
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}
#[derive(Serialize, Deserialize)]
pub struct MoleculeLod {
    max_radius: f32,
    atoms: Vec<Vec4>,
}

impl MoleculeLod {
    pub fn new(atoms: Vec<Vec4>) -> Self {
        let mut max_radius = atoms[0].w;

        for atom in &atoms {
            if atom.w > max_radius {
                max_radius = atom.w;
            }
        }

        Self { max_radius, atoms }
    }

    pub fn max_radius(&self) -> f32 {
        self.max_radius
    }

    pub fn atoms(&self) -> &[Vec4] {
        &self.atoms
    }
}
#[derive(Serialize, Deserialize)]
pub struct Molecule {
    pub name: String,
    pub bounding_box: BoundingBox,
    pub lods: Vec<MoleculeLod>,
}

impl Molecule {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }

    pub fn lods(&self) -> &[MoleculeLod] {
        &self.lods
    }
}
#[derive(Serialize, Deserialize)]
pub struct Structure {
    pub names: Vec<String>,
    pub model_matrices: Vec<Mat4>,
}
