use super::Addr;

#[allow(dead_code)]
pub(super) struct MMU {
    rom: Vec<u8>,
}

#[allow(dead_code)]
impl MMU { 
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
        }
    }
    pub fn readu8(&mut self, addr: Addr) -> u8 {
        let addr: u16 = addr.into();
        self.rom[addr as usize]
    }
    pub fn readu16(&mut self, addr: Addr) -> u16 {
        let addr: u16 = addr.into();
        u16::from_le_bytes([self.rom[addr as usize], self.rom[addr as usize +1]])
    }
    pub fn writeu8(&mut self, _addr: Addr, _value: u8) {
        unimplemented!()
    }
    pub fn writeu16(&mut self, _addr: Addr, _value: u16) {
        unimplemented!()
    }
}