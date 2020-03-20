use std::fmt;
use std::fmt::Write;
use std::ops::Range;

pub fn dissasembly_around<W: Write>(w: &mut W, traced: &Vec<Range<u16>>, rom: &[u8], pc: u16) -> Result<(), fmt::Error> {
    if let Some(range) = traced.iter().find(|&r| r.contains(&pc)) {
        let mut i: u16 = range.start;
        let mut ring = [i;7];
        let mut r = 1;
        while i < pc {
            i += (trace_opcode(i, rom).0 & 0b11) as u16;
            ring[r%ring.len()] = i;
            r += 1;
        }
        if i != pc {
            writeln!(w, "dissasembly fails, starts = {:04x}, i = {:04x}, pc = {:04x}", range.start, i, pc)?;
        }
        i = ring[r%ring.len()];
        r = if r >= ring.len() { 0 } else { ring.len() - r };
        for _ in 0..r {
            println!("      :");
        }
        while r < 13 {
            r += 1;
            if i == pc {
                write!(w, "{:04x} >> ", i).unwrap();
            } else {
                write!(w, "{:04x}  : ", i).unwrap();
            }
            i += dissasembly_opcode(w, i, rom)? as u16;
            if i >= range.end {
                break;
            }
        }
        for _ in r..13 {
            println!("      :");
        }
    } else {
        writeln!(w, "out of traced memory")?;
        let mut i = pc;
        write!(w, "{:04x} >> ", i).unwrap();
        i += dissasembly_opcode(w, i, rom)? as u16;
        for _ in 0..6 {
            write!(w, "{:04x}  : ", i).unwrap();
            i += dissasembly_opcode(w, i, rom)? as u16;
        }
    }
    Ok(())
}

pub fn dissasembly<W: Write>(w: &mut W, rom: &[u8]) -> Result<(), fmt::Error> {
    let mut pc = 0;
    let traced = trace(rom);

    for Range {start, end} in traced {
        if pc != 0 {
            writeln!(w)?;
            writeln!(w, "...")?;
            writeln!(w)?;
        }
        pc = start;
        while pc < end {
            write!(w, "{:04x} : {:02x}    ", pc, rom[pc as usize]).unwrap();
            let offset = dissasembly_opcode(w, pc, rom)?;
            if offset != 0 {
                if pc as usize >= rom.len() {
                    break;
                }
                pc = pc + offset as u16;
            }
        }
    }
    Ok(())

}

pub fn trace(rom: &[u8]) -> Vec<Range<u16>> {
    fn add_next_to_read(pc: u16, offset: u8, read: &mut Vec<Range<u16>>) -> bool {
        match read.binary_search_by_key(&pc, |r: &Range<u16>| r.end) {
            Ok(i) =>  {
                if i + 1 < read.len() && read[i].end == read[i+1].start {
                    read[i].end = read[i+1].end;
                    read.remove(i+1);
                    return false;
                } else {
                    read[i].end = pc + offset as u16;
                }
            },
            Err(_) => {
                eprintln!("add_next_to_read assuption failed at {:04x}", pc);
                // println!("-- fail!!");
            },
        }
        // println!("{:04x?}", read);
        true
    }

    fn add_jump_to_read(pc: u16, read: &mut Vec<Range<u16>>) {
        match read.binary_search_by_key(&pc, |r: &Range<u16>| r.start) {
            Ok(_) =>  {
                eprintln!("add_jump_to_read assuption failed at {:04x}", pc);
                // println!("-- jump fail!!");
            },
            Err(i) => {
                read.insert(i, pc..pc);
            },
        }
        // println!("{:04x?}", read);
        // println!("--jump {:04x}", pc);
    }

    fn check_read(pc: u16, read: &Vec<Range<u16>>) -> bool {
        use std::cmp::Ordering;
        read.binary_search_by(|r: &Range<u16>| 
            if pc < r.start { Ordering::Greater } 
            else if pc >= r.end{ Ordering::Less }
            else { Ordering::Equal }
        ).is_ok()
        // read[i-1].contains(&pc) || (i < read.len() && read[i].start == pc)
    }

    let mut pc = 0u16;
    let mut read = vec![0..0];
    let mut jumps = Vec::new();
    let mut safety_counter = 0;

    'd: loop {
        safety_counter += 1;
        if safety_counter > 0x8000 {
            panic!("Infinite Loop!!!");
        }

        let (offset, jmp) = trace_opcode(pc, rom);
        if let Some(jmp) = jmp {
            if jmp < 0x4000 {
                let i = match jumps.binary_search(&pc) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                jumps.insert(i, jmp);
            }
        }
        if add_next_to_read(pc, offset & 0b11, &mut read) && offset < 8 {
            pc = pc + offset as u16;
        } else {
            while let Some(jmp) = jumps.pop() {
                if !check_read(jmp, &read) {
                    pc = jmp;
                    add_jump_to_read(jmp, &mut read);
                    continue 'd;
                }
            }
            break;
        }
    }

    read
}

