pub(crate) const REG_DIV: u16 = 0xff04;
pub(crate) const REG_TIMA: u16 = 0xff05;
pub(crate) const REG_TMA: u16 = 0xff06;
pub(crate) const REG_TAC: u16 = 0xff07;

// https://pixelbits.16-b.it/GBEDG/timers/

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Timer {
    divider: u16,
    counter: u8,
    tm: u8,
    tc: TimerControl,
    prev_and_res: bool,
    pub(crate) interrupt: bool,
}

impl Timer {
    pub(crate) fn read_divider(&self) -> u8 {
        (self.divider >> 8) as u8
    }

    pub(crate) fn write_divider(&mut self, _val: u8) {
        self.divider = 0;
    }

    pub(crate) fn read_counter(&self) -> u8 {
        self.counter
    }

    pub(crate) fn write_counter(&mut self, val: u8) {
        self.counter = val;
    }

    pub(crate) fn read_modulo(&self) -> u8 {
        self.tm
    }

    pub(crate) fn write_modulo(&mut self, val: u8) {
        self.tm = val;
    }

    pub(crate) fn read_control(&self) -> u8 {
        self.tc.into()
    }

    pub(crate) fn write_control(&mut self, val: u8) {
        self.tc = val.into();
    }

    pub(crate) fn tick(&mut self, cpu_ticks: u16) {
        // increment divider
        self.divider += cpu_ticks;

        // A bit position of the 16-bit counter is determined based on the lower 2 bits of the TAC register, as seen here:
        // 0b00: Bit 9
        // 0b01: Bit 3
        // 0b10: Bit 5
        // 0b11: Bit 7
        let bit = match self.tc.select {
            Select::Zero => 9,
            Select::One => 3,
            Select::Two => 5,
            Select::Three => 7,
        };
        // The bit taken from the DIV counter is ANDed with the Timer Enable bit.
        // The result of this operation will be referred to as the “AND Result”.
        let new_and_res = self.tc.enable && (self.divider & (1 << bit) != 0);

        // TIMA register is incremented only when there is a falling edge
        if self.prev_and_res && !new_and_res {
            let (new_tima, of) = self.counter.overflowing_add(1);
            self.counter = if of {
                // when TIMA overflows, it's set to the value present in TMA
                // and a timer interrupt is requested.
                self.interrupt = true;
                self.tm
            } else {
                new_tima
            };
        }
        self.prev_and_res = new_and_res;
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TimerControl {
    enable: bool,
    select: Select,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
enum Select {
    #[default]
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
            _ => unreachable!(),
        };
        TimerControl { enable, select }
    }
}

impl From<TimerControl> for u8 {
    fn from(value: TimerControl) -> Self {
        (value.enable as u8) << 2 | value.select as u8
    }
}
