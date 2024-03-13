use vulkan::VulkanInstance;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  };
  
  mod vulkan;

  fn main() {

    let application_name = "Hello Triangle";
        
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
      .with_title(application_name)
      .build(&event_loop)
      .unwrap();
    
    let engine_name = "Vulkan Renderer";
    let mut vulkan_instance = VulkanInstance::new(application_name, engine_name)
      .expect("Failed to initialize Vulkan instance");
    
    vulkan_instance.configure_hardware();
    
    event_loop.run(move | event, _, control_flow | {
      *control_flow = ControlFlow::Wait;
  
      match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(_) => {
                _window.request_redraw();
            }
            _ => (),
        },
        Event::RedrawRequested(_) => {
        }
        _ => (),
    }
    });
  }