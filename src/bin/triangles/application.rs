// use wgpu_experiments::pipelines::triangles::TrianglesPipeline;
// use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton};

// use rand::Rng;
// use std::f32::consts::PI;
// use wgpu;
// use zerocopy::AsBytes;

// pub struct Application {
//     device: wgpu::Device,
//     queue: wgpu::Queue,

//     triangles_pipeline: TrianglesPipeline,
//     triangles_bind_group: wgpu::BindGroup,

//     triangles_buffer: wgpu::Buffer,
// }

// impl Application {
//     pub fn new() -> Self {
//         let adapter = wgpu::Adapter::request(
//             &wgpu::RequestAdapterOptions {
//                 power_preference: wgpu::PowerPreference::Default,
//                 backends: wgpu::BackendBit::PRIMARY,
//             },
//             // wgpu::BackendBit::PRIMARY,
//         )
//         .unwrap();

//         let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
//             extensions: wgpu::Extensions {
//                 anisotropic_filtering: false,
//             },
//             limits: wgpu::Limits::default(),
//         });

//         let triangles_pipeline = TrianglesPipeline::new(&device);
//         let triangles_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//             layout: &triangles_pipeline.bind_group_layout,
//             bindings: &[],
//         });

//         // Create list of triangles
//         let mut rng = rand::thread_rng();
//         //let ts = [(5.0 / 4.0) * PI, (7.0 / 4.0) * PI, (1.0 / 2.0) * PI];
//         let ts = [0.0, (1.0 / 2.0) * PI, PI];
//         let count = 1_000_000;
//         let mut vertices: Vec<f32> = Vec::with_capacity(3 * 2 * count);
//         for _ in 0..count {
//             let radius = 0.03f32;
//             let center_x = rng.gen_range(-1.0f32, 1.0);
//             let center_y = rng.gen_range(-1.0f32, 1.0);

//             for t in ts.iter() {
//                 let x = center_x + radius * f32::cos(*t);
//                 let y = center_y + radius * f32::sin(*t);

//                 vertices.push(x);
//                 vertices.push(y);
//             }
//         }
//         let triangles_buffer = device.create_buffer_with_data(vertices.as_bytes(), wgpu::BufferUsage::VERTEX);

//         Self {
//             device,
//             queue,

//             triangles_pipeline,
//             triangles_bind_group,

//             triangles_buffer,
//         }
//     }
// }

// impl<'a> ApplicationSkeleton<'a> for Application {
//     fn resize(&mut self, _: u32, _: u32) {
//         //
//     }

//     fn update(&mut self, _: ApplicationEvent<'a>) {
//         //
//     }

//     fn render(&mut self, frame: &wgpu::TextureView) {
//         let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
//         {
//             let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                 color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
//                     attachment: &frame,
//                     resolve_target: None,
//                     load_op: wgpu::LoadOp::Clear,
//                     store_op: wgpu::StoreOp::Store,
//                     clear_color: wgpu::Color::GREEN,
//                 }],
//                 depth_stencil_attachment: None,
//             });
//             rpass.set_pipeline(&self.triangles_pipeline.pipeline);
//             rpass.set_bind_group(0, &self.triangles_bind_group, &[]);
//             rpass.set_vertex_buffers(0, &[(&self.triangles_buffer, 0)]);
//             rpass.draw(0..3 * 1_000_000, 0..1);
//         }

//         self.queue.submit(&[encoder.finish()]);
//     }

//     fn device(&self) -> &wgpu::Device {
//         &self.device
//     }

//     fn device_mut(&mut self) -> &mut wgpu::Device {
//         &mut self.device
//     }
// }
