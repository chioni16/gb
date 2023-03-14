pub(crate) mod busio;
mod not_usable;
pub(crate) mod ram;
mod rom;

use busio::{BusIO, SResult};
use crate::{
    cpu::interrupts::{Interrupts, Interrupt},
    ppu::{
        lcdc::LCDC, palette::{BgWinPalette, ObjPalette}, status::Status, PPU, REG_BG_PALETTE, REG_CURR_SCANLINE, REG_LCDC,
        REG_SCROLL_X, REG_SCROLL_Y, REG_STAT, REG_OBJ_PALETTE_0, REG_OBJ_PALETTE_1, REG_WIN_X, REG_WIN_Y,
    },
    util::Addr, 
    timer::{Timer, REG_DIV, REG_TAC, REG_TIMA, REG_TMA},
};
use not_usable::{NotUsableHigh, NotUsableLow};
use ram::RAM;
use rom::ROM;

const DMA: u16 = 0xff46;
const BANK_REG: u16 = 0xff50;
const IER: u16 = 0xffff;
const IFR: u16 = 0xff0f;

pub(crate) struct MMU {
    pub(crate) boot_disabled: bool,
    bootrom: Option<ROM>,
    cartridge: ROM,
    external_ram: RAM,
    work_ram: RAM,
    high_ram: RAM,

    nuh: NotUsableHigh,
    nul: NotUsableLow,

    ier: Interrupts,

    // joypad: u8,
    
    ppu: PPU,
    timer: Timer,
}

impl MMU {
    fn find_region(&self, addr: Addr) -> SResult<&dyn BusIO> {
        match addr.into() {
            0x0000..0x0100 if !self.boot_disabled => Ok(self.bootrom.as_ref().unwrap()),
            0x0000..0x8000 => Ok(&self.cartridge),
            0x8000..0xa000 => Ok(&self.ppu.vram),
            0xa000..0xc000 => Ok(&self.external_ram),
            0xc000..0xe000 => Ok(&self.work_ram),
            0xe000..0xfe00 => Ok(&self.work_ram),
            0xfe00..0xfea0 => Ok(&self.ppu.oam),
            0xff80..0xffff => Ok(&self.high_ram),
            
            0xff00            => Ok(&self.nuh), // joypad
            
            0xff01 | 0xff02 => Ok(&self.nuh), // serial transfer
            0xff10..=0xff3f => Ok(&self.nuh), // audio
            0xff51..=0xff7f => Ok(&self.nuh), // io regs
            0xfea0..0xff00 => Ok(&self.nuh), // not usable
            
            
            _              => Err(format!("No read region corresponding to address: {:x?}", addr).into())
        }
    }

    fn find_region_mut<'a>(&mut self, addr: Addr) -> SResult<&mut dyn BusIO> {
        match addr.into() {
            0x0000..0x0100 if !self.boot_disabled => Ok(self.bootrom.as_mut().unwrap()),
            0x0000..0x8000 => Ok(&mut self.cartridge),
            0x8000..0xa000 => Ok(&mut self.ppu.vram),
            0xa000..0xc000 => Ok(&mut self.external_ram),
            0xc000..0xe000 => Ok(&mut self.work_ram),
            0xe000..0xfe00 => Ok(&mut self.work_ram),
            0xfe00..0xfea0 => Ok(&mut self.ppu.oam),
            0xff80..0xffff => Ok(&mut self.high_ram),

            0xff00            => Ok(&mut self.nuh), // joypad

            0xff01 | 0xff02 => Ok(&mut self.nuh), // serial transfer
            0xff10..=0xff3f => Ok(&mut self.nuh), // audio
            0xff51..=0xff7f => Ok(&mut self.nuh), // io regs
            // 0xff06 => Ok(&mut self.nuh), // timer
            0xfea0..0xff00 => Ok(&mut self.nuh), // not usable
            


            _              => Err(format!("No write region corresponding to address: {:x?}", addr).into())
        }
    }
}

