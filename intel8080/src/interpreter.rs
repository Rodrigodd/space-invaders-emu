use crate::intel8080::{ I8080State, IODevices, Memory };

#[cfg(feature = "debug")]
use {
    crate::write_adapter::WriteAdapter,
    crate::dissasembler::dissasembly_around,
    crate::dissasembler,
    std::collections::HashSet,
    std::fmt::Write,
    std::io,
    std::ops::Range,
};

macro_rules! as_expr {
    ($x:expr) => { $x };
}

macro_rules! ops {
    // r1 r2 | 0b11SSSDDD
    (@rule $state:expr; $opcode:expr; ($r1:ident $r2:ident | $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $state; $opcode; ($($tail)*) -> ( $($accum)*
            _ if $opcode == $x + 0b000001 => {
                let $r1 = &mut $state.B; let $r2 = &$state.C; $y
            },
            _ if $opcode == $x + 0b000010 => {
                let $r1 = &mut $state.B; let $r2 = &$state.D; $y
            },
            _ if $opcode == $x + 0b000011 => {
                let $r1 = &mut $state.B; let $r2 = &$state.E; $y
            },
            _ if $opcode == $x + 0b000100 => {
                let $r1 = &mut $state.B; let $r2 = &$state.H; $y
            },
            _ if $opcode == $x + 0b000101 => {
                let $r1 = &mut $state.B; let $r2 = &$state.L; $y
            },
            _ if $opcode == $x + 0b000111 => {
                let $r1 = &mut $state.B; let $r2 = &$state.A; $y
            },
            _ if $opcode == $x + 0b001000 => {
                let $r1 = &mut $state.C; let $r2 = &$state.B; $y
            },
            _ if $opcode == $x + 0b001010 => {
                let $r1 = &mut $state.C; let $r2 = &$state.D; $y
            },
            _ if $opcode == $x + 0b001011 => {
                let $r1 = &mut $state.C; let $r2 = &$state.E; $y
            },
            _ if $opcode == $x + 0b001100 => {
                let $r1 = &mut $state.C; let $r2 = &$state.H; $y
            },
            _ if $opcode == $x + 0b001101 => {
                let $r1 = &mut $state.C; let $r2 = &$state.L; $y
            },
            _ if $opcode == $x + 0b001111 => {
                let $r1 = &mut $state.C; let $r2 = &$state.A; $y
            },
            _ if $opcode == $x + 0b010000 => {
                let $r1 = &mut $state.D; let $r2 = &$state.B; $y
            },
            _ if $opcode == $x + 0b010001 => {
                let $r1 = &mut $state.D; let $r2 = &$state.C; $y
            },
            _ if $opcode == $x + 0b010011 => {
                let $r1 = &mut $state.D; let $r2 = &$state.E; $y
            },
            _ if $opcode == $x + 0b010100 => {
                let $r1 = &mut $state.D; let $r2 = &$state.H; $y
            },
            _ if $opcode == $x + 0b010101 => {
                let $r1 = &mut $state.D; let $r2 = &$state.L; $y
            },
            _ if $opcode == $x + 0b010111 => {
                let $r1 = &mut $state.D; let $r2 = &$state.A; $y
            },
            _ if $opcode == $x + 0b011000 => {
                let $r1 = &mut $state.E; let $r2 = &$state.B; $y
            },
            _ if $opcode == $x + 0b011001 => {
                let $r1 = &mut $state.E; let $r2 = &$state.C; $y
            },
            _ if $opcode == $x + 0b011010 => {
                let $r1 = &mut $state.E; let $r2 = &$state.D; $y
            },
            _ if $opcode == $x + 0b011100 => {
                let $r1 = &mut $state.E; let $r2 = &$state.H; $y
            },
            _ if $opcode == $x + 0b011101 => {
                let $r1 = &mut $state.E; let $r2 = &$state.L; $y
            },
            _ if $opcode == $x + 0b011111 => {
                let $r1 = &mut $state.E; let $r2 = &$state.A; $y
            },
            _ if $opcode == $x + 0b100000 => {
                let $r1 = &mut $state.H; let $r2 = &$state.B; $y
            },
            _ if $opcode == $x + 0b100001 => {
                let $r1 = &mut $state.H; let $r2 = &$state.C; $y
            },
            _ if $opcode == $x + 0b100010 => {
                let $r1 = &mut $state.H; let $r2 = &$state.D; $y
            },
            _ if $opcode == $x + 0b100011 => {
                let $r1 = &mut $state.H; let $r2 = &$state.E; $y
            },
            _ if $opcode == $x + 0b100101 => {
                let $r1 = &mut $state.H; let $r2 = &$state.L; $y
            },
            _ if $opcode == $x + 0b100111 => {
                let $r1 = &mut $state.H; let $r2 = &$state.A; $y
            },
            _ if $opcode == $x + 0b101000 => {
                let $r1 = &mut $state.L; let $r2 = &$state.B; $y
            },
            _ if $opcode == $x + 0b101001 => {
                let $r1 = &mut $state.L; let $r2 = &$state.C; $y
            },
            _ if $opcode == $x + 0b101010 => {
                let $r1 = &mut $state.L; let $r2 = &$state.D; $y
            },
            _ if $opcode == $x + 0b101011 => {
                let $r1 = &mut $state.L; let $r2 = &$state.E; $y
            },
            _ if $opcode == $x + 0b101100 => {
                let $r1 = &mut $state.L; let $r2 = &$state.H; $y
            },
            _ if $opcode == $x + 0b101111 => {
                let $r1 = &mut $state.L; let $r2 = &$state.A; $y
            },
            _ if $opcode == $x + 0b111000 => {
                let $r1 = &mut $state.A; let $r2 = &$state.B; $y
            },
            _ if $opcode == $x + 0b111001 => {
                let $r1 = &mut $state.A; let $r2 = &$state.C; $y
            },
            _ if $opcode == $x + 0b111010 => {
                let $r1 = &mut $state.A; let $r2 = &$state.D; $y
            },
            _ if $opcode == $x + 0b111011 => {
                let $r1 = &mut $state.A; let $r2 = &$state.E; $y
            },
            _ if $opcode == $x + 0b111100 => {
                let $r1 = &mut $state.A; let $r2 = &$state.H; $y
            },
            _ if $opcode == $x + 0b111101 => {
                let $r1 = &mut $state.A; let $r2 = &$state.L; $y
            },
        ))
    };
    // r | 0b11SSS000
    (@rule $state:expr; $opcode:expr; ($r1:ident | $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $state; $opcode; ($($tail)*) -> ( $($accum)*
            _ if $opcode == $x + 0b000_000 => {
                let $r1 = &mut $state.B as *mut u8; $y
            },
            _ if $opcode == $x + 0b001_000 => {
                let $r1 = &mut $state.C as *mut u8; $y
            },
            _ if $opcode == $x + 0b010_000 => {
                let $r1 = &mut $state.D as *mut u8; $y
            },
            _ if $opcode == $x + 0b011_000 => {
                let $r1 = &mut $state.E as *mut u8; $y
            },
            _ if $opcode == $x + 0b100_000 => {
                let $r1 = &mut $state.H as *mut u8; $y
            },
            _ if $opcode == $x + 0b101_000 => {
                let $r1 = &mut $state.L as *mut u8; $y
            },
            _ if $opcode == $x + 0b111_000 => {
                let $r1 = &mut $state.A as *mut u8; $y
            },
        ))
    };
    // r 0b11000DDD
    (@rule $state:expr; $opcode:expr; ($r1:ident $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {

        ops!(@rule $state; $opcode; ($($tail)*) -> ( $($accum)*
            _ if $opcode == $x + 0b000 => {
                let $r1 = $state.B; $y
            },
            _ if $opcode == $x + 0b001 => {
                let $r1 = $state.C; $y
            },
            _ if $opcode == $x + 0b010 => {
                let $r1 = $state.D; $y
            },
            _ if $opcode == $x + 0b011 => {
                let $r1 = $state.E; $y
            },
            _ if $opcode == $x + 0b100 => {
                let $r1 = $state.H; $y
            },
            _ if $opcode == $x + 0b101 => {
                let $r1 = $state.L; $y
            },
            _ if $opcode == $x + 0b111 => {
                let $r1 = $state.A; $y
            },
        ))
    };
    // _ if ...
    (@rule $state:expr; $opcode:expr; (_ if $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $state; $opcode; ($($tail)*) -> ( $($accum)*
            _ if $x => { $y },
        ))
    };
    // 0b11111111
    (@rule $state:expr; $opcode:expr; ($x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $state; $opcode; ($($tail)*) -> ( $($accum)*
            $x => { $y },
        ))
    };
    // end point
    (@rule $state:expr; $opcode:expr; (_ => $y:expr) -> ($($accum:tt)*) ) => {
        as_expr!(match $opcode { $($accum)* _ => $y })
    };
    (@rule $state:expr; $opcode:expr; () -> ($($accum:tt)*) ) => {
        as_expr!(match $opcode { $($accum)* _ => {} })
    };
    // entry
    {$state:expr; $opcode:expr; $($tokens:tt)* } => {
        ops!(@rule $state; $opcode; ($($tokens)*) -> () )
    };
}


// const TARGET_FREQ: u64 = 2_000_000; //Hz


pub struct Interpreter<M: Memory, I: IODevices> {
    state: I8080State,
    devices: I,
    memory: M,
    clock_count: u32,
    target_clock: u32,
    #[cfg(feature = "debug")]
    debug: bool,
    #[cfg(feature = "debug")]
    breakpoints: HashSet<u16>,
    #[cfg(feature = "debug")]
    traced: Vec<Range<u16>>,
}
impl<M: Memory, I: IODevices> Interpreter<M,I> {
    pub fn new(devices: I, mut memory: M, entries: &[u16]) -> Self {

        let mut state = I8080State::new();

        state.set_PC(entries[0]);

        Self {
            #[cfg(feature = "debug")]
            debug: false,
            #[cfg(feature = "debug")]
            breakpoints: HashSet::new(),
            #[cfg(feature = "debug")]
            traced: dissasembler::trace(&memory.get_rom(), entries),

            state,
            devices,
            memory,
            clock_count: 0,
            target_clock: 0,
        }
    }

    #[cfg(feature = "debug")]
    pub fn enter_debug_mode(&mut self) {
        self.debug = true;
    }

    fn get_opcode_size_and_clock(opcode: u8) -> (u8, u8) {
        const SIZE_AND_CLOCKS: [(u8,u8); 0x100] =[
            (1, 4), (3, 10), (1, 7 ), (1, 5 ), (1, 5 ), (1, 5 ), (2, 7 ), (1, 4 ),
            (0, 4), (1, 10), (1, 7 ), (1, 5 ), (1, 5 ), (1, 5 ), (2, 7 ), (1, 4 ),
            (0, 4), (3, 10), (1, 7 ), (1, 5 ), (1, 5 ), (1, 5 ), (2, 7 ), (1, 4 ),
            (0, 4), (1, 10), (1, 7 ), (1, 5 ), (1, 5 ), (1, 5 ), (2, 7 ), (1, 4 ),
            (0, 4), (3, 10), (3, 16), (1, 5 ), (1, 5 ), (1, 5 ), (2, 7 ), (1, 4 ),
            (0, 4), (1, 10), (3, 16), (1, 5 ), (1, 5 ), (1, 5 ), (2, 7 ), (1, 4 ),
            (0, 4), (3, 10), (3, 13), (1, 5 ), (1, 10), (1, 10), (2, 10), (1, 4 ),
            (0, 4), (1, 10), (3, 13), (1, 5 ), (1, 5 ), (1, 5 ), (2, 7 ), (1, 4 ),
            (1, 5), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 7 ), (1, 5 ),
            (1, 5), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 7 ), (1, 5 ),
            (1, 5), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 7 ), (1, 5 ),
            (1, 5), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 7 ), (1, 5 ),
            (1, 5), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 7 ), (1, 5 ),
            (1, 5), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 7 ), (1, 5 ),
            (1, 7), (1, 7 ), (1, 7 ), (1, 7 ), (1, 7 ), (1, 7 ), (1, 7 ), (1, 7 ),
            (1, 5), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 5 ), (1, 7 ), (1, 5 ),
            (1, 4), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 7 ), (1, 4 ),
            (1, 4), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 7 ), (1, 4 ),
            (1, 4), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 7 ), (1, 4 ),
            (1, 4), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 7 ), (1, 4 ),
            (1, 4), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 7 ), (1, 4 ),
            (1, 4), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 7 ), (1, 4 ),
            (1, 4), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 7 ), (1, 4 ),
            (1, 4), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 4 ), (1, 7 ), (1, 4 ),
            (1, 5), (1, 10), (3, 10), (3, 10), (3, 11), (1, 11), (2, 7 ), (1, 11),
            (1, 5), (1, 10), (3, 10), (0, 10), (3, 11), (3, 17), (2, 7 ), (1, 11),
            (1, 5), (1, 10), (3, 10), (2, 10), (3, 11), (1, 11), (2, 7 ), (1, 11),
            (1, 5), (0, 10), (3, 10), (2, 10), (3, 11), (0, 17), (2, 7 ), (1, 11),
            (1, 5), (1, 10), (3, 10), (1, 18), (3, 11), (1, 11), (2, 7 ), (1, 11),
            (1, 5), (1, 5 ), (3, 10), (1, 4 ), (3, 11), (0, 17), (2, 7 ), (1, 11),
            (1, 5), (1, 10), (3, 10), (1, 4 ), (3, 11), (1, 11), (2, 7 ), (1, 11),
            (1, 5), (1, 5 ), (3, 10), (1, 4 ), (3, 11), (0, 17), (2, 7 ), (1, 11),
        ];
        SIZE_AND_CLOCKS[opcode as usize]
    }

    /// run for 'number_of_clocks' clocks
    pub fn run(&mut self, number_of_clocks: u32) {
        self.target_clock += number_of_clocks;
        while self.clock_count < self.target_clock {
            #[cfg(feature = "debug")] {
                if self.debug {
                    let stdout = std::io::stdout();
                    let mut w = WriteAdapter(io::BufWriter::new(stdout.lock()));
                    writeln!(w).unwrap();
                    dissasembly_around(&mut w, &self.traced, &self.memory.get_rom(), self.state.get_PC()).unwrap();
                    writeln!(w).unwrap();
                    self.state.print_state(&mut w);
                    drop(w);
                    loop {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        let mut input = input.trim().split_ascii_whitespace().filter(|s| !s.trim().is_empty());
                        if let Some(command) = input.next() {
                            if command.starts_with("runto") {
                                if let Some(adress) = input.next() {
                                    if let Ok(adress) = u16::from_str_radix(adress, 16) {
                                        let mut safety = 0;
                                        while self.state.get_PC() != adress {
                                            safety += 1;
                                            if safety > 100_000 {
                                                println!("safety: after 100_000 steps, it don't reach the adress {:04x} yet", adress);
                                                break;
                                            }
                                            let opcode = self.memory.read(self.state.get_PC());
                                            let (offset, clock) = Self::get_opcode_size_and_clock(opcode);
                                            self.clock_count += clock as u32;
                                            self.state.set_PC(self.state.get_PC() + offset as u16);
                                            self.interpret_opcode(opcode);
                                        }
                                    } else { println!("error: invalid adress"); }
                                } else {
                                    println!("use 'runto <ADRESS>', where <ADRESS> is the hexadecimal adress of the opcode it will stop when reached.");
                                }
                            } else if command.starts_with("interrupt") {
                                if let Some(opcode) = input.next() {
                                    if let Ok(opcode) = u8::from_str_radix(opcode, 16) {
                                        self.interrupt(opcode);
                                        break;
                                    } else { println!("error: invalid opcode"); }
                                } else {
                                    println!("use 'interrupt <OPCODE>', where <OPCODE> is the hexadecimal that opcode will be run.");
                                }
                            } else if command.starts_with("run") {
                                self.debug = false;
                                break;
                            } else if command.starts_with("bp") {
                                if let Some(adress) = input.next() {
                                    if let Ok(adress) = u16::from_str_radix(adress, 16) {
                                        self.breakpoints.insert(adress);
                                    } else { println!("error: invalid adress"); }
                                } else {
                                    println!("use 'bp <ADRESS>', where <ADRESS> is the hexadecimal adress of the opcode it will break when reached.");
                                }
                            }
                        } else {
                            break; // do one step
                        }
                    }
                }
            }
            
            
            let opcode = self.memory.read(self.state.get_PC());
            let (offset, clock) = Self::get_opcode_size_and_clock(opcode);
            self.state.set_PC(self.state.get_PC() + offset as u16);
            self.clock_count += clock as u32;
            self.interpret_opcode(opcode);

            #[cfg(feature = "debug")] {
                if self.breakpoints.contains(&self.state.get_PC()) {
                    self.enter_debug_mode();
                }
            }
        }
    }

    /// block the current thread, running the interpreter forever. 
    /// (But you can stop it using std::process::exit in some DeviceIO)
    #[inline]
    pub fn run_forever(&mut self) -> ! {
        loop {
            let opcode = self.memory.read(self.state.get_PC());
            let (offset, clock) = Self::get_opcode_size_and_clock(opcode);
            self.state.set_PC(self.state.get_PC() + offset as u16);
            self.clock_count += clock as u32;
            self.interpret_opcode(opcode);
        }
    }

    pub fn interrupt(&mut self, opcode: u8) {
        let (_, clock) = Self::get_opcode_size_and_clock(opcode);
        self.clock_count += clock as u32;
        self.interpret_opcode(opcode);
    }

    #[allow(non_snake_case)]
