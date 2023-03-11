mod busio;
mod not_usable;
mod ram;
mod region;
mod rom;

use busio::{BusIO, SResult};
use not_usable::NotUsable;
use ram::RAM;
use rom::ROM;
use crate::{
    util::Addr,
    // timer::Timer,
    // DIVIDER_WRITE,
};

pub(crate) struct MMU {
    bootrom: Option<ROM>,
    cartridge: ROM,
    video_ram: RAM,
    external_ram: RAM,
    work_ram: RAM,
    oam: RAM,
    io_regs: RAM,
    high_ram: RAM,
    ier: u8,
    nu: NotUsable,
}

impl MMU {
    pub fn new(bootrom: Option<Vec<u8>>, cartridge: Vec<u8>) -> Self {
        Self {
            bootrom: bootrom.map(ROM::new),
            cartridge: ROM::new(cartridge),
            video_ram: RAM::new(8 * 1024, Box::new(|addr: Addr| {addr-0x8000.into()})),
            external_ram: RAM::new(8 *1024, Box::new(|addr: Addr| {addr-0xa000.into()})),
            work_ram: RAM::new(8 * 1024, Box::new(|addr: Addr| {(u16::from(addr) & 0b11_1111_1111_1111).into()})),
            oam: RAM::new(0xa0, Box::new(|addr: Addr| {addr-0xfe00.into()})),
            io_regs: RAM::new(0x80, Box::new(|addr: Addr| {addr-0xff00.into()})),
            high_ram: RAM::new(0xfe-0x80+1, Box::new(|addr: Addr| {addr-0xff80.into()})),
            ier: 0u8,
            nu: NotUsable,
        }
    }

    fn find_region(&self, addr: Addr) -> SResult<&dyn BusIO> {
        match addr.into() {
            0x0000..0x0100 if self.bootrom_enabled() => Ok(self.bootrom.as_ref().unwrap()),
            0x0000..0x8000 => Ok(&self.cartridge),
            // 0x8000..0xa000 => { println!("vram"); Ok(&self.video_ram) } ,
            // 0xa000..0xc000 => { println!("xram"); Ok(&self.external_ram) },
            // 0xc000..0xe000 => { println!("wram"); Ok(&self.work_ram) },
            // 0xe000..0xfe00 => { println!("eram"); Ok(&self.work_ram) },
            0x8000..0xa000 => Ok(&self.video_ram),
            0xa000..0xc000 => Ok(&self.external_ram),
            0xc000..0xe000 => Ok(&self.work_ram),
            0xe000..0xfe00 => Ok(&self.work_ram),
            0xfe00..0xfea0 => Ok(&self.oam),
            0xff00..0xff80 => Ok(&self.io_regs),
            0xff80..0xffff => Ok(&self.high_ram),
            0xffff         => Ok(&self.ier),
            _              => Ok(&self.nu),
            // _              => Err(format!("No region corresponding to address: {:x?}", addr).into())
        }
    }
    
    fn find_region_mut(&mut self, addr: Addr) -> SResult<&mut dyn BusIO> {
        match addr.into() {
            0x0000..0x0100 if self.bootrom_enabled() => Ok(self.bootrom.as_mut().unwrap()),
            0x0000..0x8000 => Ok(&mut self.cartridge),
            // 0x8000..0xa000 => { println!("vram"); Ok(&mut self.video_ram) } ,
            // 0xa000..0xc000 => { println!("xram"); Ok(&mut self.external_ram) },
            // 0xc000..0xe000 => { println!("wram"); Ok(&mut self.work_ram) },
            // 0xe000..0xfe00 => { println!("eram"); Ok(&mut self.work_ram) },
            0x8000..0xa000 => Ok(&mut self.video_ram),
            0xa000..0xc000 => Ok(&mut self.external_ram),
            0xc000..0xe000 => Ok(&mut self.work_ram),
            0xe000..0xfe00 => Ok(&mut self.work_ram),
            0xfe00..0xfea0 => Ok(&mut self.oam),
            0xff00..0xff80 => Ok(&mut self.io_regs),
            0xff80..0xffff => Ok(&mut self.high_ram),
            0xffff         => Ok(&mut self.ier),
            _              => Ok(&mut self.nu),
            // _              => Err(format!("No region corresponding to address: {:x?}", addr).into())
        }
    }

    #[inline]
    fn bootrom_enabled(&self) -> bool {
        self.readu8(0xff50.into()) == 0
    }

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
}

// handles choosing the right device according to the address
// decouple the addresses within the device
// from the addresses they are mapped to in the whole of address space (0x0 - 0xfffe)
// pub(crate) struct Bus (Vec<Box<dyn BusIO>>);
// #[derive(Debug)]
// pub(crate) struct MMU {
//     regions: HashMap<String, Region>,
//     pub timer: Timer,
// }

