//!
//! Pipeline implementin Screen-Space Ambient Occlusion.
//!

use crate::load_glsl;
use wgpu::*;

pub struct DepthConversionPipeline {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
}

impl DepthConversionPipeline {
    pub fn new(device: &Device) -> Self {
        // Shaders
        let cs_bytes = load_glsl(include_str!("convert.comp"), crate::ShaderStage::Compute);
        let cs_module = device.create_shader_module(&cs_bytes);

        // Bind Groups
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("SSAO bind group layout"),
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                        format: TextureFormat::R32Float,
                        readonly: false,
                    },
                },
            ],
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: ProgrammableStageDescriptor {
                module: &cs_module,
                entry_point: "main",
            },
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn create_bind_group(&self, device: &Device, depth_input: &wgpu::Buffer, depth_output: &wgpu::TextureView) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer(depth_input.slice(..)),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::TextureView(depth_output),
                },
            ],
        })
    }
}
