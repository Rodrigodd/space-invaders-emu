use crate::state::I8080State;
use crate::dissasembler;

use std::sync::mpsc::{channel, Sender};
use std::thread;


pub enum Message {
    Debug,
    Step,
}

/// Start the interpreter in a new thread. Return a sender for communication.
pub fn start(mut state: I8080State) -> Sender<Message> {
    use crate::dissasembler::dissasembly_around;
    use crate::write_adapter::WriteAdapter;
    use std::io;
    use std::fmt::Write;

    enum State {
        Debugging,
        Running,
    }

    let traced = dissasembler::trace(&state.memory, &[0x0u16, 0x8, 0x10]);
    let stdout = io::stdout();
    let mut interpreter_state = State::Debugging;

    let (send, recv) = channel();

    thread::spawn(move || loop {
        let mut w = WriteAdapter(io::BufWriter::new(stdout.lock()));
        writeln!(w).unwrap();
        dissasembly_around(&mut w, &traced, &state.memory, state.get_PC()).unwrap();
        writeln!(w).unwrap();
        state.print_state(&mut w);
        drop(w);

        let of = interpret_opcode(&mut state);
        state.set_PC(state.get_PC() + of);

        use std::sync::mpsc::TryRecvError;
        
        match interpreter_state {
            State::Debugging => loop { match recv.recv() {
                Ok(Message::Step) => break,
                Ok(Message::Debug) => { interpreter_state = State::Running; break; },
                Err(_) => return,
            }}
            State::Running => match recv.try_recv() {
                Ok(Message::Debug) => interpreter_state = State::Debugging,
                Err(TryRecvError::Disconnected) => return,
                _ => {}
            }
        }
    });

    send
}

macro_rules! as_expr {
    ($x:expr) => { $x };
}

