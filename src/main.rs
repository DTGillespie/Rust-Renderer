use std::env;

use vulkan_resources::Vertex;
use vulkan::VulkanInstance;
use ash::vk::{
  DescriptorSetLayoutBinding, 
  DescriptorType, 
  ShaderStageFlags, 
};
use pipeline::{
  PipelineConfig, 
  ShaderStageConfig,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  };
  
  mod vulkan;
  mod vulkan_resources;
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
        .allocate_resources(10);
        
      // Test Shader

      let cwd = env::current_dir().expect("Failed to get current working directory");
      let shaders_dir = cwd.join("..").join("..").join("src").join("shaders");

      let vertex_shader_path   = shaders_dir.join("vertex.spv".to_string()).to_str().unwrap().to_string();
      let fragment_shader_path = shaders_dir.join("fragment.spv".to_string()).to_str().unwrap().to_string();
      
      let pipeline_config = PipelineConfig {
        shader_stages: vec![
          ShaderStageConfig {
            stage: ShaderStageFlags::VERTEX,
            shader_path: vertex_shader_path,
            entry_point: "main".to_string()
          },
          ShaderStageConfig {
            stage: ShaderStageFlags::FRAGMENT,
            shader_path: fragment_shader_path,
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
      vulkan_instance.configure_graphics_pipeline("Primary", pipeline_layout, pipeline_config);

      let vertices: Vec<Vertex> = vec![
        Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [0.5, -0.5, 0.0],  color: [0.0, 1.0, 0.0] },
        Vertex { position: [0.0, 0.5, 0.0],   color: [0.0, 0.0, 1.0] },
      ];

      let vbo = vulkan_instance.allocate_vertex_buffer(&vertices);
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