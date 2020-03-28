use intel8080::{ IODevices, Memory };
use crate::interpreter;

struct TestDevices;
impl IODevices for TestDevices {
    fn read(&mut self, _: u8) -> u8 {
        0
    }

    fn write(&mut self, device: u8, value: u8) {
        match device {
            0 => print!("CPU HAS FAILED    ERROR EXIT="),
            1 => print!("CPU IS OPERATIONAL"),
            2 => print!("{}", value as char),
            3 => {
                println!();
                std::process::exit(0)
            },
            _ => (),
        };
    }
}

struct TestMemory {
    memory: [u8; 0x4000]
}
impl Memory for TestMemory {
    fn read(&self, adress: u16) -> u8 {
        if (adress as usize) < self.memory.len() {
            self.memory[adress as usize]
        } else {
            println!("Read out of memory! At 0x{:04x}!", adress);
            0
        }
    }

    fn write(&mut self, adress: u16, value: u8) {
        if adress < 0x05a4 {
            println!("overwriting code at 0x{:04x}!", adress);
        } else {
            if (adress as usize) < self.memory.len() {
                self.memory[adress as usize] = value;
            } else {
                println!("Write out of memory! At 0x{:04x}!", adress);
            }
        }
    }

    fn get_rom(&mut self) -> Vec<u8> {
        self.memory.to_vec()
    }
}

pub fn load_rom(memory: &mut [u8]) {
    use std::fs;
    use std::io::Read;
    let mut file = fs::File::open("rom/test.com").unwrap();
    file.read(memory).unwrap();
}

pub fn main_loop(debug: bool) {
    let mut memory = [0; 0x4000];
    load_rom(&mut memory);

    let _io = interpreter::start(
        TestDevices,
        TestMemory { memory: memory.clone(), },
        &[0x0],
        debug
    );

    loop {}
}