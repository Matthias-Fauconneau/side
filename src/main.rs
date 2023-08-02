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
            if !r.trim().is_empty() && r.trim()!=":" { use std::io::Write; std::io::stdout().write_all(r.as_bytes()).unwrap(); }
        }
        let mut r = read(8);
        assert_eq!(&r[0..2], "\r\n"); r.drain(0..2);
        while ["\r","\n"].contains(&&r[r.len()-1..]) { r.pop(); }
        print!("'{r}'");
        r
    }
    let mut distance : i16 = command(&mut hmd, "getdisplaydistance").trim().parse().unwrap();

    use winit::{event_loop::{EventLoop, ControlFlow}, window::{WindowBuilder, Fullscreen}, event::{self, Event::*, WindowEvent::*, ElementState::Pressed, VirtualKeyCode}};
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_fullscreen(Some(Fullscreen::Borderless(Some(event_loop.available_monitors().next().unwrap())))).build(&event_loop)?;
    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    assert_eq!(command(&mut hmd, "setmute 0"), "OK"); let mut on = true;
    assert_eq!(command(&mut hmd, "set2d3d 0"), "OK"); let mut software_stereo_side_by_side = false;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            RedrawRequested(window_id) if window_id == window.id() => {
                let (width, height) = {let size = window.inner_size(); (size.width, size.height)};
                surface.resize(width.try_into().unwrap(), height.try_into().unwrap()).unwrap();
                let mut buffer = surface.buffer_mut().unwrap();

                let checkerboard = |x,y| {
                    let n = 15;
                    let [i, j] = [x,y as i32].map(|x| (x/(width/n) as i32)); // FIXME: signed x
                    if i>1 && i<11 && j>=0 && j<7 && (i%2==1) ^ (j%2==1) { 0xFFFFFF } else { 0 }
                };
                if !software_stereo_side_by_side {
                    for y in 0..height {
                        for x in 0..width { // Full image horizontal shift (setdisplaydistance: -32,256)
                            buffer[(y * width + x) as usize] = checkerboard(x as i32, y as i32);
                        }
                    }
                } else {
                    for i in 0..2 {
                        for y in 0..height {
                            for x in 0..width/2 { // Side by side stereo display (/!\ Epson BT-40 is broken and keeps 1920x1080 mode, i.e rescales 960 small images)
                                buffer[(y * width + (i as u32*width/2/*broken BT-40*/ + x)) as usize] = checkerboard(
                                    x as i32*2/*Anamorphic side by side stereo display (broken BT-40)*/ + ((distance-256).max(0)*[-1,1][i]) as i32/*Software stereo display shift*/, y as i32
                                );
                            }
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
                Up|Down => { 
                    match key {
                        Up => if distance >= -32 { distance = distance-8; },
                        Down => if distance <= 256-8 || software_stereo_side_by_side { distance = distance+8 },
                        _ => unreachable!()
                    };
                    if distance <= 256 { 
                        command(&mut hmd, format!("setdisplaydistance {distance}")); 
                    } 
                    else { 
                        window.request_redraw(); 
                        println!("{distance}"); 
                    }
                }
                Space => { if on { assert_eq!(command(&mut hmd, "setmute 1"), "OK"); on=false; } else { assert_eq!(command(&mut hmd, "setmute 0"), "OK"); on=true; }}
                _ => {},
            }},
            _ => {}
        }
    });
}