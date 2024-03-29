use bytemuck::*;
use glm::vec3_to_vec4;
use glm::Mat4;
use lib3dmol::structures::{atom::AtomType, GetAtom};
use nalgebra_glm as glm;
use std::collections::HashMap;
use wgpu;
use wgpu_experiments::camera::*;
use wgpu_experiments::pdb_loader;
use wgpu_experiments::pipelines::{boxes::*, mesh::MeshPipeline, sphere_billboards::SphereBillboardPipeline, triangles::TrianglesPipeline};
use wgpu_experiments::rpdb;
use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton, Mesh};

use crate::grid::*;

pub struct ApplicationOptions {
    pub render_molecules: bool,
    pub render_grid: bool,
    pub render_aabbs: bool,
}

pub struct BoxPipelineInput {
    pub count: u32,
    pub positions: wgpu::Buffer,
    pub sizes: wgpu::Buffer,
    pub colors: wgpu::Buffer,
}

impl BoxPipelineInput {
    pub fn new(device: &wgpu::Device, positions: &[f32], sizes: &[f32], colors: &[f32]) -> Self {
        let count = positions.len() as u32 / 4u32;
        assert_eq!(positions.len(), sizes.len());
        assert_eq!(sizes.len(), colors.len());
        let positions =
            device.create_buffer_with_data(cast_slice(positions), wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST);
        let sizes = device.create_buffer_with_data(cast_slice(sizes), wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST);
        let colors = device.create_buffer_with_data(cast_slice(colors), wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST);

        BoxPipelineInput {
            count,
            positions,
            sizes,
            colors,
        }
    }
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
    pub billboards_pipeline: SphereBillboardPipeline,
    pub billboards_bind_group: wgpu::BindGroup,

    atoms_buffer: wgpu::Buffer,
    atoms_buffer_len: u32,

    voxel_grid: VoxelGrid,

    pub box_pipeline_line: BoxPipeline,
    pub box_pipeline_filled: BoxPipeline,

    // Enclosing bounding box
    pub bounding_box: BoxPipelineInput,
    pub bounding_box_bind_group: wgpu::BindGroup,

    // Grid
    pub grid: BoxPipelineInput,
    pub grid_bind_group: wgpu::BindGroup,

    // Planar Occluders
    pub planar_occluders_pipeline: TrianglesPipeline,
    pub planar_occluders_bind_group: wgpu::BindGroup,
    pub planar_occluders: wgpu::Buffer,
    pub planar_occluders_len: usize,
}