// impl MMU {
    // pub fn new(cartridge: Vec<u8>) -> Self {
    //     let mut mmu = Self {
    //         regions: HashMap::new(),
    //         timer: Timer::new(),
    //     };
    //     mmu.map(
    //         "ROM".into(),
    //         ROM::new(cartridge),
    //         0x0000.into(),
    //         0x7fff.into(),
    //         false,
    //     )
    //     .unwrap();
    //     mmu.map(
    //         "VRAM".into(),
    //         RAM::new(8 * 1024),
    //         0x8000.into(),
    //         0x9fff.into(),
    //         true,
    //     )
    //     .unwrap();
    //     mmu.map(
    //         "ERAM".into(),
    //         RAM::new(8 * 1024),
    //         0xa000.into(),
    //         0xbfff.into(),
    //         true,
    //     )
    //     .unwrap();
    //     mmu.map(
    //         "WRAM".into(),
    //         RAM::new(8 * 1024),
    //         0xc000.into(),
    //         0xdfff.into(),
    //         true,
    //     )
    //     .unwrap();
    //     mmu.map(
    //         "IOREGS".into(),
    //         RAM::new(0xff7f - 0xff00 + 1),
    //         0xff00.into(),
    //         0xff7f.into(),
    //         true,
    //     )
    //     .unwrap();
    //     mmu.map(
    //         "HRAM".into(),
    //         RAM::new(0xfffe - 0xff80 + 1),
    //         0xff80.into(),
    //         0xfffe.into(),
    //         true,
    //     )
    //     .unwrap();
    //     mmu.map(
    //         "IER".into(),
    //         RAM::new(0x01),
    //         0xffff.into(),
    //         0xffff.into(),
    //         true,
    //     )
    //     .unwrap();
    //     // mmu.writeu8(0xff44.into(), 0x90);
    //     mmu
    // }

    // pub fn map(
    //     &mut self,
    //     name: String,
    //     device: impl BusIO + 'static,
    //     start: Addr,
    //     end: Addr,
    //     remap: bool,
    // ) -> SResult<()> {
    //     // check overlap with other regions
    //     // check if address range is within bounds
    //     // overlapping regions?
    //     let o_r = self
    //         .regions
    //         .iter()
    //         .find(|(_, r)| !(r.start < start && r.end < start || start < r.start && end < r.start));
    //     if let Some((o_n, o_r)) = o_r {
    //         let o_r_s = o_r.start;
    //         let o_r_e = o_r.end;
    //         return Err(format!("The new region {name} {start}..{end} overlaps with at least one other region {o_n} {o_r_s}..{o_r_e}").into());
    //     }
    //     let region = Region::new(device, start, end, remap);
    //     self.regions.insert(name, region);
    //     Ok(())
    // }

    // pub fn unmap(index: usize) -> SResult<()> {
    //     unimplemented!()
    // }

    // fn find_region(&self, mut addr: Addr) -> SResult<&Region> {
    //     if MIRROR_START <= addr && addr <= MIRROR_END {
    //         addr = addr - 0x2000u16.into();
    //     }
    //     self.regions
    //         .iter()
    //         .find(|(_, r)| r.start <= addr && addr <= r.end)
    //         .map(|(_, r)| r)
    //         // .ok_or(panic!("Find Region: No mapping found for the address {addr}"))
    //         .ok_or(format!("Find Region: No mapping found for the address {:#x?}", addr).into())
    // }

    // fn find_region_mut(&mut self, mut addr: Addr) -> SResult<&mut Region> {
    //     if MIRROR_START <= addr && addr <= MIRROR_END {
    //         addr = addr - 0x2000u16.into();
    //         println!("new addr: {:#x?}", addr);
    //     }
    //     self.regions
    //         .iter_mut()
    //         .find(|(_, r)| r.start <= addr && addr <= r.end)
    //         .map(|(_, r)| r)
    //         .ok_or(
    //             format!(
    //                 "Find Region Mut: No mapping found for the address {:#x?}",
    //                 addr
    //             )
    //             .into(),
    //         )
    // }

    // // exposing the methods for reading and writing values to different mapped devices/regions
    // // TODO maybe I can use macros here?
    // // probably proc macros
    // pub(crate) fn readu16(&self, addr: Addr) -> u16 {
    //     let region = self.find_region(addr).unwrap();
    //     region.readu16(addr).unwrap()
    // }
    // pub(crate) fn readu8(&self, addr: Addr) -> u8 {
    //     let region = self.find_region(addr).unwrap();
    //     region.readu8(addr).unwrap()
    // }
    // pub(crate) fn writeu16(&mut self, addr: Addr, value: u16) {
    //     let region = self.find_region_mut(addr).unwrap();
    //     region.writeu16(addr, value).unwrap()
    // }
    // pub(crate) fn writeu8(&mut self, addr: Addr, mut value: u8) {
    //     if addr == 0xff01.into() {
    //         if self.readu8(0xff02.into()) == 0x81 {
    //             eprintln!("{}", value as char);
    //         }
    //         // return
    //     }
    //     // timer divider
    //     if addr == 0xff04.into() {
    //         value = 0;
    //         unsafe {
    //             DIVIDER_WRITE = true;
    //         }
    //     }

    //     let region = self.find_region_mut(addr).unwrap();
    //     region.writeu8(addr, value).unwrap()
    // }

    // debug
    // pub(crate) fn print_dbg(&self, start: Addr, len: u16) -> String {
    //     let region = self.find_region(start).unwrap();
    //     println!("{}..{}", region.start, region.end);
    //     region.device.print_dbg(start, len)
    // }
// }
