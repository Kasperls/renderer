use std::path::Path;

trait PipelineTrait {
    /// creates the pipeline
    fn create_pipeline(&self, device: &wgpu::Device, pipeline_desc: &wgpu::RenderPipelineDescriptor) -> wgpu::RenderPipeline;
    
    /// creates the pipeline layout
    fn create_pipeline_layout(&self, device: &wgpu::Device, bind_group_layouts: &[&wgpu::BindGroupLayout]) -> wgpu::PipelineLayout;
    
    // function to create a shader
    fn create_shader(&self, device: &wgpu::Device, path: &str) -> wgpu::ShaderModule {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(path.into()),
        });
        shader
    }
    
    fn get_bind_groups(&self) -> Vec<&wgpu::BindGroupLayout>;

    fn get_pipeline(&self) -> &wgpu::RenderPipeline;

    fn get_view() -> &wgpu::TextureView;
}