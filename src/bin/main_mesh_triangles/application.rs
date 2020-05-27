use crate::small_molecules_pipeline::*;

use bytemuck::*;
use nalgebra_glm::{scaling, vec2, vec4, zero, Mat4};
use std::collections::HashMap;
use wgpu;
use wgpu_experiments::camera::*;
use wgpu_experiments::pipelines::depth_conversion::DepthConversionPipeline;
use wgpu_experiments::rpdb;
use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton};

pub struct ApplicationOptions {
    pub render_depth_prepass: bool,
    pub render_aabbs: bool,
    pub render_output: bool,
}

pub struct MoleculePointer {
    pub bounding_box: rpdb::BoundingBox,
    pub lods_radii: Vec<f32>,
    pub lods_vertices: Vec<std::ops::Range<u32>>,
}

pub struct Application {
    width: u32,
    height: u32,

    pub options: ApplicationOptions,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,
    pub atomic_depth_ssbo: wgpu::Buffer,
    pub atomic_depth_texture: wgpu::Texture,
    pub atomic_depth_texture_view: wgpu::TextureView,

    pub camera: RotationCamera,
    pub camera_buffer: wgpu::Buffer,

    globals: wgpu::Buffer,

    // Spheres rendering
    pipeline: SmallMoleculesPipeline,
    pipeline_depth: SmallMoleculesPipeline,
    depth_conversion_pipeline: DepthConversionPipeline,

    molecule_name_id: HashMap<String, usize>,
    molecules_pointers: Vec<MoleculePointer>,
    molecules_ubos: Vec<Option<wgpu::Buffer>>,
    atoms_buffer: wgpu::Buffer,

    structure_model_matrices: Vec<Vec<Mat4>>,
    structure_model_matrices_buffer: Vec<wgpu::Buffer>,

    depth_only: bool,
}

impl Application {
    pub async fn new(width: u32, height: u32, instance: &wgpu::Instance, surface: &wgpu::Surface) -> Self {
        let options = ApplicationOptions {
            render_depth_prepass: true,
            render_aabbs: false,
            render_output: false,
        };

        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: Some(&surface),
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: false,
                        mesh_shaders: true,
                    },
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let aspect = width as f32 / height as f32;
        let mut camera = RotationCamera::new(aspect, 0.785398163, 0.1);
        let camera_buffer = device.create_buffer_with_data(
            cast_slice(&[camera.ubo()]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let globals = MoleculesGlobals {
            resolution: vec2(width as f32, height as f32),
        };
        let globals = device.create_buffer_with_data(cast_slice(&[globals]), wgpu::BufferUsage::UNIFORM);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let depth_texture_view = depth_texture.create_default_view();

        let atomic_depth_ssbo = device.create_buffer_with_data(
            cast_slice(&vec![4294967295u32; (width * height) as usize]),
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_SRC,
        );
        let atomic_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsage::STORAGE,
        });
        let atomic_depth_texture_view = atomic_depth_texture.create_default_view();

        // Open structure file
        let args: Vec<String> = std::env::args().collect();
        let structure_file_path: &str = &args[1];
        let structure_folder = std::path::Path::new(structure_file_path);
        let structure_file = std::fs::read_to_string(structure_file_path).expect("Could not open structure file.");
        let structure: rpdb::Structure = ron::de::from_str(&structure_file).expect("Could not deserialize structure file.");

        // Load Molecules
        let mut molecule_name_id: HashMap<String, usize> = HashMap::new();
        let mut molecules = Vec::new();
        let mut molecules_pointers = Vec::new();
        let mut molecules_ubos = Vec::new();
        let mut structure_model_matrices = Vec::new();

        let mut atoms = Vec::new();
        let mut atoms_sum = 0u32;
        let mut molecules_num = 0;
        for (name, matrix) in structure.names.iter().zip(structure.model_matrices.iter()) {
            let molecule_loaded = molecule_name_id.contains_key(name);
            if !molecule_loaded {
                // Load a molecule
                let molecule_file_str = std::fs::read_to_string(structure_folder.with_file_name(name.to_string() + ".ron"))
                    .expect("Could not open structure file.");
                let molecule: rpdb::Molecule = ron::de::from_str(&molecule_file_str).expect("Could not parse molecule.");

                let mut lods_vertices: Vec<std::ops::Range<u32>> = Vec::new();
                let mut lods_radii = Vec::new();
                for lod in molecule.lods() {
                    lods_radii.push(lod.max_radius());
                    let mut new_vertices = 0;
                    for atom in lod.atoms() {
                        atoms.extend_from_slice(&[atom.x, atom.y, atom.z, atom.w]);
                        atoms.extend_from_slice(&[atom.x, atom.y, atom.z, atom.w]);
                        atoms.extend_from_slice(&[atom.x, atom.y, atom.z, atom.w]);
                        new_vertices += 3;
                    }
                    lods_vertices.push(atoms_sum..atoms_sum + new_vertices);
                    atoms_sum += new_vertices;
                }

                molecules_pointers.push(MoleculePointer {
                    bounding_box: molecule.bounding_box,
                    lods_radii,
                    lods_vertices,
                });

                if molecule.lods()[0].atoms().len() == 52 {
                    let mut positions = [vec4(0.0, 0.0, 0.0, 0.0); 64];
                    for (i, position) in molecule.lods()[0].atoms().iter().enumerate() {
                        positions[i] = *position;
                    }
                    let scale = molecule.bounding_box.max - molecule.bounding_box.min;
                    let molecule_ubo = MoleculeUbo {
                        positions,
                        aabb_scale: vec4(scale.x, scale.y, scale.z, 1.0),
                        count: molecule.lods()[0].atoms().len() as u32,
                    };
                    let molecule_buffer = device.create_buffer_with_data(cast_slice(&[molecule_ubo]), wgpu::BufferUsage::UNIFORM);
                    molecules_ubos.push(Some(molecule_buffer));
                } else {
                    molecules_ubos.push(None);
                }

                molecules.push(molecule);
                molecule_name_id.insert(name.clone(), molecules_num);
                molecules_num += 1;
                structure_model_matrices.push(Vec::new());
            }

            structure_model_matrices[molecule_name_id[name]].push(*matrix);
        }

