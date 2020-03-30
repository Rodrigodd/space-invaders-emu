use std::sync::{
    mpsc::{channel, Sender},
};

use intel8080::{interpreter,  IODevices, Memory };

use std::{
    io::Cursor,
    thread,
};

use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
};

use pixels::{Pixels, SurfaceTexture};

use rodio;
use rodio::Source;

const SCREEN_WIDTH: u32 = 224;
const SCREEN_HEIGHT: u32 = 256;

static SOUND_BANK: [&'static [u8]; 9] = [
    include_bytes!("../sound/0.wav"),
    include_bytes!("../sound/1.wav"),
    include_bytes!("../sound/2.wav"),
    include_bytes!("../sound/3.wav"),
    include_bytes!("../sound/4.wav"),
    include_bytes!("../sound/5.wav"),
    include_bytes!("../sound/6.wav"),
    include_bytes!("../sound/7.wav"),
    include_bytes!("../sound/8.wav"),
];

enum AudioMessage {
    Play(u8),
    StartUfo,
    StopUfo,
}

struct SpaceInvadersDevices {
    shift_register: u16,
    shift_amount: u8,
    read_ports: [u8; 3],

    wport3: u8,
    wport5: u8,
    audio_channel: Sender<AudioMessage>,
}
impl SpaceInvadersDevices {
    fn new(ports: [u8; 3]) -> Self {

        let (sx, rx) = channel();

        thread::spawn(move || {
            let device = rodio::default_output_device().unwrap();
            let ufo = rodio::Sink::new(&device);
            ufo.pause();
            ufo.append(
                rodio::Decoder::new(Cursor::new(SOUND_BANK[0])).unwrap()
                    .repeat_infinite()
            );

            while let Ok(msg) = rx.recv() {
                match msg {
                    AudioMessage::Play(index) => {
                        rodio::play_raw(
                            &device,
                            rodio::Decoder::new(
                                Cursor::new(SOUND_BANK[index as usize])
                            ).unwrap().convert_samples()
                        );
                    },
                    AudioMessage::StartUfo => {
                        ufo.play();
                    },
                    AudioMessage::StopUfo => {
                        ufo.pause();
                    },
                }
            }
        });
        
        Self {
            shift_register:0,
            shift_amount: 0,
            read_ports: ports,

            wport3: 0,
            wport5: 0,
            audio_channel: sx,
        }
    }

    fn start_ufo(&mut self) {
        self.audio_channel.send(AudioMessage::StartUfo).unwrap();
    }
    fn stop_ufo(&mut self) {
        self.audio_channel.send(AudioMessage::StopUfo).unwrap();
    }

    fn play_sound(&mut self, index: u8) {
        self.audio_channel.send(AudioMessage::Play(index)).unwrap();
    }
}
impl IODevices for SpaceInvadersDevices {
    fn read(&mut self, device: u8) -> u8 {
        match device {
            i @ 0..=2 => self.read_ports[i as usize],
            3 => (self.shift_register >> (8 - self.shift_amount)) as u8,
            _ => 0
        }
    }

    fn write(&mut self, device: u8, value: u8) {
        match device {
            2 => self.shift_amount = value & 0b111,
            3 => { // sound
                let check_bit = |byte: u8, i: u8| byte & (0b1 << i) != 0;

                if check_bit(value, 0) && !check_bit(self.wport3, 0) {
                    self.start_ufo();
                } else if !check_bit(value, 0) && check_bit(self.wport3, 0) {
                    self.stop_ufo();
                }

                if check_bit(value, 1) && !check_bit(self.wport3, 1) {
                    self.play_sound(1);
                }
                if check_bit(value, 2) && !check_bit(self.wport3, 2) {
                    self.play_sound(2);
                }
                if check_bit(value, 3) && !check_bit(self.wport3, 3) {
                    self.play_sound(3);
                }
                self.wport3 = value;
            }
            4 => self.shift_register = (self.shift_register >> 8) | ((value as u16) << 8),
            5 => { // sound
                let check_bit = |byte: u8, i: u8| byte & (0b1 << i) != 0;

                if check_bit(value, 0) && !check_bit(self.wport3, 0) {
                    self.play_sound(4);
                }
                if check_bit(value, 1) && !check_bit(self.wport3, 1) {
                    self.play_sound(5);
                }
                if check_bit(value, 2) && !check_bit(self.wport3, 2) {
                    self.play_sound(6);
                }
                if check_bit(value, 3) && !check_bit(self.wport3, 3) {
                    self.play_sound(7);
                }
                if check_bit(value, 4) && !check_bit(self.wport3, 4) {
                    self.play_sound(8);
                }
                self.wport5 = value;
            }
            _ => (),
        };
    }
}

