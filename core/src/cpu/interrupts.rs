use std::ops::BitAnd;
use crate::mmu::busio::{BusIO, SResult};
use crate::util::Addr;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Interrupts {
    pub vblank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

impl Interrupts {
    pub(crate) fn set(&mut self, i: Interrupt) -> &mut Self {
        match i {
            Interrupt::VBlank => self.vblank = true,
            Interrupt::LCDStat => self.lcd_stat = true,
            Interrupt::Timer => self.timer = true,
            Interrupt::Serial => self.serial = true,
            Interrupt::Joypad => self.joypad = true,
        };
        self
    }   
    pub(crate) fn unset(&mut self, i: Interrupt) -> &mut Self {
        match i {
            Interrupt::VBlank => self.vblank = false,
            Interrupt::LCDStat => self.lcd_stat = false,
            Interrupt::Timer => self.timer = false,
            Interrupt::Serial => self.serial = false,
            Interrupt::Joypad => self.joypad = false,
        };
        self
    }   
    pub(crate) fn toggle(&mut self, i: Interrupt) -> &mut Self {
        match i {
            Interrupt::VBlank => self.vblank = !self.vblank,
            Interrupt::LCDStat => self.lcd_stat = !self.lcd_stat,
            Interrupt::Timer => self.timer = !self.timer,
            Interrupt::Serial => self.serial = !self.serial,
            Interrupt::Joypad => self.joypad = !self.joypad,
        };
        self
    }   
}

impl From<u8> for Interrupts {
    fn from(value: u8) -> Self {
        Interrupts {
            vblank: ((value >> 0) & 1) == 1,
            lcd_stat: ((value >> 1) & 1) == 1,
            timer: ((value >> 2) & 1) == 1,
            serial: ((value >> 3) & 1) == 1,
            joypad: ((value >> 4) & 1) == 1,
        }
    }
}

impl From<Interrupts> for u8 {
    fn from(value: Interrupts) -> Self {
        ((((value.joypad as u8) << 1 | value.serial as u8) << 1 | value.timer as u8) << 1
            | value.lcd_stat as u8)
            << 1
            | value.vblank as u8
    }
}

impl BitAnd for Interrupts {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let lhs: u8 = self.into();
        let rhs: u8 = rhs.into();
        (lhs & rhs).into()
    }
}

impl Interrupts {
    pub fn next_interrupt(self) -> Option<Interrupt> {
        if self.vblank {
            return Some(Interrupt::VBlank);
        }
        if self.lcd_stat {
            return Some(Interrupt::LCDStat);
        }
        if self.timer {
            return Some(Interrupt::Timer);
        }
        if self.serial {
            return Some(Interrupt::Serial);
        }
        if self.joypad {
            return Some(Interrupt::Joypad);
        }
        None
    }
}

pub(crate) enum Interrupt {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

impl BusIO for Interrupts {
    fn readu8(&self, _addr: Addr) -> SResult<u8> {
        Ok((*self).into())
    }

    fn writeu8(&mut self, _addr: Addr, value: u8) -> SResult<()> {
        *self =  value.into();
        Ok(())
    }

    fn readu16(&self, _addr: Addr) -> SResult<u16> {
        unimplemented!()
    }

    fn writeu16(&mut self, _addr: Addr, _value: u16) -> SResult<()> {
        unimplemented!()
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }

    fn as_slice(&self, _addr: Addr, _len: usize) -> SResult<&[u8]> {
        unimplemented!()
    }
}


