use crate::pipeline::{
  GraphicsPipeline, 
  PipelineConfig, 
};
use std::{
  collections::HashMap, 
  mem::{align_of, offset_of, size_of}
};
use ash::{
  util::Align, vk::{
    self, Buffer, BufferCreateInfo, BufferUsageFlags, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, DeviceMemory, DeviceSize, Extent2D, Format, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, PhysicalDevice, PhysicalDeviceMemoryProperties, PipelineLayout, PipelineLayoutCreateInfo, ShaderStageFlags, SharingMode, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate
  }, Device, Instance
};

#[repr(C, align(4))]
#[derive(Copy)]
#[derive(Clone)]
pub struct Vertex {
  pub position : [f32; 3],
  pub color    : [f32; 3]
}

impl Vertex {

  pub fn binding_description() -> VertexInputBindingDescription {
    VertexInputBindingDescription::builder()
      .binding(0)
      .stride(size_of::<Vertex>() as u32)
      .input_rate(VertexInputRate::VERTEX)
      .build()
  }

  pub fn attribute_descriptions() -> [VertexInputAttributeDescription; 2] {
    let position_attribute = VertexInputAttributeDescription::builder()
      .binding(0)
      .location(0)
      .format(Format::R32G32B32_SFLOAT)
      .offset(offset_of!(Vertex, position) as u32)
      .build();

    let color_attribute = VertexInputAttributeDescription::builder()
      .binding(0)
      .location(1)
      .format(Format::R32G32B32_SFLOAT)
      .offset(offset_of!(Vertex, color) as u32)
      .build();
    [position_attribute, color_attribute]
  }
}

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

pub struct VulkanResources {
  descriptor_pool  : DescriptorPool,
  shader_resources : HashMap<String, ShaderResources>,
  pipelines        : HashMap<String, GraphicsPipeline>
}

impl VulkanResources {
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

    VulkanResources {
      shader_resources : HashMap::new(),
      pipelines        : HashMap::new(),
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

  pub fn create_graphics_pipeline(
    &mut self, 
    device          : &Device, 
    render_pass     : vk::RenderPass, 
    pipeline_id     : &str, 
    pipeline_layout : vk::PipelineLayout, 
    pipeline_config : PipelineConfig,
    extent          : Extent2D
  ) {
    let pipeline = GraphicsPipeline::new(device, render_pass, pipeline_layout, pipeline_config, extent);
    self.pipelines.insert(pipeline_id.to_string(), pipeline);
  }

  pub fn allocate_vertex_buffer(&mut self, vertices: &[Vertex], instance: &Instance, physical_device: PhysicalDevice, device: &Device) -> (Buffer, DeviceMemory) {
    
    let buffer_info = BufferCreateInfo {
      size         : (size_of::<Vertex>() * vertices.len()) as DeviceSize,
      usage        : BufferUsageFlags::VERTEX_BUFFER,
      sharing_mode : SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let vertex_buffer = unsafe { device.create_buffer(&buffer_info, None) }.expect("Failed to create Vertex Buffer");
    
    // Find suitable memory type for Vertex Buffer
    let mem_requirements = unsafe { device.get_buffer_memory_requirements(vertex_buffer) };
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    let mem_type_index = VulkanResources::query_memory_type(mem_requirements.memory_type_bits, MemoryPropertyFlags::HOST_COHERENT, mem_properties);
  
    // Allocate Vertex Buffer
    let alloc_info = MemoryAllocateInfo {
      allocation_size   : VulkanResources::align_to(mem_requirements.size, mem_requirements.alignment),
      memory_type_index : mem_type_index,
      ..Default::default()
    };

    let buffer_memory = unsafe { device.allocate_memory(&alloc_info, None) }.expect("Failed to allocate Vertex Buffer memory");
    unsafe {
      device.bind_buffer_memory(vertex_buffer, buffer_memory, 0)
        .expect("Failed to bind Vertex Buffer Memory");
    }

    // Copy vertex data into Vertex Buffer
    let data_ptr = unsafe { device.map_memory(buffer_memory, 0, buffer_info.size, MemoryMapFlags::empty()) }.expect("Failed to map Vertex Buffer memory");
    unsafe {
      let aligned_data_ptr = data_ptr as *mut Vertex;
      aligned_data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
      device.unmap_memory(buffer_memory);
    }

    (vertex_buffer, buffer_memory)
  }

  fn align_to(value: DeviceSize, alignment: DeviceSize) -> DeviceSize {
    (value + alignment - 1) & !(alignment - 1)
  }

  fn query_memory_type(type_filter: u32, properties: MemoryPropertyFlags, mem_properties: PhysicalDeviceMemoryProperties) -> u32 {
    for i in 0..mem_properties.memory_type_count {
      if (type_filter & (1 << i)) > 0 && mem_properties.memory_types[i as usize].property_flags.contains(properties) {
        return i;
      }
    }
    panic!("Failed to query suitable memory type");
  }
}