use intel8080::{IODevices, Memory, interpreter::Interpreter};
use wasm_bindgen::prelude::*;

use std::sync::{LazyLock, Mutex};

static INTERPRETER: LazyLock<Mutex<Interpreter<SpaceInvadersMemory, SpaceInvadersDevices>>> =
    LazyLock::new(|| Mutex::new(create_interpreter()));

fn get_interpreter()
-> std::sync::MutexGuard<'static, Interpreter<SpaceInvadersMemory, SpaceInvadersDevices>> {
    INTERPRETER.lock().unwrap()
}

pub const SCREEN_WIDTH: u32 = 224;
pub const SCREEN_HEIGHT: u32 = 256;

pub struct SpaceInvadersDevices {
    shift_register: u16,
    shift_amount: u8,
    read_ports: [u8; 3],

    wport3: u8,
    wport5: u8,
}
impl SpaceInvadersDevices {
    fn new(ports: [u8; 3]) -> Self {
        Self {
            shift_register: 0,
            shift_amount: 0,
            read_ports: ports,

            wport3: 0,
            wport5: 0,
        }
    }
}
impl IODevices for SpaceInvadersDevices {
    fn read(&mut self, device: u8) -> u8 {
        match device {
            i @ 0..=2 => self.read_ports[i as usize],
            3 => (self.shift_register >> (8 - self.shift_amount)) as u8,
            _ => 0,
        }
    }

    fn write(&mut self, device: u8, value: u8) {
        match device {
            2 => self.shift_amount = value & 0b111,
            3 => {
                // sound
                let check_bit = |byte: u8, i: u8| byte & (0b1 << i) != 0;

                if check_bit(value, 0) && !check_bit(self.wport3, 0) {
                    start_ufo();
                } else if !check_bit(value, 0) && check_bit(self.wport3, 0) {
                    stop_ufo();
                }

                if check_bit(value, 1) && !check_bit(self.wport3, 1) {
                    play_sound(1);
                }
                if check_bit(value, 2) && !check_bit(self.wport3, 2) {
                    play_sound(2);
                }
                if check_bit(value, 3) && !check_bit(self.wport3, 3) {
                    play_sound(3);
                }
                self.wport3 = value;
            }
            4 => self.shift_register = (self.shift_register >> 8) | ((value as u16) << 8),
            5 => {
                // sound
                let check_bit = |byte: u8, i: u8| byte & (0b1 << i) != 0;

                if check_bit(value, 0) && !check_bit(self.wport3, 0) {
                    play_sound(4);
                }
                if check_bit(value, 1) && !check_bit(self.wport3, 1) {
                    play_sound(5);
                }
                if check_bit(value, 2) && !check_bit(self.wport3, 2) {
                    play_sound(6);
                }
                if check_bit(value, 3) && !check_bit(self.wport3, 3) {
                    play_sound(7);
                }
                if check_bit(value, 4) && !check_bit(self.wport3, 4) {
                    play_sound(8);
                }
                self.wport5 = value;
            }
            _ => (),
        };
    }
}

