use crate::utils;
use crate::pipelines::{ triangle::TrianglePipeline };
use wgpu;
use winit::event::WindowEvent;

pub struct Application {
    device: wgpu::Device,
    queue: wgpu::Queue,

    triangle_pipeline: TrianglePipeline,
    triangle_bind_group: wgpu::BindGroup,
}

impl Application {
    pub fn new() -> Self {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
            },
            wgpu::BackendBit::PRIMARY,
        )
        .unwrap();
    
        let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });

        let triangle_pipeline = TrianglePipeline::new(&device);

        let triangle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &triangle_pipeline.bind_group_layout,
            bindings: &[],
        });

        Self { device, queue, triangle_pipeline, triangle_bind_group }
    }
}

impl utils::ApplicationSkeleton for Application {
    fn resize(&mut self, width: u32, height: u32) {
        //
    }

    fn update(&mut self, event: WindowEvent) {
        //
    }

    fn render(&mut self, frame: &wgpu::TextureView) {
        let mut encoder =
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
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
            rpass.set_pipeline(&self.triangle_pipeline.pipeline);
            rpass.set_bind_group(0, &self.triangle_bind_group, &[]);
            rpass.draw(0 .. 3, 0 .. 1);
        }

        self.queue.submit(&[encoder.finish()]);
    }

    fn device(&self) -> &wgpu::Device {
        &self.device
    }
}
