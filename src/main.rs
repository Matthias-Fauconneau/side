fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hmd = serialport::new("/dev/ttyUSB0", 115200).timeout(std::time::Duration::from_millis(1000)).open()?;
    fn command(serialport: &mut (impl std::io::Read + std::io::Write), command: impl AsRef<str>) -> String {
        let command = command.as_ref();
        println!("{command}");
        serialport.write(format!("{command}\r\n").as_bytes()).unwrap(); 
        let mut read = |len| {let mut buffer: Vec<u8> = vec![0; len]; let len = serialport.read(&mut buffer).unwrap(); buffer.truncate(len); String::from_utf8(buffer).unwrap()};
        loop {
            let r = read(command.len());
            if r == command { break; }
            if !r.trim().is_empty() || r.trim()!=":" { use std::io::Write; std::io::stdout().write_all(r.as_bytes()).unwrap(); }
        }
        let mut r = read(8);
        assert_eq!(&r[0..2], "\r\n"); r.drain(0..2);
        assert_eq!(&r[r.len()-2..], "\r\n"); r.truncate(r.len()-2);
        if &r[r.len()-2..] == "\r\n" { r.truncate(r.len()-2); }
        print!("'{r}'");
        r
    }
    let mut distance : u8 = command(&mut hmd, "getdisplaydistance").trim().parse().unwrap();

    use winit::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, event::{self, Event::*, WindowEvent::*, ElementState::Pressed, VirtualKeyCode}};
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    //assert_eq!(command(&mut hmd, "set2d3d 1"), "OK"); 
    assert_eq!(command(&mut hmd, "setmute 0"), "OK");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            RedrawRequested(window_id) if window_id == window.id() => {
                let (width, height) = {let size = window.inner_size(); (size.width, size.height)};
                //let (width, stride) = (width/2, width); // Side by side stereo display (/!\ Epson BT-40 is broken and keeps 1920x1080 mode, i.e rescales 960 small images)
                surface.resize(width.try_into().unwrap(), height.try_into().unwrap()).unwrap();

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
            WindowEvent{event: CloseRequested, ..} | WindowEvent{event: KeyboardInput{input: event::KeyboardInput{virtual_keycode: Some(VirtualKeyCode::Escape), ..},..},..} => { 
                assert_eq!(command(&mut hmd, "setmute 1"), "OK");
                *control_flow = ControlFlow::Exit; 
            }
            WindowEvent{event: KeyboardInput{input: event::KeyboardInput{virtual_keycode: Some(key), state: Pressed, ..},..},..} => {use VirtualKeyCode::*; match key {
                Down => { if distance >= 8 { distance = distance-8; command(&mut hmd, format!("setdisplaydistance {distance}")); } }
                Up => { if distance <= 24 { distance = distance+8; command(&mut hmd, format!("setdisplaydistance {distance}")); } }
                _ => {},
            }},
            _ => {}
        }
    });
}