impl Application {
    pub async fn new(width: u32, height: u32, surface: &wgpu::Surface) -> Self {
        use wgpu::{Binding, BindingResource};
        let options = ApplicationOptions {
            render_molecules: true,
            render_grid: false,
            render_aabbs: false,
        };

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        println!("{}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await;

        let aspect = width as f32 / height as f32;
        let mut camera = RotationCamera::new(aspect, 0.785398163, 0.1);
        let camera_buffer = device.create_buffer_with_data(
            cast_slice(&[camera.ubo()]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth: 1 },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let depth_texture_view = depth_texture.create_default_view();

        // Open structure file
        let args: Vec<String> = std::env::args().collect();
        let molecule_file_path: &str = &args[1];
        let molecule_ron = std::fs::read_to_string(molecule_file_path).unwrap();
        let molecule: rpdb::Molecule = ron::de::from_str(&molecule_ron).unwrap();

        let mut atoms = Vec::new();
        for atom in molecule.lods[0].atoms() {
            atoms.extend_from_slice(&[atom.x, atom.y, atom.z, atom.w]);
        }

        // camera.set_distance(distance(&molecule.bounding_box.min, &molecule.bounding_box.max));
        // camera.set_speed(distance(&molecule.bounding_box.min, &molecule.bounding_box.max));

        let atoms_buffer_len = atoms.len() / 4;
        let atoms_buffer =
            device.create_buffer_with_data(cast_slice(&atoms), wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST);

        //
        let billboards_pipeline = SphereBillboardPipeline::new(&device);
        let billboards_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &billboards_pipeline.bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &camera_buffer,
                        range: 0..std::mem::size_of::<CameraUbo>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &atoms_buffer,
                        range: 0..(4 * atoms_buffer_len as usize * std::mem::size_of::<f32>()) as u64,
                    },
                },
            ],
        });

        println!("Loading done.");

        let mut voxel_grid_atoms: Vec<glm::Vec4> = molecule.lods[0].atoms().to_vec();
        let mut voxel_grid = VoxelGrid::new(&mut voxel_grid_atoms);

        let box_pipeline_line = BoxPipeline::new(&device, BoxRendering::Line);
        let box_pipeline_filled = BoxPipeline::new(&device, BoxRendering::Filled);

        let bounding_box_scale = voxel_grid.bb_diff.abs();
        let bounding_box = BoxPipelineInput::new(
            &device,
            &[0.0, 0.0, 0.0, 1.0],
            &[bounding_box_scale.x, bounding_box_scale.y, bounding_box_scale.z, 1.0],
            &[1.0, 0.0, 0.0, 1.0],
        );
        let bounding_box_buffer_size = (bounding_box.count as usize * 4usize * std::mem::size_of::<f32>()) as u64;
        let bounding_box_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &box_pipeline_line.bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &camera_buffer,
                        range: 0..std::mem::size_of::<CameraUbo>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &bounding_box.positions,
                        range: 0..bounding_box_buffer_size,
                    },
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &bounding_box.sizes,
                        range: 0..bounding_box_buffer_size,
                    },
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &bounding_box.colors,
                        range: 0..bounding_box_buffer_size,
                    },
                },
            ],
        });

        println!("Box pipeline done.");

        let grid = {
            let mut positions: Vec<f32> = Vec::new();
            let mut sizes: Vec<f32> = Vec::new();
            let mut colors: Vec<f32> = Vec::new();

            for x in 0..voxel_grid.size {
                for y in 0..voxel_grid.size {
                    for z in 0..voxel_grid.size {
                        let index = glm::vec3(x as i32, y as i32, z as i32);

                        if voxel_grid.voxels[voxel_grid.to_1d(index)] {
                            let position = voxel_grid.to_ws(index);
                            let size = voxel_grid.voxel_size;
                            let color = glm::vec3(0.0, 0.0, 1.0);

                            positions.extend_from_slice(&[position.x, position.y, position.z, 1.0]);
                            sizes.extend_from_slice(&[size.x, size.y, size.z, 1.0]);
                            colors.extend_from_slice(&[color.x, color.y, color.z, 1.0]);
                        }
                    }
                }
            }

            BoxPipelineInput::new(&device, &positions, &sizes, &colors)
        };

        println!("Grid done.");

        let grid_buffer_size = (grid.count as usize * 4usize * std::mem::size_of::<f32>()) as u64;
        let grid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &box_pipeline_filled.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &camera_buffer,
                        range: 0..std::mem::size_of::<CameraUbo>() as u64,
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer {
                        buffer: &grid.positions,
                        range: 0..grid_buffer_size,
                    },
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Buffer {
                        buffer: &grid.sizes,
                        range: 0..grid_buffer_size,
                    },
                },
                Binding {
                    binding: 3,
                    resource: BindingResource::Buffer {
                        buffer: &grid.colors,
                        range: 0..grid_buffer_size,
                    },
                },
            ],
        });

        println!("Grid buffers done.");

        let planar_occluders_pipeline = TrianglesPipeline::new(&device);
        let planar_occluders_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &planar_occluders_pipeline.bind_group_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &camera_buffer,
                    range: 0..192,
                },
            }],
        });
        let planar_occluders_len;
        let planar_occluders = {
            let planar_occluders = voxel_grid.get_planar_occluders(100000);
            println!("Ocluders: {}", planar_occluders.len() / 3);
            planar_occluders_len = planar_occluders.len();
            let mut res = Vec::new();
            for occluder in planar_occluders {
                res.push(occluder.x);
                res.push(occluder.y);
                res.push(occluder.z);
                res.push(occluder.w);
            }

            res
        };
        let planar_occluders = device.create_buffer_with_data(cast_slice(&planar_occluders), wgpu::BufferUsage::VERTEX);

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
            billboards_bind_group,

            atoms_buffer,
            atoms_buffer_len: atoms_buffer_len as u32,

            voxel_grid,

            box_pipeline_line,
            box_pipeline_filled,

            bounding_box,
            bounding_box_bind_group,

            grid,
            grid_bind_group,

            planar_occluders_pipeline,
            planar_occluders_bind_group,
            planar_occluders,
            planar_occluders_len,
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
        }

        {
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

            rpass.set_pipeline(&self.box_pipeline_line.pipeline);
            rpass.set_bind_group(0, &self.bounding_box_bind_group, &[]);
            rpass.draw(0..24, 0..1 as u32);

            if self.options.render_molecules {
                rpass.set_pipeline(&self.billboards_pipeline.pipeline);
                rpass.set_bind_group(0, &self.billboards_bind_group, &[]);
                rpass.draw(0..self.atoms_buffer_len * 3, 0..1);
            }

            if self.options.render_grid {
                rpass.set_pipeline(&self.box_pipeline_filled.pipeline);
                rpass.set_bind_group(0, &self.grid_bind_group, &[]);
                rpass.draw(0..36, 0..self.grid.count as u32);
            }
        }

        if self.options.render_aabbs {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::WHITE,
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.planar_occluders_pipeline.pipeline);
            rpass.set_bind_group(0, &self.planar_occluders_bind_group, &[]);
            rpass.set_vertex_buffer(0, &self.planar_occluders, 0, 0);
            rpass.draw(0..self.planar_occluders_len as u32, 0..1);
        }

        self.queue.submit(&[encoder.finish()]);
    }

    fn device(&self) -> &wgpu::Device {
        &self.device
    }
    fn device_mut(&mut self) -> &mut wgpu::Device {
        &mut self.device
    }
}
