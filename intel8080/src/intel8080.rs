pub trait IODevices : Send {
    fn read(&mut self, device: u8) -> u8;
    fn write(&mut self, device: u8, value: u8);
}

pub trait Memory : Send {
    fn read(&self, adress: u16) -> u8;
    fn write(&mut self, adress: u16, value: u8);

    fn get_rom(&mut self) -> Vec<u8>;

    #[inline]
    fn read_u16(&self, adress: u16) -> u16 {
        u16::from_le_bytes(
            [self.read(adress), self.read(adress + 1)]
        )
    }
}


#[repr(C)]
#[allow(non_snake_case)]
pub struct I8080State {
    pub A: u8,
    pub Flags: u8,
    pub B: u8,
    pub C: u8,
    pub D: u8,
    pub E: u8,
    pub H: u8,
    pub L: u8,
    SP: u16,
    PC: u16,
    pub interrupt_enabled: bool,
}
#[allow(non_snake_case)]
impl I8080State {
    pub fn new() -> Self {
        I8080State {
            A: 0,
            Flags: 0b0000_0010,
            B: 0,
            C: 0,
            D: 0,
            E: 0,
            H: 0,
            L: 0,
            SP: 0,
            PC: 0,
            interrupt_enabled: false,
        }
    }

    #[inline]
    pub fn get_PSW(&self) -> u16 {
        unsafe {  u16::from_be(*(&self.A as *const u8 as *const u16)) }
    }
    #[inline]
    pub fn set_PSW(&mut self, value: u16) {
        unsafe { *(&mut self.A as *mut u8 as *mut u16) = value.to_be(); }
    }

    #[inline]
    pub fn get_BC(&self) -> u16 {
        unsafe {  u16::from_be(*(&self.B as *const u8 as *const u16)) }
    }
    #[inline]
    pub fn set_BC(&mut self, value: u16) {
        unsafe { *(&mut self.B as *mut u8 as *mut u16) = value.to_be(); }
    }

    #[inline]
    pub fn get_DE(&self) -> u16 {
        unsafe {  u16::from_be(*(&self.D as *const u8 as *const u16)) }
    }
    #[inline]
    pub fn set_DE(&mut self, value: u16) {
        unsafe { *(&mut self.D as *mut u8 as *mut u16) = value.to_be(); }
    }

    #[inline]
    pub fn get_HL(&self) -> u16 {
        unsafe {  u16::from_be(*(&self.H as *const u8 as *const u16)) }
    }
    #[inline]
    pub fn set_HL(&mut self, value: u16) {
        unsafe { *(&mut self.H as *mut u8 as *mut u16) = value.to_be(); }
    }

    #[inline]
    pub fn get_SP(&self) -> u16 {
        u16::from_le(self.SP)
    }
    #[inline]
    pub fn set_SP(&mut self, value: u16) {
        self.SP = value.to_le();
    }
    #[inline]
    pub fn get_PC(&self) -> u16 {
        u16::from_le(self.PC)
    }
    #[inline]
    pub fn set_PC(&mut self, value: u16) {
        self.PC = value.to_le();
    }
    #[inline]
    pub fn push_stack<M: Memory>(&mut self, value: u16, memory: &mut M) {
        memory.write(self.get_SP() - 2, (value.to_le() & 0xff) as u8);
        memory.write(self.get_SP() - 1, ((value.to_le() >> 8) & 0xff) as u8);
        self.SP = (u16::from_le(self.SP) - 2).to_le();
    }
    #[inline]
    pub fn pop_stack<M: Memory>(&mut self, memory: &M) -> u16 {
        self.SP = (u16::from_le(self.SP) + 2).to_le();
        memory.read(self.get_SP() - 2) as u16 |
        (memory.read(self.get_SP() - 1) as u16) << 8
        
    }

    #[inline]
    /// set all flags
    pub fn set_flags(&mut self, result: u8, carry: bool, auxcarry: bool) {
        self.Flags = 0b0000_0010
            | (carry as u8) // carry
            | ((!(result.count_ones() as u8) & 1) << 2) // parity
            | (result & ((auxcarry as u8) << 4)) // auxcarry
            | ( ((result == 0) as u8) << 6 ) // zero
            | (result & 0b1000_0000); // sign
    }

    #[inline]
    /// set all flags except carry
    pub fn set_flags_ex(&mut self, result: u8, auxcarry: bool) {
        self.Flags = 0b0000_0010
            | (self.Flags & 1) // carry
            | ((!(result.count_ones() as u8) & 1) << 2) // parity
            | (result & ((auxcarry as u8) << 4)) // auxcarry
            | ( ((result == 0) as u8) << 6 ) // zero
            | (result & 0b1000_0000); // sign
    }

    #[inline]
    pub fn set_carry(&mut self, carry: bool) {
        self.Flags = (self.Flags & !1) | carry as u8;
    }

    #[inline]
    pub fn on_aux_carry(&self) -> bool {
        (self.Flags & 0b0001_0000) != 0
    }

    #[inline]
    pub fn on_carry(&self) -> bool {
        (self.Flags & 0b0000_0001) != 0
    }

    #[inline]
    pub fn on_zero(&self) -> bool {
        (self.Flags & 0b0100_0000) != 0
    }

    #[inline]
    pub fn on_positive(&self) -> bool {
        (self.Flags & 0b1000_0000) == 0
    }

    #[inline]
    pub fn on_parity_even(&self) -> bool {
        (self.Flags & 0b0000_0100) != 0
    }

    pub fn print_state<W: std::fmt::Write>(&self, w: &mut W) {
        writeln!(w, "B  C  D  E  H  L  A  SZ_A_P_C <- Flags").unwrap();
        writeln!(w, "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:08b}", self.B, self.C, self.D, self.E, self.H, self.L, self.A, self.Flags).unwrap();
        writeln!(w, "PC: {:04x}   SP: {:04x}", self.get_PC(), self.get_SP()).unwrap();
    }
}