        let atoms_buffer = device.create_buffer_with_data(cast_slice(&atoms), wgpu::BufferUsage::VERTEX);

        let pipeline = SmallMoleculesPipeline::new(&device, false);
        let pipeline_depth = SmallMoleculesPipeline::new(&device, true);
        let depth_conversion_pipeline = DepthConversionPipeline::new(&device);

        let mut structure_model_matrices_buffer = Vec::new();
        for (i, structure_molecule) in structure_model_matrices.iter().enumerate() {
            let mut matrices = Vec::new();
            for m in structure_molecule {
                matrices.extend_from_slice(m.as_slice());
            }
            structure_model_matrices_buffer
                .push(device.create_buffer_with_data(cast_slice(&matrices), wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST));
        }

        Self {
            width,
            height,
            options,

            device,
            queue,

            depth_texture,
            depth_texture_view,

            atomic_depth_ssbo,
            atomic_depth_texture,
            atomic_depth_texture_view,

            camera,
            camera_buffer,

            globals,

            pipeline,
            pipeline_depth,
            depth_conversion_pipeline,

            molecule_name_id,
            molecules_pointers,
            molecules_ubos,
            atoms_buffer,

            structure_model_matrices,
            structure_model_matrices_buffer,

            depth_only: false,
        }
    }

    pub fn options(&self) -> &ApplicationOptions {
        &self.options
    }

    pub fn queue_mut(&mut self) -> &mut wgpu::Queue {
        &mut self.queue
    }
}

impl ApplicationSkeleton for Application {
    fn resize(&mut self, _: u32, _: u32) {
        //
    }

    fn update(&mut self, event: ApplicationEvent) {
        match event {
            ApplicationEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    if input.state == winit::event::ElementState::Pressed && keycode == winit::event::VirtualKeyCode::D {
                        self.depth_only = !self.depth_only;
                    }
                }
            }
            _ => {}
        };

        self.camera.update(event);
    }

    fn render(&mut self, frame: &wgpu::TextureView) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let size = std::mem::size_of::<CameraUbo>();
            let camera_buffer = self
                .device
                .create_buffer_with_data(cast_slice(&[self.camera.ubo()]), wgpu::BufferUsage::COPY_SRC);

            encoder.copy_buffer_to_buffer(&camera_buffer, 0, &self.camera_buffer, 0, size as wgpu::BufferAddress);
        }

        let mut bind_groups = Vec::new();
        for (molecule_index, molecule) in self.molecules_pointers.iter().enumerate() {
            if let Some(ref molecule_ubo) = self.molecules_ubos[molecule_index] {
                let bind_group = if self.depth_only {
                    self.pipeline_depth.create_bind_group(
                        &self.device,
                        &self.camera_buffer,
                        &self.globals,
                        &molecule_ubo,
                        &self.structure_model_matrices_buffer[molecule_index],
                        &self.atomic_depth_ssbo,
                    )
                } else {
                    self.pipeline.create_bind_group(
                        &self.device,
                        &self.camera_buffer,
                        &self.globals,
                        &molecule_ubo,
                        &self.structure_model_matrices_buffer[molecule_index],
                        &self.atomic_depth_ssbo,
                    )
                };
                bind_groups.push(Some(bind_group));
            } else {
                bind_groups.push(None);
            }
        }

        let depth_bind_group = self.depth_conversion_pipeline.create_bind_group(&self.device, &self.atomic_depth_ssbo, &self.atomic_depth_texture_view);

        {
            let color_attachments = if self.depth_only {
                vec![]
            } else {
                vec![wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::WHITE,
                }]
            };
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &color_attachments,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_depth: 0.0,
                    clear_stencil: 0,
                }),
            });

            if self.depth_only {
                rpass.set_mesh_pipeline(&self.pipeline_depth.pipeline);
            } else {
                rpass.set_mesh_pipeline(&self.pipeline.pipeline);
            }

            for (molecule_index, molecule) in self.molecules_pointers.iter().enumerate() {
                if let Some(ref molecule_ubo) = self.molecules_ubos[molecule_index] {
                    rpass.set_bind_group(0, bind_groups[molecule_index].as_ref().unwrap(), &[]);
                    let tasks_count = self.structure_model_matrices[molecule_index].len();
                    // let tasks_count = tasks_count - (tasks_count % 32);
                    // let tasks_count = tasks_count / 32;
                    rpass.draw_mesh_tasks(tasks_count as u32);
                }
            }
        }

        // Depth compute conversion
        {
            let mut cpass = encoder.begin_compute_pass();

            cpass.set_pipeline(&self.depth_conversion_pipeline.pipeline);
            cpass.set_bind_group(0, &depth_bind_group, &[]);
            cpass.dispatch((self.width + 15) / 16, (self.height + 15) / 16, 1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    fn device(&self) -> &wgpu::Device {
        &self.device
    }
    fn device_mut(&mut self) -> &mut wgpu::Device {
        &mut self.device
    }
}
