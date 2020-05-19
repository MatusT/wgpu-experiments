use wgpu_experiments::camera::*;
use wgpu_experiments::pipelines::{mesh::MeshPipeline, sphere_billboards::*};
use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton, Mesh};

extern crate alloc;

use bytemuck::*;
use wgpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshType {
    Billboard,
    Cube,
    Icosahedron1,
    Icosahedron2,
    Icosahedron3,
    Icosahedron4,
    Icosahedron5,
}

impl MeshType {
    pub fn all() -> [MeshType; 7] {
        [
            MeshType::Billboard,
            MeshType::Cube,
            MeshType::Icosahedron1,
            MeshType::Icosahedron2,
            MeshType::Icosahedron3,
            MeshType::Icosahedron4,
            MeshType::Icosahedron5,
        ]
    }
}

impl From<MeshType> for String {
    fn from(mesh_type: MeshType) -> String {
        match mesh_type {
            MeshType::Billboard => String::from("Billboard"),
            MeshType::Cube => String::from("Cube"),
            MeshType::Icosahedron1 => String::from("Icosahedron 1"),
            MeshType::Icosahedron2 => String::from("Icosahedron 2"),
            MeshType::Icosahedron3 => String::from("Icosahedron 3"),
            MeshType::Icosahedron4 => String::from("Icosahedron 4"),
            MeshType::Icosahedron5 => String::from("Icosahedron 5"),
        }
    }
}

impl From<MeshType> for &str {
    fn from(mesh_type: MeshType) -> &'static str {
        match mesh_type {
            MeshType::Billboard => "Billboard",
            MeshType::Cube => "Cube",
            MeshType::Icosahedron1 => "Icosahedron 1",
            MeshType::Icosahedron2 => "Icosahedron 2",
            MeshType::Icosahedron3 => "Icosahedron 3",
            MeshType::Icosahedron4 => "Icosahedron 4",
            MeshType::Icosahedron5 => "Icosahedron 5",
        }
    }
}

impl From<MeshType> for usize {
    fn from(mesh_type: MeshType) -> usize {
        match mesh_type {
            MeshType::Billboard => 0,
            MeshType::Cube => 1,
            MeshType::Icosahedron1 => 2,
            MeshType::Icosahedron2 => 3,
            MeshType::Icosahedron3 => 4,
            MeshType::Icosahedron4 => 5,
            MeshType::Icosahedron5 => 6,
        }
    }
}

pub struct ApplicationOptions {
    pub mesh: MeshType,
    pub n: i32,
}

pub struct Application {
    width: u32,
    height: u32,

    pub options: ApplicationOptions,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub pipeline: MeshPipeline,
    pub billboards_pipeline: SphereBillboardPipeline,
    pub billboards_preprocess: BillboardsPreprocessPipeline,
    pub billboards_passthrough: BillboardsPassthroughPipeline,

    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,
    pub dumb_texture_0: wgpu::TextureView,

    pub multisampled_framebuffer: wgpu::TextureView,

    pub meshes: Vec<Mesh>,

    pub camera: RotationCamera,
    pub camera_buffer: wgpu::Buffer,

    pub positions_len: usize,
    pub positions_instanced_buffer: wgpu::Buffer,
    pub positions_clip_space_buffer: wgpu::Buffer,
}

