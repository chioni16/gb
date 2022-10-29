use std::ops::{Add, AddAssign};

mod flags;
mod instruction;
mod mmu;
mod registers;

use flags::Flags;
use instruction::decode;
use mmu::MMU;
use registers::Registers;

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

    // stack
    pub fn push(&mut self, v: u16) {
        self.mmu.writeu16(self.sp, v);
    }

    pub fn set_sp(&mut self, v: u16) {
        self.sp = v.into();
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
