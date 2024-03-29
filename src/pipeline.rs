use ash::{
  vk::{self, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo, ShaderModule, ShaderStageFlags},
  Device
};
use std::{ffi::CString, io::Read, path::Path};

pub struct ShaderStageConfig {
  pub stage       : ShaderStageFlags,
  pub shader_path : String,
  pub entry_point : String
}

pub struct PipelineConfig {
  pub shader_stages: Vec<ShaderStageConfig>
}

pub struct ShaderStage {
  pub stage            : ShaderStageFlags,
  pub module           : ShaderModule,
  pub entry_point_name : CString
}

impl ShaderStage {
  fn new(device: &Device, config: &ShaderStageConfig) -> Self {
    let shader_code      = GraphicsPipeline::load_shader(&config.shader_path);
    let module      = GraphicsPipeline::create_shader_module(device, &shader_code);
    let entry_point_name = CString::new(config.entry_point).unwrap();
    ShaderStage {
      stage: config.stage,
      module,
      entry_point_name
    }
  }
}

pub struct GraphicsPipeline {
  pub pipeline            : Pipeline,
  pub pipeline_layout     : PipelineLayout,
  shader_stages: Vec<ShaderStage>
}

impl GraphicsPipeline {
  pub fn new(device: &Device, render_pass: vk::RenderPass, pipeline_layout: vk::PipelineLayout, pipeline_config: PipelineConfig) -> Self {
    
    let shader_stages: Vec<ShaderStage> = pipeline_config
      .shader_stages
      .iter()
      .map(|config| ShaderStage::new(device, config))
      .collect();
    
    let pipeline_shader_stages: Vec<PipelineShaderStageCreateInfo> = shader_stages.iter().map(|stage| {
      PipelineShaderStageCreateInfo::builder()
        .stage(stage.stage)
        .module(stage.module)
        .name(stage.entry_point_name.as_c_str())
        .build()
    }).collect();

    Self {
      pipeline: vk::Pipeline::null(),
      pipeline_layout,
      shader_stages
    }
  }

  pub fn load_shader(path: &str) -> Vec<u8> {
    let mut file = std::fs::File::open(path).expect("Failure opening file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failure reading file");
    buffer
  }

  fn bytes_to_u32_slice(bytes: &[u8]) -> Vec<u32> {
    assert!(bytes.len() % 4 == 0, "Shader byte code length is not algined to 4");
    bytes.chunks(4).map(|chunk| {
      u32::from_le_bytes(chunk.try_into().expect("Slice with incorrect length"))
    }).collect()
  }

  fn create_shader_module(device: &Device, src: &[u8]) -> ShaderModule {
    let code_u32 = Self::bytes_to_u32_slice(src);
    let create_info = vk::ShaderModuleCreateInfo::builder()
      .code(&code_u32)
      .build();

    unsafe {
      device.create_shader_module(&create_info, None)
        .expect("Failed to create Shader Module")
    }
  }
}