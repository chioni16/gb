use crate::mmu::busio::{BusIO, SResult};
use crate::util::Addr;

use super::rom_bank::{RomBank, ROM_BANK_SIZE};
// use super::ram_bank::{RamBank, RAM_BANK_SIZE};

const ROM_SIZE_ADDR: u16 = 0x148;
const NUM_OF_RAMS_ADDR: u16 = 0x149;

#[derive(Debug)]
pub(crate) struct MBC1{
    roms: Vec<RomBank>,
    // rams: Vec<RamBank>,

    cur_rom: u8,
    // secondary: u8, // 2 bit

    ram_enabled: bool,
    use_secondary: bool,
}

impl MBC1 {
    pub(crate) fn new(raw: Vec<u8>, _mbc_mode: u8) -> Self {
        let num_rams = raw[NUM_OF_RAMS_ADDR as usize] as usize;
        println!("num_rams: {:x}", num_rams);
        assert!(num_rams <= 4);
        
        let rom_size =  32usize * (1 << raw[ROM_SIZE_ADDR as usize]);
        println!("raw_rom_size: {}, rom_size: {}", raw[ROM_SIZE_ADDR as usize], rom_size);

        let s = Self {
            roms: raw.chunks(ROM_BANK_SIZE as usize).map(|c| c.to_vec()).map(RomBank::new).collect(),
            // rams: vec![RamBank::new(); num_rams],

            cur_rom: 1,
            // secondary: 0, 

            ram_enabled: false,
            use_secondary: false,
        };

        println!("num of rom banks created: {}", s.roms.len());
        for r in &s.roms {
            println!("{:x?}", &r.0[..][..20]);
        }
        // panic!();
        s
    }
}

impl BusIO for MBC1 {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        // if addr == 0x65fd.into() {
        //     panic!("read from 0x65fd, cur_rom: {:?}, secondary:  {:?}, ram_enabled: {:?}, use_secondary: {:?}", self.cur_rom, self.secondary, self.ram_enabled, self.use_secondary);
        // }
        match addr.into() {
            0x0000..0x4000 => { 
                // let index = if self.use_secondary { self.secondary << 5  } else { 0 } as usize;
                let index = 0usize;
                self.roms[index].readu8(addr)
            }
            0x4000..0x8000 => {
                // let index = if self.use_secondary {self.secondary << 5 | self.cur_rom } else { self.cur_rom } as usize;
                let index =  self.cur_rom as usize;
                // let index = index % self.roms.len(); // don't know if it's a good idea to use mod here
                self.roms[index].readu8(addr) 
            }
            0xa000..0xc000 => {
                // if self.ram_enabled { 
                //     let index = if self.use_secondary { self.secondary } else { 0 } as usize;
                //     let index = index % self.rams.len(); // don't know if it's a good idea to use mod here
                //     self.rams[index].readu8(addr) 
                // } else {
                    Ok(0xff)
                // }
            }
            _              => Err(format!("MBC1 readu8 - invalid addr: {:x?}", addr).into())
        }
    }

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        match addr.into() {
            0x0000..0x2000 => { if value & 0x0f == 0xa { panic!("write {:x?} to ram_enabled @ {:x?}", value, addr)}; self.ram_enabled = value & 0x0f == 0xa},
            0x2000..0x4000 => {
                let cur_rom = value & 0b0001_1111;
                self.cur_rom = if cur_rom == 0 {1} else {cur_rom};
            }
            // 0x4000..0x6000 => { panic!("write to secondary");self.secondary = value & 0b11},
            // 0x6000..0x8000 => { panic!("write to use_secondary");self.use_secondary = value != 0},
            0xa000..0xc000 => {
                // if self.ram_enabled { 
                //     let index = if self.use_secondary { self.secondary } else { 0 } as usize;
                //     let index = index % self.rams.len(); // don't know if it's a good idea to use mod here
                //     self.rams[index].writeu8(addr, value)?; 
                // } 
            }
            _              => return Err(format!("MBC1 writeu8 - invalid addr: {:x?}", addr).into())
        };
        Ok(())
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        Ok(u16::from_le_bytes([
            self.readu8(addr)?,
            self.readu8(addr + 1.into())?
        ]))
    }

    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        if addr > 0x0000.into() && addr < 0x8000.into() {
            panic!("MBC1 writeu16 @ {:x?}", addr);
        }
        let value = value.to_le_bytes();
        self.writeu8(addr, value[0])?;
        self.writeu8(addr + 1.into(), value[1])?;
        Ok(()) 
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        unimplemented!() // expecting this to blow up 
    }

    fn print_dbg(&self, start: Addr, len: u16) -> String {
        unimplemented!()
    }
}