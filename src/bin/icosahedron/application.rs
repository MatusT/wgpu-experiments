use wgpu_experiments::camera::*;
use wgpu_experiments::pipelines::mesh::MeshPipeline;
use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton};

extern crate alloc;

use obj::*;
use safe_transmute::*;
use std::fs::File;
use std::io::BufReader;
use wgpu;

pub struct ApplicationOptions {
    pub n: i32,
}

pub struct Application {
    width: u32,
    height: u32,
    options: ApplicationOptions,

    device: wgpu::Device,
    queue: wgpu::Queue,

    pipeline: MeshPipeline,
    bind_group: wgpu::BindGroup,

    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,

    multisampled_framebuffer: wgpu::TextureView,

    icosahedron_vertices: wgpu::Buffer,
    icosahedron_normals: wgpu::Buffer,
    icosahedron_indices: wgpu::Buffer,
    icosahedron_indices_len: u32,

    camera: RotationCamera,
    camera_buffer: wgpu::Buffer,

    positions_instanced_buffer: wgpu::Buffer,
}

impl Application {
    pub fn new(width: u32, height: u32) -> Self {
        let options = ApplicationOptions { n: 100 };

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                backends: wgpu::BackendBit::PRIMARY,
            },
            // wgpu::BackendBit::PRIMARY,
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
        let icosahedron_file = BufReader::new(File::open("cube.obj").unwrap());
        let icosahedron_obj: Obj = load_obj(icosahedron_file).unwrap();

        let mut icosahedron_vertices = Vec::new();
        for v in icosahedron_obj.vertices.iter() {
            icosahedron_vertices.push(v.position[0]);
            icosahedron_vertices.push(v.position[1]);
            icosahedron_vertices.push(v.position[2]);
        }
        // let icosahedron_vertices = device.create_buffer_with_data(transmute_to_bytes(&icosahedron_vertices), wgpu::BufferUsage::VERTEX);
        let icosahedron_vertices = device
            .create_buffer_mapped::<f32>(icosahedron_vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&icosahedron_vertices);

        let mut icosahedron_normals = Vec::new();
        for v in icosahedron_obj.vertices.iter() {
            icosahedron_normals.push(v.normal[0]);
            icosahedron_normals.push(v.normal[1]);
            icosahedron_normals.push(v.normal[2]);
        }
        // let icosahedron_normals = device.create_buffer_with_data(transmute_to_bytes(&icosahedron_normals), wgpu::BufferUsage::VERTEX);
        let icosahedron_normals = device
            .create_buffer_mapped::<f32>(icosahedron_normals.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&icosahedron_normals);

        let icosahedron_indices = icosahedron_obj.indices;
        let icosahedron_indices_len = icosahedron_indices.len() as u32;
        // let icosahedron_indices = device.create_buffer_with_data(transmute_to_bytes(&icosahedron_indices), wgpu::BufferUsage::INDEX);
        let icosahedron_indices = device
            .create_buffer_mapped(icosahedron_indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&icosahedron_indices);

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
        // let positions_instanced_buffer = device.create_buffer_with_data(transmute_to_bytes(&positions), wgpu::BufferUsage::STORAGE_READ);
        let positions_instanced_buffer = device
            .create_buffer_mapped::<f32>(positions.len(), wgpu::BufferUsage::STORAGE_READ)
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

            icosahedron_vertices,
            icosahedron_normals,
            icosahedron_indices,
            icosahedron_indices_len,

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

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
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
            rpass.set_vertex_buffers(0, &[(&self.icosahedron_vertices, 0), (&self.icosahedron_normals, 0)]);
            rpass.set_index_buffer(&self.icosahedron_indices, 0);
            rpass.draw_indexed(
                0..self.icosahedron_indices_len,
                0,
                0..(self.options.n * self.options.n * self.options.n) as u32,
            );
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
