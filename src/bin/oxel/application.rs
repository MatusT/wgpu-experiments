use nalgebra_glm as glm;
use safe_transmute::*;
use wgpu;
use wgpu_experiments::camera::*;
use wgpu_experiments::pipelines::{boxes::*, mesh::MeshPipeline};
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
        let positions = device
            .create_buffer_mapped::<f32>(positions.len(), wgpu::BufferUsage::STORAGE_READ)
            .fill_from_slice(&positions);
        let sizes = device
            .create_buffer_mapped::<f32>(sizes.len(), wgpu::BufferUsage::STORAGE_READ)
            .fill_from_slice(&sizes);
        let colors = device
            .create_buffer_mapped::<f32>(colors.len(), wgpu::BufferUsage::STORAGE_READ)
            .fill_from_slice(&colors);

        BoxPipelineInput {
            count,
            positions,
            sizes,
            colors,
        }
    }
}

pub struct Application {
    width: u32,
    height: u32,

    pub options: ApplicationOptions,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,

    pub multisampled_framebuffer: wgpu::TextureView,

    pub camera: RotationCamera,
    pub camera_buffer: wgpu::Buffer,

    // Spheres rendering
    pub mesh_pipeline: MeshPipeline,
    pub mesh_bind_group: wgpu::BindGroup,
    pub mesh: Mesh,

    pub box_pipeline_line: BoxPipeline,
    pub box_pipeline_filled: BoxPipeline,

    // Enclosing bounding box
    pub bounding_box: BoxPipelineInput,
    pub bounding_box_bind_group: wgpu::BindGroup,

    // Grid
    pub grid: BoxPipelineInput,
    pub grid_bind_group: wgpu::BindGroup,

    // Occluders,
    pub occluders: BoxPipelineInput,
    pub occluders_bind_group: wgpu::BindGroup,
}

