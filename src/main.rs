use std::env::args;
use std::io;

mod space_invaders;
mod test_machine;
mod write_adapter;

use intel8080::{dissasembler, interpreter};

use dissasembler::*;
use write_adapter::WriteAdapter;

fn main() {
    let mut args = args();
    let _ = args.next();
    let mut disassembly = false;
    let mut test = false;
    let mut debug = false;
    for arg in args {
        if arg.starts_with("-debug") {
            debug = true;
        } else if arg.starts_with("test") {
            test = true;
        } else if arg.starts_with("-d") {
            disassembly = true;
        }
    }
    if test {
        if disassembly {
            use test_machine::load_rom;
            let mut rom = [0; 0x2000];
            load_rom(&mut rom);
            let mut stdout = WriteAdapter(io::stdout());
            dissasembly(&mut stdout, &rom, &[0x0]).unwrap();
        } else {
            test_machine::main_loop(debug);
        }
    } else {
        if disassembly {
            use space_invaders::load_rom;
            let mut rom = [0; 0x2000];
            load_rom(&mut rom);
            let mut stdout = WriteAdapter(io::stdout());
            dissasembly(&mut stdout, &rom, &[0x0u16, 0x8, 0x10]).unwrap();
        } else {
            space_invaders::main_loop(debug);
        }
    }
}
