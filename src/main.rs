use std::{cell::Cell, env};

use vulkan_resources::Vertex;
use vulkan::{VulkanInstance, MAX_FRAMES_IN_FLIGHT};
use ash::vk::{
  DescriptorSetLayoutBinding, DescriptorType, ShaderStageFlags 
};
use pipeline::{
  PipelineConfig, 
  ShaderStageConfig,
};
use winit::{
    event::{self, Event, WindowEvent},
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
        .create_framebuffers()
        .allocate_resources(10)
        .create_command_pool()
        .allocate_command_buffers()
        .create_synchronization_objects();

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
      vulkan_instance.configure_graphics_pipeline("PRIMARY", pipeline_layout, pipeline_config);

      let vertices: Vec<Vertex> = vec![
        Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [0.5, -0.5, 0.0],  color: [0.0, 1.0, 0.0] },
        Vertex { position: [0.0, 0.5, 0.0],   color: [0.0, 0.0, 1.0] },
      ];

      vulkan_instance.allocate_vertex_buffer(&vertices);
    }

    /* Render Loop */
    let current_frame = Cell::new(0);

    //event_loop.run(move |event, _, control_flow| {
    event_loop.run(move |event, control_flow| {
      //*control_flow = ControlFlow::Poll;

      match event {

        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => return,

        /* Main Draw Loop */
        Event::WindowEvent {
          event: WindowEvent::RedrawRequested,
          ..
        } => {

          let frame_index = current_frame.get();
          let image_index = match vulkan_instance.acquire_next_image_index(frame_index) {
            Ok(index) => index,
            Err(_) => {
              println!("Failed to acquire next Image Index");
              return;
            }
          };
          
          if let Some(fence) = vulkan_instance.get_image_in_flight(image_index as usize) {
            vulkan_instance
          }

          current_frame.set((frame_index + 1) & MAX_FRAMES_IN_FLIGHT)
        },
        
        Event::MainEventsCleard => {
          _window.request_redraw();
        },
        _ => (),
      }
    });
  }

  /*

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Poll; // Continuously run the event loop, even if no events are being received.

    match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit, // Exit the loop when the window is closed.

        Event::RedrawRequested(_) => { // This is triggered for every frame to be drawn.
            let image_index = match vulkan_instance.acquire_next_image_index(&image_available_semaphores[current_frame]) {
                Ok(index) => index,
                Err(_) => {
                    // Handle error (e.g., recreate swap chain if needed)
                    return;
                }
            };

            if let Some(fence) = images_in_flight[image_index as usize] {
                vulkan_instance.logical_device.wait_for_fences(&[fence], true, u64::MAX).expect("Failed to wait for fence");
            }
            images_in_flight[image_index as usize] = Some(in_flight_fences[current_frame]);

            // Assuming command_buffer recording and other Vulkan operations are correctly handled here

            let wait_semaphores = [image_available_semaphores[current_frame]];
            let signal_semaphores = [render_finished_semaphores[current_frame]];
            let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&[command_buffer]) // Your command buffer for the current frame
                .signal_semaphores(&signal_semaphores)
                .build();

            vulkan_instance.logical_device.reset_fences(&[in_flight_fences[current_frame]]).expect("Failed to reset fence");
            vulkan_instance.graphics_queue.submit(&[submit_info], in_flight_fences[current_frame]).expect("Failed to submit draw command buffer");

            let swapchains = [vulkan_instance.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices)
                .build();

            match vulkan_instance.present_queue.present_khr(&present_info) {
                Ok(_) => (),
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                    // Handle swap chain recreation
                },
                Err(e) => panic!("Failed to present swapchain image: {:?}", e),
            }

            current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        },

        Event::MainEventsCleared => {
            // Ensure the window is always requested to redraw, enabling continuous animation.
            window.request_redraw();
        }

        _ => (),
    }
  });
  */