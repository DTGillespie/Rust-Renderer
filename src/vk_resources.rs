use crate::pipeline::{self, GraphicsPipeline};
use std::collections::HashMap;
use ash::{
  vk::{self, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, PipelineLayout, PipelineLayoutCreateInfo, ShaderStageFlags},
  Device,
};

pub struct ShaderResources {
  descriptor_layouts : Vec<DescriptorSetLayout>,
  descriptor_sets    : Vec<DescriptorSet>,
}

impl ShaderResources {
  fn new() -> Self {
    ShaderResources {
      descriptor_layouts : Vec::new(),
      descriptor_sets    : Vec::new(),
    }
  }
}

pub struct VkResourceManager {
  shader_resources : HashMap<String, ShaderResources>,
  descriptor_pool  : DescriptorPool,
  pipelines        : Vec<GraphicsPipeline>
}

impl VkResourceManager {
  pub fn new(device: &ash::Device, max_sets: u32) -> Self {

    let pool_sizes = [DescriptorPoolSize {
      ty: DescriptorType::UNIFORM_BUFFER,
      descriptor_count: max_sets,
    }];

    let pool_info = DescriptorPoolCreateInfo::builder()
      .pool_sizes(&pool_sizes)
      .max_sets(max_sets)
      .build();

    let descriptor_pool = unsafe {
      device.create_descriptor_pool(&pool_info, None)
        .expect("Failed to create Descriptor Pool")
    };

    VkResourceManager {
      shader_resources : HashMap::new(),
      pipelines        : Vec::new(),
      descriptor_pool,
    }
  }

  pub fn create_shader_resources(&mut self, shader_id: &str) -> &mut Self {
    self.shader_resources.insert(shader_id.to_string(), ShaderResources::new());
    self
  }

  pub fn new_descriptor_layout(&mut self, device: &Device, shader_id: &str, bindings: Vec<DescriptorSetLayoutBinding>) -> &mut Self {

    let layout_info = DescriptorSetLayoutCreateInfo::builder()
      .bindings(&bindings)
      .build();

    let descriptor_layout = unsafe {
      device.create_descriptor_set_layout(&layout_info, None)
        .expect("Failed to create Descriptor Set Layout")
    };

    if let Some(shader_resources) = self.shader_resources.get_mut(shader_id) {
      shader_resources.descriptor_layouts.push(descriptor_layout);
    } else {
      println!("Shader ID not found. Ensure shader resources have been allocated before attempting to add a layout");
    }

    self
  }

  pub fn allocate_shader_descriptor_sets(&mut self, device: &Device, shader_id: &str) {

    if let Some(shader_resources) = self.shader_resources.get(shader_id) {
      let layouts: Vec<DescriptorSetLayout> = shader_resources.descriptor_layouts.iter().copied().collect();
      let allocate_info = DescriptorSetAllocateInfo::builder()
        .descriptor_pool(self.descriptor_pool)
        .set_layouts(&layouts)
        .build();

      let sets = unsafe {
        device.allocate_descriptor_sets(&allocate_info)
          .expect("Failed to allocate Descriptor Sets")
      };

      if let Some(shader_resources) = self.shader_resources.get_mut(shader_id) {
        shader_resources.descriptor_sets = sets;
      }
    } else {
      println!("Shader ID not found. Ensure shader resources have been allocated before attempting to add a layout");
    }
  }

  pub fn create_pipeline_layout(&mut self, device: &Device, shader_id: &str) -> PipelineLayout {
    if let Some(shader_resources) = self.shader_resources.get(shader_id) {
      let layouts: Vec<DescriptorSetLayout> = shader_resources.descriptor_layouts.iter().copied().collect();
      let pipeline_layout_info = PipelineLayoutCreateInfo::builder()
        .set_layouts(&layouts)
        .build();

      let pipeline_layout = unsafe {
        device.create_pipeline_layout(&pipeline_layout_info, None)
          .expect("Failed to create Pipeline Layout")
      };

      pipeline_layout
    } else {
      panic!("Shader ID not found. Ensure shader resources have been allocated before attemtping to create Pipeline Layout");
    }
  }

  pub fn create_graphics_pipeline(&mut self, device: &Device, render_pass: vk::RenderPass, pipeline_layout: vk::PipelineLayout) {
    /*
    let pipeline = GraphicsPipeline::new(device, render_pass, pipeline_layout);
    self.pipelines.push(pipeline);
    self.pipelines.len() - 1
    */
  }
}