use ash::{
  vk,
  Device,
  vk::Pipeline,
  vk::PipelineLayout,
  vk::ShaderModule
};

pub struct GraphicsPipeline {
  pub pipeline            : Pipeline,
  pub pipeline_layout     : PipelineLayout,
  vertex_shader_module    : ShaderModule,
  fragment_shader_modyule : ShaderModule,
}

impl GraphicsPipeline {
  pub fn new() -> Self {
    
  }
}