#[derive(Debug)]
pub(super) struct Flags {
    pub zero: bool,
    pub subtraction: bool,
    pub half_carry: bool,
    pub carry: bool,
}

#[allow(dead_code)]
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