
#[repr(C)]
#[allow(non_snake_case)]
pub struct I8080State {
    pub B: u8,
    pub C: u8,
    pub D: u8,
    pub E: u8,
    pub H: u8,
    pub L: u8,
    pub A: u8,
    pub Flags: u8,
    SP: u16,
    PC: u16,
    pub memory: [u8; 0x4000],
}
#[allow(non_snake_case)]
impl I8080State {
    pub fn new() -> Self {
        I8080State {
            B: 0,
            C: 0,
            D: 0,
            E: 0,
            H: 0,
            L: 0,
            A: 0,
            Flags: 0,
            SP: 0,
            PC: 0,

            memory: [0; 0x4000],
        }
    }

    pub fn get_PSW(&self) -> u16 {
        unsafe {  u16::from_be(*(&self.A as *const u8 as *const u16)) }
    }
    pub fn set_PSW(&mut self, value: u16) {
        unsafe { *(&mut self.A as *mut u8 as *mut u16) = value.to_be(); }
    }

    pub fn get_BC(&self) -> u16 {
        unsafe {  u16::from_be(*(&self.B as *const u8 as *const u16)) }
    }
    pub fn set_BC(&mut self, value: u16) {
        unsafe { *(&mut self.B as *mut u8 as *mut u16) = value.to_be(); }
    }

    pub fn get_DE(&self) -> u16 {
        unsafe {  u16::from_be(*(&self.D as *const u8 as *const u16)) }
    }
    pub fn set_DE(&mut self, value: u16) {
        unsafe { *(&mut self.D as *mut u8 as *mut u16) = value.to_be(); }
    }

    pub fn get_HL(&self) -> u16 {
        unsafe {  u16::from_be(*(&self.H as *const u8 as *const u16)) }
    }
    pub fn set_HL(&mut self, value: u16) {
        unsafe { *(&mut self.H as *mut u8 as *mut u16) = value.to_be(); }
    }

    pub fn get_SP(&self) -> u16 {
        u16::from_le(self.SP)
    }
    pub fn set_SP(&mut self, value: u16) {
        self.SP = value.to_le();
    }

    pub fn get_PC(&self) -> u16 {
        u16::from_le(self.PC)
    }
    pub fn set_PC(&mut self, value: u16) {
        self.PC = value.to_le();
    }

    pub fn push_stack(&mut self, value: u16) {
        unsafe { *(&mut self.memory[(u16::from_le(self.SP) as usize) - 2] as *mut u8 as *mut u16) = value.to_le(); }
        self.SP = (u16::from_le(self.SP) - 2).to_le();
    }
    pub fn pop_stack(&mut self) -> u16 {
        self.SP = (u16::from_le(self.SP) + 2).to_le();
        unsafe { u16::from_le(*(&mut self.memory[(u16::from_le(self.SP) as usize) - 2] as *mut u8 as *mut u16)) }
    }

    pub fn get_op(&self) -> u8 {
        self.memory[self.PC as usize]
    }

    pub fn get_u8(&self) -> u8 {
        self.memory[self.PC as usize + 1]
    }

    pub fn get_u16(&self) -> u16 {
        u16::from_le_bytes(
            [self.memory[self.PC as usize + 1], self.memory[self.PC as usize + 2]]
        )
    }

    pub fn get_memory(&self, adress: u16) -> u8 {
        self.memory[(adress & 0x3fff) as usize]
    }

    pub fn set_memory(&mut self, adress: u16, value: u8) {
        self.memory[(adress & 0x3fff) as usize] = value;
    }

    /// set all flags
    pub fn set_flags(&mut self, result: u8, carry: bool, auxcarry: bool) {
        self.Flags = 
           (carry as u8) // carry
         | ((!(result.count_ones() as u8) & 1) << 2) // parity
         | (result & ((auxcarry as u8) << 4)) // auxcarry
         | ( ((result == 0) as u8) << 6 ) // zero
         | (result & 0b1000_0000); // sign
    }

    /// set all flags except carry
    pub fn set_flags_ex(&mut self, result: u8, auxcarry: bool) {
        self.Flags = 
           (self.Flags & 1) // carry
         | ((!(result.count_ones() as u8) & 1) << 2) // parity
         | (result & ((auxcarry as u8) << 4)) // auxcarry
         | ( ((result == 0) as u8) << 6 ) // zero
         | (result & 0b1000_0000); // sign
    }

    pub fn set_carry(&mut self, carry: bool) {
        self.Flags = (self.Flags & !1) | carry as u8;
    }

    pub fn on_aux_carry(&self) -> bool {
        (self.Flags & 0b0001_0000) != 0
    }

    pub fn on_carry(&self) -> bool {
        (self.Flags & 0b0000_0001) != 0
    }

    pub fn on_zero(&self) -> bool {
        (self.Flags & 0b0100_0000) != 0
    }

    pub fn on_positive(&self) -> bool {
        (self.Flags & 0b1000_0000) == 0
    }

    pub fn on_parity_even(&self) -> bool {
        (self.Flags & 0b0000_0100) == 0
    }

    pub fn print_state<W: std::fmt::Write>(&self, w: &mut W) {
        writeln!(w, "B  C  D  E  H  L  A  SZ_A_P_C <- Flags").unwrap();
        writeln!(w, "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:08b}", self.B, self.C, self.D, self.E, self.H, self.L, self.A, self.Flags).unwrap();
        writeln!(w, "PC: {:04x}   SP: {:04x}", self.get_PC(), self.get_SP()).unwrap();
    }
}