impl Application {
    pub fn new(width: u32, height: u32) -> Self {
        use wgpu::{Binding, BindingResource};
        let options = ApplicationOptions {
            render_molecules: true,
            render_grid: false,
            render_aabbs: false,
        };

        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            backends: wgpu::BackendBit::PRIMARY,
        })
        .unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });

        let aspect = width as f32 / height as f32;
        let camera = RotationCamera::new(aspect, 0.785398163, 0.1);
        let camera_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[camera.ubo()]);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d { width, height, depth: 1 },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let depth_texture_view = depth_texture.create_default_view();

        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: wgpu::Extent3d { width, height, depth: 1 },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        };

        let multisampled_framebuffer = device.create_texture(multisampled_frame_descriptor).create_default_view();

        //
        let atoms = vec![glm::vec4(1.0, 1.0, 1.0, 1.0), glm::vec4(-1.0, -1.0, -1.0, 1.0)];
        let mut atom_positions = Vec::new();
        for atom in atoms.iter() {
            atom_positions.push(atom.x);
        }
        let voxel_grid = VoxelGrid::new(atoms);

        //
        let mesh = Mesh::from_obj(&device, "icosahedron_3.obj", 1.0);
        let mesh_positions = device
            .create_buffer_mapped::<f32>(4, wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[0.0, 0.0, 0.0, 0.0]);
        let mesh_pipeline = MeshPipeline::new(&device);
        let mesh_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mesh_pipeline.bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &camera_buffer,
                        range: 0..192,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &mesh_positions,
                        range: 0..(4 * std::mem::size_of::<f32>()) as u64,
                    },
                },
            ],
        });

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
            layout: &box_pipeline_line.bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &camera_buffer,
                        range: 0..192,
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

        let grid_to_position = |input: glm::TVec3<u32>| -> glm::Vec3 {
            let input_f32 = glm::vec3(input.x as f32, input.y as f32, input.z as f32);
            let voxel_center = glm::vec3(
                input_f32.x * voxel_grid.voxel_size.x + voxel_grid.voxel_size.x / 2.0,
                input_f32.y * voxel_grid.voxel_size.y + voxel_grid.voxel_size.y / 2.0,
                input_f32.z * voxel_grid.voxel_size.z + voxel_grid.voxel_size.z / 2.0,
            );

            voxel_center + voxel_grid.bb_min
        };

        let grid_3d_to_1d = |input: glm::TVec3<u32>| -> usize {
            let width = voxel_grid.size as usize;
            let height = voxel_grid.size as usize;
            let x = input.x as usize;
            let y = input.y as usize;
            let z = input.z as usize;

            (width * height * z) + (width * y) + x
        };

        let grid = {
            let mut positions: Vec<f32> = Vec::new();
            let mut sizes: Vec<f32> = Vec::new();
            let mut colors: Vec<f32> = Vec::new();

            for x in 0..voxel_grid.size {
                for y in 0..voxel_grid.size {
                    for z in 0..voxel_grid.size {
                        let index = glm::vec3(x, y, z);

                        if voxel_grid.voxels[grid_3d_to_1d(index)].filled {
                            let position = grid_to_position(index);
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

        let grid_buffer_size = (grid.count as usize * 4usize * std::mem::size_of::<f32>()) as u64;
        let grid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &box_pipeline_line.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &camera_buffer,
                        range: 0..192,
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

        let occluders = {
            let mut positions: Vec<f32> = Vec::new();
            let mut sizes: Vec<f32> = Vec::new();
            let mut colors: Vec<f32> = Vec::new();

            for (bb_min, bb_max) in voxel_grid.occluders.iter() {
                let bb_max: glm::Vec3 = grid_to_position(*bb_max);
                let bb_min: glm::Vec3 = grid_to_position(*bb_min);

                let position = (bb_max + bb_min) * 0.5;
                let size = (bb_max - bb_min).abs();
                let color = glm::vec3(0.0, 1.0, 0.0);

                positions.extend_from_slice(&[position.x, position.y, position.z, 1.0]);
                sizes.extend_from_slice(&[size.x, size.y, size.z, 1.0]);
                colors.extend_from_slice(&[color.x, color.y, color.z, 1.0]);
            }

            BoxPipelineInput::new(&device, &positions, &sizes, &colors)
        };

        let occluders_buffer_size = (occluders.count as usize * 4usize * std::mem::size_of::<f32>()) as u64;
        let occluders_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &box_pipeline_filled.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &camera_buffer,
                        range: 0..192,
                    },
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer {
                        buffer: &occluders.positions,
                        range: 0..occluders_buffer_size,
                    },
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Buffer {
                        buffer: &occluders.sizes,
                        range: 0..occluders_buffer_size,
                    },
                },
                Binding {
                    binding: 3,
                    resource: BindingResource::Buffer {
                        buffer: &occluders.colors,
                        range: 0..occluders_buffer_size,
                    },
                },
            ],
        });

        Self {
            width,
            height,
            options,

            device,
            queue,
            depth_texture,
            depth_texture_view,

            multisampled_framebuffer,

            camera,
            camera_buffer,

            mesh_pipeline,
            mesh_bind_group,
            mesh,

            box_pipeline_line,
            box_pipeline_filled,

            bounding_box,
            bounding_box_bind_group,

            grid,
            grid_bind_group,

            occluders,
            occluders_bind_group,
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
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        {
            let size = std::mem::size_of::<CameraUbo>();
            let camera_buffer = self.device.create_buffer_mapped(size, wgpu::BufferUsage::COPY_SRC);
            camera_buffer.data.copy_from_slice(transmute_to_bytes(&[self.camera.ubo()]));
            let camera_buffer = camera_buffer.finish();

            encoder.copy_buffer_to_buffer(&camera_buffer, 0, &self.camera_buffer, 0, size as wgpu::BufferAddress);
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.multisampled_framebuffer,
                    resolve_target: Some(frame),
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

            rpass.set_bind_group(0, &self.grid_bind_group, &[]);
            rpass.draw(0..24, 0..self.grid.count as u32);

            rpass.set_pipeline(&self.box_pipeline_filled.pipeline);
            rpass.set_bind_group(0, &self.occluders_bind_group, &[]);
            rpass.draw(0..36, 0..self.occluders.count as u32);
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
