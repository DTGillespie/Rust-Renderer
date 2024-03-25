use ash::{
  vk,
  Device,
  vk::DescriptorSetLayout,
  vk::DescriptorPool,
  vk::DescriptorPoolSize,
  vk::DescriptorSet,
};

/*
pub struct VkResourceManager {
  pub descriptor_layouts: vk::DescriptorL
}
*/

pub struct DescriptorLayouts {
  pub layouts: Vec<DescriptorSetLayout>
}

impl DescriptorLayouts {
  pub fn new(device: &Device) -> Self {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
      .binding(0) // Shader Index Binding
      .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
      .descriptor_count(1)
      .stage_flags(vk::ShaderStageFlags::VERTEX)
      .build();

    let layout_bindings = [ubo_layout_binding];

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
      .bindings(&layout_bindings)
      .build();
    
    let layout = unsafe {
      device.create_descriptor_set_layout(&layout_info, None)
        .expect("Failed to create Descriptor Set Layout")
    };

    DescriptorLayouts {
      layouts: vec![layout],
    }
  }
}

pub struct _DescriptorPool {
  pub pool: vk::DescriptorPool
}

impl _DescriptorPool {
  pub fn new(device: &ash::Device, max_sets: u32, pool_sizes: &[vk::DescriptorPoolSize]) -> Self {
    
  }
}

pub struct DescriptorSets {
  pub sets: Vec<DescriptorSet>
}

impl DescriptorSets {
  pub fn new(device: &ash::Device, pool: &DescriptorPool, layouts: &DescriptorLayouts) -> Self {

  }
}