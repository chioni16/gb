use std::{ops::{Add, AddAssign, Sub, SubAssign}, fmt::Display};

mod instruction;
mod mmu;
mod registers;

use instruction::decode;
use mmu::MMU;
use registers::Registers;
use crate::cpu::registers::Flags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub (crate) struct Addr(u16);
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

impl Sub for Addr {
    type Output = Addr;
    fn sub(self, rhs: Self) -> Self::Output {
        Addr(self.0 - rhs.0)
    }
}

impl AddAssign for Addr {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl SubAssign for Addr {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;        
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

impl Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

pub(crate) struct CPU {
    mmu: MMU,
    regs: Registers, 
    pc: Addr,
    sp: Addr,
}

impl CPU {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            mmu: MMU::new(rom), 
            regs: Registers::new(),
            pc: Addr::new(),
            sp: Addr::new(),
        }
    }

    pub fn no_boot(&mut self) {
        self.regs.a = 0x01;
        self.regs.f = 0xb0.into();
        self.regs.b = 0x00;
        self.regs.c = 0x13;
        self.regs.d = 0x00;
        self.regs.e = 0xd8;
        self.regs.h = 0x01;
        self.regs.l = 0x4d;
        self.pc = 0x0100.into();
        self.sp = 0xfffe.into();
    }
   
    pub(crate) fn step(&mut self) {
        println!("A: {:0>2X} F: {:0>2X} B: {:0>2X} C: {:0>2X} D: {:0>2X} E: {:0>2X} H: {:0>2X} L: {:0>2X} SP: {:0>4X} PC: 00:{:0>4X} ({:0>2X} {:0>2X} {:0>2X} {:0>2X})", self.regs.a, <Flags as Into<u8>>::into(self.regs.f), self.regs.b, self.regs.c, self.regs.d, self.regs.e, self.regs.h, self.regs.l, self.sp.0, self.pc.0, self.mmu.readu8(self.pc),self.mmu.readu8(self.pc+1.into()),self.mmu.readu8(self.pc+2.into()),self.mmu.readu8(self.pc+3.into()));
        let mut opcode = self.readu8() as u16;
        if opcode == 0xcb {
            opcode = opcode << 8 | self.readu8() as u16;
        }
        // println!("{:#x?}\t", opcode);
        let _cycles = decode(opcode, self);
    }

    // stack
    pub fn push_stack(&mut self, v: u16) {
        // TODO 
        // invariance? sp pointing to the location where next piece of information
        // can be written? 
        self.sp -= 2.into();
        self.mmu.writeu16(self.sp, v);
    }
    pub fn pop_stack(&mut self) -> u16 {
        let v = self.mmu.readu16(self.sp);
        self.sp += 2.into();
        v
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
