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
    pub SP: u16,
    pub PC: u16,
    pub memory: [u8; 0x4000],
}
#[allow(non_snake_case)]
impl I8080State {
    pub fn new() -> Self {
        I8080State {
            A: 0,
            Flags: 0,
            B: 0,
            C: 0,
            D: 0,
            E: 0,
            H: 0,
            L: 0,
            SP: 0,
            PC: 0,

            memory: [0; 0x4000],
        }
    }

    pub fn get_AF(&self) -> u16 {
        unsafe {  u16::from_le(*(&self.A as *const u8 as *const u16)) }
    }
    pub fn set_AF(&mut self, value: u16) {
        unsafe { *(&mut self.A as *mut u8 as *mut u16) = value.to_le(); }
    }

    pub fn get_BC(&self) -> u16 {
        unsafe {  u16::from_le(*(&self.B as *const u8 as *const u16)) }
    }
    pub fn set_BC(&mut self, value: u16) {
        unsafe { *(&mut self.B as *mut u8 as *mut u16) = value.to_le(); }
    }

    pub fn get_DE(&self) -> u16 {
        unsafe {  u16::from_le(*(&self.D as *const u8 as *const u16)) }
    }
    pub fn set_DE(&mut self, value: u16) {
        unsafe { *(&mut self.D as *mut u8 as *mut u16) = value.to_le(); }
    }

    pub fn get_HL(&self) -> u16 {
        unsafe {  u16::from_le(*(&self.H as *const u8 as *const u16)) }
    }
    pub fn set_HL(&mut self, value: u16) {
        unsafe { *(&mut self.H as *mut u8 as *mut u16) = value.to_le(); }
    }
}