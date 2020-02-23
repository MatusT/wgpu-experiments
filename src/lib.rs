pub mod camera;
pub mod pipelines;

use winit;
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

pub fn load_glsl(code: &str, stage: ShaderStage) -> Vec<u32> {
    let ty = match stage {
        ShaderStage::Vertex => glsl_to_spirv::ShaderType::Vertex,
        ShaderStage::Fragment => glsl_to_spirv::ShaderType::Fragment,
        ShaderStage::Compute => glsl_to_spirv::ShaderType::Compute,
    };

    wgpu::read_spirv(glsl_to_spirv::compile(&code, ty).unwrap()).unwrap()
}

pub enum ApplicationEvent<'a> {
    DeviceEvent(winit::event::DeviceEvent),
    WindowEvent(winit::event::WindowEvent<'a>),
}

pub trait ApplicationSkeleton<'a> {
    fn resize(&mut self, width: u32, height: u32);

    fn update(&mut self, event: ApplicationEvent<'a>);

    fn render(&mut self, frame: &wgpu::TextureView);

    fn device(&self) -> &wgpu::Device;
}
