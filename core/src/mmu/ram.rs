use super::busio::{BusIO, SResult};
use crate::util::Addr;

#[derive(Debug)]
pub(super) struct RAM(Vec<u8>);

impl RAM {
    pub(super) fn new(size: usize) -> Self {
        Self(vec![0; size])
    }
}

impl BusIO for RAM {
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

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        let addr: u16 = addr.into();
        self.0[addr as usize] = value;
        Ok(())
    }

    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        // Little Endian
        let addr: u16 = addr.into();
        let value = value.to_le_bytes();
        self.0[addr as usize] = value[0];
        self.0[addr as usize + 1] = value[1];
        Ok(())
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}