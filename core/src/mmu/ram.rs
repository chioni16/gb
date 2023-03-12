use std::fmt::Debug;

use super::busio::{BusIO, SResult};
use crate::util::Addr;

pub(crate) struct RAM {
    buffer: Vec<u8>,
    map: Box<dyn Fn(Addr) -> Addr>,
}

impl Debug for RAM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RAM: {:?}", self.buffer)
    }
}

impl RAM {
    pub(crate) fn new(size: usize, f: Box<dyn Fn(Addr) -> Addr>) -> Self {
        Self {
            buffer: vec![0; size],
            map: f,
        }
    }

    pub(crate) fn copy_from_slice(&mut self, src: &[u8]) {
        self.buffer.copy_from_slice(src);
    }

}

impl BusIO for RAM {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        let addr: u16 = (self.map)(addr).into();
        Ok(self.buffer[addr as usize])
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        let addr: u16 = (self.map)(addr).into();
        Ok(u16::from_le_bytes([
            self.buffer[addr as usize],
            self.buffer[addr as usize + 1],
        ]))
    }

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        let addr: u16 = (self.map)(addr).into();
        self.buffer[addr as usize] = value;
        Ok(())
    }

    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        // Little Endian
        let addr: u16 = (self.map)(addr).into();
        let value = value.to_le_bytes();
        self.buffer[addr as usize] = value[0];
        self.buffer[addr as usize + 1] = value[1];
        Ok(())
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        let addr: u16 = (self.map)(addr).into();
        Ok(&self.buffer[addr as usize..][..len])
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}
