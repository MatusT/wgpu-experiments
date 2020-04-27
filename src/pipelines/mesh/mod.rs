use crate::{load_glsl, ShaderStage};
use wgpu;
pub struct MeshPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl MeshPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        // Shaders
        let vs_bytes = load_glsl(include_str!("mesh.vert"), ShaderStage::Vertex);
        let fs_bytes = load_glsl(include_str!("mesh.frag"), ShaderStage::Fragment);
        let vs_module = device.create_shader_module(&vs_bytes);
        let fs_module = device.create_shader_module(&fs_bytes);

        // Bind Groups
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Mesh pipeline bind group layout"),
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
            ],
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: 12,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[wgpu::VertexAttributeDescriptor {
                            offset: 0,
                            format: wgpu::VertexFormat::Float3,
                            shader_location: 0,
                        }],
                    },
                    wgpu::VertexBufferDescriptor {
                        stride: 12,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[wgpu::VertexAttributeDescriptor {
                            offset: 0,
                            format: wgpu::VertexFormat::Float3,
                            shader_location: 1,
                        }],
                    },
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}
