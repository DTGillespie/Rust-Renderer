use ash::{
  vk,
  Device,
  vk::DescriptorSetLayout,
  vk::DescriptorPool,
  vk::DescriptorPoolSize,
  vk::DescriptorSet,
};

use crate::vulkan::VulkanInstance;

pub struct VkResourceManager {
  pub descriptor_layouts : VkrDescriptorLayouts,
  pub descriptor_pool    : VkrDescriptorPool,
  pub descriptor_sets    : VkrDescriptorSets,
}

impl VkResourceManager {
  pub fn new(device: &ash::Device, max_sets: u32) -> Self {
    let descriptor_layouts = VkrDescriptorLayouts::new(device);
    let pool_sizes = [vk::DescriptorPoolSize {
      ty: vk::DescriptorType::UNIFORM_BUFFER,
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
  pub layouts: Vec<vk::DescriptorSetLayout>
}

impl VkrDescriptorLayouts {
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

    VkrDescriptorLayouts {
      layouts: vec![layout],
    }
  }
}

pub struct VkrDescriptorPool {
  pub pool: vk::DescriptorPool
}

impl VkrDescriptorPool {
  pub fn new(device: &ash::Device, max_sets: u32, pool_sizes: &[vk::DescriptorPoolSize]) -> Self {
    let pool_info = vk::DescriptorPoolCreateInfo::builder()
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
  pub sets: Vec<vk::DescriptorSet>
}

impl VkrDescriptorSets {
  pub fn new(device: &ash::Device, pool: &VkrDescriptorPool, layouts: &VkrDescriptorLayouts, num_sets: usize) -> Self {
    let layouts_ref: Vec<vk::DescriptorSetLayout> = layouts.layouts.iter().copied().collect();
    let allocate_info = vk::DescriptorSetAllocateInfo::builder()
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