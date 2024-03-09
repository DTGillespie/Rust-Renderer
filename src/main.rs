use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  };
  
  fn main() {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
      .with_title("Hello Triangle")
      .build(&event_loop)
      .unwrap();
  
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