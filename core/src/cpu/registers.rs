#[derive(Debug, Clone, Copy)]
pub(super) struct Flags {
    pub zero: bool,
    pub subtraction: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl Flags {
    pub fn new() -> Self {
        Self { 
            zero: false,
            subtraction: false,
            half_carry: false,
            carry: false,
        }
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        let mut f = Self::new();     
        f.zero = (value >> 7) == 1; 
        f.subtraction = (value >> 6) == 1;
        f.half_carry = (value >> 5) == 1;
        f.carry = (value >> 4) == 1;
        f
    }
}

impl From<Flags> for u8 {
    fn from(value: Flags) -> Self {
        ((((value.zero as u8) << 1 
        | value.subtraction as u8) << 1
        | value.half_carry as u8) << 1
        | value.carry as u8) << 4
    }
}

pub(super) struct Registers {
    pub a: u8,
    pub f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    pub fn new() -> Self {
        Self { 
            a: 0,
            f: Flags::new(),
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }

    pub fn get_a(&self) -> u8 {
        self.a
    }
    pub fn get_b(&self) -> u8 {
        self.b
    }
    pub fn get_c(&self) -> u8 {
        self.c
    }
    pub fn get_d(&self) -> u8 {
        self.d
    }
    pub fn get_e(&self) -> u8 {
        self.e
    }
    pub fn get_f(&self) -> u8 {
        self.f.into()
    }
    pub fn get_h(&self) -> u8 {
        self.h
    }
    pub fn get_l(&self) -> u8 {
        self.l
    }
    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }
    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }
    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }
    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | self.get_f() as u16
    }

    pub fn set_a(&mut self, v: u8) {
        self.a = v;
    }
    pub fn set_b(&mut self, v: u8) {
        self.b = v;
    }
    pub fn set_c(&mut self, v: u8) {
        self.c = v;
    }
    pub fn set_d(&mut self, v: u8) {
        self.d = v;
    }
    pub fn set_e(&mut self, v: u8) {
        self.e = v;
    }
    pub fn set_f(&mut self, v: u8) {
        self.f = v.into();
    }
    pub fn set_h(&mut self, v: u8) {
        self.h = v;
    }
    pub fn set_l(&mut self, v: u8) {
        self.l = v;
    }
    pub fn set_bc(&mut self, v: u16) {
        self.c = lsb(v);
        self.b = msb(v);
    }
    pub fn set_de(&mut self, v: u16) {
        self.e = lsb(v);
        self.d = msb(v);
    }
    pub fn set_hl(&mut self, v: u16) {
        self.l = lsb(v);
        self.h = msb(v)
    }
    pub fn incr_hl(&mut self) {
        let v = self.get_hl()+1;
        self.set_hl(v);
    }
    pub fn decr_hl(&mut self) {
        let v = self.get_hl()-1;
        self.set_hl(v);
    }
    pub fn set_af(&mut self, v: u16) {
        // In BC, B is the high byte and C the low. So C appears first in memory.
        // https://www.reddit.com/r/EmuDev/comments/tqdt9b/gameboy_endianness_and_registers/ 
        let v = v.to_be_bytes();
        self.set_a(v[0]);
        self.set_f(v[1]);
    }
}

fn msb(v: u16) -> u8 {(
    (v & 0xff00) >> 8) as u8
}

fn lsb(v: u16) -> u8 {
    (v & 0x00ff) as u8
}