// write the dissasembly of the opcode, return the next offset
    fn interpret_opcode(&mut self, opcode: u8) {
        ops!{ self.state; opcode;
            r1 r2 | 0b01000000 => { // MOV  r1, r2| Move register to register            | 01DDDSSS        |  5   
                *r1 = *r2;
            },
            r 0b01110000 => { // MOV  M, r  | Move register to memory              | 01110SSS        |  7   
                let m = self.state.get_HL();
                self.memory.write(m, r);
            },
            r | 0b01000110 => { // MOV  r, M  | Move memory to register              | 01DDD110        |  7   
                let m = self.state.get_HL();
                unsafe { *r = self.memory.read(m); }
            },
            0b01110110 => { // HLT        | Halt                                 | 01110110        |  7   
                println!("{:04x}  : op {:02x} is unimplemented", self.state.get_PC(), opcode);
            },
            r | 0b00000110 => { // MVI  r     | Move immediate to register              | 00DDD110        |  7   
                let immediate = self.memory.read(self.state.get_PC()-1);
                unsafe { *r = immediate; }
            },
            0b00110110 => { // MVI  M     | Move immediate to memory                | 00110110        | 10   
                let immediate = self.memory.read(self.state.get_PC()-1);
                let m = self.state.get_HL();
                self.memory.write(m, immediate);
            },
            r | 0b00000100 => { // INR  r     | Increment register                   | 00DDD100        |  5   
                unsafe {
                    let (sum, _) = (*r).overflowing_add(1);
                    self.state.set_flags_ex(sum, (*r) & 0xf == 0xf);
                    *r = sum;
                }
            },
            r | 0b00000101 => { // DCR  r     | Decrement register                   | 00DDD101        |  5   
                unsafe {
                    let (sum, _) = (*r).overflowing_sub(1);
                    self.state.set_flags_ex(sum, (*r) & 0xf == 0x0);
                    *r = sum;
                }
            },
            0b00110100 => { // INR  M     | Increment memory                     | 00110100        | 10   
                let m = self.state.get_HL();
                let value = self.memory.read(m);

                let (sum, _) = value.overflowing_add(1);
                self.state.set_flags_ex(sum, value & 0xf == 0xf);
                self.memory.write(m, sum);
            },
            0b00110101 => { // DCR  M     | Decrement memory                     | 00110101        | 10   
                let m = self.state.get_HL();
                let value = self.memory.read(m);

                let (sum, _) = value.overflowing_sub(1);
                self.state.set_flags_ex(sum, value & 0xf == 0x0);
                self.memory.write(m, sum);
            },
            r 0b10000000 => { // ADD  r     | Add register to A                    | 10000SSS        |  4   
                let (sum, carry) = self.state.A.overflowing_add(r);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + (r & 0xf) > 0xf);
                self.state.A = sum;
            },
            r 0b10001000 => { // ADC  r     | Add register to A with carry         | 10001SSS        |  4   
                let (sum, carry) = self.state.A.overflowing_add(r.wrapping_add(self.state.on_carry() as u8));
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + (r & 0xf) + self.state.on_carry() as u8 > 0xf);
                self.state.A = sum;
            },
            r 0b10010000 => { // SUB  r     | Subtract register from A             | 10010SSS        |  4   
                let (sum, carry) = self.state.A.overflowing_sub(r);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + ((!r).wrapping_add(1) & 0xf) > 0xf);
                self.state.A = sum;
            },
            r 0b10011000 => { // SBB  r     | Subtract register from A with borrow | 10011SSS        |  4   
                let value = r.wrapping_add(self.state.on_carry() as u8);
                let (sum, carry) = self.state.A.overflowing_sub(value);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + ((!value).wrapping_add(1) & 0xf) > 0xf);
                self.state.A = sum;
            },
            r 0b10100000 => { // ANA  r     | And register with A                  | 10100SSS        |  4   
                self.state.A = self.state.A & r;
                self.state.set_flags(self.state.A, false, self.state.on_aux_carry());
            },
            r 0b10101000 => { // XRA  r     | Exclusive Or register with A         | 10101SSS        |  4   
                self.state.A = self.state.A ^ r;
                self.state.set_flags(self.state.A, false, self.state.on_aux_carry());
            },
            r 0b10110000 => { // ORA  r     | Or register with A                   | 10110SSS        |  4   
                self.state.A = self.state.A | r;
                self.state.set_flags(self.state.A, false, self.state.on_aux_carry());
            },
            r 0b10111000 => { // CMP  r     | Compare register with A              | 10111SSS        |  4   
                let (sum, carry) = self.state.A.overflowing_sub(r);
                self.state.set_flags(sum, carry, ((self.state.A & 0xf) + ((!r).wrapping_add(1) & 0xf)) > 0xf );
            },
            0b10000110 => { // ADD  M     | Add memory to A                      | 10000110        |  7   
                let value = self.memory.read(self.state.get_HL());
                let (sum, carry) = self.state.A.overflowing_add(value);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + (value & 0xf) > 0xf);
                self.state.A = sum;
            },
            0b10001110 => { // ADC  M     | Add memory to A with carry           | 10001110        |  7   
                let value = self.memory.read(self.state.get_HL());
                let (sum, carry) = self.state.A.overflowing_add(value.wrapping_add(self.state.on_carry() as u8));
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + (value & 0xf) + self.state.on_carry() as u8 > 0xf);
                self.state.A = sum;
            },
            0b10010110 => { // SUB  M     | Subtract memory from A               | 10010110        |  7   
                let value = self.memory.read(self.state.get_HL());
                let (sum, carry) = self.state.A.overflowing_sub(value);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + ((!value).wrapping_add(1) & 0xf) > 0xf);
                self.state.A = sum;
            },
            0b10011110 => { // SBB  M     | Subtract memory from A with borrow   | 10011110        |  7   
                let value = self.memory.read(self.state.get_HL()).wrapping_add(self.state.on_carry() as u8);
                let (sum, carry) = self.state.A.overflowing_sub(value);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + ((!value).wrapping_add(1) & 0xf) > 0xf);
                self.state.A = sum;
            },
            0b10100110 => { // ANA  M     | And memory with A                    | 10100110        |  7   
                let value = self.memory.read(self.state.get_HL());
                self.state.A = self.state.A & value;
                self.state.set_flags(self.state.A, false, self.state.on_aux_carry());
            },
            0b10101110 => { // XRA  M     | Exclusive Or memory with A           | 10101110        |  7   
                let value = self.memory.read(self.state.get_HL());
                self.state.A = self.state.A ^ value;
                self.state.set_flags(self.state.A, false, self.state.on_aux_carry());
            },
            0b10110110 => { // ORA  M     | Or memory with A                     | 10110110        |  7   
                let value = self.memory.read(self.state.get_HL());
                self.state.A = self.state.A | value;
                self.state.set_flags(self.state.A, false, self.state.on_aux_carry());
            },
            0b10111110 => { // CMP  M     | Compare memory with A                | 10111110        |  7   
                let value = self.memory.read(self.state.get_HL());
                let (sum, carry) = self.state.A.overflowing_sub(value);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + ((!value).wrapping_add(1) & 0xf) > 0xf );
            },
            0b11000110 => { // ADI        | Add immediate to A                   | 11000110        |  7   
                let immediate = self.memory.read(self.state.get_PC()-1);
                let (sum, carry) = self.state.A.overflowing_add(immediate);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + (immediate & 0xf) > 0xf);
                self.state.A = sum;
            },
            0b11001110 => { // ACI        | Add immediate to A with carry        | 11001110        |  7   
                let immediate = self.memory.read(self.state.get_PC()-1);
                let (sum, carry) = self.state.A.overflowing_add(immediate.wrapping_add(self.state.on_carry() as u8));
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + (immediate & 0xf) + self.state.on_carry() as u8 > 0xf);
                self.state.A = sum;
            },
            0b11010110 => { // SUI        | Subtract immediate from A            | 11010110        |  7   
                let immediate = self.memory.read(self.state.get_PC()-1);
                let (sum, carry) = self.state.A.overflowing_sub(immediate);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + ((!immediate).wrapping_add(1) & 0xf) > 0xf);
                self.state.A = sum;
            },
            0b11011110 => { // SBI        | Subtract immediate from A with borrow| 11011110        |  7   
                let immediate = self.memory.read(self.state.get_PC()-1).wrapping_add(self.state.on_carry() as u8);
                let (sum, carry) = self.state.A.overflowing_sub(immediate);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + ((!immediate).wrapping_add(1) & 0xf) > 0xf);
                self.state.A = sum;
            },
            0b11100110 => { // ANI        | And immediate with A                 | 11100110        |  7   
                let immediate = self.memory.read(self.state.get_PC()-1);
                self.state.A = self.state.A & immediate;
                self.state.set_flags(self.state.A, false, self.state.on_aux_carry());
            },
            0b11101110 => { // XRI        | Exclusive Or immediate with A        | 11101110        |  7   
                let immediate = self.memory.read(self.state.get_PC()-1);
                self.state.A = self.state.A ^ immediate;
                self.state.set_flags(self.state.A, false, self.state.on_aux_carry());
            },
            0b11110110 => { // ORI        | Or immediate with A                  | 11110110        |  7   
                let immediate = self.memory.read(self.state.get_PC()-1);
                self.state.A = self.state.A | immediate;
                self.state.set_flags(self.state.A, false, self.state.on_aux_carry());
            },
            0b11111110 => { // CPI        | Compare immediate with A             | 11111110        |  7   
                let immediate = self.memory.read(self.state.get_PC()-1);
                let (sum, carry) = self.state.A.overflowing_sub(immediate);
                self.state.set_flags(sum, carry, (self.state.A & 0xf) + ((!immediate).wrapping_add(1) & 0xf) > 0xf );
            },
            0b00000111 => { // RLC        | Rotate A left                        | 00000111        |  4   
                self.state.set_carry((self.state.A & 0b1000_0000) != 0);
                self.state.A = self.state.A.rotate_left(1);
            },
            0b00001111 => { // RRC        | Rotate A right                       | 00001111        |  4   
                self.state.set_carry((self.state.A & 0b0000_0001) != 0);
                self.state.A = self.state.A.rotate_right(1);
            },
            0b00010111 => { // RAL        | Rotate A left through carry          | 00010111        |  4   
                let carry = (self.state.A & 0b1000_0000) != 0;
                self.state.A = (self.state.A << 1) | self.state.on_carry() as u8;
                self.state.set_carry(carry);
            },
            0b00011111 => { // RAR        | Route A right through carry          | 00011111        |  4   
                let carry = (self.state.A & 0b0000_0001) != 0;
                self.state.A = (self.state.A >> 1) | ((self.state.on_carry() as u8) << 7);
                self.state.set_carry(carry);
            },
            0b11000011 => { // JMP        | Jump unconditional                   | 11000011        | 10   
                let adress = self.memory.read_u16(self.state.get_PC() - 2);
                self.state.set_PC(adress);
            },
            0b11011010 => { // JC         | Jump on carry                        | 11011010        | 10   
                if self.state.on_carry() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.set_PC(adress);
                }
            },
            0b11010010 => { // JNC        | Jump on no carry                     | 11010010        | 10   
                if !self.state.on_carry() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.set_PC(adress);
                }
            },
            0b11001010 => { // JZ         | Jump on zero                         | 11001010        | 10   
                if self.state.on_zero() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.set_PC(adress);
                }
            },
            0b11000010 => { // JNZ        | Jump on no zero                      | 11000010        | 10   
                if !self.state.on_zero() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.set_PC(adress);
                }
            },
            0b11110010 => { // JP         | Jump on positive                     | 11110010        | 10   
                if self.state.on_positive() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.set_PC(adress);
                }
            },
            0b11111010 => { // JM         | Jump on minus                        | 11111010        | 10   
                if !self.state.on_positive() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.set_PC(adress);
                }
            },
            0b11101010 => { // JPE        | Jump on parity even                  | 11101010        | 10   
                if self.state.on_parity_even() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.set_PC(adress);
                }
            },
            0b11100010 => { // JPO        | Jump on parity odd                   | 11100010        | 10   
                if !self.state.on_parity_even() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.set_PC(adress);
                }
            },
            0b11001101 => { // CALL       | Call unconditional                   | 11001101        | 17   
                let adress = self.memory.read_u16(self.state.get_PC() - 2);
                self.state.push_stack(self.state.get_PC(), &mut self.memory);
                self.state.set_PC(adress);
            },
            0b11011100 => { // CC         | Call on carry                        | 11011100        | 11/17
                if self.state.on_carry() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.push_stack(self.state.get_PC(), &mut self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11010100 => { // CNC        | Call on no carry        | 11010100        | 11/17
                if !self.state.on_carry() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.push_stack(self.state.get_PC(), &mut self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11001100 => { // CZ         | Call on zero                         | 11001100        | 11/17
                if self.state.on_zero() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.push_stack(self.state.get_PC(), &mut self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11000100 => { // CNZ        | Call on no zero                      | 11000100        | 11/17
                if !self.state.on_zero() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.push_stack(self.state.get_PC(), &mut self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11110100 => { // CP         | Call on positive                     | 11110100        | 11/17
                if self.state.on_positive() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.push_stack(self.state.get_PC(), &mut self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11111100 => { // CM         | Call on minus                        | 11111100        | 11/17
                if !self.state.on_positive() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.push_stack(self.state.get_PC(), &mut self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11101100 => { // CPE        | Call on parity even                  | 11101100        | 11/17
                if self.state.on_parity_even() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.push_stack(self.state.get_PC(), &mut self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11100100 => { // CPO        | Call on parity odd                   | 11100100        | 11/17
                if !self.state.on_parity_even() {
                    let adress = self.memory.read_u16(self.state.get_PC() - 2);
                    self.state.push_stack(self.state.get_PC(), &mut self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11001001 => { // RET        | Return                               | 11001001        | 10   
                let adress = self.state.pop_stack(&self.memory);
                self.state.set_PC(adress);
            },
            0b11011000 => { // RC         | Return on carry                      | 11011000        | 5/11 
                if self.state.on_carry() {
                    let adress = self.state.pop_stack(&self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11010000 => { // RNC        | Return on no carry                   | 11010000        | 5/11 
                if !self.state.on_carry() {
                    let adress = self.state.pop_stack(&self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11001000 => { // RZ         | Return on zero                       | 11001000        | 5/11 
                if self.state.on_zero() {
                    let adress = self.state.pop_stack(&self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11000000 => { // RNZ        | Return on no zero                    | 11000000        | 5/11 
                if !self.state.on_zero() {
                    let adress = self.state.pop_stack(&self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11110000 => { // RP         | Return on positive                   | 11110000        | 5/11 
                if self.state.on_positive() {
                    let adress = self.state.pop_stack(&self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11111000 => { // RM         | Return on minus                      | 11111000        | 5/11 
                if !self.state.on_positive() {
                    let adress = self.state.pop_stack(&self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11101000 => { // RPE        | Return on parity even                | 11101000        | 5/11 
                if self.state.on_parity_even() {
                    let adress = self.state.pop_stack(&self.memory);
                    self.state.set_PC(adress);
                }
            },
            0b11100000 => { // RPO        | Return on parity odd                 | 11100000        | 5/11 
                if !self.state.on_parity_even() {
                    let adress = self.state.pop_stack(&self.memory);
                    self.state.set_PC(adress);
                }
            },
            _ if opcode & 0b11000111 == 0b11000111 => { // RST        | Restart                              | 11AAA111        | 11   
                let adress = opcode & 0b00111000;
                self.state.push_stack(self.state.get_PC(), &mut self.memory);
                self.state.set_PC(adress as u16);
            },
            0b11011011 => { // IN         | Input                                | 11011011        | 10   
                let device = self.memory.read(self.state.get_PC()-1);
                self.state.A = self.devices.read(device);
            },
            0b11010011 => { // OUT        | Output                               | 11010011        | 10   
                let device = self.memory.read(self.state.get_PC()-1);
                self.devices.write(device, self.state.A);
            },
            0b00000001 => { // LXI  B     | Load immediate register Pair B & C   | 00000001        | 10   
                let immediate = self.memory.read_u16(self.state.get_PC() - 2);
                self.state.set_BC(immediate);
            },
            0b00010001 => { // LXI  D     | Load immediate register pair D & E   | 00010001        | 10   
                let immediate = self.memory.read_u16(self.state.get_PC() - 2);
                self.state.set_DE(immediate);
            },
            0b00100001 => { // LXI  H     | Load immediate register pair H & L   | 00100001        | 10   
                let immediate = self.memory.read_u16(self.state.get_PC() - 2);
                self.state.set_HL(immediate);
            },
            0b00110001 => { // LXI  SP    | Load immediate stack pointer         | 00110001        | 10   
                let immediate = self.memory.read_u16(self.state.get_PC() - 2);
                self.state.set_SP(immediate);
            },
            0b11000101 => { // PUSH B     | Push register Pair B & C on stack    | 11000101        | 11   
                self.state.push_stack(self.state.get_BC(), &mut self.memory);
            },
            0b11010101 => { // PUSH D     | Push register Pair D & E on stack    | 11010101        | 11   
                self.state.push_stack(self.state.get_DE(), &mut self.memory);
            },
            0b11100101 => { // PUSH H     | Push register Pair H & L on stack    | 11100101        | 11   
                self.state.push_stack(self.state.get_HL(), &mut self.memory);
            },
            0b11110101 => { // PUSH PSW   | Push A and Flags on stack            | 11110001        | 11   
                self.state.push_stack(self.state.get_PSW(), &mut self.memory);
            },
            0b11000001 => { // POP  B     | Pop register pair B & C off stack    | 11000001        | 10   
                let value = self.state.pop_stack(&self.memory);
                self.state.set_BC(value);
            },
            0b11010001 => { // POP  D     | Pop register pair D & E off stack    | 11010001        | 10   
                let value = self.state.pop_stack(&self.memory);
                self.state.set_DE(value);
            },
            0b11100001 => { // POP  H     | Pop register pair H & L off stick    | 11100001        | 10   
                let value = self.state.pop_stack(&self.memory);
                self.state.set_HL(value);
            },
            0b11110001 => { // POP  PSW   | Pop A and Flags off stack            | 11110001        | 10   
                let value = self.state.pop_stack(&self.memory);
                self.state.set_PSW(value);
            },
            0b00110010 => { // STA        | Store A direct                       | 00110010        | 13   
                let immediate = self.memory.read_u16(self.state.get_PC() - 2);
                self.memory.write(immediate, self.state.A);
            },
            0b00111010 => { // LDA        | Load A direct                        | 00111010        | 13   
                let immediate = self.memory.read_u16(self.state.get_PC() - 2);
                self.state.A = self.memory.read(immediate);
            },
            0b11101011 => { // XCHG       | Exchange D & E, H & L Registers      | 11101011        | 4    
                let de = self.state.get_DE();
                self.state.set_DE(self.state.get_HL());
                self.state.set_HL(de);
            },
            0b11100011 => { // XTHL       | Exchange top of stack, H & L         | 11100011        | 18   
                let b2 = self.memory.read(self.state.get_SP() + 1);
                let b1 = self.memory.read(self.state.get_SP());
                self.memory.write(self.state.get_SP() + 1, self.state.H);
                self.memory.write(self.state.get_SP(), self.state.L);
                self.state.H = b2;
                self.state.L = b1;
            },
            0b11111001 => { // SPHL       | H & L to stack pointer               | 11111001        | 5    
                self.state.set_SP(self.state.get_HL());
            },
            0b11101001 => { // PCHL       | H & L to program counter             | 11101001        | 5    
                self.state.set_PC(self.state.get_HL());
            },
            0b00001001 => { // DAD  B     | Add B & C to H & L                   | 00001001        | 10   
                let (sum, carry) = self.state.get_BC().overflowing_add(self.state.get_HL());
                self.state.set_carry(carry);
                self.state.set_HL(sum);
            },
            0b00011001 => { // DAD  D     | Add D & E to H & L                   | 00011001        | 10   
                let (sum, carry) = self.state.get_DE().overflowing_add(self.state.get_HL());
                self.state.set_carry(carry);
                self.state.set_HL(sum);
            },
            0b00101001 => { // DAD  H     | Add H & L to H & L                   | 00101001        | 10   
                let (sum, carry) = self.state.get_HL().overflowing_add(self.state.get_HL());
                self.state.set_carry(carry);
                self.state.set_HL(sum);
            },
            0b00111001 => { // DAD  SP    | Add stack pointer to H & L           | 00111001        | 10   
                let (sum, carry) = self.state.get_SP().overflowing_add(self.state.get_HL());
                self.state.set_carry(carry);
                self.state.set_HL(sum);
            },
            0b00000010 => { // STAX B     | Store A indirect                     | 00000010        | 7    
                let adress = self.state.get_BC();
                self.memory.write(adress, self.state.A);
            },
            0b00010010 => { // STAX D     | Store A Indirect                     | 00010010        | 7    
                let adress = self.state.get_DE();
                self.memory.write(adress, self.state.A);
            },
            0b00001010 => { // LDAX B     | Load A indirect                      | 00001010        | 7    
                let adress = self.state.get_BC();
                self.state.A = self.memory.read(adress);
            },
            0b00011010 => { // LDAX D     | Load A indirect                      | 00011010        | 7    
                let adress = self.state.get_DE();
                self.state.A = self.memory.read(adress);
            },
            0b00000011 => { // INX  B     | Increment B & C registers            | 00000011        | 5    
                self.state.set_BC(self.state.get_BC().wrapping_add(1));
            },
            0b00010011 => { // INX  D     | Increment D & E registers            | 00010011        | 5    
                self.state.set_DE(self.state.get_DE().wrapping_add(1));
            },
            0b00100011 => { // INX  H     | Increment H & L registers            | 00100011        | 5    
                self.state.set_HL(self.state.get_HL().wrapping_add(1));
            },
            0b00110011 => { // INX  SP    | Increment stack pointer              | 00110011        | 5    
                self.state.set_SP(self.state.get_SP().wrapping_add(1));
            },
            0b00001011 => { // DCX  B     | Decrement B & C                      | 00001011        | 5    
                self.state.set_BC(self.state.get_BC().wrapping_sub(1));
            },
            0b00011011 => { // DCX  D     | Decrement D & E                      | 00011011        | 5    
                self.state.set_DE(self.state.get_DE().wrapping_sub(1));
            },
            0b00101011 => { // DCX  H     | Decrement H & L                      | 00101011        | 5    
                self.state.set_HL(self.state.get_HL().wrapping_sub(1));
            },
            0b00111011 => { // DCX  SP    | Decrement stack pointer              | 00111011        | 5    
                self.state.set_SP(self.state.get_SP().wrapping_sub(1));
            },
            0b00101111 => { // CMA        | Complement A                         | 00101111        | 4    
                self.state.A = !self.state.A;
            },
            0b00110111 => { // STC        | Set carry                            | 00110111        | 4    
                self.state.set_carry(true);
            },
            0b00111111 => { // CMC        | Complement carry                     | 00111111        | 4    
                self.state.set_carry(!self.state.on_carry());
            },
            0b00100111 => { // DAA        | Decimal adjust A                     | 00100111        | 4    
                let mut carry = self.state.on_carry();
                let mut aux_carry = false;
                if self.state.A & 0xf > 9 || self.state.on_aux_carry() {
                    aux_carry = (self.state.A & 0xf) + 6 > 0xf;
                    self.state.A += 6;
                }
                if (self.state.A & 0xf0) >> 4 > 9 || self.state.on_carry() {
                    let (sum, c) = self.state.A.overflowing_add(6 << 4);
                    self.state.A = sum;
                    carry = carry || c;
                }
                self.state.set_flags(self.state.A, carry, aux_carry);
            },
            0b00100010 => { // SHLD       | Store H & L direct                   | 00100010        | 16   
                let adr = self.memory.read_u16(self.state.get_PC() - 2);
                self.memory.write(adr, self.state.L);
                self.memory.write(adr+1, self.state.H);
            },
            0b00101010 => { // LHLD       | Load H & L direct                    | 00101010        | 16   
                let adr = self.memory.read_u16(self.state.get_PC() - 2);
                self.state.L = self.memory.read(adr);
                self.state.H = self.memory.read(adr+1);
            },
            0b11111011 => { // EI         | Enable Interrupts                    | 11111011        | 4    
                self.state.interrupt_enabled = true;
            },
            0b11110011 => { // DI         | Disable Interrupts                   | 11110011        | 4    
                self.state.interrupt_enabled = false;
            },
            0b00000000 => { // NOP        | No operation                         | 00000000        | 4    
            },        
            _ => {
                println!("<{:04x}: UNDEFINED OPCODE {:02X}>", self.state.get_PC(), opcode);
            }
        }
    }
}