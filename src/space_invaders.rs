use std::sync::Arc;
use std::sync::atomic::{ AtomicU8, Ordering };

use intel8080::{ IODevices, Memory };
use intel8080::interpreter;

use std::time::{ Duration, Instant };
use std::thread;

use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
};
use pixels::{Pixels, SurfaceTexture};

const SCREEN_WIDTH: u32 = 224;
const SCREEN_HEIGHT: u32 = 256;

struct SpaceInvadersDevices {
    shift_register: u16,
    shift_amount: u8,
    ports: Arc<[AtomicU8; 3]>,
}
impl SpaceInvadersDevices {
    fn new(ports: Arc<[AtomicU8; 3]>) -> Self {
        Self {
            shift_register:0,
            shift_amount: 0,
            ports
        }
    }
}
impl IODevices for SpaceInvadersDevices {
    fn read(&mut self, device: u8) -> u8 {
        match device {
            i @ 0..=2 => self.ports[i as usize].load(Ordering::Relaxed),
            3 => (self.shift_register >> (8 - self.shift_amount)) as u8,
            _ => 0
        }
    }

    fn write(&mut self, device: u8, value: u8) {
        match device {
            2 => self.shift_amount = value & 0b111,
            4 => self.shift_register = (self.shift_register >> 8) | ((value as u16) << 8),
            _ => (),
        };
    }
}

struct SpaceInvadersMemory {
    memory: Arc<[AtomicU8; 0x4000]>
}
impl Memory for SpaceInvadersMemory {
    #[inline]
    fn read(&self, mut adress: u16) -> u8 {
        if adress > 0x4000 {
            adress = (adress % 0x2000) + 0x2000;
        }
        if (adress as usize) < self.memory.len() {
            self.memory[adress as usize].load(Ordering::Relaxed)
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
                self.memory[adress as usize].store(value, Ordering::Relaxed);
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
            rom.push(self.memory[i].load(Ordering::Relaxed));
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

fn render_screen(screen: &mut [u8], memory: &[AtomicU8]) {
    for x in 0..SCREEN_WIDTH {
        for y in 0..SCREEN_HEIGHT {
            let i = (x*SCREEN_HEIGHT + y) as usize;
            if i/8 >= memory.len() {
                println!("x: {}, y: {}, i: {}, i/8: {}, memory.len(): {}", x, y, i, i/8, memory.len());
                return;
            }
            let m = memory[i/8].load(Ordering::Relaxed);
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

    let ports = Arc::new([
        (0b0000_1111).into(),
        (0b0000_1000).into(),
        (0b0000_0000).into(),
    ]);
    
    use std::mem;
    let mut memory = [0; 0x4000];
    load_rom(&mut memory);
    let memory: Arc<[AtomicU8; 0x4000]> = Arc::new(unsafe {
        mem::transmute::<_, [AtomicU8; 0x4000]>(memory)
    });

    let mut interpreter_io = interpreter::start(
        SpaceInvadersDevices::new(ports.clone()),
        SpaceInvadersMemory { memory: memory.clone(), },
        &[0x0u16, 0x8, 0x10],
        debug,
    );
    let mut clock = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let elapsed = clock.elapsed();
                clock = Instant::now();
                if elapsed.as_micros() < 16666 {
                    thread::sleep(Duration::from_micros(16666) - elapsed);
                }
                render_screen(pixels.get_frame(), &memory[0x2400..]);
                pixels.render();
                interpreter_io.interrupt(0b11010111); // RST 2 (0xd7)
                thread::sleep(Duration::from_millis(8));
                interpreter_io.interrupt(0b11001111); // RST 1 (0xcf)
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
                    VirtualKeyCode::Left   => { ports[1].fetch_or(0b0010_0000, Ordering::Relaxed); }, // P1 LEFT
                    VirtualKeyCode::Right  => { ports[1].fetch_or(0b0100_0000, Ordering::Relaxed); }, // P1 RIGHT },
                    VirtualKeyCode::Z      => { ports[1].fetch_or(0b0001_0000, Ordering::Relaxed); }, // P1 SHOOT
                    VirtualKeyCode::C      => { ports[1].fetch_or(0b0000_0001, Ordering::Relaxed); }, // COIN
                    VirtualKeyCode::Return => { ports[1].fetch_or(0b0000_0100, Ordering::Relaxed); }, // P1 START
                    #[cfg(feature = "debug")]
                    VirtualKeyCode::Escape => if !interpreter_io.toogle_debug_mode() {
                        *control_flow = ControlFlow::Exit;
                    },
                    _ => (),
                },
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode: Some(key), state: ElementState::Released, .. },
                    is_synthetic: false,
                    .. 
                } => match key { // RELEASED
                    VirtualKeyCode::Left   => { ports[1].fetch_and(!0b0010_0000, Ordering::Relaxed); }, // P1 LEFT
                    VirtualKeyCode::Right  => { ports[1].fetch_and(!0b0100_0000, Ordering::Relaxed); }, // P1 RIGHT 
                    VirtualKeyCode::Z      => { ports[1].fetch_and(!0b0001_0000, Ordering::Relaxed); }, // P1 SHOOT
                    VirtualKeyCode::C      => { ports[1].fetch_and(!0b0000_0001, Ordering::Relaxed); }, // COIN
                    VirtualKeyCode::Return => { ports[1].fetch_and(!0b0000_0100, Ordering::Relaxed); }, // P1 START
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
