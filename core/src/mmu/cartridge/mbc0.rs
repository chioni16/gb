use crate::mmu::busio::{BusIO, SResult};
use crate::util::Addr;

pub(super) const MBC0_ROM_SIZE: u16 = 0x8000;
pub(super) const MBC0_RAM_SIZE: u16 = 0x2000;
pub(super) const EXTERNAL_RAM_START: u16 = 0xa000;
pub(super) const EXTERNAL_RAM_END: u16 = EXTERNAL_RAM_START + MBC0_RAM_SIZE;

#[derive(Debug)]
pub(crate) struct MBC0 {
    rom: [u8; MBC0_ROM_SIZE as usize], 
    ram: [u8; MBC0_RAM_SIZE as usize], 
}

impl MBC0 {
    pub(crate) fn new(mut raw: Vec<u8>) -> Self {
        raw.resize(MBC0_ROM_SIZE as usize, 0);
        let rom = raw.try_into().unwrap();
        Self {
            rom,
            ram: [0; MBC0_RAM_SIZE as usize],
        }
    }
}

impl BusIO for MBC0 {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        let addr: u16 = addr.into();
        match addr {
            0..MBC0_ROM_SIZE                     => Ok(self.rom[addr as usize]),
            EXTERNAL_RAM_START..EXTERNAL_RAM_END => Ok(self.ram[(addr & 0x1fff) as usize]),
            _                                    => Err(format!("MBC0 received readu8 addr: {:x?}", addr).into()),
        }
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        let addr: u16 = addr.into();
        match addr {
            // should be MBC0_SIZE-1, but I guess this should work
            0..MBC0_ROM_SIZE => Ok(u16::from_le_bytes([self.rom[addr as usize], self.rom[addr as usize + 1]])),
            EXTERNAL_RAM_START..EXTERNAL_RAM_END => {
                let addr = addr & 0x1fff;
                Ok(u16::from_le_bytes([self.ram[addr as usize], self.ram[addr as usize + 1]]))
            }
            _            => Err(format!("MBC0 received readu16 addr: {:x?}", addr).into()),
        }
    }

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        let addr: u16 = addr.into();
        match addr {
            0..MBC0_ROM_SIZE                     => {}
            EXTERNAL_RAM_START..EXTERNAL_RAM_END => self.ram[(addr & 0x1fff) as usize] = value,
            _                                    => return Err(format!("MBC0 received writeu8 addr: {:x?}, value: {:x?}", addr, value).into()),
        };
        Ok(())
    }

    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        let addr: u16 = addr.into();
        match addr {
            // should be MBC0_SIZE-1, but I guess this should work
            0..MBC0_ROM_SIZE => {}
            EXTERNAL_RAM_START..EXTERNAL_RAM_END => {
                let addr = addr & 0x1fff;
                let value = value.to_le_bytes();
                self.ram[addr as usize] = value[0];
                self.ram[addr as usize + 1] = value[1];
            }
            _            => return Err(format!("MBC0 received writeu16 addr: {:x?}, value: {:x?}", addr, value).into()),
        };
        Ok(())
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        let addr: u16 = addr.into();
        let s = match addr {
            0..MBC0_ROM_SIZE                     => &self.rom[addr as usize..],
            EXTERNAL_RAM_START..EXTERNAL_RAM_END => &self.ram[(addr & 0x1fff) as usize..],
            _                                    => return Err(format!("MBC0 received as_slice addr: {:x?}", addr).into()),
        };
        Ok(&s[..len])
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}