struct SpaceInvadersMemory {
    memory: [u8; 0x4000]
}
impl Memory for SpaceInvadersMemory {
    #[inline]
    fn read(&self, mut adress: u16) -> u8 {
        if adress > 0x4000 {
            adress = (adress % 0x2000) + 0x2000;
        }
        if (adress as usize) < self.memory.len() {
            self.memory[adress as usize]
        } else {
            #[cfg(feature = "debug")]
            println!("Reading out of memory! At {:04x}!", adress);
            0
        }
    }

    fn write(&mut self, mut adress: u16, value: u8) {
        if adress > 0x4000 {
            adress = (adress % 0x2000) + 0x2000;
        }
        if (adress as usize) < self.memory.len() {
            if adress >= 0x2000 { // adress < 0x2000 is ROM
                self.memory[adress as usize] = value;
            } else {
                #[cfg(feature = "debug")]
                println!("Writing to ROM?");
            }
        } else {
            #[cfg(feature = "debug")]
            println!("Writing out of memory! At {:04x}!", adress);
        }
    }

    fn get_rom(&mut self) -> Vec<u8> {
        let mut rom = Vec::with_capacity(0x2000);
        for i in 0..0x2000 {
            rom.push(self.memory[i]);
        }
        rom
    }
}

pub fn load_rom(buf: &mut [u8]) {
    use std::io::Read;
    use std::fs;
    let mut andress = 0;
    for e in ['h', 'g', 'f', 'e'].iter() {
        let mut file = fs::File::open(format!("rom/invaders.{}", e)).unwrap();
        file.read(&mut buf[andress..andress+0x800]).unwrap();
        andress += 0x800;
    }
}

fn render_screen(screen: &mut [u8], memory: &[u8]) {
    for x in 0..SCREEN_WIDTH {
        for y in 0..SCREEN_HEIGHT {
            let i = (x*SCREEN_HEIGHT + y) as usize;
            if i/8 >= memory.len() {
                println!("x: {}, y: {}, i: {}, i/8: {}, memory.len(): {}", x, y, i, i/8, memory.len());
                return;
            }
            let m = memory[i/8];
            let c = if (m >> (i%8)) & 0x1 != 0 { 0xff } else { 0x0 };
            let p = ((SCREEN_HEIGHT - y - 1)*SCREEN_WIDTH + x) as usize*4;
            screen[p]     = if y >= 72 || (y < 16 && (x < 16 || x >= 102))                         { c } else { 0 };
            screen[p + 1] = if y < 192 || y >= 224                                                 { c } else { 0 };
            screen[p + 2] = if (y >= 72 && y < 192) || y>= 224 || (y < 16 && (x < 16 || x >= 102)) { c } else { 0 };
            screen[p + 3] = 0xff;
        }
    }
}