impl MMU {
    pub fn new(bootrom: Option<Vec<u8>>, cartridge: Vec<u8>) -> Self {
        Self {
            boot_disabled: false, 

            bootrom: bootrom.map(ROM::new),
            cartridge: ROM::new(cartridge),
            external_ram: RAM::new(8 * 1024, Box::new(|addr: Addr| addr - 0xa000.into()), 0),
            work_ram: RAM::new(
                8 * 1024,
                Box::new(|addr: Addr| (u16::from(addr) & 0b11_1111_1111_1111).into()),
                0,
            ),
            high_ram: RAM::new(0xfe - 0x80 + 1, Box::new(|addr: Addr| addr - 0xff80.into()), 0),
            nuh: NotUsableHigh,
            nul: NotUsableLow,

            ier: { let mut i: Interrupts = Default::default(); i.set(Interrupt::VBlank); i},

            // joypad: 0,

            ppu: PPU::new(),
            timer: Default::default(),
        }
    }

    pub(crate) fn readu16(&self, addr: Addr) -> u16 {
        let region = self.find_region(addr).unwrap();
        region.readu16(addr).unwrap()
    }
    pub(crate) fn readu8(&self, addr: Addr) -> u8 {
        match addr.into() {
            REG_LCDC          => self.ppu.lcdc.into(),
            REG_STAT          => self.ppu.status.into(),
            REG_SCROLL_X      => self.ppu.scx,
            REG_SCROLL_Y      => self.ppu.scy,
            REG_WIN_X         => self.ppu.wx,
            REG_WIN_Y         => self.ppu.wy,
            REG_CURR_SCANLINE => self.ppu.curr_scanline,

            REG_DIV           => self.timer.read_divider(),
            REG_TIMA          => self.timer.read_counter(),
            REG_TMA           => self.timer.read_modulo(),
            REG_TAC           => self.timer.read_control(),
            
            BANK_REG          => if self.boot_disabled { 1 } else { 0 },

            IER               => self.ier.into(),
            IFR               => self.ifr().into(),

            // 0x8000..0xa000    => self.ppu.vram.readu8(addr).unwrap(),

            _ => {
                let region = self.find_region(addr).unwrap();
                region.readu8(addr).unwrap()
            }
        }
    }
    pub(crate) fn writeu16(&mut self, addr: Addr, value: u16) {
        let region = self.find_region_mut(addr).unwrap();
        region.writeu16(addr, value).unwrap()
    }

    pub(crate) fn writeu8(&mut self, addr: Addr, value: u8) {
        match addr.into() {
            REG_LCDC          => self.ppu.lcdc = LCDC::from(value),
            REG_STAT          => self.ppu.status = Status::from(value),
            REG_SCROLL_X      => self.ppu.scx = value,
            REG_SCROLL_Y      => self.ppu.scy = value,
            REG_WIN_X         => self.ppu.wx = value,
            REG_WIN_Y         => self.ppu.wy = value,
            REG_BG_PALETTE    => self.ppu.bgp = BgWinPalette::from(value),
            REG_OBJ_PALETTE_0 => self.ppu.obp0 = ObjPalette::from(value),
            REG_OBJ_PALETTE_1 => self.ppu.obp1 = ObjPalette::from(value),

            REG_DIV           => self.timer.write_divider(value),
            REG_TIMA          => self.timer.write_counter(value),
            REG_TMA           => self.timer.write_modulo(value),
            REG_TAC           => self.timer.write_control(value),

            IER               => self.ier = value.into(),
            IFR               => self.ifr_set(value),
            // 0x8000..0xa000    => self.ppu.vram.writeu8(addr, value).unwrap(),

            DMA               => {
                // println!("start dma");
                let addr = ((value as u16) << 8).into();
                let mem_reg = self.find_region(addr).unwrap();
                let slice = mem_reg.as_slice(addr, 0xa0).unwrap().to_owned();
                self.ppu.dma(&slice);
                // println!("end dma");
            }

            BANK_REG       => self.boot_disabled = value != 0,
            _ => {
                let region = self.find_region_mut(addr).unwrap();
                region.writeu8(addr, value).unwrap()
            }
        }
    }

    pub(crate) fn tick(&mut self, cpu_ticks: u64) {
        self.ppu.tick(cpu_ticks);
        self.timer.tick(cpu_ticks as u16);
    }

    fn ifr(&self) -> Interrupts {
        let mut ifr: Interrupts = Default::default();
        if self.ppu.vblank_interrupt {
            ifr.set(Interrupt::VBlank);
        }
        if self.timer.interrupt {
            ifr.set(Interrupt::Timer);
        }
        ifr
    }

    fn ifr_set(&mut self, value: u8) {
        let ifr: Interrupts = value.into();
        self.ppu.vblank_interrupt = ifr.vblank;
        self.timer.interrupt = ifr.timer;
    }
}
