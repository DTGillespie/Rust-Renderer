use std::env;
use ash::vk::{DescriptorSetLayoutBinding, DescriptorType, ShaderStageFlags};
use pipeline::{
  GraphicsPipeline, PipelineConfig, ShaderStageConfig
};
use vulkan::VulkanInstance;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  };
  
  mod vulkan;
  mod vk_resources;
  mod pipeline;

  fn main() {

    let application_name = "Real Engine";
    let event_loop = EventLoop::new().unwrap();
    let _window = WindowBuilder::new()
      .with_title(application_name)
      .build(&event_loop)
      .unwrap();
    
    let engine_name = "Vulkan Renderer";
    let mut vulkan_instance = VulkanInstance::new(application_name, engine_name)
      .expect("Vulkan initialization failed");
    unsafe {
      vulkan_instance
        .create_surface(&_window).expect("Vulkan surface creation failed")
        .configure_hardware()
        .create_logical_device().expect("Failed to create Logical Device")
        .create_swapchain(&_window).unwrap()
        .create_render_pass().expect("Failed to create Render Pass")
        .bind_resources(10);
        
      // Test Shader
      let pipeline_config = PipelineConfig {
        shader_stages: vec![
          ShaderStageConfig {
            stage: ShaderStageFlags::VERTEX,
            shader_path: "shaders/vertex.spv".to_string(),
            entry_point: "main".to_string()
          },
          ShaderStageConfig {
            stage: ShaderStageFlags::FRAGMENT,
            shader_path: "shaders/fragment.spv".to_string(),
            entry_point: "main".to_string()
          }
        ]
      };

      let bindings = vec![
        DescriptorSetLayoutBinding::builder()
          .binding(0)
          .descriptor_type(DescriptorType::UNIFORM_BUFFER)
          .descriptor_count(1)
          .stage_flags(ShaderStageFlags::VERTEX)
          .build(),
        DescriptorSetLayoutBinding::builder()
          .binding(1)
          .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
          .descriptor_count(1)
          .stage_flags(ShaderStageFlags::FRAGMENT)
          .build(),
      ];

      
      vulkan_instance.define_shader("Demo", bindings); // Defines Descriptor Layouts and allocate Sets
      
      let pipeline_layout = vulkan_instance.create_pipeline_layout("Demo");
      vulkan_instance.bind_graphics_pipeline(pipeline_layout);
    }

    let _ = event_loop.run(move |event, elwt| {
      let mut _control_flow = ControlFlow::Wait;

      match event {
        Event::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => elwt.exit(),
          WindowEvent::Resized(_) => {
            _window.request_redraw();
          },
          WindowEvent::RedrawRequested => {
            // Vulkan Drawing Implementation
          }
          _ => (),
        },
        Event::NewEvents(_) => {

        },
        Event::DeviceEvent { device_id, event } => {

        },
        Event::UserEvent(_) => {

        },
        Event::Suspended => {
          
        },
        Event::Resumed => {

        },
        Event::AboutToWait => {

        },
        Event::LoopExiting => {

        },
        Event::MemoryWarning => {

        },
      }
    });
  }