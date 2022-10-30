use super::Addr;
use std::collections::HashMap;
use std::fmt::Debug;

type SResult<T> = Result<T, Box<dyn std::error::Error>>;

// common interface to read and write from/to addresses
pub(crate) trait BusIO: Debug {
    fn readu8(&self, addr: Addr) -> SResult<u8>;
    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()>;

    fn readu16(&self, addr: Addr) -> SResult<u16>;
    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()>;

    fn print_dbg(&self, start: Addr, len: u16) -> String;
}

#[derive(Debug)]
struct ROM(Vec<u8>);

impl ROM { 
    fn new(mut rom: Vec<u8>) -> Self {
        rom.resize(32*1024, 0);
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
        Ok(u16::from_le_bytes([self.0[addr as usize], self.0[addr as usize +1]]))
    }
    fn writeu8(&mut self, _addr: Addr, _value: u8) -> SResult<()>{
        unimplemented!()
    }
    fn writeu16(&mut self, _addr: Addr, _value: u16) -> SResult<()> {
        unimplemented!()
    }
    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        "".into()
    }
}

#[derive(Debug)]
struct RAM(Vec<u8>);

impl RAM {
    fn new(size: usize) -> Self {
        Self(vec![0; size])
    }
}

impl BusIO for RAM {
    fn readu16(&self, addr: Addr) -> SResult<u16> {
        let addr: u16 = addr.into();
        Ok(u16::from_le_bytes([self.0[addr as usize], self.0[addr as usize +1]]))
    }
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        let addr: u16 = addr.into();
        Ok(self.0[addr as usize])
    }
    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        // Little Endian 
        let addr: u16 = addr.into();
        let value = value.to_le_bytes();
        self.0[addr as usize] = value[0];
        self.0[addr as usize + 1] = value[1];
        Ok(())
    }
    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        let addr: u16 = addr.into();
        self.0[addr as usize] = value;
        Ok(())
    }
    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        "".into()
    }
}

#[derive(Debug)]
struct Region {
    device: Box<dyn BusIO + 'static>,
    start: Addr,
    end: Addr,
    remap: bool,
}

impl Region {
    fn new(device: impl BusIO + 'static, start: Addr, end: Addr, remap: bool) -> Self {
        Self {
            device: Box::new(device),
            start,
            end,
            remap,
        }
    }
    
    fn remap_addr(&self, addr: Addr) -> Addr {
        if self.remap {
            addr - self.start
        } else {
            addr
        }
    }
    fn readu16(&self, addr: Addr) -> SResult<u16> {
        let addr = self.remap_addr(addr);
        self.device.readu16(addr)
    }
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        let addr = self.remap_addr(addr);
        self.device.readu8(addr)
    }
    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        let addr = self.remap_addr(addr);
        self.device.writeu16(addr, value)
    }
    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        let addr = self.remap_addr(addr);
        self.device.writeu8(addr, value)
    }
}

// handles choosing the right device according to the address
// decouple the addresses within the device
// from the addresses they are mapped to in the whole of address space (0x0 - 0xfffe)
// pub(crate) struct Bus (Vec<Box<dyn BusIO>>);
#[derive(Debug)]
pub(crate) struct MMU(HashMap<String, Region>);

impl MMU {
    pub fn new(cartridge: Vec<u8>) -> Self {
        let mut mmu = Self(HashMap::new());
        mmu.map("ROM".into(), ROM::new(cartridge), 0x0000.into(), 0x7fff.into(), false).unwrap();
        mmu.map("VRAM".into(), RAM::new(8 * 1024), 0x8000.into(), 0x9fff.into(), true).unwrap();
        mmu.map("ERAM".into(), RAM::new(8 * 1024), 0xa000.into(), 0xbfff.into(), true).unwrap();
        mmu.map("WRAM".into(), RAM::new(8 * 1024), 0xc000.into(), 0xdfff.into(), true).unwrap();
        mmu.map("IOREGS".into(), RAM::new(0xff7f-0xff00+1), 0xff00.into(), 0xff7f.into(), true).unwrap();
        mmu.map("HRAM".into(), RAM::new(0xfffe-0xff80+1), 0xff80.into(), 0xfffe.into(), true).unwrap();
        mmu
    }

    pub fn map(
        &mut self,
        name: String,
        device: impl BusIO + 'static,
        start: Addr,
        end: Addr,
        remap: bool,
    ) -> SResult<()> {
        // check overlap with other regions
        // check if address range is within bounds
        // overlapping regions? 
        let o_r = self.0.iter().find(|(_, r)| {
            !(r.start < start && r.end < start || start < r.start && end < r.start)
        });
        if let Some((o_n, o_r)) = o_r {
            let o_r_s = o_r.start;
            let o_r_e = o_r.end;
            return Err(format!("The new region {name} {start}..{end} overlaps with at least one other region {o_n} {o_r_s}..{o_r_e}").into());
        }
        let region = Region::new(device, start, end, remap);
        self.0.insert(name, region);
        Ok(())
    }

    pub fn unmap(index: usize) -> SResult<()> {
        unimplemented!()
    }

    fn find_region(&self, addr: Addr) -> SResult<&Region> {
        self.0.iter()
            .find(|(_, r)| r.start <= addr && addr <= r.end )
            .map(|(_, r)| r)
            // .ok_or(panic!("Find Region: No mapping found for the address {addr}"))
            .ok_or(format!("Find Region: No mapping found for the address {addr}").into())
    }

    fn find_region_mut(&mut self, addr: Addr) -> SResult<&mut Region> {
        self.0.iter_mut()
            .find(|(_, r)| r.start <= addr && addr <= r.end )
            .map(|(_, r)| r)
            .ok_or(format!("Find Region Mut: No mapping found for the address {addr}").into())
    }

    // exposing the methods for reading and writing values to different mapped devices/regions
    // TODO maybe I can use macros here? 
    // probably proc macros
    pub(crate) fn readu16(&self, addr: Addr) -> u16 {
        let region = self.find_region(addr).unwrap();
        region.readu16(addr).unwrap()
    }
    pub(crate) fn readu8(&self, addr: Addr) -> u8 {
        let region = self.find_region(addr).unwrap();
        region.readu8(addr).unwrap()
    }
    pub(crate) fn writeu16(&mut self, addr: Addr, value: u16) {
        let region = self.find_region_mut(addr).unwrap();
        region.writeu16(addr, value).unwrap()
    }
    pub(crate) fn writeu8(&mut self, addr: Addr, value: u8) {
        let region = self.find_region_mut(addr).unwrap();
        region.writeu8(addr, value).unwrap()
    }

    // debug
    pub(crate) fn print_dbg(&self, start: Addr, len: u16) -> String {
        let region = self.find_region(start).unwrap();
        println!("{}..{}", region.start, region.end);
        region.device.print_dbg(start, len)
    }
}