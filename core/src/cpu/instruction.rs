use super::{CPU, Addr};

pub(super) fn decode(opcode: u16, cpu: &mut CPU) -> Result<u8, u16>{
    match opcode {
        0x0000 => {/*NOP*/}
        0x0010 => {/*STOP*/}
        0x0076 => {/*HALT*/}
        0x00f3 => {/*DI*/}
        0x00fb => {/*EI*/}
        0x0001 | 0x0011 | 0x0021 | 0x0031 => {
            // LD R16G1 [u16]
            // Flags: - - - -
            let val = cpu.readu16();
            let dst = R16G1::try_from(get_p(opcode as u8)).unwrap();
            write_to_r16_group1(cpu, dst, val);
        }
        0x0002 | 0x0012 | 0x0022 | 0x0032 => {
            // LD [R16G2] A
            // Flags: - - - -
            write_to_r16_group2(cpu, opcode as u8, cpu.regs.get_a());
        }
        0x0003 | 0x0013 | 0x0023 | 0x0033 => {
            // INC R16G1
            // Flags: - - - - 
            let reg = R16G1::try_from(get_p(opcode as u8)).unwrap();
            let val = read_from_r16_group1(cpu, reg);
            write_to_r16_group1(cpu, reg, val+1);
        }
        0x0004 | 0x0014 | 0x0024 | 0x0034 | 0x000c | 0x001c | 0x002c | 0x003c => {
            // INC r8
            // Flags: Z 0 H -
            let target = R8::try_from(get_y(opcode as u8)).unwrap();
            let v = read_from_r8(cpu, target) + 1;
            write_to_r8(cpu, target, v);
            cpu.flags.zero =  v == 0;
            cpu.flags.subtraction = false;
            cpu.flags.half_carry = bit_3_overflow(v, 1);
        }
        0x0005 | 0x0015 | 0x0025 | 0x0035 | 0x000d | 0x001d | 0x002d | 0x003d => {
            // DEC r8
            // Flags: Z 1 H -
            let target = R8::try_from(get_y(opcode as u8)).unwrap();
            let v = read_from_r8(cpu, target) - 1;
            write_to_r8(cpu, target, v);
            cpu.flags.zero = v == 0;
            cpu.flags.subtraction = true;
            cpu.flags.half_carry = bit_4_borrow(v, 1);
        }
        0x0006 | 0x0016 | 0x0026 | 0x0036 | 0x000e | 0x001e | 0x002e | 0x003e => {
            // LD r8 [u8]
            // Flags: - - - -
            let v = cpu.readu8();
            let dst = R8::try_from(get_y(opcode as u8)).unwrap();
            write_to_r8(cpu, dst, v);
        }
        0x0009 | 0x0019 | 0x0029 | 0x0039 => {
            // ADD HL R16G1
            // Flags: - - - - 
            let dst = R16G1::HL;
            let src = R16G1::try_from(get_p(opcode as u8)).unwrap();
            let val = read_from_r16_group1(cpu, src) + read_from_r16_group1(cpu, dst);
            write_to_r16_group1(cpu, dst, val);
        }
        0x000a | 0x001a | 0x002a | 0x003a => {
            // LD A [R16G2]
            // Flags: - - - -
            let v = read_from_r16_group2(cpu, opcode as u8);
            cpu.regs.set_a(v);
        }
        0x000b | 0x001b | 0x002b | 0x003b => {
            // DEC R16G1
            // Flags: - - - - 
            let reg = R16G1::try_from(get_p(opcode as u8)).unwrap();
            let val = read_from_r16_group1(cpu, reg);
            write_to_r16_group1(cpu, reg, val-1);
        }
        0x0040..=0x0075 | 0x0077..=0x007f => {
            // Flags: - - - - 
            r8_to_r8(cpu, opcode as u8);
        }
        0x0080..=0x00bf => {
            // Flags: perform_alu8
            let src = R8::try_from(get_z(opcode as u8)).unwrap();
            let op2 = read_from_r8(cpu, src);
            let operation = AluOp::try_from(get_y(opcode as u8)).unwrap();
            perform_alu8(cpu, operation, op2);
        }
        0x00c6 | 0x00d6 | 0x00e6 | 0x00f6 | 0x00ce | 0x00de | 0x00ee | 0x00fe => {
            // Flags: perform_alu8
            let op2 = cpu.readu8();
            let operation = AluOp::try_from(get_y(opcode as u8)).unwrap();
            perform_alu8(cpu, operation, op2);
        }


        // cbprefixed

        0xcb00..=0xcb3f => {
            // 00 opcode(group 3) r8
            //                  y  z
            // Flags: Z 0 0 C/0  (0 for swap)
            let target = R8::try_from(get_z(opcode as u8)).unwrap();
            let res = read_from_r8(cpu, target);
            let operation = SrOp::try_from(get_y(opcode as u8)).unwrap();
            let (res, carry) = perform_sr8(cpu, operation, res);
            write_to_r8(cpu, target, res); 
            cpu.flags.zero = res == 0;
            cpu.flags.subtraction = false;
            cpu.flags.half_carry = false;
            cpu.flags.carry = carry;
        }

        0xcb40..=0xcb7f => {
            // BIT bit, r8
            //  01   y   z
            // Flags Z 0 1 - 
            let src = R8::try_from(get_z(opcode as u8)).unwrap();
            let res = read_from_r8(cpu, src);
            let bit = get_y(opcode as u8);
            let res = get_nth_bit(res, bit);
            cpu.flags.zero = !res;
            cpu.flags.subtraction = false;
            cpu.flags.half_carry = true;
        }
        
        0xcb80..=0xcbbf => {
            // RES bit, r8
            //  10   y   z
            // Flags: - - - - 
            let target = R8::try_from(get_z(opcode as u8)).unwrap();
            let res = read_from_r8(cpu, target);
            let bit = get_y(opcode as u8);
            let res = unset_nth_bit(res, bit);
            write_to_r8(cpu, target, res);
        }
        0xcbc0..=0xcbff => {
            // SET bit, r8
            //  11   y   z
            // Flags: - - - - 
            let target = R8::try_from(get_z(opcode as u8)).unwrap();
            let res = read_from_r8(cpu, target);
            let bit = get_y(opcode as u8);
            let res = set_nth_bit(res, bit);
            write_to_r8(cpu, target, res);
        }
        _ => unimplemented!()
    };
    Ok(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum R8 {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    HL = 6,
    A = 7,
}

impl TryFrom<u8> for R8 {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let r = match value {
            0 => R8::B,
            1 => R8::C,
            2 => R8::D,
            3 => R8::E,
            4 => R8::H,
            5 => R8::L,
            6 => R8::HL,
            7 => R8::A,
            _ => return Err(()),
        };
        Ok(r)
    }
}

fn read_from_r8(cpu: &mut CPU, src: R8) -> u8 {
    match src {
        R8::B => cpu.regs.get_b(),
        R8::C => cpu.regs.get_c(),
        R8::D => cpu.regs.get_d(),
        R8::E => cpu.regs.get_e(),
        R8::H => cpu.regs.get_h(),
        R8::L => cpu.regs.get_l(),
        R8::HL => cpu.mmu.readu8(cpu.regs.get_hl().into()),
        R8::A => cpu.regs.get_a(),
    }
}

fn write_to_r8(cpu: &mut CPU, dst: R8, v: u8) {
    match dst {
        R8::B => cpu.regs.set_b(v),
        R8::C => cpu.regs.set_c(v),
        R8::D => cpu.regs.set_d(v),
        R8::E => cpu.regs.set_e(v),
        R8::H => cpu.regs.set_h(v),
        R8::L => cpu.regs.set_l(v),
        R8::HL => cpu.mmu.writeu8(cpu.regs.get_hl().into(), v),
        R8::A => cpu.regs.set_a(v),
    }

}

fn r8_to_r8(cpu: &mut CPU, opcode: u8) {
    let src = R8::try_from(get_z(opcode)).unwrap();
    let v = read_from_r8(cpu, src);
    let dst = R8::try_from(get_y(opcode)).unwrap();
    write_to_r8(cpu, dst, v);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum R16G1 {
    BC = 0,
    DE = 1,
    HL = 2,
    SP = 3,
}

impl TryFrom<u8> for R16G1 {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let r = match value {
            0 => R16G1::BC,
            1 => R16G1::DE,
            2 => R16G1::HL,
            3 => R16G1::SP,
            _ => return Err(()),
        };
        Ok(r)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum R16G2 {
    BC = 0,
    DE = 1,
    HLI = 2,
    HLD = 3,
}

impl TryFrom<u8> for R16G2 {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let r = match value {
            0 => R16G2::BC,
            1 => R16G2::DE,
            2 => R16G2::HLI,
            3 => R16G2::HLD,
            _ => return Err(()),
        };
        Ok(r)
    }
}

fn read_from_r16_group1(cpu: &mut CPU, reg: R16G1) -> u16 {
    match reg {
        R16G1::BC => cpu.regs.get_bc(),
        R16G1::DE => cpu.regs.get_de(),
        R16G1::HL => cpu.regs.get_hl(),
        R16G1::SP => cpu.sp.into(),
    }
}

fn write_to_r16_group1(cpu: &mut CPU, reg: R16G1, val: u16) {
    match reg {
        R16G1::BC => cpu.regs.set_bc(val),
        R16G1::DE => cpu.regs.set_de(val),
        R16G1::HL => cpu.regs.set_hl(val),
        R16G1::SP => cpu.sp = val.into(),
    }
}

fn get_addr_from_r16_group2(cpu: &mut CPU, opcode: u8) -> Addr {
    let reg = R16G2::try_from(get_p(opcode)).unwrap();
    let addr = match reg {
        R16G2::BC => cpu.regs.get_bc(),
        R16G2::DE => cpu.regs.get_de(),
        R16G2::HLI => {
            let addr = cpu.regs.get_hl();
            cpu.regs.incr_hl();
            addr
        }
        R16G2::HLD => {
            let addr = cpu.regs.get_hl();
            cpu.regs.decr_hl();
            addr
        }
    };
    addr.into()
}

fn write_to_r16_group2(cpu: &mut CPU, opcode: u8, value: u8) {
    let addr = get_addr_from_r16_group2(cpu, opcode);
    cpu.mmu.writeu8(addr, value);
}

fn read_from_r16_group2(cpu: &mut CPU, opcode: u8) -> u8 {
    let addr = get_addr_from_r16_group2(cpu, opcode);
    cpu.mmu.readu8(addr)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum AluOp {
    ADD = 0,
    ADC = 1,
    SUB = 2,
    SBC = 3,
    AND = 4,
    XOR = 5,
    OR  = 6,
    CP  = 7,
}

impl TryFrom<u8> for AluOp {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let r = match value {
            0 => AluOp::ADD, 
            1 => AluOp::ADC, 
            2 => AluOp::SUB,
            3 => AluOp::SBC,
            4 => AluOp::AND,
            5 => AluOp::XOR, 
            6 => AluOp::OR,
            7 => AluOp::CP,
            _ => return Err(()),
        };
        Ok(r)
    }
}

fn perform_alu8(cpu: &mut CPU, operation: AluOp, op2: u8) {
    let a = cpu.regs.get_a();
    match operation {
        AluOp::ADD => {
            // Flags: Z 0 H C 
            let (res, of) = a.overflowing_add(op2);
            cpu.flags.zero = res == 0;
            cpu.flags.subtraction = false;
            cpu.flags.half_carry = bit_3_overflow(a, op2);
            cpu.flags.carry = of;
            cpu.regs.set_a(res);
        }
        AluOp::ADC => {
            // Flags: Z 0 H C 
            let (res, of) = a.carrying_add(op2, cpu.flags.carry);
            cpu.flags.zero = res == 0;
            cpu.flags.subtraction = false;
            cpu.flags.half_carry = bit_3_overflow(a, op2);
            cpu.flags.carry = of;
            cpu.regs.set_a(res);
        }
        AluOp::SUB => {
            // Flags: Z 1 H C 
            let (res, of) = a.overflowing_sub(op2);
            cpu.flags.zero = res == 0;
            cpu.flags.subtraction = true;
            cpu.flags.half_carry = bit_4_borrow(a, op2);
            cpu.flags.carry = of;
            cpu.regs.set_a(res);
        }
        AluOp::SBC => {
            // Flags: Z 1 H C 
            let (res, of) = a.borrowing_sub(op2, cpu.flags.carry);
            cpu.flags.zero = res == 0;
            cpu.flags.subtraction = true;
            cpu.flags.half_carry = bit_4_borrow(a, op2);
            cpu.flags.carry = of;
            cpu.regs.set_a(res);
        }
        AluOp::AND => {
            // Flags: Z 0 1 0 
            let res = a & op2;
            cpu.flags.zero = res == 0;
            cpu.flags.subtraction = false;
            cpu.flags.half_carry = true;
            cpu.flags.carry = false;
            cpu.regs.set_a(res);
        }
        AluOp::XOR => {
            // Flags: Z 0 0 0 
            let res = a ^ op2;
            cpu.flags.zero = res == 0;
            cpu.flags.subtraction = false;
            cpu.flags.half_carry = false;
            cpu.flags.carry = false;
            cpu.regs.set_a(res);
        }
        AluOp::OR => {
            // Flags: Z 0 0 0 
            let res = a | op2;
            cpu.flags.zero = res == 0;
            cpu.flags.subtraction = false;
            cpu.flags.half_carry = false;
            cpu.flags.carry = false;
            cpu.regs.set_a(res);
        }
        AluOp::CP => {
            // Flags: Z 1 H C 
            let (res, of) = a.overflowing_sub(op2);
            cpu.flags.zero = res == 0;
            cpu.flags.subtraction = true;
            cpu.flags.half_carry = bit_4_borrow(a, op2);
            cpu.flags.carry = of;
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum SrOp {
    RLC  = 0,
    RRC  = 1,
    RL   = 2,
    RR   = 3,
    SLA  = 4,
    SRA  = 5,
    SWAP = 6,
    SRL  = 7,
}
impl TryFrom<u8> for SrOp {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let r = match value {
            0 => SrOp::RLC, 
            1 => SrOp::RRC, 
            2 => SrOp::RL,
            3 => SrOp::RR,
            4 => SrOp::SLA,
            5 => SrOp::SRA, 
            6 => SrOp::SWAP,
            7 => SrOp::SRL,
            _ => return Err(()),
        };
        Ok(r)
    }
}

fn perform_sr8(cpu: &mut CPU, operation: SrOp, value: u8) -> (u8, bool) {
    match operation {
        // Flags: Z 0 0 C
        SrOp::RLC  => (value.rotate_left(1), get_nth_bit(value, 7)),
        // Flags: Z 0 0 C
        SrOp::RRC  => (value.rotate_right(1), get_nth_bit(value, 0)),
        // Flags: Z 0 0 C
        SrOp::RL   => (nth_bit_to(value << 1, 0, cpu.flags.carry), get_nth_bit(value, 7)),
        // Flags: Z 0 0 C
        SrOp::RR    => (nth_bit_to(value >> 1, 7, cpu.flags.carry), get_nth_bit(value, 0)),
        // Flags: Z 0 0 C
        SrOp::SLA  => (value << 1, get_nth_bit(value, 7)),
        // Flags: Z 0 0 C
        SrOp::SRA  => (nth_bit_to(value >> 1, 7, get_nth_bit(value, 7)), get_nth_bit(value, 0)),
        // Flags: Z 0 0 0
        SrOp::SWAP => (swap_nibbles(value), false),
        // Flags: Z 0 0 C
        SrOp::SRL  => (value >> 1, get_nth_bit(value, 0)),
    }
}

// bit operations
fn get_nth_bit(val: u8, n: u8) -> bool {
    assert!(n < 8);
    ((val & (1 << n)) >> n) == 1
}

fn nth_bit_to(val: u8, n: u8, set: bool) -> u8 {
    if set {
        set_nth_bit(val, n)
    } else {
        unset_nth_bit(val, n)
    }
}

fn set_nth_bit(val: u8, n: u8) -> u8 {
    assert!(n < 8);
    val | (1 << n)
}
fn unset_nth_bit(val: u8, n: u8) -> u8 {
    assert!(n < 8);
    val & !(1 << n)
}
fn toggle_nth_bit(val: u8, n: u8) -> u8 {
    assert!(n < 8);
    val ^ (1 << n)
}
fn swap_nibbles(val: u8) -> u8 {
    ((val & 0x0f) << 4) | ((val & 0xf0) >> 4)
}

fn bit_3_overflow(op1: u8, op2: u8) -> bool {
    (((op1 & 0xf) + (op2 & 0xf)) & 0x10) == 0x10
}

fn bit_4_borrow(op1: u8, op2: u8) -> bool {
    (op1 & 0x0F) < (op2 & 0x0F)
}
// x = the opcode's 1st octal digit (i.e. bits 7-6)
// y = the opcode's 2nd octal digit (i.e. bits 5-3)
// z = the opcode's 3rd octal digit (i.e. bits 2-0)
// p = y rightshifted one position (i.e. bits 5-4)
// q = y modulo 2 (i.e. bit 3)
// diagram and explanation:
// https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html 

fn get_z(val: u8) -> u8 {
    val & 0b00000111
}
fn get_y(val: u8) -> u8 {
    (val & 0b00111000) >> 3
}
fn get_p(val: u8) -> u8 {
    (val & 0b00110000) >> 4
}