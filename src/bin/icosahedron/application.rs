use wgpu_experiments::camera::*;
use wgpu_experiments::pipelines::mesh::MeshPipeline;
use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton};

extern crate alloc;

use nalgebra_glm as glm;
use obj::*;
use safe_transmute::*;
use std::fs::File;
use std::io::BufReader;
use wgpu;
use winit;

pub struct Application {
    width: u32,
    height: u32,

    device: wgpu::Device,
    queue: wgpu::Queue,

    pipeline: MeshPipeline,
    bind_group: wgpu::BindGroup,

    icosahedron_vertices: wgpu::Buffer,
    icosahedron_indices: wgpu::Buffer,
    icosahedron_indices_len: u32,

    camera: RotationCamera,
    camera_buffer: wgpu::Buffer,
}

impl Application {
    pub fn new(width: u32, height: u32) -> Self {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
            },
            wgpu::BackendBit::PRIMARY,
        )
        .unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });

        let pipeline = MeshPipeline::new(&device);

        // Icosahedron
        let icosahedron_file = BufReader::new(File::open("icosahedron.obj").unwrap());
        let icosahedron_obj: Obj = load_obj(icosahedron_file).unwrap();

        let mut icosahedron_vertices = Vec::new();
        for v in icosahedron_obj.vertices.iter() {
            icosahedron_vertices.push(v.position[0]);
            icosahedron_vertices.push(v.position[1]);
            icosahedron_vertices.push(v.position[2]);
        }
        let icosahedron_indices = icosahedron_obj.indices;
        let icosahedron_indices_len = icosahedron_indices.len() as u32;

        let icosahedron_vertices = device.create_buffer_with_data(transmute_to_bytes(&icosahedron_vertices), wgpu::BufferUsage::VERTEX);
        let icosahedron_indices = device.create_buffer_with_data(transmute_to_bytes(&icosahedron_indices), wgpu::BufferUsage::INDEX);

        let aspect = width as f32 / height as f32;
        let camera = RotationCamera::new(aspect, 0.785398163, 0.1);
        let camera_buffer = device.create_buffer_with_data(
            transmute_to_bytes(&[camera.ubo()]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &camera_buffer,
                    range: 0..192,
                },
            }],
        });

        Self {
            width,
            height,

            device,
            queue,

            pipeline,
            bind_group,

            icosahedron_vertices,
            icosahedron_indices,
            icosahedron_indices_len,

            camera,
            camera_buffer,
        }
    }
}

impl<'a> ApplicationSkeleton<'a> for Application {
    fn resize(&mut self, _: u32, _: u32) {
        //
    }

    fn update(&mut self, event: ApplicationEvent<'a>) {
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
                    attachment: &frame,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::GREEN,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffers(0, &[(&self.icosahedron_vertices, 0)]);
            rpass.set_index_buffer(&self.icosahedron_indices, 0);
            rpass.draw_indexed(0..self.icosahedron_indices_len, 0, 0..1);
        }

        self.queue.submit(&[encoder.finish()]);
    }

    fn device(&self) -> &wgpu::Device {
        &self.device
    }
}
