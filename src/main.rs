use std::fs;
use std::io;

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
    let mut stdout = WriteAdapter(io::stdout());
    dissasembly(&mut stdout, &state.memory).unwrap();
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