macro_rules! ops {
    // r1 r2 | 0b11SSSDDD
    (@rule $state:expr; ($r1:ident $r2:ident | $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $state; ($($tail)*) -> ( $($accum)*
            _ if $state.get_op() == $x + 0b000001 => {
                let $r1 = &mut $state.B; let $r2 = &$state.C; $y
            },
            _ if $state.get_op() == $x + 0b000010 => {
                let $r1 = &mut $state.B; let $r2 = &$state.D; $y
            },
            _ if $state.get_op() == $x + 0b000011 => {
                let $r1 = &mut $state.B; let $r2 = &$state.E; $y
            },
            _ if $state.get_op() == $x + 0b000100 => {
                let $r1 = &mut $state.B; let $r2 = &$state.H; $y
            },
            _ if $state.get_op() == $x + 0b000101 => {
                let $r1 = &mut $state.B; let $r2 = &$state.L; $y
            },
            _ if $state.get_op() == $x + 0b000111 => {
                let $r1 = &mut $state.B; let $r2 = &$state.A; $y
            },
            _ if $state.get_op() == $x + 0b001000 => {
                let $r1 = &mut $state.C; let $r2 = &$state.B; $y
            },
            _ if $state.get_op() == $x + 0b001010 => {
                let $r1 = &mut $state.C; let $r2 = &$state.D; $y
            },
            _ if $state.get_op() == $x + 0b001011 => {
                let $r1 = &mut $state.C; let $r2 = &$state.E; $y
            },
            _ if $state.get_op() == $x + 0b001100 => {
                let $r1 = &mut $state.C; let $r2 = &$state.H; $y
            },
            _ if $state.get_op() == $x + 0b001101 => {
                let $r1 = &mut $state.C; let $r2 = &$state.L; $y
            },
            _ if $state.get_op() == $x + 0b001111 => {
                let $r1 = &mut $state.C; let $r2 = &$state.A; $y
            },
            _ if $state.get_op() == $x + 0b010000 => {
                let $r1 = &mut $state.D; let $r2 = &$state.B; $y
            },
            _ if $state.get_op() == $x + 0b010001 => {
                let $r1 = &mut $state.D; let $r2 = &$state.C; $y
            },
            _ if $state.get_op() == $x + 0b010011 => {
                let $r1 = &mut $state.D; let $r2 = &$state.E; $y
            },
            _ if $state.get_op() == $x + 0b010100 => {
                let $r1 = &mut $state.D; let $r2 = &$state.H; $y
            },
            _ if $state.get_op() == $x + 0b010101 => {
                let $r1 = &mut $state.D; let $r2 = &$state.L; $y
            },
            _ if $state.get_op() == $x + 0b010111 => {
                let $r1 = &mut $state.D; let $r2 = &$state.A; $y
            },
            _ if $state.get_op() == $x + 0b011000 => {
                let $r1 = &mut $state.E; let $r2 = &$state.B; $y
            },
            _ if $state.get_op() == $x + 0b011001 => {
                let $r1 = &mut $state.E; let $r2 = &$state.C; $y
            },
            _ if $state.get_op() == $x + 0b011010 => {
                let $r1 = &mut $state.E; let $r2 = &$state.D; $y
            },
            _ if $state.get_op() == $x + 0b011100 => {
                let $r1 = &mut $state.E; let $r2 = &$state.H; $y
            },
            _ if $state.get_op() == $x + 0b011101 => {
                let $r1 = &mut $state.E; let $r2 = &$state.L; $y
            },
            _ if $state.get_op() == $x + 0b011111 => {
                let $r1 = &mut $state.E; let $r2 = &$state.A; $y
            },
            _ if $state.get_op() == $x + 0b100000 => {
                let $r1 = &mut $state.H; let $r2 = &$state.B; $y
            },
            _ if $state.get_op() == $x + 0b100001 => {
                let $r1 = &mut $state.H; let $r2 = &$state.C; $y
            },
            _ if $state.get_op() == $x + 0b100010 => {
                let $r1 = &mut $state.H; let $r2 = &$state.D; $y
            },
            _ if $state.get_op() == $x + 0b100011 => {
                let $r1 = &mut $state.H; let $r2 = &$state.E; $y
            },
            _ if $state.get_op() == $x + 0b100101 => {
                let $r1 = &mut $state.H; let $r2 = &$state.L; $y
            },
            _ if $state.get_op() == $x + 0b100111 => {
                let $r1 = &mut $state.H; let $r2 = &$state.A; $y
            },
            _ if $state.get_op() == $x + 0b101000 => {
                let $r1 = &mut $state.L; let $r2 = &$state.B; $y
            },
            _ if $state.get_op() == $x + 0b101001 => {
                let $r1 = &mut $state.L; let $r2 = &$state.C; $y
            },
            _ if $state.get_op() == $x + 0b101010 => {
                let $r1 = &mut $state.L; let $r2 = &$state.D; $y
            },
            _ if $state.get_op() == $x + 0b101011 => {
                let $r1 = &mut $state.L; let $r2 = &$state.E; $y
            },
            _ if $state.get_op() == $x + 0b101100 => {
                let $r1 = &mut $state.L; let $r2 = &$state.H; $y
            },
            _ if $state.get_op() == $x + 0b101111 => {
                let $r1 = &mut $state.L; let $r2 = &$state.A; $y
            },
            _ if $state.get_op() == $x + 0b111000 => {
                let $r1 = &mut $state.A; let $r2 = &$state.B; $y
            },
            _ if $state.get_op() == $x + 0b111001 => {
                let $r1 = &mut $state.A; let $r2 = &$state.C; $y
            },
            _ if $state.get_op() == $x + 0b111010 => {
                let $r1 = &mut $state.A; let $r2 = &$state.D; $y
            },
            _ if $state.get_op() == $x + 0b111011 => {
                let $r1 = &mut $state.A; let $r2 = &$state.E; $y
            },
            _ if $state.get_op() == $x + 0b111100 => {
                let $r1 = &mut $state.A; let $r2 = &$state.H; $y
            },
            _ if $state.get_op() == $x + 0b111101 => {
                let $r1 = &mut $state.A; let $r2 = &$state.L; $y
            },
        ))
    };
    // r | 0b11SSS000
    (@rule $state:expr; ($r1:ident | $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $state; ($($tail)*) -> ( $($accum)*
            _ if $state.get_op() == $x + 0b000_000 => {
                let $r1 = &mut $state.B as *mut u8; $y
            },
            _ if $state.get_op() == $x + 0b001_000 => {
                let $r1 = &mut $state.C as *mut u8; $y
            },
            _ if $state.get_op() == $x + 0b010_000 => {
                let $r1 = &mut $state.D as *mut u8; $y
            },
            _ if $state.get_op() == $x + 0b011_000 => {
                let $r1 = &mut $state.E as *mut u8; $y
            },
            _ if $state.get_op() == $x + 0b100_000 => {
                let $r1 = &mut $state.H as *mut u8; $y
            },
            _ if $state.get_op() == $x + 0b101_000 => {
                let $r1 = &mut $state.L as *mut u8; $y
            },
            _ if $state.get_op() == $x + 0b111_000 => {
                let $r1 = &mut $state.A as *mut u8; $y
            },
        ))
    };
    // r 0b11000DDD
    (@rule $state:expr; ($r1:ident $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {

        ops!(@rule $state; ($($tail)*) -> ( $($accum)*
            _ if $state.get_op() == $x + 0b000 => {
                let $r1 = $state.B; $y
            },
            _ if $state.get_op() == $x + 0b001 => {
                let $r1 = $state.C; $y
            },
            _ if $state.get_op() == $x + 0b010 => {
                let $r1 = $state.D; $y
            },
            _ if $state.get_op() == $x + 0b011 => {
                let $r1 = $state.E; $y
            },
            _ if $state.get_op() == $x + 0b100 => {
                let $r1 = $state.H; $y
            },
            _ if $state.get_op() == $x + 0b101 => {
                let $r1 = $state.L; $y
            },
            _ if $state.get_op() == $x + 0b111 => {
                let $r1 = $state.A; $y
            },
        ))
    };
    // _ if ...
    (@rule $state:expr; (_ if $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $state; ($($tail)*) -> ( $($accum)*
            _ if $x => { $y },
        ))
    };
    // 0b11111111
    (@rule $state:expr; ($x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $state; ($($tail)*) -> ( $($accum)*
            $x => { $y },
        ))
    };
    // end point
    (@rule $state:expr; (_ => $y:expr) -> ($($accum:tt)*) ) => {
        as_expr!(match $state.get_op() { $($accum)* _ => $y })
    };
    (@rule $state:expr; () -> ($($accum:tt)*) ) => {
        as_expr!(match $state.get_op() { $($accum)* _ => {} })
    };
    // entry
    {$state:expr; $($tokens:tt)* } => {
        ops!(@rule $state; ($($tokens)*) -> () )
    };
}


#[allow(non_snake_case)]
// write the dissasembly of the opcode, return the next offset
fn interpret_opcode(state: &mut I8080State) -> u16 {
    ops!{ state;
        r1 r2 | 0b01000000 => { // MOV  r1, r2| Move register to register            | 01DDDSSS        |  5   
            *r1 = *r2;
            1
        },
        r 0b01110000 => { // MOV  M, r  | Move register to memory              | 01110SSS        |  7   
            let M = state.get_HL();
            state.set_memory(M, r);
            1
        },
        r | 0b01000110 => { // MOV  r, M  | Move memory to register              | 01DDD110        |  7   
            let M = state.get_HL();
            unsafe { *r = state.get_memory(M); }
            
            1
        },
        0b01110110 => { // HLT        | Halt                                 | 01110110        |  7   
            println!("{:04x}  : op {:02x} is unimplemented", state.get_PC(), state.get_op());
            1
        },
        r | 0b00000110 => { // MVI  r     | Move immediate to register              | 00DDD110        |  7   
            let immediate = state.get_u8();
            unsafe { *r = immediate; }
            2
        },
        0b00110110 => { // MVI  M     | Move immediate to memory                | 00110110        | 10   
            let immediate = state.get_u8();
            let M = state.get_HL();
            state.set_memory(M, immediate);
            2
        },
        r | 0b00000100 => { // INR  r     | Increment register                   | 00DDD100        |  5   
            unsafe {
                let (sum, _) = (*r).overflowing_add(1);
                state.set_flags_ex(sum, (*r) & 0xf == 0xf);
                *r = sum;
            }
            1
        },
        r | 0b00000101 => { // DCR  r     | Decrement register                   | 00DDD101        |  5   
            unsafe {
                let (sum, _) = (*r).overflowing_sub(1);
                state.set_flags_ex(sum, (*r) & 0xf == 0x0);
                *r = sum;
            }
            1
        },
        0b00110100 => { // INR  M     | Increment memory                     | 00110100        | 10   
            let M = state.get_HL();
            let value = state.get_memory(M);

            let (sum, _) = value.overflowing_add(1);
            state.set_flags_ex(sum, value & 0xf == 0xf);
            state.set_memory(M, sum);
            1
        },
        0b00110101 => { // DCR  M     | Decrement memory                     | 00110101        | 10   
            let M = state.get_HL();
            let value = state.get_memory(M);

            let (sum, _) = value.overflowing_sub(1);
            state.set_flags_ex(sum, value & 0xf == 0x0);
            state.set_memory(M, sum);
            1
        },
        r 0b10000000 => { // ADD  r     | Add register to A                    | 10000SSS        |  4   
            let (sum, carry) = state.A.overflowing_add(r);
            state.set_flags(sum, carry, (state.A & 0xf) + (r & 0xf) > 0xf);
            state.A = sum;
            1
        },
        r 0b10001000 => { // ADC  r     | Add register to A with carry         | 10001SSS        |  4   
            let (sum, carry) = state.A.overflowing_add(r + state.on_carry() as u8);
            state.set_flags(sum, carry, (state.A & 0xf) + (r & 0xf) + state.on_carry() as u8 > 0xf);
            state.A = sum;
            1
        },
        r 0b10010000 => { // SUB  r     | Subtract register from A             | 10010SSS        |  4   
            let (sum, carry) = state.A.overflowing_sub(r);
            state.set_flags(sum, carry, (state.A & 0xf) + (-(r as i8) as u8 & 0xf) > 0xf);
            state.A = sum;
            1
        },
        r 0b10011000 => { // SBB  r     | Subtract register from A with borrow | 10011SSS        |  4   
            let (sum, carry) = state.A.overflowing_sub(r.wrapping_add(1));
            state.set_flags(sum, carry, (state.A & 0xf) + (-(r.wrapping_add(1) as i8) as u8 & 0xf) > 0xf);
            state.A = sum;
            1
        },
        r 0b10100000 => { // ANA  r     | And register with A                  | 10100SSS        |  4   
            state.A = state.A & r;
            state.set_flags(state.A, false, state.on_aux_carry());
            1
        },
        r 0b10101000 => { // XRA  r     | Exclusive Or register with A         | 10101SSS        |  4   
            state.A = state.A ^ r;
            state.set_flags(state.A, false, state.on_aux_carry());
            1
        },
        r 0b10110000 => { // ORA  r     | Or register with A                   | 10110SSS        |  4   
            state.A = state.A | r;
            state.set_flags(state.A, false, state.on_aux_carry());
            1
        },
        r 0b10111000 => { // CMP  r     | Compare register with A              | 10111SSS        |  4   
            let (sum, carry) = state.A.overflowing_sub(r);
            state.set_flags(sum, carry, ((state.A & 0xf) + ((-(r as i8)) as u8 & 0xf)) > 0xf );
            1
        },
        0b10000110 => { // ADD  M     | Add memory to A                      | 10000110        |  7   
            let value = state.get_memory(state.get_HL());
            let (sum, carry) = state.A.overflowing_add(value);
            state.set_flags(sum, carry, (state.A & 0xf) + (value & 0xf) > 0xf);
            state.A = sum;
            1
        },
        0b10001110 => { // ADC  M     | Add memory to A with carry           | 10001110        |  7   
            let value = state.get_memory(state.get_HL());
            let (sum, carry) = state.A.overflowing_add(value + state.on_carry() as u8);
            state.set_flags(sum, carry, (state.A & 0xf) + (value & 0xf) + state.on_carry() as u8 > 0xf);
            state.A = sum;
            1
        },
        0b10010110 => { // SUB  M     | Subtract memory from A               | 10010110        |  7   
            let value = state.get_memory(state.get_HL());
            let (sum, carry) = state.A.overflowing_sub(value);
            state.set_flags(sum, carry, (state.A & 0xf) + (-(value as i8) as u8 & 0xf) > 0xf);
            state.A = sum;
            1
        },
        0b10011110 => { // SBB  M     | Subtract memory from A with borrow   | 10011110        |  7   
            let value = state.get_memory(state.get_HL());
            let (sum, carry) = state.A.overflowing_sub(value.wrapping_add(1));
            state.set_flags(sum, carry, (state.A & 0xf) + (-(value.wrapping_add(1) as i8) as u8 & 0xf) > 0xf);
            state.A = sum;
            1
        },
        0b10100110 => { // ANA  M     | And memory with A                    | 10100110        |  7   
            let value = state.get_memory(state.get_HL());
            state.A = state.A & value;
            state.set_flags(state.A, false, state.on_aux_carry());
            1
        },
        0b10101110 => { // XRA  M     | Exclusive Or memory with A           | 10101110        |  7   
            let value = state.get_memory(state.get_HL());
            state.A = state.A ^ value;
            state.set_flags(state.A, false, state.on_aux_carry());
            1
        },
        0b10110110 => { // ORA  M     | Or memory with A                     | 10110110        |  7   
            let value = state.get_memory(state.get_HL());
            state.A = state.A | value;
            state.set_flags(state.A, false, state.on_aux_carry());
            1
        },
        0b10111110 => { // CMP  M     | Compare memory with A                | 10111110        |  7   
            let value = state.get_memory(state.get_HL());
            let (sum, carry) = state.A.overflowing_sub(value);
            state.set_flags(sum, carry, ((state.A & 0xf) + ((-(value as i8)) as u8 & 0xf)) > 0xf );
            1
        },
        0b11000110 => { // ADI        | Add immediate to A                   | 11000110        |  7   
            let immediate = state.get_u8();
            let (sum, carry) = state.A.overflowing_add(immediate);
            state.set_flags(sum, carry, (state.A & 0xf) + (immediate & 0xf) > 0xf);
            state.A = sum;
            2
        },
        0b11001110 => { // ACI        | Add immediate to A with carry        | 11001110        |  7   
            let immediate = state.get_u8();
            let (sum, carry) = state.A.overflowing_add(immediate + state.on_carry() as u8);
            state.set_flags(sum, carry, (state.A & 0xf) + (immediate & 0xf) + state.on_carry() as u8 > 0xf);
            state.A = sum;
            2
        },
        0b11010110 => { // SUI        | Subtract immediate from A            | 11010110        |  7   
            let immediate = state.get_u8();
            let (sum, carry) = state.A.overflowing_sub(immediate);
            state.set_flags(sum, carry, (state.A & 0xf) + (-(immediate as i8) as u8 & 0xf) > 0xf);
            state.A = sum;
            2
        },
        0b11011110 => { // SBI        | Subtract immediate from A with borrow| 11011110        |  7   
            let immediate = state.get_u8();
            let (sum, carry) = state.A.overflowing_sub(immediate.wrapping_add(1));
            state.set_flags(sum, carry, (state.A & 0xf) + (-(immediate.wrapping_add(1) as i8) as u8 & 0xf) > 0xf);
            state.A = sum;
            2
        },
        0b11100110 => { // ANI        | And immediate with A                 | 11100110        |  7   
            let immediate = state.get_u8();
            state.A = state.A & immediate;
            state.set_flags(state.A, false, state.on_aux_carry());
            2
        },
        0b11101110 => { // XRI        | Exclusive Or immediate with A        | 11101110        |  7   
            let immediate = state.get_u8();
            state.A = state.A ^ immediate;
            state.set_flags(state.A, false, state.on_aux_carry());
            2
        },
        0b11110110 => { // ORI        | Or immediate with A                  | 11110110        |  7   
            let immediate = state.get_u8();
            state.A = state.A | immediate;
            state.set_flags(state.A, false, state.on_aux_carry());
            2
        },
        0b11111110 => { // CPI        | Compare immediate with A             | 11111110        |  7   
            let immediate = state.get_u8();
            let (sum, carry) = state.A.overflowing_sub(immediate);
            state.set_flags(sum, carry, ((state.A & 0xf) + ((-(immediate as i8)) as u8 & 0xf)) > 0xf );
            2
        },
        0b00000111 => { // RLC        | Rotate A left                        | 00000111        |  4   
            state.set_carry((state.A & 0b1000_0000) != 0);
            state.A = state.A << 1;
            1
        },
        0b00001111 => { // RRC        | Rotate A right                       | 00001111        |  4   
            state.set_carry((state.A & 0b0000_0001) != 0);
            state.A = state.A >> 1;
            1
        },
        0b00010111 => { // RAL        | Rotate A left through carry          | 00010111        |  4   
            let carry = (state.A & 0b1000_0000) != 0;
            state.A = (state.A << 1) & state.on_carry() as u8;
            state.set_carry(carry);
            1
        },
        0b00011111 => { // RAR        | Route A right through carry          | 00011111        |  4   
            let carry = (state.A & 0b0000_0001) != 0;
            state.A = (state.A >> 1) & ((state.on_carry() as u8) << 7);
            state.set_carry(carry);
            1
        },
        0b11000011 => { // JMP        | Jump unconditional                   | 11000011        | 10   
            let adress = state.get_u16();
            state.set_PC(adress);
            0
        },
        0b11011010 => { // JC         | Jump on carry                        | 11011010        | 10   
            if state.on_carry() {
                let adress = state.get_u16();
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11010010 => { // JNC        | Jump on no carry                     | 11010010        | 10   
            if !state.on_carry() {
                let adress = state.get_u16();
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11001010 => { // JZ         | Jump on zero                         | 11001010        | 10   
            if state.on_zero() {
                let adress = state.get_u16();
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11000010 => { // JNZ        | Jump on no zero                      | 11000010        | 10   
            if !state.on_zero() {
                let adress = state.get_u16();
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11110010 => { // JP         | Jump on positive                     | 11110010        | 10   
            if state.on_positive() {
                let adress = state.get_u16();
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11111010 => { // JM         | Jump on minus                        | 11111010        | 10   
            if !state.on_positive() {
                let adress = state.get_u16();
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11101010 => { // JPE        | Jump on parity even                  | 11101010        | 10   
            if state.on_parity_even() {
                let adress = state.get_u16();
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11100010 => { // JPO        | Jump on parity odd                   | 11100010        | 10   
            if !state.on_parity_even() {
                let adress = state.get_u16();
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11001101 => { // CALL       | Call unconditional                   | 11001101        | 17   
            let adress = state.get_u16();
            state.push_stack(state.get_PC() + 3);
            state.set_PC(adress);
            0
        },
        0b11011100 => { // CC         | Call on carry                        | 11011100        | 11/17
            if state.on_carry() {
                let adress = state.get_u16();
                state.push_stack(state.get_PC() + 3);
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11010100 => { // CNC        | Call on no carry        | 11010100        | 11/17
            if !state.on_carry() {
                let adress = state.get_u16();
                state.push_stack(state.get_PC() + 3);
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11001100 => { // CZ         | Call on zero                         | 11001100        | 11/17
            if state.on_zero() {
                let adress = state.get_u16();
                state.push_stack(state.get_PC() + 3);
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11000100 => { // CNZ        | Call on no zero                      | 11000100        | 11/17
            if !state.on_zero() {
                let adress = state.get_u16();
                state.push_stack(state.get_PC() + 3);
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11110100 => { // CP         | Call on positive                     | 11110100        | 11/17
            if state.on_positive() {
                let adress = state.get_u16();
                state.push_stack(state.get_PC() + 3);
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11111100 => { // CM         | Call on minus                        | 11111100        | 11/17
            if !state.on_positive() {
                let adress = state.get_u16();
                state.push_stack(state.get_PC() + 3);
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11101100 => { // CPE        | Call on parity even                  | 11101100        | 11/17
            if state.on_parity_even() {
                let adress = state.get_u16();
                state.push_stack(state.get_PC() + 3);
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11100100 => { // CPO        | Call on parity odd                   | 11100100        | 11/17
            if !state.on_parity_even() {
                let adress = state.get_u16();
                state.push_stack(state.get_PC() + 3);
                state.set_PC(adress);
                0
            } else { 3 }
        },
        0b11001001 => { // RET        | Return                               | 11001001        | 10   
            let adress = state.pop_stack();
            state.set_PC(adress);
            0
        },
        0b11011000 => { // RC         | Return on carry                      | 11011000        | 5/11 
            if state.on_carry() {
                let adress = state.pop_stack();
                state.set_PC(adress);
                0
            } else { 1 }
        },
        0b11010000 => { // RNC        | Return on no carry                   | 11010000        | 5/11 
            if !state.on_carry() {
                let adress = state.pop_stack();
                state.set_PC(adress);
                0
            } else { 1 }
        },
        0b11001000 => { // RZ         | Return on zero                       | 11001000        | 5/11 
            if state.on_zero() {
                let adress = state.pop_stack();
                state.set_PC(adress);
                0
            } else { 1 }
        },
        0b11000000 => { // RNZ        | Return on no zero                    | 11000000        | 5/11 
            if !state.on_zero() {
                let adress = state.pop_stack();
                state.set_PC(adress);
                0
            } else { 1 }
        },
        0b11110000 => { // RP         | Return on positive                   | 11110000        | 5/11 
            if state.on_positive() {
                let adress = state.pop_stack();
                state.set_PC(adress);
                0
            } else { 1 }
        },
        0b11111000 => { // RM         | Return on minus                      | 11111000        | 5/11 
            if !state.on_positive() {
                let adress = state.pop_stack();
                state.set_PC(adress);
                0
            } else { 1 }
        },
        0b11101000 => { // RPE        | Return on parity even                | 11101000        | 5/11 
            if state.on_parity_even() {
                let adress = state.pop_stack();
                state.set_PC(adress);
                0
            } else { 1 }
        },
        0b11100000 => { // RPO        | Return on parity odd                 | 11100000        | 5/11 
            if !state.on_parity_even() {
                let adress = state.pop_stack();
                state.set_PC(adress);
                0
            } else { 1 }
        },
        _ if state.get_op() & 0b11000111 == 0b11000111 => { // RST        | Restart                              | 11AAA111        | 11   
            let adress = state.get_op() & 0b00111000;
            state.push_stack(state.get_PC() + 3);
            state.set_PC(adress as u16);
            0
        },
        0b11011011 => { // IN         | Input                                | 11011011        | 10   
            let device = state.get_u8();
            println!("{:04x}  : op {:02x} is unimplemented", state.get_PC(), state.get_op());
            2
        },
        0b11010011 => { // OUT        | Output                               | 11010011        | 10   
            let device = state.get_u8();
            println!("{:04x}  : op {:02x} is unimplemented", state.get_PC(), state.get_op());
            2
        },
        0b00000001 => { // LXI  B     | Load immediate register Pair B & C   | 00000001        | 10   
            let immediate = state.get_u16();
            state.set_BC(immediate);
            3
        },
        0b00010001 => { // LXI  D     | Load immediate register pair D & E   | 00010001        | 10   
            let immediate = state.get_u16();
            state.set_DE(immediate);
            3
        },
        0b00100001 => { // LXI  H     | Load immediate register pair H & L   | 00100001        | 10   
            let immediate = state.get_u16();
            state.set_HL(immediate);
            3
        },
        0b00110001 => { // LXI  SP    | Load immediate stack pointer         | 00110001        | 10   
            let immediate = state.get_u16();
            state.set_SP(immediate);
            3
        },
        0b11000101 => { // PUSH B     | Push register Pair B & C on stack    | 11000101        | 11   
            state.push_stack(state.get_BC());
            1
        },
        0b11010101 => { // PUSH D     | Push register Pair D & E on stack    | 11010101        | 11   
            state.push_stack(state.get_DE());
            1
        },
        0b11100101 => { // PUSH H     | Push register Pair H & L on stack    | 11100101        | 11   
            state.push_stack(state.get_HL());
            1
        },
        0b11110101 => { // PUSH PSW   | Push A and Flags on stack            | 11110001        | 11   
            state.push_stack(state.get_PSW());
            1
        },
        0b11000001 => { // POP  B     | Pop register pair B & C off stack    | 11000001        | 10   
            let value = state.pop_stack();
            state.set_BC(value);
            1
        },
        0b11010001 => { // POP  D     | Pop register pair D & E off stack    | 11010001        | 10   
            let value = state.pop_stack();
            state.set_DE(value);
            1
        },
        0b11100001 => { // POP  H     | Pop register pair H & L off stick    | 11100001        | 10   
            let value = state.pop_stack();
            state.set_HL(value);
            1
        },
        0b11110001 => { // POP  PSW   | Pop A and Flags off stack            | 11110001        | 10   
            let value = state.pop_stack();
            state.set_PSW(value);
            1
        },
        0b00110010 => { // STA        | Store A direct                       | 00110010        | 13   
            let immediate = state.get_u16();
            state.set_memory(immediate, state.A);
            3
        },
        0b00111010 => { // LDA        | Load A direct                        | 00111010        | 13   
            let immediate = state.get_u16();
            state.A = state.get_memory(immediate);
            3
        },
        0b11101011 => { // XCHG       | Exchange D & E, H & L Registers      | 11101011        | 4    
            let de = state.get_DE();
            state.set_DE(state.get_HL());
            state.set_HL(de);
            1
        },
        0b11100011 => { // XTHL       | Exchange top of stack, H & L         | 11100011        | 18   
            let b2 = state.get_memory(state.get_SP() + 1);
            let b1 = state.get_memory(state.get_SP());
            state.set_memory(state.get_SP() + 1, state.H);
            state.set_memory(state.get_SP(), state.L);
            state.H = b2;
            state.L = b1;
            1
        },
        0b11111001 => { // SPHL       | H & L to stack pointer               | 11111001        | 5    
            state.set_SP(state.get_HL());
            1
        },
        0b11101001 => { // PCHL       | H & L to program counter             | 11101001        | 5    
            state.set_PC(state.get_HL());
            0
        },
        0b00001001 => { // DAD  B     | Add B & C to H & L                   | 00001001        | 10   
            let (sum, carry) = state.get_BC().overflowing_add(state.get_HL());
            state.set_carry(carry);
            state.set_HL(sum);
            1
        },
        0b00011001 => { // DAD  D     | Add D & E to H & L                   | 00011001        | 10   
            let (sum, carry) = state.get_DE().overflowing_add(state.get_HL());
            state.set_carry(carry);
            state.set_HL(sum);
            1
        },
        0b00101001 => { // DAD  H     | Add H & L to H & L                   | 00101001        | 10   
            let (sum, carry) = state.get_HL().overflowing_add(state.get_HL());
            state.set_carry(carry);
            state.set_HL(sum);
            1
        },
        0b00111001 => { // DAD  SP    | Add stack pointer to H & L           | 00111001        | 10   
            let (sum, carry) = state.get_SP().overflowing_add(state.get_HL());
            state.set_carry(carry);
            state.set_HL(sum);
            1
        },
        0b00000010 => { // STAX B     | Store A indirect                     | 00000010        | 7    
            let adress = state.get_BC();
            state.set_memory(adress, state.A);
            1
        },
        0b00010010 => { // STAX D     | Store A Indirect                     | 00010010        | 7    
            let adress = state.get_DE();
            state.set_memory(adress, state.A);
            1
        },
        0b00001010 => { // LDAX B     | Load A indirect                      | 00001010        | 7    
            let adress = state.get_BC();
            state.A = state.get_memory(adress);
            1
        },
        0b00011010 => { // LDAX D     | Load A indirect                      | 00011010        | 7    
            let adress = state.get_DE();
            state.A = state.get_memory(adress);
            1
        },
        0b00000011 => { // INX  B     | Increment B & C registers            | 00000011        | 5    
            state.set_BC(state.get_BC().wrapping_add(1));
            1
        },
        0b00010011 => { // INX  D     | Increment D & E registers            | 00010011        | 5    
            state.set_DE(state.get_DE().wrapping_add(1));
            1
        },
        0b00100011 => { // INX  H     | Increment H & L registers            | 00100011        | 5    
            state.set_HL(state.get_HL().wrapping_add(1));
            1
        },
        0b00110011 => { // INX  SP    | Increment stack pointer              | 00110011        | 5    
            state.set_SP(state.get_SP().wrapping_add(1));
            1
        },
        0b00001011 => { // DCX  B     | Decrement B & C                      | 00001011        | 5    
            state.set_BC(state.get_BC().wrapping_sub(1));
            1
        },
        0b00011011 => { // DCX  D     | Decrement D & E                      | 00011011        | 5    
            state.set_DE(state.get_DE().wrapping_sub(1));
            1
        },
        0b00101011 => { // DCX  H     | Decrement H & L                      | 00101011        | 5    
            state.set_HL(state.get_HL().wrapping_sub(1));
            1
        },
        0b00111011 => { // DCX  SP    | Decrement stack pointer              | 00111011        | 5    
            state.set_SP(state.get_SP().wrapping_sub(1));
            1
        },
        0b00101111 => { // CMA        | Complement A                         | 00101111        | 4    
            state.A = !state.A;
            1
        },
        0b00110111 => { // STC        | Set carry                            | 00110111        | 4    
            state.set_carry(true);
            1
        },
        0b00111111 => { // CMC        | Complement carry                     | 00111111        | 4    
            state.set_carry(!state.on_carry());
            1
        },
        0b00100111 => { // DAA        | Decimal adjust A                     | 00100111        | 4    
            let mut carry = state.on_carry();
            let mut aux_carry = false;
            if state.A & 0xf > 9 || state.on_aux_carry() {
                aux_carry = state.A & 0xf + 6 > 0xf;
                state.A += 6;
            }
            if state.A & 0xf0 >> 4 > 9 || state.on_carry() {
                let (sum, c) = state.A.overflowing_add(6 << 4);
                state.A = sum;
                carry = carry || c;
            }
            state.set_flags(state.A, carry, aux_carry);
            1
        },
        0b00100010 => { // SHLD       | Store H & L direct                   | 00100010        | 16   
            let adr = state.get_u16();
            state.set_memory(adr, state.L);
            state.set_memory(adr+1, state.H);
            3
        },
        0b00101010 => { // LHLD       | Load H & L direct                    | 00101010        | 16   
            let adr = state.get_u16();
            state.L = state.get_memory(adr);
            state.H = state.get_memory(adr+1);
            3
        },
        0b11111011 => { // EI         | Enable Interrupts                    | 11111011        | 4    
            println!("{:04x}  : op {:02x} is unimplemented", state.get_PC(), state.get_op());
            1
        },
        0b11110011 => { // DI         | Disable Interrupts                   | 11110011        | 4    
            println!("{:04x}  : op {:02x} is unimplemented", state.get_PC(), state.get_op());
            1
        },
        0b00000000 => { // NOP        | No operation                         | 00000000        | 4    
            1
        },        
        _ => {
            println!("<{:04x}: UNDEFINED OPCODE {:02X}>", state.get_PC(), state.get_op());
            1
        }
    }
}