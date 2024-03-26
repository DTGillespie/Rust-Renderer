use ash::{
  vk,
  Device,
  vk::DescriptorSetLayout,
  vk::DescriptorSetLayoutBinding,
  vk::DescriptorSetLayoutCreateInfo,
  vk::DescriptorPoolCreateInfo,
  vk::DescriptorSet,
  vk::DescriptorType,
  vk::DescriptorPool,
  vk::DescriptorPoolSize,
  vk::ShaderStageFlags,
  vk::DescriptorSetAllocateInfo,
};

pub struct VkResourceManager {
  pub descriptor_layouts : VkrDescriptorLayouts,
  pub descriptor_pool    : VkrDescriptorPool,
  pub descriptor_sets    : VkrDescriptorSets,
}

impl VkResourceManager {
  pub fn new(device: &ash::Device, max_sets: u32) -> Self {
    let descriptor_layouts = VkrDescriptorLayouts::new(device);
    let pool_sizes = [DescriptorPoolSize {
      ty: DescriptorType::UNIFORM_BUFFER,
      descriptor_count: max_sets,
    }];

    let descriptor_pool = VkrDescriptorPool::new(device, max_sets, &pool_sizes);
    let descriptor_sets = VkrDescriptorSets::new(device, &descriptor_pool, &descriptor_layouts, max_sets as usize);
  
    VkResourceManager {
      descriptor_layouts,
      descriptor_pool,
      descriptor_sets,
    }
  }
}

pub struct VkrDescriptorLayouts {
  pub layouts: Vec<DescriptorSetLayout>
}

impl VkrDescriptorLayouts {
  pub fn new(device: &Device) -> Self {
    let ubo_layout_binding = DescriptorSetLayoutBinding::builder()
      .binding(0) // Shader Index Binding
      .descriptor_type(DescriptorType::UNIFORM_BUFFER)
      .descriptor_count(1)
      .stage_flags(ShaderStageFlags::VERTEX)
      .build();

    let layout_bindings = [ubo_layout_binding];
    let layout_info = DescriptorSetLayoutCreateInfo::builder()
      .bindings(&layout_bindings)
      .build();
    
    let layout = unsafe {
      device.create_descriptor_set_layout(&layout_info, None)
        .expect("Failed to create Descriptor Set Layout")
    };

    VkrDescriptorLayouts {
      layouts: vec![layout],
    }
  }
}

pub struct VkrDescriptorPool {
  pub pool: DescriptorPool
}

impl VkrDescriptorPool {
  pub fn new(device: &ash::Device, max_sets: u32, pool_sizes: &[DescriptorPoolSize]) -> Self {
    let pool_info = DescriptorPoolCreateInfo::builder()
      .pool_sizes(pool_sizes)
      .max_sets(max_sets)
      .build();

    let pool = unsafe {
      device.create_descriptor_pool(&pool_info, None)
        .expect("Failed to create Descriptor Pool")
    };

    VkrDescriptorPool{ pool }
  }
}

pub struct VkrDescriptorSets {
  pub sets: Vec<DescriptorSet>
}

impl VkrDescriptorSets {
  pub fn new(device: &ash::Device, pool: &VkrDescriptorPool, layouts: &VkrDescriptorLayouts, num_sets: usize) -> Self {
    let layouts_ref: Vec<DescriptorSetLayout> = layouts.layouts.iter().copied().collect();
    let allocate_info = DescriptorSetAllocateInfo::builder()
      .descriptor_pool(pool.pool)
      .set_layouts(&layouts_ref)
      .build();

    let sets = unsafe {
      device.allocate_descriptor_sets(&allocate_info)
        .expect("Failed to allocate Descriptor Sets")
    };

    VkrDescriptorSets { sets }
  }
}