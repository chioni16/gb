#[allow(dead_code)]
pub(super) struct Registers {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
}

impl Registers {
    pub fn new() -> Self {
        Self { 
            a: 0,
            f: 0,
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
        self.a
    }
    pub fn get_c(&self) -> u8 {
        self.a
    }
    pub fn get_d(&self) -> u8 {
        self.a
    }
    pub fn get_e(&self) -> u8 {
        self.a
    }
    pub fn get_h(&self) -> u8 {
        self.a
    }
    pub fn get_l(&self) -> u8 {
        self.a
    }
    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }
    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_a(&mut self, v: u8) {
        self.a = v;
    }
    pub fn set_b(&mut self, v: u8) {
        self.a = v;
    }
    pub fn set_c(&mut self, v: u8) {
        self.a = v;
    }
    pub fn set_d(&mut self, v: u8) {
        self.a = v;
    }
    pub fn set_e(&mut self, v: u8) {
        self.a = v;
    }
    pub fn set_h(&mut self, v: u8) {
        self.a = v;
    }
    pub fn set_l(&mut self, v: u8) {
        self.a = v;
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
}

fn msb(v: u16) -> u8 {(
    (v & 0xff00) >> 8) as u8
}

fn lsb(v: u16) -> u8 {
    (v & 0x00ff) as u8
}