use super::busio::{BusIO, SResult};
use crate::util::Addr;

#[derive(Debug)]
pub(super) struct ROM(Vec<u8>);

impl ROM {
    pub(super) fn new(mut rom: Vec<u8>) -> Self {
        rom.resize(32 * 1024, 0);
        Self(rom)
    }
}

impl BusIO for ROM {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        let addr: u16 = addr.into();
        Ok(self.0[addr as usize])
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        let addr: u16 = addr.into();
        Ok(u16::from_le_bytes([
            self.0[addr as usize],
            self.0[addr as usize + 1],
        ]))
    }

    fn writeu8(&mut self, _addr: Addr, _value: u8) -> SResult<()> {
        // panic!("write request for ROM, addr: {:#x?}", addr);
        // Memory Bank Controllers 
        Ok(())
    }

    fn writeu16(&mut self, _addr: Addr, _value: u16) -> SResult<()> {
        // panic!("write request for ROM, addr: {:#x?}", addr);
        // Memory Bank Controllers 
        Ok(())
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}