fn get_u16(index: u16, rom: &[u8]) -> u16 {
    u16::from_le_bytes(
        [rom[index as usize + 1], rom[index as usize + 2]]
    )
}

fn get_u8(index: u16, rom: &[u8]) -> u8 {
    rom[index as usize + 1]
}

fn jump_instruction<W: Write>(w: &mut W, name: &str, pc: u16, rom: &[u8]) -> Result<u8, fmt::Error> {
    let adr = get_u16(pc, rom);
    writeln!(w, "{:<4} {:04x}", name, adr)?;
    Ok(3)
}

// return the next offset and jump destiny, if any
fn trace_opcode(pc: u16, rom: &[u8]) -> (u8, Option<u16>) {
    match rom[pc as usize] {
        0x00 | 0x02 | 0x03 | 0x04 | 0x05 | 0x07 | 0x09 | 0x0a | 0x0b | 0x0c | 0x0d | 0x0f | 0x12 | 0x13 | 0x14 | 0x15 | 0x17 | 0x19 | 0x1a | 0x1b | 0x1c | 0x1d | 0x1f | 0x23 | 0x24 | 0x25 | 0x27 | 0x29 | 0x2b | 0x2c | 0x2d | 0x2f | 0x33 | 0x34 | 0x35 | 0x37 | 0x39 | 0x3b | 0x3c | 0x3d | 0x3f | 0x40 | 0x41 | 0x42 | 0x43 | 0x44 | 0x45 | 0x46 | 0x47 | 0x48 | 0x49 | 0x4a | 0x4b | 0x4c | 0x4d | 0x4e | 0x4f | 0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x57 | 0x58 | 0x59 | 0x5a | 0x5b | 0x5c | 0x5d | 0x5e | 0x5f | 0x60 | 0x61 | 0x62 | 0x63 | 0x64 | 0x65 | 0x66 | 0x67 | 0x68 | 0x69 | 0x6a | 0x6b | 0x6c | 0x6d | 0x6e | 0x6f | 0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x76 | 0x77 | 0x78 | 0x79 | 0x7a | 0x7b | 0x7c | 0x7d | 0x7e | 0x7f | 0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x86 | 0x87 | 0x88 | 0x89 | 0x8a | 0x8b | 0x8c | 0x8d | 0x8e | 0x8f | 0x90 | 0x91 | 0x92 | 0x93 | 0x94 | 0x95 | 0x96 | 0x97 | 0x98 | 0x99 | 0x9a | 0x9b | 0x9c | 0x9d | 0x9e | 0x9f | 0xa0 | 0xa1 | 0xa2 | 0xa3 | 0xa4 | 0xa5 | 0xa6 | 0xa7 | 0xa8 | 0xa9 | 0xaa | 0xab | 0xac | 0xad | 0xae | 0xaf | 0xb0 | 0xb1 | 0xb2 | 0xb3 | 0xb4 | 0xb5 | 0xb6 | 0xb7 | 0xb8 | 0xb9 | 0xba | 0xbb | 0xbc | 0xbd | 0xbe | 0xbf | 0xc0 | 0xc1 | 0xc5 | 0xc7 | 0xc8 | 0xcf | 0xd0 | 0xd1 | 0xd5 | 0xd7 | 0xd8 | 0xdf | 0xe0 | 0xe1 | 0xe3 | 0xe5 | 0xe7 | 0xe8 | 0xe9 | 0xeb | 0xef | 0xf0 | 0xf1 | 0xf3 | 0xf5 | 0xf7 | 0xf8 | 0xf9 | 0xfb | 0xff => {
            (1, None)
        }
        0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e | 0xc6 | 0xce | 0xd3 | 0xd6 | 0xdb | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
            (2, None)
        }
        0x01 | 0x11 | 0x21 | 0x22 | 0x2a | 0x31 | 0x32 | 0x3a | 0xc4 | 0xcc | 0xd4 | 0xdc | 0xe4 | 0xec | 0xf4 | 0xfc => {
            (3, None)
        }
        0xc2 | 0xca | 0xd2 | 0xda | 0xe2 | 0xea | 0xf2 | 0xfa => { // JUMPS IF
            let adr = get_u16(pc, rom);
            (3, Some(adr))
        }
        0xc3 => { // JMP
            let adr = get_u16(pc, rom);
            (8 + 3, Some(adr)) // offsets > 8 indicate the program don't continue to the next opcode
        }
        0xc9 => { // RET
            (8 + 1, None)
        }
        0xcd => { // CALL adr
            let adr = get_u16(pc, rom);
            (3, Some(adr))
        }
        _ => {
            (8 + 1, None)
        }
    }
}

