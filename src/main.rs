use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};

use std::fs;
use std::io;
use std::env::args;

mod dissasembler;
mod write_adapter;
mod state;
mod interpreter;

use dissasembler::*;
use write_adapter::WriteAdapter;
use state::I8080State;


fn main() {
    let mut state = I8080State::new();
    load_rom(&mut state.memory);

    let mut args = args();
    let _ = args.next();
    if args.any(|arg| arg.starts_with("-d")) {
        let mut stdout = WriteAdapter(io::stdout());
        dissasembly(&mut stdout, &state.memory, &[0x0u16, 0x8, 0x10]).unwrap();
    } else {
        main_loop(state);
    }
}

fn main_loop(state: I8080State) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(200, 150))
        .build(&event_loop)
        .unwrap();

    let sender = interpreter::start(state);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode: Some(key), state: ElementState::Pressed, .. },
                    is_synthetic: false,
                    .. 
                }  => match key {
                    VirtualKeyCode::Right => match sender.send(interpreter::Message::Step) {
                        Ok(()) => (),
                        Err(_) => *control_flow = ControlFlow::Exit,
                    },
                    VirtualKeyCode::Return => match sender.send(interpreter::Message::Debug) {
                        Ok(()) => (),
                        Err(_) => *control_flow = ControlFlow::Exit,
                    },
                    _ => (),
                }
                _ => ()
            },
            _ => (),
        }
    });
}


fn load_rom(buf: &mut [u8]) {
    let mut andress = 0;
    for e in ['h', 'g', 'f', 'e'].iter() {
        use std::io::Read;
        let mut file = fs::File::open(format!("rom/invaders.{}", e)).unwrap();
        file.read(&mut buf[andress..andress+0x800]).unwrap();
        andress += 0x800;
    }
}