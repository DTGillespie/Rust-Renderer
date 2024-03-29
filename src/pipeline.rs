use std::{ffi::CString, io::Read,};
use ash::{
  vk::{
    self, ColorComponentFlags, CullModeFlags, Extent2D, FrontFace, GraphicsPipelineCreateInfo, Offset2D, Pipeline, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendAttachmentStateBuilder, PipelineColorBlendStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, Rect2D, SampleCountFlags, ShaderModule, ShaderStageFlags, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate, Viewport
  },
  Device
};

use crate::vulkan_resources::Vertex;

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
    let entry_point_name = CString::new(config.entry_point.as_str()).unwrap();
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
  shader_stages           : Vec<ShaderStage>
}

impl GraphicsPipeline {
  pub fn new(
    device: &Device, 
    render_pass     : vk::RenderPass, 
    pipeline_layout : vk::PipelineLayout, 
    pipeline_config : PipelineConfig,
    extent          : Extent2D
  ) -> Self {
    
    let shader_stages: Vec<ShaderStage> = pipeline_config
      .shader_stages
      .iter()
      .map(|config| ShaderStage::new(device, config))
      .collect();
    
    let input_assembly = PipelineInputAssemblyStateCreateInfo::builder()
      .topology(PrimitiveTopology::TRIANGLE_LIST)
      .primitive_restart_enable(false)
      .build();

    let viewport = Viewport {
      x: 0.0,
      y: 0.0,
      width    : extent.width as f32,
      height   : extent.height as f32,
      min_depth : 0.0,
      max_depth : 1.0
    };

    let scissor = Rect2D {
      offset: Offset2D {x: 0, y: 0},
      extent
    };

    let rasterizer = PipelineRasterizationStateCreateInfo::builder()
      .depth_clamp_enable(false)
      .rasterizer_discard_enable(false) // Disables output to framebuffer
      .polygon_mode(PolygonMode::FILL)  // Solid: FILL, Wireframe: LINE
      .line_width(1.0)                  // Wireframe Line Width
      .cull_mode(CullModeFlags::BACK)   // Backface culling (NONE, BACK, FRONT, FRONT_AND_BACK)
      .front_face(FrontFace::CLOCKWISE) // (CLOCKWISE, COUNTER_CLOCKWISE)
      .build();

    let multisampling = PipelineMultisampleStateCreateInfo::builder()
      .sample_shading_enable(false)
      .rasterization_samples(SampleCountFlags::TYPE_1) // No Multisampling
      .build();

    let color_blend_attachment = PipelineColorBlendAttachmentState::builder()
      .color_write_mask(ColorComponentFlags::R | ColorComponentFlags::G | ColorComponentFlags::B | ColorComponentFlags::A)
      .blend_enable(false)
      .build();

    let color_blending = PipelineColorBlendStateCreateInfo::builder()
      .logic_op_enable(false)
      .attachments(&[color_blend_attachment])
      .build();

      let pipeline_shader_stages: Vec<PipelineShaderStageCreateInfo> = shader_stages.iter().map(|stage| {
        PipelineShaderStageCreateInfo::builder()
          .stage(stage.stage)
          .module(stage.module)
          .name(stage.entry_point_name.as_c_str())
          .build()
      }).collect();

      let vertex_binding_description = Vertex::binding_description();
      let vertex_attribute_descriptions = Vertex::attribute_descriptions();
      let vertex_input_info = PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&[vertex_binding_description])
        .vertex_attribute_descriptions(&vertex_attribute_descriptions)
        .build();
  
      let viewport_state = PipelineViewportStateCreateInfo::builder()
        .viewports(&[viewport])
        .scissors(&[scissor])
        .build();

      let pipeline_info = GraphicsPipelineCreateInfo::builder()
        .stages(&pipeline_shader_stages)
        .vertex_input_state(&vertex_input_info)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .color_blend_state(&color_blending)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .build();

      let graphics_pipeline = unsafe {
        device.create_graphics_pipelines(PipelineCache::null(), &[pipeline_info], None)
          .expect("Failed to create Graphics Pipeline")[0]
      };
      
    Self {
      pipeline: vk::Pipeline::null(),
      pipeline_layout,
      shader_stages
    }
  }

  pub fn load_shader(path: &str) -> Vec<u8> {
    let msg = format!("Failure loading shader from source: {}", path);
    let mut file = std::fs::File::open(path).expect(&msg);
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