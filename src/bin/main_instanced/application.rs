use bytemuck::*;
use glm::Mat4;
use nalgebra_glm as glm;
use std::collections::HashMap;
use wgpu;
use wgpu_experiments::camera::*;
use wgpu_experiments::pipelines::{
    boxes::BoxDepthPipeline,
    sphere_billboards::{SphereBillboardInstancedDepthPipeline, SphereBillboardInstancedPipeline},
};
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

pub struct StructurePointer {}

pub struct Application {
    width: u32,
    height: u32,

    pub options: ApplicationOptions,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,

    pub camera: RotationCamera,
    pub camera_buffer: wgpu::Buffer,

    // Spheres rendering
    billboards_pipeline: SphereBillboardInstancedPipeline,
    billboards_bind_groups: Vec<wgpu::BindGroup>,
    billboards_depth_pipeline: SphereBillboardInstancedDepthPipeline,
    billboards_depth_bind_groups: Vec<wgpu::BindGroup>,
    box_depth_pipeline_write: BoxDepthPipeline,
    box_depth_pipeline_read: BoxDepthPipeline,

    molecule_name_id: HashMap<String, usize>,

    molecules_pointers: Vec<MoleculePointer>,
    atoms_buffer: wgpu::Buffer,

    structure_model_matrices: Vec<Vec<Mat4>>,
    structure_model_matrices_buffer: Vec<wgpu::Buffer>,

    aabbs_len: u64,
    aabbs_matrices: wgpu::Buffer,
    aabbs_fragments: wgpu::Buffer,
}

impl Application {
    pub async fn new(width: u32, height: u32, instance: &wgpu::Instance, surface: &wgpu::Surface) -> Self {
        let options = ApplicationOptions {
            render_depth_prepass: false,
            render_aabbs: false,
            render_output: true,
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

                molecules.push(molecule);
                molecule_name_id.insert(name.clone(), molecules_num);
                molecules_num += 1;
                structure_model_matrices.push(Vec::new());
            }

            structure_model_matrices[molecule_name_id[name]].push(*matrix);
        }

        let atoms_buffer = device.create_buffer_with_data(cast_slice(&atoms), wgpu::BufferUsage::VERTEX);

        println!("Pipeline");
        let billboards_pipeline = SphereBillboardInstancedPipeline::new(&device);
        let billboards_depth_pipeline = SphereBillboardInstancedDepthPipeline::new(&device);
        let box_depth_pipeline_write = BoxDepthPipeline::new(&device, true);
        let box_depth_pipeline_read = BoxDepthPipeline::new(&device, false);

        let mut billboards_bind_groups = Vec::new();
        let mut billboards_depth_bind_groups = Vec::new();
        let mut structure_model_matrices_buffer = Vec::new();
        let mut aabbs_matrices = Vec::new();
        for (i, structure_molecule) in structure_model_matrices.iter().enumerate() {
            let mut matrices = Vec::new();
            for m in structure_molecule {
                matrices.extend_from_slice(m.as_slice());

                let aabb_matrix = m * glm::scaling(&(molecules[i].bounding_box.max - molecules[i].bounding_box.min));
                aabbs_matrices.extend_from_slice(aabb_matrix.as_slice());
            }
            structure_model_matrices_buffer
                .push(device.create_buffer_with_data(cast_slice(&matrices), wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST));

            billboards_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &billboards_pipeline.bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(camera_buffer.slice(0..0)),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(structure_model_matrices_buffer.last().unwrap().slice(0..0)),
                    },
                ],
            }));

            billboards_depth_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &billboards_depth_pipeline.bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(camera_buffer.slice(0..0)),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(structure_model_matrices_buffer.last().unwrap().slice(0..0)),
                    },
                ],
            }));
        }

        let aabbs_len = (aabbs_matrices.len() / 16) as u64;
        let aabbs_matrices = device.create_buffer_with_data(
            cast_slice(&aabbs_matrices),
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
        );
        let aabbs_fragments = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: aabbs_len * std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
        });

        Self {
            width,
            height,
            options,

            device,
            queue,
            depth_texture,
            depth_texture_view,

            camera,
            camera_buffer,

            billboards_pipeline,
            billboards_bind_groups,
            billboards_depth_pipeline,
            billboards_depth_bind_groups,
            box_depth_pipeline_write,
            box_depth_pipeline_read,

            molecule_name_id,

            molecules_pointers,
            atoms_buffer,

            structure_model_matrices,
            structure_model_matrices_buffer,

            aabbs_len,
            aabbs_matrices,
            aabbs_fragments,
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

            let tmp_buffer = self
                .device
                .create_buffer_with_data(cast_slice(&vec![0.0f32; self.aabbs_len as usize]), wgpu::BufferUsage::COPY_SRC);
            encoder.copy_buffer_to_buffer(
                &tmp_buffer,
                0,
                &self.aabbs_fragments,
                0,
                (4 * self.aabbs_len) as wgpu::BufferAddress,
            );
        }

        let box_depth_bind_group =
            self.box_depth_pipeline_read
                .create_bind_group(&self.device, &self.camera_buffer, &self.aabbs_matrices, &self.aabbs_fragments);

        // Depth pre-pass
        if self.options.render_depth_prepass {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[],
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

            rpass.set_pipeline(&self.billboards_depth_pipeline.pipeline);
            rpass.set_vertex_buffer(0, self.atoms_buffer.slice(0..0));

            for (molecule_index, molecule) in self.molecules_pointers.iter().enumerate() {
                if molecule.lods_vertices[0].end - molecule.lods_vertices[0].start == 156 {
                    rpass.set_bind_group(0, &self.billboards_depth_bind_groups[molecule_index], &[]);
                    rpass.draw(
                        molecule.lods_vertices[0].clone(),
                        0..self.structure_model_matrices[molecule_index].len() as u32,
                    );
                }
            }
        }

        // AABB removal
        if self.options.render_aabbs {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_load_op: wgpu::LoadOp::Load,
                    depth_store_op: wgpu::StoreOp::Clear,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_depth: 0.0,
                    clear_stencil: 0,
                }),
            });

            rpass.set_pipeline(&self.box_depth_pipeline_read.pipeline);
            rpass.set_bind_group(0, &box_depth_bind_group, &[]);
            rpass.draw(0..36, 0..self.aabbs_len as u32);
        }

        if self.options.render_output {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::WHITE,
                }],
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

            rpass.set_pipeline(&self.billboards_pipeline.pipeline);
            rpass.set_vertex_buffer(0, self.atoms_buffer.slice(0..0));

            for (molecule_index, molecule) in self.molecules_pointers.iter().enumerate() {
                rpass.set_bind_group(0, &self.billboards_bind_groups[molecule_index], &[]);
                // println!("{:?}",molecule.lods_vertices[0].end - molecule.lods_vertices[0].start);
                if molecule.lods_vertices[0].end - molecule.lods_vertices[0].start == 156 {
                    // println!("yes");
                    rpass.draw(
                        molecule.lods_vertices[0].clone(),
                        0..self.structure_model_matrices[molecule_index].len() as u32,
                    );
                }
            }
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
