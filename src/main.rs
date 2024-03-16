use vulkan::VulkanInstance;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  };
  
  mod vulkan;

  fn main() {

    let application_name = "Hello Triangle";
        
    let event_loop = EventLoop::new().unwrap();
    let _window = WindowBuilder::new()
      .with_title(application_name)
      .build(&event_loop)
      .unwrap();
    
    let engine_name = "Vulkan Renderer";
    let mut vulkan_instance = VulkanInstance::new(application_name, engine_name)
      .expect("Failed to initialize Vulkan instance");
    
    vulkan_instance.configure_hardware();

    unsafe {
      vulkan_instance.create_surface(&_window)
        .expect("Failed to create Vulkan surface");
    }
    
    let _ = event_loop.run(move |event, elwt| {
      let mut _control_flow = ControlFlow::Wait;

      match event {
        Event::WindowEvent { event, window_id } => match event {
          WindowEvent::CloseRequested => elwt.exit(),
          WindowEvent::Resized(_) => {
            _window.request_redraw();
          },
          WindowEvent::RedrawRequested => {
            // Vulkan Drawing Implementation
          }
          _ => (),
        },
        Event::NewEvents(_) => todo!(),
        Event::DeviceEvent { device_id, event } => todo!(),
        Event::UserEvent(_) => todo!(),
        Event::Suspended => todo!(),
        Event::Resumed => todo!(),
        Event::AboutToWait => todo!(),
        Event::LoopExiting => todo!(),
        Event::MemoryWarning => todo!(),
      }
    });
  }