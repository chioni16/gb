use crate::mmu::busio::{BusIO, SResult};
use crate::util::Addr;

use super::rom_bank::{RomBank, ROM_BANK_SIZE};
use super::ram_bank::{RamBank, RAM_BANK_SIZE};

const ROM_SIZE_ADDR: u16 = 0x148;
const NUM_OF_RAMS_ADDR: u16 = 0x149;

#[derive(Debug)]
pub(crate) struct MBC5 {
    roms: Vec<RomBank>,
    rams: Vec<RamBank>,

    cur_rom_lower: u8,
    cur_rom_higher: u8,
    // secondary: u8, // 2 bit

    cur_ram: u8,
    ram_enabled: bool,
    // use_secondary: bool,
}

impl MBC5 {
    pub(crate) fn new(raw: Vec<u8>, _mbc_mode: u8) -> Self {
        let num_rams = raw[NUM_OF_RAMS_ADDR as usize] as usize;
        println!("num_rams: {:x}", num_rams);
        
        let rom_size =  32usize * (1 << raw[ROM_SIZE_ADDR as usize]);
        println!("raw_rom_size: {}, rom_size: {}", raw[ROM_SIZE_ADDR as usize], rom_size);

        let s = Self {
            roms: raw.chunks(ROM_BANK_SIZE as usize).map(|c| c.to_vec()).map(RomBank::new).collect(),
            rams: vec![RamBank::new(); num_rams],

            cur_rom_lower: 0,
            cur_rom_higher: 0,
            // secondary: 0, 

            cur_ram: 0,
            ram_enabled: false,
            // use_secondary: false,
        };

        println!("num of rom banks created: {}", s.roms.len());
        for r in &s.roms {
            println!("{:x?}", &r.0[..][..20]);
        }
        s
    }

    fn get_rom_number(&self) -> usize {
        ((self.cur_rom_higher as usize & 1) << 8) | (self.cur_rom_lower as usize)
    }
}

impl BusIO for MBC5 {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        // if addr == 0x65fd.into() {
        //     panic!("read from 0x65fd, cur_rom: {:?}, secondary:  {:?}, ram_enabled: {:?}, use_secondary: {:?}", self.cur_rom, self.secondary, self.ram_enabled, self.use_secondary);
        // }
        match addr.into() {
            0x0000..0x4000 => { 
                let index = 0;
                self.roms[index].readu8(addr)
            }
            0x4000..0x8000 => {
                let index = self.get_rom_number();
                let index = index % self.roms.len(); // don't know if it's a good idea to use mod here
                self.roms[index].readu8(addr) 
            }
            0xa000..0xc000 => {
                if self.ram_enabled { 
                    let index = self.cur_ram as usize;
                    let index = index % self.rams.len(); // don't know if it's a good idea to use mod here
                    println!("reading from ram: {}", index);
                    self.rams[index].readu8(addr) 
                } else {
                    Ok(0xff)
                }
            }
            _              => Err(format!("MBC5 readu8 - invalid addr: {:x?}", addr).into())
        }
    }

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        match addr.into() {
            0x0000..0x2000 => self.ram_enabled = value & 0x0f == 0xa,
            0x2000..0x3000 => self.cur_rom_lower = value,
            0x3000..0x4000 => self.cur_rom_higher = value & 1,
            0x4000..0x6000 => self.cur_ram = value & 0x0f,
            0x6000..0x8000 => {}
            0xa000..0xc000 => {
                if self.ram_enabled { 
                    let index = self.cur_ram as usize;
                    let index = index % self.rams.len(); // don't know if it's a good idea to use mod here
                    println!("writing to ram: {}", index);
                    self.rams[index].writeu8(addr, value)?; 
                } 
            }
            _              => return Err(format!("MBC5 writeu8 - invalid addr: {:x?}", addr).into())
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
            panic!("MBC5 writeu16 @ {:x?}", addr);
        }
        let value = value.to_le_bytes();
        self.writeu8(addr, value[0])?;
        self.writeu8(addr + 1.into(), value[1])?;
        Ok(()) 
    }

    fn as_slice(&self, _addr: Addr, _len: usize) -> SResult<&[u8]> {
        unimplemented!() // expecting this to blow up 
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}