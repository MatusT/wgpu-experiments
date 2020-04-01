use wgpu_experiments::camera::*;
use wgpu_experiments::pipelines::mesh::MeshPipeline;
use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton, Mesh};

extern crate alloc;

use obj::*;
use safe_transmute::*;
use std::fs::File;
use std::io::BufReader;
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
    pub bind_group: wgpu::BindGroup,

    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,

    pub multisampled_framebuffer: wgpu::TextureView,

    pub meshes: Vec<Mesh>,

    pub camera: RotationCamera,
    pub camera_buffer: wgpu::Buffer,

    pub positions_instanced_buffer: wgpu::Buffer,
}

impl Application {
    pub fn new(width: u32, height: u32) -> Self {
        let options = ApplicationOptions {
            mesh: MeshType::Cube,
            n: 50,
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

        let pipeline = MeshPipeline::new(&device);

        let meshes = vec![
            Mesh::from_obj(&device, "cube.obj"),
            Mesh::from_obj(&device, "cube.obj"),
            Mesh::from_obj(&device, "icosahedron_1.obj"),
            Mesh::from_obj(&device, "icosahedron_2.obj"),
            Mesh::from_obj(&device, "icosahedron_3.obj"),
            Mesh::from_obj(&device, "icosahedron_4.obj"),
            Mesh::from_obj(&device, "icosahedron_5.obj"),
        ];

        let aspect = width as f32 / height as f32;
        let camera = RotationCamera::new(aspect, 0.785398163, 0.1);
        let camera_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[camera.ubo()]);

        let mut positions = Vec::new();
        for x in -options.n / 2..options.n / 2 {
            for y in -options.n / 2..options.n / 2 {
                for z in -options.n / 2..options.n / 2 {
                    positions.push(x as f32);
                    positions.push(y as f32);
                    positions.push(z as f32);
                    positions.push(0.0);
                }
            }
        }
        let positions_instanced_buffer = device
            .create_buffer_mapped::<f32>(positions.len(), wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&positions);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.bind_group_layout,
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
                        buffer: &positions_instanced_buffer,
                        range: 0..(positions.len() * std::mem::size_of::<f32>()) as u64,
                    },
                },
            ],
        });

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

        Self {
            width,
            height,
            options,

            device,
            queue,

            pipeline,
            bind_group,

            depth_texture,
            depth_texture_view,

            multisampled_framebuffer,

            meshes,

            camera,
            camera_buffer,

            positions_instanced_buffer,
        }
    }

    pub fn options(&self) -> &ApplicationOptions {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut ApplicationOptions {
        &mut self.options
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

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let size = std::mem::size_of::<CameraUbo>();
            let camera_buffer = self.device.create_buffer_mapped(size, wgpu::BufferUsage::COPY_SRC);
            camera_buffer.data.copy_from_slice(transmute_to_bytes(&[self.camera.ubo()]));
            let camera_buffer = camera_buffer.finish();

            encoder.copy_buffer_to_buffer(&camera_buffer, 0, &self.camera_buffer, 0, size as wgpu::BufferAddress);
        }

        {
            let mesh_index: usize = self.options.mesh.into();
            let mesh = &self.meshes[mesh_index];
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.multisampled_framebuffer,
                    resolve_target: Some(frame),
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::GREEN,
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
            rpass.set_pipeline(&self.pipeline.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffers(0, &[(&mesh.vertices(), 0), (&mesh.normals(), 0)]);
            rpass.set_index_buffer(&mesh.indices(), 0);
            rpass.draw_indexed(0..mesh.indices_len(), 0, 0..n as u32);
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
