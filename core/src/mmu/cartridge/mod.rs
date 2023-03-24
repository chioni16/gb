mod mbc0;
mod mbc1;
mod mbc5;
mod ram_bank;
mod rom_bank;

use super::busio::{BusIO, SResult};
use crate::util::Addr;
use mbc0::MBC0;
use mbc1::MBC1;
use mbc5::MBC5;

pub(crate) const MBC_MODE_ADDR: usize = 0x147;

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum Cartridge {
    MBC0(MBC0), // No MBC
    MBC1(MBC1),
    MBC5(MBC5),
}

impl Cartridge {
    pub(crate) fn new(raw: Vec<u8>) -> Self {
        let mbc_mode = raw[MBC_MODE_ADDR];
        println!("mbc mode: {:x}", mbc_mode);
        match mbc_mode {
            0 => Cartridge::MBC0(MBC0::new(raw)),
            1 | 2 | 3 => Cartridge::MBC1(MBC1::new(raw, mbc_mode)),
            19 => Cartridge::MBC5(MBC5::new(raw, mbc_mode)),
            _ => panic!("unsupported MBC type: {}", mbc_mode)
        }
    }
}

impl BusIO for Cartridge {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        match self {
            Self::MBC0(mbc0) => mbc0.readu8(addr),
            Self::MBC1(mbc1) => mbc1.readu8(addr),
            Self::MBC5(mbc5) => mbc5.readu8(addr),
            _ => unimplemented!()
        }
    }

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        match self {
            Self::MBC0(mbc0) => mbc0.writeu8(addr, value),
            Self::MBC1(mbc1) => mbc1.writeu8(addr, value),
            Self::MBC5(mbc5) => mbc5.writeu8(addr, value),
            _ => unimplemented!()
        }
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        match self {
            Self::MBC0(mbc0) => mbc0.readu16(addr),
            Self::MBC1(mbc1) => mbc1.readu16(addr),
            Self::MBC5(mbc5) => mbc5.readu16(addr),
            _ => unimplemented!()
        }
    }

    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        match self {
            Self::MBC0(mbc0) => mbc0.writeu16(addr, value),
            Self::MBC1(mbc1) => mbc1.writeu16(addr, value),
            Self::MBC5(mbc5) => mbc5.writeu16(addr, value),
            _ => unimplemented!()
        }
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        match self {
            Self::MBC0(mbc0) => mbc0.as_slice(addr, len),
            Self::MBC1(mbc1) => mbc1.as_slice(addr, len),
            Self::MBC5(mbc5) => mbc5.as_slice(addr, len),
            _ => unimplemented!()
        }
    }

    fn print_dbg(&self, start: Addr, len: u16) -> String {
        match self {
            Self::MBC0(mbc0) => mbc0.print_dbg(start, len),
            Self::MBC1(mbc1) => mbc1.print_dbg(start, len),
            Self::MBC5(mbc5) => mbc5.print_dbg(start, len),
            _ => unimplemented!()
        }
    }
}


