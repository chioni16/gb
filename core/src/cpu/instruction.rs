use super::{CPU, Addr};
use crate::mmu::MMU;

pub(super) fn decode(opcode: u16, cpu: &mut CPU, mmu: &mut MMU) -> usize {
    // println!("{:#x}", opcode);
    match opcode {
        0x0000 => {
            /*NOP*/
            4
        }
        0x0010 => {
            /*STOP*/
            4
        }
        0x0076 => {
            /*HALT*/
            4
        }
        0x00f3 => {
            /*DI*/
            4
        }
        0x00fb => {
            /*EI*/
            4
        }
        
        
        //////////////////    x8/lsm           ////////////////////////
        0x0002 | 0x0012 | 0x0022 | 0x0032 => {
            // LD [R16G2] A
            // Flags: - - - -
            // Cycles: 8
            write_to_r16_group2(cpu, mmu, opcode as u8, cpu.regs.get_a());
            8
        }
        
        0x000a | 0x001a | 0x002a | 0x003a => {
            // LD A [R16G2]
            // Flags: - - - -
            // Cycles: 8
            let v = read_from_r16_group2(cpu, mmu, opcode as u8);
            cpu.regs.set_a(v);
            8
        }
        
        0x0006 | 0x0016 | 0x0026 | 0x0036 | 0x000e | 0x001e | 0x002e | 0x003e => {
            // LD r8 [u8]
            // Flags: - - - -
            // Cycles: 8/12(hl)
            let v = cpu.readu8(mmu);
            let (dst, is_hl) = get_r8_reg(get_y(opcode as u8));
            write_to_r8(cpu, mmu, dst, v);
            if is_hl { 12 } else { 8 }
        }
        
        0x0040..=0x0075 | 0x0077..=0x007f => {
            // LD r8 r8
            // Flags: - - - - 
            // Cycles: 4/8(hl)
            let (src, is_hl_src) = get_r8_reg(get_z(opcode as u8));
            let v = read_from_r8(cpu, mmu, src);
            let (dst, is_hl_dst) = get_r8_reg(get_y(opcode as u8));
            write_to_r8(cpu, mmu, dst, v);
            if is_hl_src || is_hl_dst { 8 } else { 4 }
        }
        // load from A
        // Flags: - - - -
        // Cycles: 12, 8, 16 (in order)
        0x00e0 => {
            let value = cpu.regs.get_a();
            let addr = (0xff00 + cpu.readu8(mmu) as u16).into();
            mmu.writeu8(addr, value);
            12
        }
        0x00e2 => {
            let value = cpu.regs.get_a();
            let addr = (0xff00 + cpu.regs.get_c() as u16).into();
            mmu.writeu8(addr, value);
            8
        }
        0x00ea => {
            let value = cpu.regs.get_a();
            let addr = cpu.readu16(mmu).into();
            mmu.writeu8(addr, value);
            16
        }

        // load to A
        // Flags: - - - -
        // Cycles: 12, 8, 16 (in order)
        0x00f0 => {
            let addr = (0xff00 + cpu.readu8(mmu) as u16).into();
            let value = mmu.readu8(addr);
            cpu.regs.set_a(value);
            12
        }
        0x00f2 => {
            let addr = (0xff00 + cpu.regs.get_c() as u16).into();
            let value = mmu.readu8(addr);
            cpu.regs.set_a(value);
            8
        }
        0x00fa => {
            let addr = cpu.readu16(mmu).into();
            let value = mmu.readu8(addr);
            cpu.regs.set_a(value);
            16
        }
        //////////////////    x8/lsm           ////////////////////////

        //////////////////    x16/lsm           ////////////////////////
        0x0001 | 0x0011 | 0x0021 | 0x0031 => {
            // LD R16G1 [u16]
            // Flags: - - - -
            // Cycles: 12
            let val = cpu.readu16(mmu);
            let dst = R16G1::try_from(get_p(opcode as u8)).unwrap();
            write_to_r16_group1(cpu, dst, val);
            12
        }

        0x0008 => {
            // LD [u16] SP
            // Flags: - - - -
            // Cycles: 20
            let target: Addr = cpu.readu16(mmu).into();
            let value: u16 = cpu.sp.into();
            mmu.writeu16(target, value);
            20
        }
        
        0x00c1 | 0x00d1 | 0x00e1 | 0x00f1 => {
            // POP r16g3
            // Flags: - - - - (except POP AF, but no explicit changes done to flags)
            // Cycles: 12
            let val = cpu.pop_stack(mmu);
            let dst = R16G3::try_from(get_p(opcode as u8)).unwrap();
            write_to_r16_group3(cpu, dst, val);
            12
        }
        0x00c5 | 0x00d5 | 0x00e5 | 0x00f5 => {
            // PUSH r16g3
            // Flags: - - - - 
            // Cycles: 16
            let src = R16G3::try_from(get_p(opcode as u8)).unwrap();
            let val = read_from_r16_group3(cpu, src);
            cpu.push_stack(mmu, val);
            16
        }
        0x00f9 => {
            // LD SP, HL
            // Flags: - - - -
            // Cycles: 8
            let value = cpu.regs.get_hl();
            cpu.sp = value.into();
            8
        }
        //////////////////    x16/lsm           ////////////////////////

        /////////////////     x8/alu          /////////////////////////
        0x0004 | 0x0014 | 0x0024 | 0x0034 | 0x000c | 0x001c | 0x002c | 0x003c => {
            // INC r8
            // Flags: Z 0 H -
            // Cycles: 4/12(hl)
            let (target, is_hl) = get_r8_reg(get_y(opcode as u8));
            let value = read_from_r8(cpu, mmu, target);
            let new_value = value + 1;
            write_to_r8(cpu, mmu, target, new_value);
            cpu.regs.f.zero =  new_value == 0;
            cpu.regs.f.subtraction = false;
            // eprintln!("INC: {}, {}, {}", v, 1, bit_3_overflow(v, 1));
            cpu.regs.f.half_carry = bit_3_overflow(value, 1);
            if is_hl { 12 } else { 4 }
        }
        0x0005 | 0x0015 | 0x0025 | 0x0035 | 0x000d | 0x001d | 0x002d | 0x003d => {
            // DEC r8
            // Flags: Z 1 H -
            // Cycles: 4/12(hl)
            let (target, is_hl) = get_r8_reg(get_y(opcode as u8));
            let value = read_from_r8(cpu, mmu, target);
            let new_value = value - 1;
            write_to_r8(cpu, mmu, target, new_value);
            cpu.regs.f.zero = new_value == 0;
            cpu.regs.f.subtraction = true;
            cpu.regs.f.half_carry = bit_4_borrow(value, 1);
            if is_hl { 12 } else { 4 }
        }
        0x0080..=0x00bf => {
            // ALU A r8
            // Flags: perform_alu8
            // Cycles: 4/8(hl)
            let (src, is_hl) = get_r8_reg(get_z(opcode as u8));
            let op2 = read_from_r8(cpu, mmu, src);
            let operation = AluOp::try_from(get_y(opcode as u8)).unwrap();
            perform_alu8(cpu, operation, op2);
            if is_hl { 8 } else { 4 }
        }

        0x00c6 | 0x00d6 | 0x00e6 | 0x00f6 | 0x00ce | 0x00de | 0x00ee | 0x00fe => {
            // ALU A [u8]
            // Flags: perform_alu8
            // Cycles: 8
            let op2 = cpu.readu8(mmu);
            let operation = AluOp::try_from(get_y(opcode as u8)).unwrap();
            perform_alu8(cpu, operation, op2);
            8
        }

        /////////////////     x8/alu          /////////////////////////

        /////////////////     x16/alu          /////////////////////////
        
        0x0003 | 0x0013 | 0x0023 | 0x0033 => {
            // INC R16G1
            // Flags: - - - - 
            // Cycles: 8
            let reg = R16G1::try_from(get_p(opcode as u8)).unwrap();
            let val = read_from_r16_group1(cpu, reg);
            write_to_r16_group1(cpu, reg, val+1);
            8
        }

        0x0009 | 0x0019 | 0x0029 | 0x0039 => {
            // ADD HL R16G1
            // Flags: - 0 H C 
            // Cycles: 8
            let dst = R16G1::HL;
            let src = R16G1::try_from(get_p(opcode as u8)).unwrap();
            let val1 = read_from_r16_group1(cpu, src);
            let val2 = read_from_r16_group1(cpu, dst);
            let (new_val, of) = val1.overflowing_add(val2);
            write_to_r16_group1(cpu, dst, new_val);
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = bit_11_overflow(val1, val2);
            cpu.regs.f.carry = of;
            8
        }

        0x000b | 0x001b | 0x002b | 0x003b => {
            // DEC R16G1
            // Flags: - - - - 
            // Cycles: 8
            let reg = R16G1::try_from(get_p(opcode as u8)).unwrap();
            let val = read_from_r16_group1(cpu, reg);
            write_to_r16_group1(cpu, reg, val-1);
            8
        }

        0x00e8 => {
            // ADD SP i8
            // Flags: 0 0 H C
            // Cycles: 16
            let value = cpu.readu8(mmu) as i8;
            let sp: u16 = cpu.sp.into();
            let (new_sp, _) = sp.overflowing_add_signed(value as i16);
            cpu.sp = new_sp.into();

            cpu.regs.f.zero = false;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = bit_3_overflow(sp as u8, value as u8);
            let (_, of) = (sp as u8).overflowing_add(value as u8);
            cpu.regs.f.carry = of;
            16
        }

        0x00f8 => {
            // LD HL,SP+i8 
            // Flags: 0 0 H C
            // Cycles: 12
            let value = cpu.readu8(mmu) as i8;
            let sp: u16 = cpu.sp.into();
            let (new_value, _) = sp.overflowing_add_signed(value as i16);
            cpu.regs.set_hl(new_value);

            cpu.regs.f.zero = false;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = bit_3_overflow(sp as u8, value as u8);
            let (_, of) = (sp as u8).overflowing_add(value as u8);
            cpu.regs.f.carry = of;
            12
        }
        /////////////////     x16/alu          /////////////////////////
        
        /////////////////     control/br      /////////////////////////
        0x0018 => {
            // JR i8
            // Flags: - - - -
            // Cycles: 12
            let jmp = cpu.readu8(mmu) as i8;
            let cur_pc: u16 = cpu.pc.into();
            let next_pc = cur_pc.wrapping_add_signed(jmp as i16);
            cpu.pc = next_pc.into();
            12
        }

        0x0020 | 0x0030 | 0x0028 | 0x0038 => {
            // JR Condition i8
            // Flags: - - - -
            // Cycles: 8/12(br taken)
            let condition = Condition::try_from(get_r(opcode as u8)).unwrap();
            let jmp = cpu.readu8(mmu) as i8;
            if check_condition(cpu, condition) {
                // branch taken
                let cur_pc: u16 = cpu.pc.into();
                let next_pc = cur_pc.wrapping_add_signed(jmp as i16);
                cpu.pc = next_pc.into();
                12
            } else {
                // branch not taken
                8
            }
        }

        0x00c3 => {
            // JP u16
            // Flags: - - - -
            // Cycles: 16
            let next_pc = cpu.readu16(mmu).into();
            cpu.pc = next_pc;
            16
        }
        0x00c2 | 0x00d2 | 0x00ca | 0x00da => {
            // JP Condition u16
            // Flags: - - - -
            // Cycles: 12/16(br taken)
            let condition = Condition::try_from(get_r(opcode as u8)).unwrap();
            let next_pc = cpu.readu16(mmu).into();
            if check_condition(cpu, condition) {
                // branch taken
                cpu.pc = next_pc;
                16
            } else {
                // branch not taken
                12
            }
        }
        0x00e9 => {
            // JP HL
            // Flags: - - - -
            // Cycles: 4
            let new_pc = cpu.regs.get_hl().into();
            cpu.pc = new_pc;
            4
        }

        0x00cd => {
            // CALL u16
            // Flags: - - - -
            // Cycles: 24
            let next_pc = cpu.readu16(mmu);
            cpu.push_stack(mmu, cpu.pc.into());
            cpu.pc = next_pc.into();
            24
        }

        0x00c4 | 0x00d4 | 0x00cc | 0x00dc => {
            // CALL Condition u16
            // Flags: - - - -
            // Cycles: 12/24(br taken)
            let condition = Condition::try_from(get_r(opcode as u8)).unwrap();
            let next_pc = cpu.readu16(mmu);
            if check_condition(cpu, condition) {
                // branch taken
                cpu.push_stack(mmu, cpu.pc.into());
                cpu.pc = next_pc.into();
                24
            } else {
                // branch not taken
                12
            }
        }

        0x00c9 => {
            // RET
            // Flags: - - - -
            // Cycles: 16
            let ret_pc = cpu.pop_stack(mmu);
            cpu.pc = ret_pc.into();
            16
        }

        0x00d9 => {
            // RETI
            // Flags: - - - -
            // Cycles: 16
            // todo!();
            let ret_pc = cpu.pop_stack(mmu);
            cpu.pc = ret_pc.into();
            16
        }
        0x00c0 | 0x00d0 | 0x00c8 | 0x00d8 => {
            // Conditional RET
            // Flags: - - - -
            // Cycles: 8/20(br taken)
            let condition = Condition::try_from(get_r(opcode as u8)).unwrap();
            if check_condition(cpu, condition) {
                // branch taken
                let ret_pc = cpu.pop_stack(mmu);
                cpu.pc = ret_pc.into();
                20
            } else {
                // branch not taken
                8
            }
        }

        0x00c7 | 0x00d7 | 0x00e7 | 0x00f7 | 0x00cf | 0x00df | 0x00ef | 0x00ff => {
            // RST
            // Flags: - - - -
            // Cycles: 16
            let vec = get_y(opcode as u8) as u16;
            cpu.push_stack(mmu, cpu.pc.into());
            cpu.pc = vec.into();
            16
        }

        /////////////////     control/br      /////////////////////////
        

        0x0007 | 0x0017 | 0x0027 | 0x0037 | 0x000f | 0x001f | 0x002f | 0x003f => {
            let operation = AccFlagOp::try_from(get_y(opcode as u8)).unwrap();
            perform_acc_flag(cpu, operation);
            // todo!()
            4
        }

        // cbprefixed

        0xcb00..=0xcb3f => {
            // 00 opcode(group 3) r8
            //                  y  z
            // Flags: Z 0 0 C/0  (0 for swap)
            // Cycles: 8/16(hl)
            let (target, is_hl) = get_r8_reg(get_z(opcode as u8));
            let res = read_from_r8(cpu, mmu, target);
            let operation = SrOp::try_from(get_y(opcode as u8)).unwrap();
            let (res, carry) = perform_sr8(cpu, operation, res);
            write_to_r8(cpu, mmu, target, res); 
            cpu.regs.f.zero = res == 0;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = false;
            cpu.regs.f.carry = carry;
            if is_hl { 16 } else { 8 }
        }

        0xcb40..=0xcb7f => {
            // BIT bit, r8
            //  01   y   z
            // Flags Z 0 1 - 
            // Cycles: 8/12(hl)
            let (src, is_hl) = get_r8_reg(get_z(opcode as u8));
            let res = read_from_r8(cpu, mmu, src);
            let bit = get_y(opcode as u8);
            let res = get_nth_bit(res, bit);
            cpu.regs.f.zero = !res;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = true;
            if is_hl { 12 } else { 8 }
        }
        
        0xcb80..=0xcbbf => {
            // RES bit, r8
            //  10   y   z
            // Flags: - - - - 
            // Cycles: 8/16(hl)
            let (target, is_hl) = get_r8_reg(get_z(opcode as u8));
            let res = read_from_r8(cpu, mmu, target);
            let bit = get_y(opcode as u8);
            let res = unset_nth_bit(res, bit);
            write_to_r8(cpu, mmu, target, res);
            if is_hl { 16 } else { 8 }
        }
        0xcbc0..=0xcbff => {
            // SET bit, r8
            //  11   y   z
            // Flags: - - - - 
            // Cycles: 8/16(hl)
            let (target, is_hl) = get_r8_reg(get_z(opcode as u8));
            let res = read_from_r8(cpu, mmu, target);
            let bit = get_y(opcode as u8);
            let res = set_nth_bit(res, bit);
            write_to_r8(cpu, mmu, target, res);
            if is_hl { 16 } else { 8 }
        }
        _ => panic!("Unimplemented opcode: {:#x}", opcode),
    }
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

fn get_r8_reg(oct: u8) -> (R8, bool) {
    assert!(oct < 8);
    let r8 = R8::try_from(oct).unwrap();
    let is_hl = match r8 {
        R8::HL => true, 
        _      => false,
    };
    (r8, is_hl)
}

fn read_from_r8(cpu: &mut CPU, mmu: &MMU, src: R8) -> u8 {
    match src {
        R8::B => cpu.regs.get_b(),
        R8::C => cpu.regs.get_c(),
        R8::D => cpu.regs.get_d(),
        R8::E => cpu.regs.get_e(),
        R8::H => cpu.regs.get_h(),
        R8::L => cpu.regs.get_l(),
        R8::HL => mmu.readu8(cpu.regs.get_hl().into()),
        R8::A => cpu.regs.get_a(),
    }
}

fn write_to_r8(cpu: &mut CPU, mmu: &mut MMU, dst: R8, v: u8) {
    match dst {
        R8::B => cpu.regs.set_b(v),
        R8::C => cpu.regs.set_c(v),
        R8::D => cpu.regs.set_d(v),
        R8::E => cpu.regs.set_e(v),
        R8::H => cpu.regs.set_h(v),
        R8::L => cpu.regs.set_l(v),
        R8::HL => mmu.writeu8(cpu.regs.get_hl().into(), v),
        R8::A => cpu.regs.set_a(v),
    }

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
        R16G2::DE => {
            let v = cpu.regs.get_de();
            // println!("yolo {:#x}", v);
            v
        }
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

fn write_to_r16_group2(cpu: &mut CPU, mmu: &mut MMU, opcode: u8, value: u8) {
    let addr = get_addr_from_r16_group2(cpu, opcode);
    mmu.writeu8(addr, value);
}

fn read_from_r16_group2(cpu: &mut CPU, mmu: &MMU, opcode: u8) -> u8 {
    let addr = get_addr_from_r16_group2(cpu, opcode);
    let val = mmu.readu8(addr);
    // println!("yolo2 {:#x}", val);
    val
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum R16G3 {
    BC = 0,
    DE = 1,
    HL = 2,
    AF = 3,
}

impl TryFrom<u8> for R16G3 {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let r = match value {
            0 => R16G3::BC,
            1 => R16G3::DE,
            2 => R16G3::HL,
            3 => R16G3::AF,
            _ => return Err(()),
        };
        Ok(r)
    }
}

fn read_from_r16_group3(cpu: &mut CPU, reg: R16G3) -> u16 {
    match reg {
        R16G3::BC => cpu.regs.get_bc(),
        R16G3::DE => cpu.regs.get_de(),
        R16G3::HL => cpu.regs.get_hl(),
        R16G3::AF => cpu.regs.get_af(),
    }
}

fn write_to_r16_group3(cpu: &mut CPU, reg: R16G3, val: u16) {
    match reg {
        R16G3::BC => cpu.regs.set_bc(val),
        R16G3::DE => cpu.regs.set_de(val),
        R16G3::HL => cpu.regs.set_hl(val),
        R16G3::AF => cpu.regs.set_af(val),
    }
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
            cpu.regs.f.zero = res == 0;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = bit_3_overflow(a, op2);
            cpu.regs.f.carry = of;
            cpu.regs.set_a(res);
        }
        AluOp::ADC => {
            // Flags: Z 0 H C 
            let (res, of) = a.carrying_add(op2, cpu.regs.f.carry);
            cpu.regs.f.zero = res == 0;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = bit_3_overflow(a, op2) || bit_3_overflow(a+op2, cpu.regs.f.carry as u8);
            cpu.regs.f.carry = of;
            cpu.regs.set_a(res);
        }
        AluOp::SUB => {
            // Flags: Z 1 H C 
            let (res, of) = a.overflowing_sub(op2);
            cpu.regs.f.zero = res == 0;
            cpu.regs.f.subtraction = true;
            cpu.regs.f.half_carry = bit_4_borrow(a, op2);
            cpu.regs.f.carry = of;
            cpu.regs.set_a(res);
        }
        AluOp::SBC => {
            // Flags: Z 1 H C 
            let (res, of) = a.borrowing_sub(op2, cpu.regs.f.carry);
            cpu.regs.f.zero = res == 0;
            cpu.regs.f.subtraction = true;
            cpu.regs.f.half_carry = bit_4_borrow(a, op2) || bit_4_borrow(a-op2, cpu.regs.f.carry as u8);
            cpu.regs.f.carry = of;
            cpu.regs.set_a(res);
        }
        AluOp::AND => {
            // Flags: Z 0 1 0 
            let res = a & op2;
            cpu.regs.f.zero = res == 0;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = true;
            cpu.regs.f.carry = false;
            cpu.regs.set_a(res);
        }
        AluOp::XOR => {
            // Flags: Z 0 0 0 
            let res = a ^ op2;
            cpu.regs.f.zero = res == 0;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = false;
            cpu.regs.f.carry = false;
            cpu.regs.set_a(res);
        }
        AluOp::OR => {
            // Flags: Z 0 0 0 
            let res = a | op2;
            cpu.regs.f.zero = res == 0;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = false;
            cpu.regs.f.carry = false;
            cpu.regs.set_a(res);
        }
        AluOp::CP => {
            // Flags: Z 1 H C 
            let (res, of) = a.overflowing_sub(op2);
            cpu.regs.f.zero = res == 0;
            cpu.regs.f.subtraction = true;
            cpu.regs.f.half_carry = bit_4_borrow(a, op2);
            cpu.regs.f.carry = of;
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
        SrOp::RL   => (nth_bit_to(value << 1, 0, cpu.regs.f.carry), get_nth_bit(value, 7)),
        // Flags: Z 0 0 C
        SrOp::RR    => (nth_bit_to(value >> 1, 7, cpu.regs.f.carry), get_nth_bit(value, 0)),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Condition {
    NZ = 0,
    Z  = 1,
    NC = 2,
    C  = 3,
}

impl TryFrom<u8> for Condition {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let r = match value {
            0 => Condition::NZ, 
            1 => Condition::Z, 
            2 => Condition::NC,
            3 => Condition::C,
            _ => return Err(()),
        };
        Ok(r)
    }
}

fn check_condition(cpu: &CPU, conditon: Condition) -> bool {
    match conditon {
        Condition::NZ => !cpu.regs.f.zero,
        Condition::Z  => cpu.regs.f.zero,
        Condition::NC => !cpu.regs.f.carry,
        Condition::C  => cpu.regs.f.carry,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum AccFlagOp {
    RLCA = 0,
    RRCA = 1,
    RLA  = 2,
    RRA  = 3,
    DAA  = 4,
    CPL  = 5,
    SCF  = 6, 
    CCF  = 7,
}

impl TryFrom<u8> for AccFlagOp {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let r = match value {
            0 => AccFlagOp::RLCA, 
            1 => AccFlagOp::RRCA, 
            2 => AccFlagOp::RLA,
            3 => AccFlagOp::RRA,
            4 => AccFlagOp::DAA,
            5 => AccFlagOp::CPL,
            6 => AccFlagOp::SCF,
            7 => AccFlagOp::CCF,
            _ => return Err(()),
        };
        Ok(r)
    }
}

fn perform_acc_flag(cpu: &mut CPU, operation: AccFlagOp) {
    match operation {
        AccFlagOp::RLCA => {
            let value = cpu.regs.get_a();
            let new_value = value.rotate_left(1);
            cpu.regs.set_a(new_value);
            cpu.regs.f.zero = false;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = false;
            cpu.regs.f.carry = get_nth_bit(value, 7);
        }
        AccFlagOp::RRCA => {
            let value = cpu.regs.get_a();
            let new_value = value.rotate_right(1);
            cpu.regs.set_a(new_value);
            cpu.regs.f.zero = false;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = false;
            cpu.regs.f.carry = get_nth_bit(value, 0);
        }
        AccFlagOp::RLA => {
            let value = cpu.regs.get_a();
            let new_value = nth_bit_to(value << 1, 0, cpu.regs.f.carry);
            cpu.regs.set_a(new_value);
            cpu.regs.f.zero = false;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = false;
            cpu.regs.f.carry = get_nth_bit(value, 7);
        }
        AccFlagOp::RRA => {
            let value = cpu.regs.get_a();
            let new_value = nth_bit_to(value >> 1, 7, cpu.regs.f.carry);
            cpu.regs.set_a(new_value);
            cpu.regs.f.zero = false;
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = false;
            cpu.regs.f.carry = get_nth_bit(value, 0);
        }
        AccFlagOp::DAA => {
            // https://forums.nesdev.org/viewtopic.php?t=15944#:~:text=The%20DAA%20instruction%20adjusts%20the,%2C%20lower%20nybble%2C%20or%20both.
            // Flags: Z - 0 C
            let mut value = cpu.regs.get_a();
            if !cpu.regs.f.subtraction {
                if cpu.regs.f.carry || value > 0x99 {
                    value += 0x60;
                    cpu.regs.f.carry = true;   
                }
                if cpu.regs.f.half_carry || (value & 0x0f) > 0x09 {
                    value += 0x06;
                }
            } else {
                if cpu.regs.f.carry {
                    value -= 0x60;
                }
                if cpu.regs.f.half_carry {
                    value -= 0x06;
                }
            }
            cpu.regs.set_a(value);
            cpu.regs.f.zero = value == 0;
            cpu.regs.f.half_carry = false;
        }
        AccFlagOp::CPL => {
            // Flags: - 1 1 -
            let a = cpu.regs.get_a();
            cpu.regs.set_a(!a);
            cpu.regs.f.subtraction = true;
            cpu.regs.f.half_carry = true;
        }
        AccFlagOp::SCF => {
            // Flags: - 0 0 !C
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = false;
            cpu.regs.f.carry = true;
        }
        AccFlagOp::CCF => {
            // Flags: - 0 0 !C
            cpu.regs.f.subtraction = false;
            cpu.regs.f.half_carry = false;
            cpu.regs.f.carry = !cpu.regs.f.carry;
        }
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

fn bit_11_overflow(op1: u16, op2: u16) -> bool {
    (((op1 & 0x0fff) + (op2 & 0x0fff)) & 0x1000) == 0x1000
}

// fn bit_7_overflow(op1: u8, op2: u8) -> bool {
//     (((op1 & 0xff) + (op2 & 0xff)) & 0x100) == 0x100
// }

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

fn get_r(val: u8) -> u8 {
    (val & 0b00011000) >> 3
}