use vulkan::VulkanInstance;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  };
  
  mod vulkan;

  fn main() {

    let application_name = "Hephaestus";
    let event_loop = EventLoop::new().unwrap();
    let _window = WindowBuilder::new()
      .with_title(application_name)
      .build(&event_loop)
      .unwrap();
    
    let engine_name = "Vulkan Renderer";
    let mut vulkan_instance = VulkanInstance::new(application_name, engine_name)
      .expect("Vulkan initialization failed");
    
    unsafe {
      vulkan_instance.create_surface(&_window)
        .expect("Vulkan surface creation failed")
        .configure_hardware()
        .create_logical_device()
        .expect("Failed to create logical device");
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