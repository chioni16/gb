mod instruction;
pub(crate) mod interrupts;
mod registers;

use super::mmu::MMU;
use crate::util::Addr;
use instruction::decode;
use interrupts::{Interrupt, Interrupts};
use registers::Flags;
use registers::Registers;

const INT_ENABLE_ADDR: Addr = Addr::from(0xffff);
const INT_REQUEST_ADDR: Addr = Addr::from(0xff0f);

pub(crate) struct CPU {
    regs: Registers,
    pc: Addr,
    sp: Addr,
    ime: IMEState,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            pc: Addr::new(),
            sp: Addr::new(),
            ime: IMEState::Disabled,
        }
    }

    pub fn no_boot(&mut self, mmu: &mut MMU) {
        self.regs.a = 0x11;
        self.regs.f = 0x80.into();
        self.regs.b = 0x00;
        self.regs.c = 0x00;
        self.regs.d = 0xff;
        self.regs.e = 0x56;
        self.regs.h = 0x00;
        self.regs.l = 0x0d;
        self.pc = 0x0100.into();
        self.sp = 0xfffe.into();

        // turn off DMG boot rom
        // mmu.writeu8(0xff50.into(), 1);
        mmu.boot_disabled = true;
    }

    // pub fn no_boot(&mut self) {
    //     self.regs.a = 0x01;
    //     self.regs.f = 0xb0.into();
    //     self.regs.b = 0x00;
    //     self.regs.c = 0x13;
    //     self.regs.d = 0x00;
    //     self.regs.e = 0xd8;
    //     self.regs.h = 0x01;
    //     self.regs.l = 0x4d;
    //     self.pc = 0x0101.into();
    //     self.sp = 0xfffe.into();
    //     self.ime = IMEState::Enabled;
    // }

    pub(crate) fn step(&mut self, mmu: &mut MMU) -> u64 {
        if self.handle_ime(mmu) {
            return 20;
        }
        // println!("A: {:0>2X} F: {:0>2X} B: {:0>2X} C: {:0>2X} D: {:0>2X} E: {:0>2X} H: {:0>2X} L: {:0>2X} SP: {:0>4X} PC: 00:{:0>4X} ({:0>2X} {:0>2X} {:0>2X} {:0>2X})", self.regs.a, <Flags as Into<u8>>::into(self.regs.f), self.regs.b, self.regs.c, self.regs.d, self.regs.e, self.regs.h, self.regs.l, self.sp.0, self.pc.0, mmu.readu8(self.pc),mmu.readu8(self.pc+1.into()),mmu.readu8(self.pc+2.into()),mmu.readu8(self.pc+3.into()));
        let mut opcode = self.readu8(mmu) as u16;
        if opcode == 0xcb {
            opcode = opcode << 8 | self.readu8(mmu) as u16;
        }

        // println!("{:#x?}\t", opcode);
        let ticks = decode(opcode, self, mmu);
        ticks
    }

    // interrupts
    fn handle_ime(&mut self, mmu: &mut MMU) -> bool {
        match self.ime {
            IMEState::Enabled => {
                let mut request = self.get_interrupt_request(mmu);
                let enable = self.get_interrupt_enable(mmu);

                let interrupts = enable & request;
                if let Some(interrupt) = interrupts.next_interrupt() {
                    self.ime = IMEState::Disabled;
                    self.push_stack(mmu, self.pc.into());
                    match interrupt {
                        Interrupt::VBlank => {
                            request.vblank = false;
                            self.pc = 0x40.into();
                        }
                        Interrupt::LCDStat => {
                            request.lcd_stat = false;
                            self.pc = 0x48.into();
                        }
                        Interrupt::Timer => {
                            request.timer = false;
                            self.pc = 0x50.into();
                        }
                        Interrupt::Serial => {
                            request.serial = false;
                            self.pc = 0x58.into();
                        }
                        Interrupt::Joypad => {
                            request.joypad = false;
                            self.pc = 0x60.into();
                        }
                    }
                    self.set_interrupt_request(mmu, request);
                    return true;
                }
            }
            IMEState::Intermediate1 => self.ime = IMEState::Intermediate2,
            IMEState::Intermediate2 => self.ime = IMEState::Enabled,
            IMEState::Disabled => {}
        }
        false
    }

    fn get_interrupt_enable(&self, mmu: &MMU) -> Interrupts {
        mmu.readu8(INT_ENABLE_ADDR).into()
    }

    fn get_interrupt_request(&self, mmu: &MMU) -> Interrupts {
        mmu.readu8(INT_REQUEST_ADDR).into()
    }
    fn set_interrupt_request(&self, mmu: &mut MMU, request: Interrupts) {
        mmu.writeu8(INT_REQUEST_ADDR, request.into());
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

#[derive(Debug, Clone, Copy)]
enum IMEState {
    Disabled,
    Enabled,
    Intermediate1,
    Intermediate2,
}
