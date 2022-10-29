use super::{CPU, Addr};

pub(super) fn decode(opcode: u16, cpu: &mut CPU) -> Result<u8, u16>{
    match opcode {
        0x0000 => {}
        0x0001 => {
            let v = cpu.readu16();
            cpu.regs.set_bc(v);
        }
        0x0002 | 0x0012 | 0x0022 | 0x0032 => write_to_r16_group2(cpu, opcode as u8, cpu.regs.get_a()),
        0x0006 | 0x0016 | 0x0026 | 0x0036 | 0x000e | 0x001e | 0x002e | 0x003e => {
            let v = cpu.readu8();
            write_to_r8(cpu, opcode as u8, v);
        }
        0x000a | 0x001a | 0x002a | 0x003a => {
            let v = read_from_r16_group2(cpu, opcode as u8);
            cpu.regs.set_a(v);
        }
       
        0x0011 => {
            let v = cpu.readu16();
            cpu.regs.set_de(v);
        }
        
        0x0021 => {
            let v = cpu.readu16();
            cpu.regs.set_hl(v);
        }
        
        0x0031 => {
            let v = cpu.readu16();
            cpu.push(v);
        }
        
        0x0040..=0x0075 | 0x0077..=0x007f => {
            r8_to_r8(cpu, opcode as u8);
        }
        // 0x0080..=0x00bf => {
        //     let op2 = read_from_r8(cpu, opcode as u8);
        //     perform_alu8(cpu, opcode as u8, op2);
        // }
        _ => unimplemented!()
    };
    Ok(0)
}

fn read_from_r8(cpu: &mut CPU, opcode: u8) -> u8 {
    let src = opcode & 0b00000111;

    match src {
        0 => cpu.regs.get_b(),
        1 => cpu.regs.get_c(),
        2 => cpu.regs.get_d(),
        3 => cpu.regs.get_e(),
        4 => cpu.regs.get_h(),
        5 => cpu.regs.get_l(),
        6 => cpu.mmu.readu8(cpu.regs.get_hl().into()),
        7 => cpu.regs.get_a(),
        _ => unreachable!()
    }
}

fn write_to_r8(cpu: &mut CPU, opcode: u8, v: u8) {
    let dst = (opcode & 0b00111000) >> 3;

    match dst {
        0 => cpu.regs.set_b(v),
        1 => cpu.regs.set_c(v),
        2 => cpu.regs.set_d(v),
        3 => cpu.regs.set_e(v),
        4 => cpu.regs.set_h(v),
        5 => cpu.regs.set_l(v),
        6 => cpu.mmu.writeu8(cpu.regs.get_hl().into(), v),
        7 => cpu.regs.set_a(v),
        _ => unreachable!()
    }

}

fn r8_to_r8(cpu: &mut CPU, opcode: u8) {
    let v = read_from_r8(cpu, opcode);
    write_to_r8(cpu, opcode, v);
}


fn get_addr_from_r16_group2(cpu: &mut CPU, opcode: u8) -> Addr {
    let reg = (opcode & 0b00110000) >> 4;
    let addr = match reg {
        0 => cpu.regs.get_bc(),
        1 => cpu.regs.get_de(),
        2 => {
            let addr = cpu.regs.get_hl();
            cpu.regs.incr_hl();
            addr
        }
        3 => {
            let addr = cpu.regs.get_hl();
            cpu.regs.decr_hl();
            addr
        }
        _ => unreachable!()
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

// fn perform_alu8(cpu: &mut CPU, opcode: u8, op2: u8) {
//     let operation = (opcode & 0b00111000) >> 3;
//     let a = cpu.regs.get_a();
//     let v = match operation {
//         0 => {
//             add8(a, op2)
//         }
//         1 => ,
//         2 => ,
//         3 => ,
//         4 => ,
//         5 => ,
//         6 => ,
//         7 => ,
//         _ => unreachable!()
//     };
//     cpu.regs.set_a(v);
// }

// fn add8(op1: u8, op2: u8) -> (u8, bool) {
//     op1 + op2
// }
// fn adc8(op1: u8, op2: u8, c: bool) -> bool {
//     u8::carrying_add(self, rhs, carry)
// }