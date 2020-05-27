use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{Vec2, Vec4};
use wgpu::*;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MoleculeUbo {
    pub positions: [Vec4; 64],
    pub aabb_scale: Vec4,
    pub count: u32,
}

unsafe impl Zeroable for MoleculeUbo {}
unsafe impl Pod for MoleculeUbo {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MoleculesGlobals {
    pub resolution: Vec2,
}

unsafe impl Zeroable for MoleculesGlobals {}
unsafe impl Pod for MoleculesGlobals {}
pub struct SmallMoleculesPipeline {
    pub pipeline: MeshPipeline,
    pub bind_group_layout: BindGroupLayout,
}

impl SmallMoleculesPipeline {
    pub fn new(device: &Device, depth_only: bool) -> Self {
        // Shaders
        let ms = include_bytes!("small_molecules.mesh.spv");
        let ms_module = device.create_shader_module(&read_spirv(std::io::Cursor::new(&ms[..])).unwrap());

        let fs_color = include_bytes!("small_molecules.frag.spv");
        let fs_depth_only = include_bytes!("small_molecules_depth.frag.spv");
        let fs = if depth_only {
            read_spirv(std::io::Cursor::new(&fs_depth_only[..]))
        } else {
            read_spirv(std::io::Cursor::new(&fs_color[..]))
        };
        let fs_module = device.create_shader_module(&fs.unwrap());

        // Bind Groups
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Box bind group layout"),
            bindings: &[
                // Camera
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::MESH,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                // Globals
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::MESH,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                // Molecule information
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::MESH,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                // AABB's model matrices
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::MESH,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                // Depth buffer
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStage::MESH,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: false,
                    },
                },
            ],
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let color_states = if depth_only {
            vec![]
        } else {
            vec![wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }]
        };
        let pipeline = device.create_mesh_pipeline(&MeshPipelineDescriptor {
            layout: &pipeline_layout,
            task_stage: None,
            mesh_stage: ProgrammableStageDescriptor {
                module: &ms_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Cw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &color_states,
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil_front: StencilStateFaceDescriptor::IGNORE,
                stencil_back: StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn create_bind_group(
        &self,
        device: &Device,
        camera: &Buffer,
        globals: &Buffer,
        molecule: &Buffer,
        model_matrices: &Buffer,
        depth: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer(camera.slice(..)),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer(globals.slice(..)),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Buffer(molecule.slice(..)),
                },
                Binding {
                    binding: 3,
                    resource: BindingResource::Buffer(model_matrices.slice(..)),
                },
                Binding {
                    binding: 4,
                    resource: BindingResource::Buffer(depth.slice(..)),
                },
            ],
        })
    }
}