macro_rules! as_expr {
    ($x:expr) => { $x };
}

macro_rules! ops {
    // r1 r2 | 0b11SSSDDD
    (@rule $rom:expr; $pc:expr; ($r1:ident $r2:ident | $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $rom; $pc; ($($tail)*) -> ( $($accum)*
            _ if $rom[$pc as usize] == $x + 0b000000 => {
                let $r1 = "B"; let $r2 = "B"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b000001 => {
                let $r1 = "C"; let $r2 = "B"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b000010 => {
                let $r1 = "D"; let $r2 = "B"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b000011 => {
                let $r1 = "E"; let $r2 = "B"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b000100 => {
                let $r1 = "H"; let $r2 = "B"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b000101 => {
                let $r1 = "L"; let $r2 = "B"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b000111 => {
                let $r1 = "A"; let $r2 = "B"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b001000 => {
                let $r1 = "B"; let $r2 = "C"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b001001 => {
                let $r1 = "C"; let $r2 = "C"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b001010 => {
                let $r1 = "D"; let $r2 = "C"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b001011 => {
                let $r1 = "E"; let $r2 = "C"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b001100 => {
                let $r1 = "H"; let $r2 = "C"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b001101 => {
                let $r1 = "L"; let $r2 = "C"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b001111 => {
                let $r1 = "A"; let $r2 = "C"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b010000 => {
                let $r1 = "B"; let $r2 = "D"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b010001 => {
                let $r1 = "C"; let $r2 = "D"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b010010 => {
                let $r1 = "D"; let $r2 = "D"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b010011 => {
                let $r1 = "E"; let $r2 = "D"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b010100 => {
                let $r1 = "H"; let $r2 = "D"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b010101 => {
                let $r1 = "L"; let $r2 = "D"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b010111 => {
                let $r1 = "A"; let $r2 = "D"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b011000 => {
                let $r1 = "B"; let $r2 = "E"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b011001 => {
                let $r1 = "C"; let $r2 = "E"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b011010 => {
                let $r1 = "D"; let $r2 = "E"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b011011 => {
                let $r1 = "E"; let $r2 = "E"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b011100 => {
                let $r1 = "H"; let $r2 = "E"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b011101 => {
                let $r1 = "L"; let $r2 = "E"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b011111 => {
                let $r1 = "A"; let $r2 = "E"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b100000 => {
                let $r1 = "B"; let $r2 = "H"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b100001 => {
                let $r1 = "C"; let $r2 = "H"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b100010 => {
                let $r1 = "D"; let $r2 = "H"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b100011 => {
                let $r1 = "E"; let $r2 = "H"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b100100 => {
                let $r1 = "H"; let $r2 = "H"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b100101 => {
                let $r1 = "L"; let $r2 = "H"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b100111 => {
                let $r1 = "A"; let $r2 = "H"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b101000 => {
                let $r1 = "B"; let $r2 = "L"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b101001 => {
                let $r1 = "C"; let $r2 = "L"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b101010 => {
                let $r1 = "D"; let $r2 = "L"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b101011 => {
                let $r1 = "E"; let $r2 = "L"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b101100 => {
                let $r1 = "H"; let $r2 = "L"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b101101 => {
                let $r1 = "L"; let $r2 = "L"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b101111 => {
                let $r1 = "A"; let $r2 = "L"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b111000 => {
                let $r1 = "B"; let $r2 = "A"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b111001 => {
                let $r1 = "C"; let $r2 = "A"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b111010 => {
                let $r1 = "D"; let $r2 = "A"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b111011 => {
                let $r1 = "E"; let $r2 = "A"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b111100 => {
                let $r1 = "H"; let $r2 = "A"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b111101 => {
                let $r1 = "L"; let $r2 = "A"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b111111 => {
                let $r1 = "A"; let $r2 = "A"; $y
            },
        ))
    };
    // r | 0b11SSS000
    (@rule $rom:expr; $pc:expr; ($r1:ident | $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $rom; $pc; ($($tail)*) -> ( $($accum)*
            _ if $rom[$pc as usize] == $x + 0b000_000 => {
                let $r1 = "B"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b001_000 => {
                let $r1 = "C"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b010_000 => {
                let $r1 = "D"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b011_000 => {
                let $r1 = "E"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b100_000 => {
                let $r1 = "H"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b101_000 => {
                let $r1 = "L"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b111_000 => {
                let $r1 = "A"; $y
            },
        ))
    };
    // r 0b11000DDD
    (@rule $rom:expr; $pc:expr; ($r1:ident $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $rom; $pc; ($($tail)*) -> ( $($accum)*
            _ if $rom[$pc as usize] == $x + 0b000 => {
                let $r1 = "B"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b001 => {
                let $r1 = "C"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b010 => {
                let $r1 = "D"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b011 => {
                let $r1 = "E"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b100 => {
                let $r1 = "H"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b101 => {
                let $r1 = "L"; $y
            },
            _ if $rom[$pc as usize] == $x + 0b111 => {
                let $r1 = "A"; $y
            },
        ))
    };
    (@rule $rom:expr; $pc:expr; (_ if $x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $rom; $pc; ($($tail)*) -> ( $($accum)*
            _ if $x => { $y },
        ))
    };
    (@rule $rom:expr; $pc:expr; ($x:expr => $y:expr, $($tail:tt)*) -> ($($accum:tt)*) ) => {
        ops!(@rule $rom; $pc; ($($tail)*) -> ( $($accum)*
            $x => { $y },
        ))
    };
    (@rule $rom:expr; $pc:expr; (_ => $y:expr) -> ($($accum:tt)*) ) => {
        as_expr!(match $rom [$pc as usize] { $($accum)* _ => $y })
    };
    (@rule $rom:expr; $pc:expr; () -> ($($accum:tt)*) ) => {
        as_expr!(match $rom [$pc as usize] { $($accum)* _ => {} })
    };
    
    {$rom:expr; $pc:expr; $($tokens:tt)* } => {
        ops!(@rule $rom; $pc; ($($tokens)*) -> () )
    };
}


// write the dissasembly of the opcode, return the next offset
fn dissasembly_opcode<W: Write>(w: &mut W, pc: u16, rom: &[u8]) -> Result<u8, fmt::Error> {
    Ok(ops!{ rom; pc;
        r1 r2 | 0b01000000 => { // MOV  r1, r2| Move register to register            | 01DDDSSS        |  5   
            writeln!(w, "MOV  {}, {}", r1, r2)?; 1
        },
        r 0b01110000 => { // MOV  M, r  | Move register to memory              | 01110SSS        |  7   
            writeln!(w, "MOV  M, {}", r)?; 1
        },
        r | 0b01000110 => { // MOV  r, M  | Move memory to register              | 01DDD110        |  7   
            writeln!(w, "MOV  {}, M  ", r)?; 1
        },
        0b01110110 => { // HLT        | Halt                                 | 01110110        |  7   
            writeln!(w, "HLT        ")?; 1
        },
        r | 0b00000110 => { // MVI  r     | Move immediate register              | 00DDD110        |  7   
            let immediate = get_u8(pc, rom);
            writeln!(w, "MVI  {} {:02x}", r, immediate)?; 2
        },
        0b00110110 => { // MVI  M     | Move immediate memory                | 00110110        | 10   
            let immediate = get_u8(pc, rom);
            writeln!(w, "MVI  M {:02x}", immediate)?; 2
        },
        r | 0b00000100 => { // INR  r     | Increment register                   | 00DDD100        |  5   
            writeln!(w, "INR  {}     ", r)?; 1
        },
        r | 0b00000101 => { // DCR  r     | Decrement register                   | 00DDD101        |  5   
            writeln!(w, "DCR  {}     ", r)?; 1
        },
        0b00110100 => { // INR  M     | Increment memory                     | 00110100        | 10   
            let adr = get_u16(pc, rom);
            writeln!(w, "INR  {:04x}     ",adr)?; 3
        },
        0b00110101 => { // DCR  M     | Decrement memory                     | 00110101        | 10   
            let adr = get_u16(pc, rom);
            writeln!(w, "DCR  {:04x}     ",adr)?; 3
        },
        r 0b10000000 => { // ADD  r     | Add register to A                    | 10000SSS        |  4   
            writeln!(w, "ADD  {}     ", r)?; 1
        },
        r 0b10001000 => { // ADC  r     | Add register to A with carry         | 10001SSS        |  4   
            writeln!(w, "ADC  {}     ", r)?; 1
        },
        r 0b10010000 => { // SUB  r     | Subtract register from A             | 10010SSS        |  4   
            writeln!(w, "SUB  {}     ", r)?; 1
        },
        r 0b10011000 => { // SBB  r     | Subtract register from A with borrow | 10011SSS        |  4   
            writeln!(w, "SBB  {}     ", r)?; 1
        },
        r 0b10100000 => { // ANA  r     | And register with A                  | 10100SSS        |  4   
            writeln!(w, "ANA  {}     ", r)?; 1
        },
        r 0b10101000 => { // XRA  r     | Exclusive Or register with A         | 10101SSS        |  4   
            writeln!(w, "XRA  {}     ", r)?; 1
        },
        r 0b10110000 => { // ORA  r     | Or register with A                   | 10110SSS        |  4   
            writeln!(w, "ORA  {}     ", r)?; 1
        },
        r 0b10111000 => { // CMP  r     | Compare register with A              | 10111SSS        |  4   
            writeln!(w, "CMP  {}     ", r)?; 1
        },
        0b10000110 => { // ADD  M     | Add memory to A                      | 10000110        |  7   
            let adr = get_u16(pc, rom);
            writeln!(w, "ADD  {:04x}     ",adr)?; 3
        },
        0b10001110 => { // ADC  M     | Add memory to A with carry           | 10001110        |  7   
            let adr = get_u16(pc, rom);
            writeln!(w, "ADC  {:04x}     ",adr)?; 3
        },
        0b10010110 => { // SUB  M     | Subtract memory from A               | 10010110        |  7   
            let adr = get_u16(pc, rom);
            writeln!(w, "SUB  {:04x}     ",adr)?; 3
        },
        0b10011110 => { // SBB  M     | Subtract memory from A with borrow   | 10011110        |  7   
            let adr = get_u16(pc, rom);
            writeln!(w, "SBB  {:04x}     ",adr)?; 3
        },
        0b10100110 => { // ANA  M     | And memory with A                    | 10100110        |  7   
            let adr = get_u16(pc, rom);
            writeln!(w, "ANA  {:04x}     ",adr)?; 3
        },
        0b10101110 => { // XRA  M     | Exclusive Or memory with A           | 10101110        |  7   
            let adr = get_u16(pc, rom);
            writeln!(w, "XRA  {:04x}     ",adr)?; 3
        },
        0b10110110 => { // ORA  M     | Or memory with A                     | 10110110        |  7   
            let adr = get_u16(pc, rom);
            writeln!(w, "ORA  {:04x}     ",adr)?; 3
        },
        0b10111110 => { // CMP  M     | Compare memory with A                | 10111110        |  7   
            let adr = get_u16(pc, rom);
            writeln!(w, "C{:04x}P  M     ",adr)?; 3
        },
        0b11000110 => { // ADI        | Add immediate to A                   | 11000110        |  7   
            let immediate = get_u8(pc, rom);
            writeln!(w, "ADI {:02x}", immediate)?; 2
        },
        0b11001110 => { // ACI        | Add immediate to A with carry        | 11001110        |  7   
            let immediate = get_u8(pc, rom);
            writeln!(w, "ACI {:02x}", immediate)?; 2
        },
        0b11010110 => { // SUI        | Subtract immediate from A            | 11010110        |  7   
            let immediate = get_u8(pc, rom);
            writeln!(w, "SUI {:02x}", immediate)?; 2
        },
        0b11011110 => { // SBI        | Subtract immediate from A with borrow| 11011110        |  7   
            let immediate = get_u8(pc, rom);
            writeln!(w, "SBI {:02x}", immediate)?; 2
        },
        0b11100110 => { // ANI        | And immediate with A                 | 11100110        |  7   
            let immediate = get_u8(pc, rom);
            writeln!(w, "ANI {:02x}", immediate)?; 2
        },
        0b11101110 => { // XRI        | Exclusive Or immediate with A        | 11101110        |  7   
            let immediate = get_u8(pc, rom);
            writeln!(w, "XRI {:02x}", immediate)?; 2
        },
        0b11110110 => { // ORI        | Or immediate with A                  | 11110110        |  7   
            let immediate = get_u8(pc, rom);
            writeln!(w, "ORI {:02x}", immediate)?; 2
        },
        0b11111110 => { // CPI        | Compere immediate with A             | 11111110        |  7   
            let immediate = get_u8(pc, rom);
            writeln!(w, "CPI {:02x}", immediate)?; 2
        },
        0b00000111 => { // RLC        | Rotate A left                        | 00000111        |  4   
            writeln!(w, "RLC        ")?; 1
        },
        0b00001111 => { // RRC        | Rotate A right                       | 00001111        |  4   
            writeln!(w, "RRC        ")?; 1
        },
        0b00010111 => { // RAL        | Rotate A left through carry          | 00010111        |  4   
            writeln!(w, "RAL        ")?; 1
        },
        0b00011111 => { // RAR        | Route A right through carry          | 00011111        |  4   
            writeln!(w, "RAR        ")?; 1
        },
        0b11000011 => { // JMP        | Jump unconditional                   | 11000011        | 10   
            jump_instruction(w, "JMP", pc, rom)?
        },
        0b11011010 => { // JC         | Jump on carry                        | 11011010        | 10   
            jump_instruction(w, "JC", pc, rom)?
        },
        0b11010010 => { // JNC        | Jump on no carry                     | 11010010        | 10   
            jump_instruction(w, "JNC", pc, rom)?
        },
        0b11001010 => { // JZ         | Jump on zero                         | 11001010        | 10   
            jump_instruction(w, "JZ", pc, rom)?
        },
        0b11000010 => { // JNZ        | Jump on no zero                      | 11000010        | 10   
            jump_instruction(w, "JNZ", pc, rom)?
        },
        0b11110010 => { // JP         | Jump on positive                     | 11110010        | 10   
            jump_instruction(w, "JP", pc, rom)?
        },
        0b11111010 => { // JM         | Jump on minus                        | 11111010        | 10   
            jump_instruction(w, "JM", pc, rom)?
        },
        0b11101010 => { // JPE        | Jump on parity even                  | 11101010        | 10   
            jump_instruction(w, "JPE", pc, rom)?
        },
        0b11100010 => { // JPO        | Jump on parity odd                   | 11100010        | 10   
            jump_instruction(w, "JPO", pc, rom)?
        },
        0b11001101 => { // CALL       | Call unconditional                   | 11001101        | 17   
            jump_instruction(w, "CALL", pc, rom)?
        },
        0b11011100 => { // CC         | Call on carry                        | 11011100        | 11/17
            jump_instruction(w, "CC", pc, rom)?
        },
        0b11010100 => { // CNC        | Call on no tarry Call on tern        | 11010100        | 11/17
            jump_instruction(w, "CNC", pc, rom)?
        },
        0b11001100 => { // CZ         | Call on zero                         | 11001100        | 11/17
            jump_instruction(w, "CZ", pc, rom)?
        },
        0b11000100 => { // CNZ        | Call on no zero                      | 11000100        | 11/17
            jump_instruction(w, "CNZ", pc, rom)?
        },
        0b11110100 => { // CP         | Call on positive                     | 11110100        | 11/17
            jump_instruction(w, "CP", pc, rom)?
        },
        0b11111100 => { // CM         | Call on minus                        | 11111100        | 11/17
            jump_instruction(w, "CM", pc, rom)?
        },
        0b11101100 => { // CPE        | Call on parity even                  | 11101100        | 11/17
            jump_instruction(w, "CPE", pc, rom)?
        },
        0b11100100 => { // CPO        | Call on parity odd                   | 11100100        | 11/17
            jump_instruction(w, "CPO", pc, rom)?
        },
        0b11001001 => { // RET        | Return                               | 11001001        | 10   
            writeln!(w, "RET        ")?; 1
        },
        0b11011000 => { // RC         | Return on carry                      | 11011000        | 5/11 
            writeln!(w, "RC         ")?; 1
        },
        0b11010000 => { // RNC        | Return on no carry                   | 11010000        | 5/11 
            writeln!(w, "RNC        ")?; 1
        },
        0b11001000 => { // RZ         | Return on zero                       | 11001000        | 5/11 
            writeln!(w, "RZ         ")?; 1
        },
        0b11000000 => { // RNZ        | Return on no zero                    | 11000000        | 5/11 
            writeln!(w, "RNZ        ")?; 1
        },
        0b11110000 => { // RP         | Return on positive                   | 11110000        | 5/11 
            writeln!(w, "RP         ")?; 1
        },
        0b11111000 => { // RM         | Return on minus                      | 11111000        | 5/11 
            writeln!(w, "RM         ")?; 1
        },
        0b11101000 => { // RPE        | Return on parity even                | 11101000        | 5/11 
            writeln!(w, "RPE        ")?; 1
        },
        0b11100000 => { // RPO        | Return on parity odd                 | 11100000        | 5/11 
            writeln!(w, "RPO        ")?; 1
        },
        _ if rom[pc as usize] & 0b11000111 == 0b11000111 => { // RST        | Restart                              | 11AAA111        | 11   
            writeln!(w, "RST {:03b}   ", (rom[pc as usize] & 0b00111000) >> 3 )?; 1
        },
        0b11011011 => { // IN         | Input                                | 11011011        | 10   
            let device = get_u8(pc, rom);
            writeln!(w, "IN {:02x}", device)?; 2
        },
        0b11010011 => { // OUT        | Output                               | 11010011        | 10   
            let device = get_u8(pc, rom);
            writeln!(w, "OUT {:02x}", device)?; 2
        },
        0b00000001 => { // LXI  B     | Load immediate register Pair B & C   | 00000001        | 10   
            let immediate = get_u16(pc, rom);
            writeln!(w, "LXI  B {:04x}", immediate)?; 3
        },
        0b00010001 => { // LXI  D     | Load immediate register pair D & E   | 00010001        | 10   
            let immediate = get_u16(pc, rom);
            writeln!(w, "LXI  D {:04x}", immediate)?; 3
        },
        0b00100001 => { // LXI  H     | Load immediate register pair H & L   | 00100001        | 10   
            let immediate = get_u16(pc, rom);
            writeln!(w, "LXI  H {:04x}", immediate)?; 3
        },
        0b00110001 => { // LXI  SP    | Load immediate stack pointer         | 00110001        | 10   
            let immediate = get_u16(pc, rom);
            writeln!(w, "LXI  SP {:04x}", immediate)?; 3
        },
        0b11000101 => { // PUSH B     | Push register Pair B & C on stack    | 11000101        | 11   
            writeln!(w, "PUSH B     ")?; 1
        },
        0b11010101 => { // PUSH D     | Push register Pair D & E on stack    | 11010101        | 11   
            writeln!(w, "PUSH D     ")?; 1
        },
        0b11100101 => { // PUSH H     | Push register Pair H & L on stack    | 11100101        | 11   
            writeln!(w, "PUSH H     ")?; 1
        },
        0b11110101 => { // PUSH PSW   | Push A and Flags on stack            | 11110001        | 11   
            writeln!(w, "PUSH PSW   ")?; 1
        },
        0b11000001 => { // POP  B     | Pop register pair B & C off stack    | 11000001        | 10   
            writeln!(w, "POP  B     ")?; 1
        },
        0b11010001 => { // POP  D     | Pop register pair D & E off stack    | 11010001        | 10   
            writeln!(w, "POP  D     ")?; 1
        },
        0b11100001 => { // POP  H     | Pop register pair H & L off stick    | 11100001        | 10   
            writeln!(w, "POP  H     ")?; 1
        },
        0b11110001 => { // POP  PSW   | Pop A and Flags off stack            | 11110001        | 10   
            writeln!(w, "POP  PSW   ")?; 1
        },
        0b00110010 => { // STA        | Store A direct                       | 00110010        | 13   
            let immediate = get_u16(pc, rom);
            writeln!(w, "STA {:04x}", immediate)?; 3
        },
        0b00111010 => { // LDA        | Load A direct                        | 00111010        | 13   
            let immediate = get_u16(pc, rom);
            writeln!(w, "LDA {:04x}", immediate)?; 3
        },
        0b11101011 => { // XCHG       | Exchange D & E, H & L Registers      | 11101011        | 4    
            writeln!(w, "XCHG       ")?; 1
        },
        0b11100011 => { // XTHL       | Exchange top of stack, H & L         | 11100011        | 18   
            writeln!(w, "XTHL       ")?; 1
        },
        0b11111001 => { // SPHL       | H & L to stack pointer               | 11111001        | 5    
            writeln!(w, "SPHL       ")?; 1
        },
        0b11101001 => { // PCHL       | H & L to program counter             | 11101001        | 5    
            writeln!(w, "PCHL       ")?; 1
        },
        0b00001001 => { // DAD  B     | Add B & C to H & L                   | 00001001        | 10   
            writeln!(w, "DAD  B     ")?; 1
        },
        0b00011001 => { // DAD  D     | Add D & E to H & L                   | 00011001        | 10   
            writeln!(w, "DAD  D     ")?; 1
        },
        0b00101001 => { // DAD  H     | Add H & L to H & L                   | 00101001        | 10   
            writeln!(w, "DAD  H     ")?; 1
        },
        0b00111001 => { // DAD  SP    | Add stack pointer to H & L           | 00111001        | 10   
            writeln!(w, "DAD  SP    ")?; 1
        },
        0b00000010 => { // STAX B     | Store A indirect                     | 00000010        | 7    
            writeln!(w, "STAX B     ")?; 1
        },
        0b00010010 => { // STAX D     | Store A Indirect                     | 00010010        | 7    
            writeln!(w, "STAX D     ")?; 1
        },
        0b00001010 => { // LDAX B     | Load A indirect                      | 00001010        | 7    
            writeln!(w, "LDAX B     ")?; 1
        },
        0b00011010 => { // LDAX D     | Load A indirect                      | 00011010        | 7    
            writeln!(w, "LDAX D     ")?; 1
        },
        0b00000011 => { // INX  B     | Increment B & C registers            | 00000011        | 5    
            writeln!(w, "INX  B     ")?; 1
        },
        0b00010011 => { // INX  D     | Increment D & E registers            | 00010011        | 5    
            writeln!(w, "INX  D     ")?; 1
        },
        0b00100011 => { // INX  H     | Increment H & L registers            | 00100011        | 5    
            writeln!(w, "INX  H     ")?; 1
        },
        0b00110011 => { // INX  SP    | Increment stack pointer              | 00110011        | 5    
            writeln!(w, "INX  SP    ")?; 1
        },
        0b00001011 => { // DCX  B     | Decrement B & C                      | 00001011        | 5    
            writeln!(w, "DCX  B     ")?; 1
        },
        0b00011011 => { // DCX  D     | Decrement D & E                      | 00011011        | 5    
            writeln!(w, "DCX  D     ")?; 1
        },
        0b00101011 => { // DCX  H     | Decrement H & L                      | 00101011        | 5    
            writeln!(w, "DCX  H     ")?; 1
        },
        0b00111011 => { // DCX  SP    | Decrement stack pointer              | 00111011        | 5    
            writeln!(w, "DCX  SP    ")?; 1
        },
        0b00101111 => { // CMA        | Complement A                         | 00101111        | 4    
            writeln!(w, "CMA        ")?; 1
        },
        0b00110111 => { // STC        | Set carry                            | 00110111        | 4    
            writeln!(w, "STC        ")?; 1
        },
        0b00111111 => { // CMC        | Complement carry                     | 00111111        | 4    
            writeln!(w, "CMC        ")?; 1
        },
        0b00100111 => { // DAA        | Decimal adjust A                     | 00100111        | 4    
            writeln!(w, "DAA        ")?; 1
        },
        0b00100010 => { // SHLD       | Store H & L direct                   | 00100010        | 16   
            let immediate = get_u16(pc, rom);
            writeln!(w, "SHLD  {:04x}", immediate)?; 3
        },
        0b00101010 => { // LHLD       | Load H & L direct                    | 00101010        | 16   
            let immediate = get_u16(pc, rom);
            writeln!(w, "LHLD  {:04x}", immediate)?; 3
        },
        0b11111011 => { // EI         | Enable Interrupts                    | 11111011        | 4    
            writeln!(w, "EI         ")?; 1
        },
        0b11110011 => { // DI         | Disable Interrupts                   | 11110011        | 4    
            writeln!(w, "DI         ")?; 1
        },
        0b00000000 => { // NOP        | No operation                         | 00000000        | 4    
            writeln!(w, "NOP        ")?; 1
        },        
        _ => {
            writeln!(w, "<UNDEFINED>")?;
            1
        }
    })
}