impl Application {
    pub async fn new(width: u32, height: u32, surface: &wgpu::Surface) -> Self {
        let options = ApplicationOptions {
            mesh: MeshType::Billboard,
            n: 300,
        };

        // let adapter = &wgpu::Adapter::enumerate(wgpu::BackendBit::PRIMARY)[1];
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
        println!("{:?}", adapter.get_info().backend);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await;

        let meshes = vec![
            Mesh::from_obj(&device, "cube.obj", 1.0),
            Mesh::from_obj(&device, "cube.obj", 1.0),
            Mesh::from_obj(&device, "icosahedron_1.obj", 1.0),
            Mesh::from_obj(&device, "icosahedron_2.obj", 1.0),
            Mesh::from_obj(&device, "icosahedron_3.obj", 1.0),
            Mesh::from_obj(&device, "icosahedron_4.obj", 1.0),
            Mesh::from_obj(&device, "icosahedron_5.obj", 1.0),
        ];

        let aspect = width as f32 / height as f32;
        let mut camera = RotationCamera::new(aspect, 0.785398163, 0.1);
        let camera_buffer = device.create_buffer_with_data(
            cast_slice(&[camera.ubo()]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let mut positions = Vec::new();
        for z in -options.n / 2..options.n / 2 {
            for x in -options.n / 2..options.n / 2 {
                for y in -options.n / 2..options.n / 2 {
                    positions.push(x as f32);
                    positions.push(y as f32);
                    positions.push(z as f32);
                    positions.push(1.0);
                }
            }
        }
        let positions_instanced_buffer = device.create_buffer_with_data(
            cast_slice(&positions),
            wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST,
        );
        let positions_len = positions.len();

        let positions_clip_space_buffer_size = (options.n * options.n * options.n) as usize * 3 * 4 * std::mem::size_of::<f32>();
        let positions_clip_space_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: positions_clip_space_buffer_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::VERTEX,
        });

        let billboards_pipeline = SphereBillboardPipeline::new(&device);
        let pipeline = MeshPipeline::new(&device);
        let billboards_preprocess = BillboardsPreprocessPipeline::new(&device);
        let billboards_passthrough = BillboardsPassthroughPipeline::new(&device);

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

        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth: 1 },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        };
        let multisampled_framebuffer = device.create_texture(multisampled_frame_descriptor).create_default_view();

        let dumb_texture_0 = &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth: 1 },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        };
        let dumb_texture_0 = device.create_texture(dumb_texture_0).create_default_view();

        Self {
            width,
            height,
            options,

            device,
            queue,

            pipeline,

            billboards_pipeline,
            billboards_preprocess,
            billboards_passthrough,

            depth_texture,
            depth_texture_view,
            multisampled_framebuffer,
            dumb_texture_0,

            meshes,

            camera,
            camera_buffer,

            positions_len,
            positions_instanced_buffer,
            positions_clip_space_buffer,
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
        let n = self.options.n * self.options.n * self.options.n;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let size = std::mem::size_of::<CameraUbo>();
            let camera_buffer = self
                .device
                .create_buffer_with_data(cast_slice(&[self.camera.ubo()]), wgpu::BufferUsage::COPY_SRC);

            encoder.copy_buffer_to_buffer(&camera_buffer, 0, &self.camera_buffer, 0, size as wgpu::BufferAddress);
        }

        // {
        //     let mut preprocess_pass = encoder.begin_compute_pass();
        //     preprocess_pass.set_bind_group(0, &self.billboards_preprocess_bind_group, &[]);
        //     preprocess_pass.set_pipeline(&self.billboards_preprocess.pipeline);
        //     preprocess_pass.dispatch((n / 1024) as u32, 1, 1);
        // }

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.camera_buffer,
                        range: 0..192,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.positions_instanced_buffer,
                        range: 0..(self.positions_len * std::mem::size_of::<f32>()) as u64,
                    },
                },
            ],
        });
        let billboards_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.billboards_pipeline.bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.camera_buffer,
                        range: 0..192,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.positions_instanced_buffer,
                        range: 0..(self.positions_len * std::mem::size_of::<f32>()) as u64,
                    },
                },
            ],
        });

        // let billboards_preprocess = BillboardsPreprocessPipeline::new(&device);
        // let billboards_preprocess_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: None,
        //     layout: &billboards_preprocess.bind_group_layout,
        //     bindings: &[
        //         wgpu::Binding {
        //             binding: 0,
        //             resource: wgpu::BindingResource::Buffer {
        //                 buffer: &camera_buffer,
        //                 range: 0..192,
        //             },
        //         },
        //         wgpu::Binding {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Buffer {
        //                 buffer: &positions_instanced_buffer,
        //                 range: 0..(positions.len() * std::mem::size_of::<f32>()) as u64,
        //             },
        //         },
        //         wgpu::Binding {
        //             binding: 2,
        //             resource: wgpu::BindingResource::Buffer {
        //                 buffer: &positions_clip_space_buffer,
        //                 range: 0..positions_clip_space_buffer_size as u64,
        //             },
        //         },
        //     ],
        // });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame,
                        resolve_target: None,
                        // attachment: &self.multisampled_framebuffer,
                        // resolve_target: Some(frame),
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::GREEN,
                    },
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.dumb_texture_0,
                        resolve_target: None,
                        // attachment: &self.multisampled_framebuffer,
                        // resolve_target: Some(frame),
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::GREEN,
                    },
                ],
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

            if self.options.mesh == MeshType::Billboard {
                rpass.set_pipeline(&self.billboards_pipeline.pipeline);
                rpass.set_bind_group(0, &billboards_bind_group, &[]);
                rpass.draw(0..(n * 3) as u32, 0..1);

            // rpass.set_pipeline(&self.billboards_passthrough.pipeline);
            // rpass.set_vertex_buffer(0, &self.positions_clip_space_buffer, 0, 0);
            // rpass.draw(0..(n * 3) as u32, 0..1);
            } else {
                let mesh_index: usize = self.options.mesh.into();
                let mesh = &self.meshes[mesh_index];

                rpass.set_pipeline(&self.pipeline.pipeline);
                rpass.set_bind_group(0, &bind_group, &[]);
                rpass.set_vertex_buffer(0, &mesh.vertices(), 0, 0);
                rpass.set_vertex_buffer(1, &mesh.normals(), 0, 0);
                rpass.set_index_buffer(&mesh.indices(), 0, 0);
                rpass.draw_indexed(0..mesh.indices_len(), 0, 0..n as u32);
            }
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
