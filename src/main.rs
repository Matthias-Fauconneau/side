use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let (width, height) = {let size = window.inner_size(); (size.width, size.height)};
                //let (width, stride) = (width/2, width); // Side by side stereo display (/!\ Epson BT-40 is broken and keeps 1920x1080 mode, i.e rescales 960 small images)
                surface.resize(stride.try_into().unwrap(), height.try_into().unwrap()).unwrap();

                let mut buffer = surface.buffer_mut().unwrap();
                for i in 0..2 {
                    for y in 0..height {
                        for x in 0..width/2 { // Side by side stereo display (/!\ Epson BT-40 is broken and keeps 1920x1080 mode, i.e rescales 960 small images)
                            let index = y as usize * width as usize + (i*width/2 + x) as usize;
                            let x = x*2; // Anamorphic side by side stereo display (/!\ Epson BT-40 is broken and keeps 1920x1080 mode, i.e rescales 960 small images)
                            buffer[index] = if ((x/(width/5))%2 == 1) ^ ((y/(width/5))%2 == 1) { 0xFFFFFF } else { 0 };
                        }
                    }
                }

                buffer.present().unwrap();
            }
            Event::WindowEvent {event: WindowEvent::CloseRequested, window_id} if window_id == window.id() => { *control_flow = ControlFlow::Exit; }
            _ => {}
        }
    });
}