use std::ops::{Add, AddAssign};

mod instruction;
mod mmu;

use instruction::decode;
use mmu::MMU;

#[allow(dead_code)]
struct Registers {
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
    fn new() -> Self {
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
}


struct Flags {

}

#[allow(dead_code)]
impl Flags {
    fn new() -> Self {
        Self {  }
    }
    fn set() {}
    fn unset() {}
    fn toggle() {}
}

// impl From<u8> for Flags {
//     fn from(value: u8) -> Self {
        
//     }
// }

// impl From<Flags> for u8 {
//     fn from(value: Flags) -> Self {
        
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Addr(u16);
impl Addr {
    fn new() -> Self {
        Self(0)
    }
}

impl Add for Addr {
    type Output = Addr;
    fn add(self, rhs: Self) -> Self::Output {
        Addr(self.0 + rhs.0)
    }
}

impl AddAssign for Addr {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl From<u16> for Addr {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Addr> for u16 {
    fn from(value: Addr) -> Self {
        value.0
    }
}

#[allow(dead_code)]
pub(crate) struct CPU {
    mmu: MMU,
    regs: Registers, 
    flags: Flags, 
    pc: Addr,
    sp: Addr,
}

#[allow(dead_code)]
impl CPU {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            mmu: MMU::new(rom), 
            regs: Registers::new(),
            flags: Flags::new(),
            pc: Addr::new(),
            sp: Addr::new(),
        }
    }
   
    pub(crate) fn step(&mut self) {
        print!("{:#x?}\t", self.pc.0);
        let mut opcode = self.readu8() as u16;
        if opcode == 0xcb {
            opcode = opcode << 8 | self.readu8() as u16;
        }
        print!("{:#x?}\t", opcode);
        decode(opcode, self).unwrap();
    }
    
    // mmu
    pub fn readu8(&mut self) -> u8 {
        let ret = self.mmu.readu8(self.pc);
        self.pc += 1.into();
        ret
    }
    pub fn readu16(&mut self) -> u16 {
        let ret = self.mmu.readu16(self.pc);
        self.pc += 2.into();
        ret
    }
    pub fn writeu8(&mut self, value: u8) {
        self.mmu.writeu8(self.pc, value);
    }
    pub fn writeu16(&mut self, value: u16) {
        self.mmu.writeu16(self.pc, value);
    }

}