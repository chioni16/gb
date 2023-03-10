use std::ops::BitAnd;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Interrupts {
    pub vblank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
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
