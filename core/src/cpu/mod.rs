mod instruction;
mod registers;

use super::Addr;
use super::mmu::MMU;
use instruction::decode;
use registers::Registers;
use registers::Flags;


pub(crate) struct CPU {
    regs: Registers, 
    pc: Addr,
    sp: Addr,
}

impl CPU {
    pub fn new() -> Self {
        Self {
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
   
    pub(crate) fn step(&mut self, mmu: &mut MMU) -> usize {
        println!("A: {:0>2X} F: {:0>2X} B: {:0>2X} C: {:0>2X} D: {:0>2X} E: {:0>2X} H: {:0>2X} L: {:0>2X} SP: {:0>4X} PC: 00:{:0>4X} ({:0>2X} {:0>2X} {:0>2X} {:0>2X})", self.regs.a, <Flags as Into<u8>>::into(self.regs.f), self.regs.b, self.regs.c, self.regs.d, self.regs.e, self.regs.h, self.regs.l, self.sp.0, self.pc.0, mmu.readu8(self.pc),mmu.readu8(self.pc+1.into()),mmu.readu8(self.pc+2.into()),mmu.readu8(self.pc+3.into()));
        let mut opcode = self.readu8(mmu) as u16;
        if opcode == 0xcb {
            opcode = opcode << 8 | self.readu8(mmu) as u16;
        }
        // println!("{:#x?}\t", opcode);
        decode(opcode, self, mmu)
    }

    // stack
    pub fn push_stack(&mut self, mmu: &mut MMU, v: u16) {
        // TODO 
        // invariance? sp pointing to the location where next piece of information
        // can be written? 
        self.sp -= 2.into();
        mmu.writeu16(self.sp, v);
    }
    pub fn pop_stack(&mut self, mmu: &MMU) -> u16 {
        let v = mmu.readu16(self.sp);
        self.sp += 2.into();
        v
    }

    pub fn set_sp(&mut self, v: u16) {
        self.sp = v.into();
    }

    // mmu
    pub fn readu8(&mut self, mmu: &MMU) -> u8 {
        let ret = mmu.readu8(self.pc);
        self.pc += 1.into();
        ret
    }
    pub fn readu16(&mut self, mmu: &MMU) -> u16 {
        let ret = mmu.readu16(self.pc);
        self.pc += 2.into();
        ret
    }
    pub fn writeu8(&mut self, value: u8, mmu: &mut MMU) {
        mmu.writeu8(self.pc, value);
    }
    pub fn writeu16(&mut self, value: u16, mmu: &mut MMU) {
        mmu.writeu16(self.pc, value);
    }

}