pub struct SpaceInvadersMemory {
    pub memory: [u8; 0x4000],
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
            0
        }
    }

    fn write(&mut self, mut adress: u16, value: u8) {
        if adress > 0x4000 {
            adress = (adress % 0x2000) + 0x2000;
        }
        if (adress as usize) < self.memory.len() && adress >= 0x2000 {
            // adress < 0x2000 is ROM
            self.memory[adress as usize] = value;
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
    const H: &[u8] = include_bytes!("../../rom/invaders.h");
    const G: &[u8] = include_bytes!("../../rom/invaders.g");
    const F: &[u8] = include_bytes!("../../rom/invaders.f");
    const E: &[u8] = include_bytes!("../../rom/invaders.e");

    buf[0..H.len()].clone_from_slice(H);
    buf[0x800..0x800 + G.len()].clone_from_slice(G);
    buf[0x1000..0x1000 + F.len()].clone_from_slice(F);
    buf[0x1800..0x1800 + E.len()].clone_from_slice(E);
}

pub fn render_screen(screen: &mut [u8], memory: &[u8]) {
    for x in 0..SCREEN_WIDTH {
        for y in 0..SCREEN_HEIGHT {
            let i = (x * SCREEN_HEIGHT + y) as usize;
            let m = memory[i / 8];
            let c = if (m >> (i % 8)) & 0x1 != 0 { 0xff } else { 0x0 };
            let p = ((SCREEN_HEIGHT - y - 1) * SCREEN_WIDTH + x) as usize * 4;
            screen[p] = if y >= 72 || (y < 16 && !(16..102).contains(&x)) {
                c
            } else {
                0
            };
            screen[p + 1] = if !(192..224).contains(&y) { c } else { 0 };
            screen[p + 2] =
                if (72..192).contains(&y) || y >= 224 || (y < 16 && !(16..102).contains(&x)) {
                    c
                } else {
                    0
                };
            screen[p + 3] = c;
        }
    }
}

pub fn create_interpreter() -> Interpreter<SpaceInvadersMemory, SpaceInvadersDevices> {
    let ports = [0b0000_1111, 0b0000_1000, 0b0000_0000];

    let mut memory = [0; 0x4000];
    load_rom(&mut memory);

    Interpreter::new(
        SpaceInvadersDevices::new(ports),
        SpaceInvadersMemory { memory },
        &[0x0u16, 0x8, 0x10],
    )
}

#[wasm_bindgen(module = "/sound.js")]
extern "C" {
    fn play_sound(i: u8);
    fn start_ufo();
    fn stop_ufo();
}

#[wasm_bindgen]
pub fn key_down(key: u8) {
    let mut interpreter = get_interpreter();
    match key {
        1 => {
            // LEFT
            interpreter.devices.read_ports[1] |= 0b0010_0000; // P1
            interpreter.devices.read_ports[2] |= 0b0010_0000; // P2
        }
        2 => {
            // RIGHT
            interpreter.devices.read_ports[1] |= 0b0100_0000; // P1
            interpreter.devices.read_ports[2] |= 0b0100_0000; // P2
        }
        3 => {
            // SHOOT
            interpreter.devices.read_ports[1] |= 0b0001_0000; // P1
            interpreter.devices.read_ports[2] |= 0b0001_0000; // P2
        }
        4 => {
            interpreter.devices.read_ports[1] |= 0b0000_0001;
        } // COIN
        5 => {
            interpreter.devices.read_ports[1] |= 0b0000_0100;
        } // P1 START
        6 => {
            interpreter.devices.read_ports[1] |= 0b0000_0010;
        } // P2 START
        _ => (),
    }
}

#[wasm_bindgen]
pub fn key_up(key: u8) {
    let mut interpreter = get_interpreter();
    match key {
        1 => {
            // LEFT
            interpreter.devices.read_ports[1] &= !0b0010_0000; // P1
            interpreter.devices.read_ports[2] &= !0b0010_0000; // P2
        }
        2 => {
            // RIGHT
            interpreter.devices.read_ports[1] &= !0b0100_0000; // P1
            interpreter.devices.read_ports[2] &= !0b0100_0000; // P2
        }
        3 => {
            // SHOOT
            interpreter.devices.read_ports[1] &= !0b0001_0000; // P1
            interpreter.devices.read_ports[2] &= !0b0001_0000; // P2
        }
        4 => {
            interpreter.devices.read_ports[1] &= !0b0000_0001;
        } // COIN
        5 => {
            interpreter.devices.read_ports[1] &= !0b0000_0100;
        } // P1 START
        6 => {
            interpreter.devices.read_ports[1] &= !0b0000_0010;
        } // P2 START
        _ => (),
    }
}

#[wasm_bindgen]
pub fn run_frame() -> Box<[u8]> {
    let mut interpreter = get_interpreter();

    interpreter.run(2_000_000 / 120);
    interpreter.interrupt(0b11010111); // RST 2 (0xd7)
    interpreter.run(2_000_000 / 120);
    interpreter.interrupt(0b11001111); // RST 1 (0xcf)

    let mut screen: [u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize] =
        [0; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize];
    render_screen(&mut screen, &interpreter.memory.memory[0x2400..]);

    screen.to_vec().into_boxed_slice()
}