pub fn main_loop(debug: bool) {
    let event_loop = EventLoop::new();

    // let window = WindowBuilder::new()
    //     .with_inner_size(LogicalSize::new(200, 150))
    //     .build(&event_loop)
    //     .unwrap();

    let (window, surface, p_width, p_height, _) =
        create_window("SPACE INVADERS!!", &event_loop);

    let surface_texture = SurfaceTexture::new(p_width, p_height, surface);

    let mut pixels = Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture).unwrap();

    let ports = [
        (0b0000_1111).into(),
        (0b0000_1000).into(),
        (0b0000_0000).into(),
    ];
    
    let mut memory = [0; 0x4000];
    load_rom(&mut memory);

    let mut interpreter = interpreter::Interpreter::new(
        SpaceInvadersDevices::new(ports),
        SpaceInvadersMemory { memory: memory, },
        &[0x0u16, 0x8, 0x10],
        // debug,
    );
    #[cfg(feature = "debug")] {
        if debug {
            interpreter.enter_debug_mode();
        }
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                render_screen(pixels.get_frame(), &interpreter.memory.memory[0x2400..]);
                pixels.render();
                interpreter.run(2_000_000/120);
                interpreter.interrupt(0b11010111); // RST 2 (0xd7)
                interpreter.run(2_000_000/120);
                interpreter.interrupt(0b11001111); // RST 1 (0xcf)
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode: Some(key), state: ElementState::Pressed, .. },
                    is_synthetic: false,
                    .. 
                }  => match key { // PRESSED
                    VirtualKeyCode::Left => { // LEFT
                        interpreter.devices.read_ports[1] |= 0b0010_0000; // P1
                        interpreter.devices.read_ports[2] |= 0b0010_0000; // P2
                    },
                    VirtualKeyCode::Right => { // RIGHT
                        interpreter.devices.read_ports[1] |= 0b0100_0000; // P1
                        interpreter.devices.read_ports[2] |= 0b0100_0000; // P2
                    },
                    VirtualKeyCode::Z => { // SHOOT
                        interpreter.devices.read_ports[1] |= 0b0001_0000; // P1
                        interpreter.devices.read_ports[2] |= 0b0001_0000; // P2
                    },
                    VirtualKeyCode::C => { interpreter.devices.read_ports[1] |= 0b0000_0001; }, // COIN
                    VirtualKeyCode::Return => { interpreter.devices.read_ports[1] |= 0b0000_0100; }, // P1 START
                    VirtualKeyCode::Back => { interpreter.devices.read_ports[1] |= 0b0000_0010; }, // P2 START
                    #[cfg(feature = "debug")]
                    VirtualKeyCode::Escape => interpreter.enter_debug_mode(),
                    _ => (),
                },
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode: Some(key), state: ElementState::Released, .. },
                    is_synthetic: false,
                    .. 
                } => match key { // RELEASED
                    VirtualKeyCode::Left => { // LEFT
                        interpreter.devices.read_ports[1] &= !0b0010_0000; // P1
                        interpreter.devices.read_ports[2] &= !0b0010_0000; // P2
                    },
                    VirtualKeyCode::Right => { // RIGHT
                        interpreter.devices.read_ports[1] &= !0b0100_0000; // P1
                        interpreter.devices.read_ports[2] &= !0b0100_0000; // P2
                    },
                    VirtualKeyCode::Z => { // SHOOT
                        interpreter.devices.read_ports[1] &= !0b0001_0000; // P1
                        interpreter.devices.read_ports[2] &= !0b0001_0000; // P2
                    },
                    VirtualKeyCode::C => { interpreter.devices.read_ports[1] &= !0b0000_0001; }, // COIN
                    VirtualKeyCode::Return => { interpreter.devices.read_ports[1] &= !0b0000_0100; }, // P1 START
                    VirtualKeyCode::Back => { interpreter.devices.read_ports[1] &= !0b0000_0010; }, // P2 START
                    _ => (),
                },
                _ => ()
            },
            _ => (),
        }
    });
}

fn create_window(
    title: &str,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, pixels::wgpu::Surface, u32, u32, f64) {

    // Create a hidden window so we can estimate a good default window size
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(&event_loop)
        .unwrap();
    let hidpi_factor = window.scale_factor();

    // Get dimensions
    let width = SCREEN_WIDTH as f64;
    let height = SCREEN_HEIGHT as f64;
    let (monitor_width, monitor_height) = {
        let size = window.current_monitor().size();
        (size.width as f64 / hidpi_factor, size.height as f64 / hidpi_factor)
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round();

    // Resize, center, and display the window
    let min_size: LogicalSize<f64> = PhysicalSize::new(width, height).to_logical(hidpi_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let surface = pixels::wgpu::Surface::create(&window);
    let size: PhysicalSize<u32> = default_size.to_physical(hidpi_factor);

    (
        window,
        surface,
        size.width,
        size.height,
        hidpi_factor,
    )
}
