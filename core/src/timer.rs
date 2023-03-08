use crate::{Addr, mmu::MMU, DIVIDER_WRITE};

const DIV:  Addr = Addr::from(0xff04);
const TIMA: Addr = Addr::from(0xff05);
const TMA:  Addr = Addr::from(0xff06);
const TAC:  Addr = Addr::from(0xff07);

#[derive(Debug)]
pub(crate) struct Timer {
    divider: DivTimer,
    counter: Counter,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            divider: DivTimer::new(),
            counter: Counter::new(),
        }
    }

    pub fn tick(&mut self, mmu: &mut MMU, cpu_ticks: u16) {
        self.divider.tick(mmu, cpu_ticks);
        self.counter.tick(mmu, self.divider.0);
    }

    pub fn write_divider(&mut self) {
        self.divider.0 = 0;
    }
}
#[derive(Debug)]
pub(crate) struct DivTimer(u16);

impl DivTimer {
    pub fn new() -> Self {
        Self(0)
    }
    // pub fn read(&self) -> u8 {
    //     (self.0 >> 8) as u8
    // }

    // pub fn write(&mut self, mmu: &mut MMU, _value: u8) {
    //     self.0 = 0;
    //     mmu.writeu8(DIV, 0); 
    // }

    pub fn tick(&mut self, mmu: &mut MMU, cpu_ticks: u16) {
        unsafe {
            if DIVIDER_WRITE {
                self.0 = 0;
                DIVIDER_WRITE = false;
            }
        }
        let new_tick =  self.0.wrapping_add(cpu_ticks);
        if new_tick as u8 == 0 {
            mmu.writeu8(DIV, (new_tick >> 8) as u8); 
        }
        self.0 = new_tick;
    }
}

#[derive(Debug)]
pub(crate) struct Counter(bool);

impl Counter {
    pub fn new() -> Self {
        Self(false)
    }
    // pub fn read(&self) -> u8 {
    //     (self.0 >> 8) as u8
    // }

    // pub fn write(&mut self, mmu: &mut MMU, _value: u8) {
    //     self.0 = 0;
    //     mmu.writeu8(DIV, 0); 
    // }

    pub fn tick(&mut self, mmu: &mut MMU, div: u16) {
        // The “Timer Enable” bit (Bit 2) is extracted from the value 
        // in the TAC register and stored for the next step.
        let tc = mmu.readu8(TAC);
        let tc = TimerControl::from(tc);
        // A bit position of the 16-bit counter is determined based on the lower 2 bits of the TAC register, as seen here:
        // 0b00: Bit 9
        // 0b01: Bit 3
        // 0b10: Bit 5
        // 0b11: Bit 7
        let bit = match tc.select {
            Select::Zero => 9,
            Select::One  => 3,
            Select::Two  => 5,
            Select::Three => 7,
        };
        // The bit taken from the DIV counter is ANDed with the Timer Enable bit. 
        // The result of this operation will be referred to as the “AND Result”.
        let new = tc.enable && div & (1 << bit) != 0;
        if self.0 && !new {
            let tima = mmu.readu8(TIMA);
            let (mut new_tima, of) = tima.overflowing_add(1);
            if of {
                new_tima = mmu.readu8(TMA);
            }
            mmu.writeu8(TIMA, new_tima);
        }
        self.0 = new;
    }
}   

struct TimerControl {
    enable: bool,
    select: Select,
}

#[repr(u8)]
enum Select {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

impl From<u8> for TimerControl {
    fn from(value: u8) -> Self {
        let enable = (value & 0b0000_0100) >> 2 == 1;
        let select = match value & 0b0000_0011 {
            0 => Select::Zero,
            1 => Select::One,
            2 => Select::Two,
            3 => Select::Three,
            _ => unreachable!()
        };
        TimerControl { enable, select }
    }
}

impl From<TimerControl> for u8 {
    fn from(value: TimerControl) -> Self {
        (value.enable as u8) << 2 
        | value.select as u8
    }
}