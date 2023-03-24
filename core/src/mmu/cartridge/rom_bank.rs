use crate::mmu::busio::{BusIO, SResult};
use crate::util::Addr;

const ROM_BANK_SIZE_ORDER: u8 = 14;
pub(crate) const ROM_BANK_SIZE: usize = 1 << ROM_BANK_SIZE_ORDER;

#[derive(Debug)]
pub(super) struct RomBank(pub [u8; ROM_BANK_SIZE]);

impl RomBank {
    pub(super) fn new(raw: Vec<u8>) -> Self {
        let inner: [u8; ROM_BANK_SIZE] = raw.try_into().unwrap();
        Self(inner)
    }

    fn get_index(addr: Addr) -> usize {
        let index = <Addr as Into<u16>>::into(addr) & ((1 << ROM_BANK_SIZE_ORDER) - 1);
        index as usize
    }
}

impl BusIO for RomBank {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        let index = Self::get_index(addr);
        Ok(self.0[index])
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        unreachable!()
        // let index = Self::get_index(addr);
        // Ok(u16::from_le_bytes([
        //     self.0[index],
        //     self.0[index + 1],
        // ]))
    }

    fn writeu8(&mut self, _addr: Addr, _value: u8) -> SResult<()> {
        unreachable!()
    }

    fn writeu16(&mut self, _addr: Addr, _value: u16) -> SResult<()> {
        unreachable!()
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        unreachable!()
        // let index = Self::get_index(addr);
        // Ok(&self.0[index..][..len])
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}