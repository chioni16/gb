use crate::mmu::busio::{BusIO, SResult};
use crate::util::Addr;

// pub(crate) const RAM_BANK_SIZE: usize = 0x2000;
const RAM_BANK_SIZE_ORDER: u8 = 13;
pub(crate) const RAM_BANK_SIZE: usize = 1 << RAM_BANK_SIZE_ORDER;

#[derive(Debug, Clone)]
pub(super) struct RamBank([u8; RAM_BANK_SIZE]);

impl RamBank {
    pub(super) fn new() -> Self {
        Self([0; RAM_BANK_SIZE])
    }

    fn get_index(addr: Addr) -> usize {
        let index = <Addr as Into<u16>>::into(addr) & ((1 << RAM_BANK_SIZE_ORDER) - 1);
        index as usize
    }
}

impl BusIO for RamBank {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        let index = Self::get_index(addr);
        Ok(self.0[index])
    }

    fn readu16(&self, _addr: Addr) -> SResult<u16> {
        unreachable!()
        // Ok(u16::from_le_bytes([
        //     self.readu8(addr)?,
        //     self.readu8(addr+1.into())?,
        // ]))
    }

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        let index = Self::get_index(addr);
        self.0[index] = value;
        Ok(())
    }

    fn writeu16(&mut self, _addr: Addr, _value: u16) -> SResult<()> {
        unreachable!()
        // panic!("write request for ROM, addr: {:#x?}", addr);
        // Memory Bank Controllers
        // Ok(())
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        let addr: u16 = addr.into();
        Ok(&self.0[addr as usize..][..len])
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}