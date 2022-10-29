use crate::parse::{ Instruction, Opcode };

use quote::{quote, format_ident, __private::TokenStream};

pub(crate) fn execute(opcodes: Vec<(Opcode, Instruction)>) -> Result<String, Box<dyn std::error::Error>> {
    let mut functions = quote!();
    #[allow(nonstandard_style)]
    for (opcode, i) in opcodes.clone() {
        // TODO not sure what to do when there are 2 cycle values
        let cycles = i.cycles[0];
        let len_of_inst = i.length;
        let mnemonic = i.mnemonic;

        let mut body = quote!();
        // if opcode < 0xcb00 && i.length > 1 {
        //     for op in i.operands {
        //         if let Some(length) = op.bytes {
        //             let name = op.name;
        //             let method = format_ident!("readu{}", length*8);
        //             let op = quote!({
        //                 print!(" {} ", #name);
        //                 print!(" {:#x} ", cpu.#method());
        //             });
        //             body.extend(op);
        //         }
        //     }
        // }
        let group = i.group.as_str();
        match group {
            "x8/lsm" => {
                // 8-bit load instructions transfer one byte of data between two 8-bit registers, 
                // or between one 8-bit register and location in memory
                assert!(opcode < 0xcb00);
                match mnemonic.as_str() {
                    "LD"  => { 
                        let v = getter8(&i.operand2.unwrap());
                        // {{ i.operands[0] | setter(bits=i.bits) }}v);
                    }
                    "LDH" => {}
                    _ => unreachable!()
                }
            }
            "x16/lsm" => {
                assert!(opcode < 0xcb00);
                match mnemonic.as_str() {
                    "LD"   => {}
                    "POP"  => {}
                    "PUSH" => {}
                    _ => unreachable!()
                }
            }
            "x8/alu" => {
                assert!(opcode < 0xcb00);
                match mnemonic.as_str() {
                    "ADC" => {}
                    "ADD" => {}
                    "AND" => {}
                    "CCF" => {}
                    "CP"  => {}
                    "CPL" => {}
                    "DAA" => {}
                    "DEC" => {}
                    "INC" => {}
                    "OR"  => {}
                    "SBC" => {}
                    "SCF" => {}
                    "SUB" => {}
                    "XOR" => {}
                    _     => unreachable!()
                }
            }
            "x16/alu" => {
                assert!(opcode < 0xcb00);
                match mnemonic.as_str() {
                    "ADD" => {} 
                    "DEC" => {} 
                    "INC" => {}
                    _     => unreachable!()
                }
            }
            "x8/rsb" => {
                if opcode < 0xcb00 {
                    // unprefixed
                    match mnemonic.as_str() {
                        "RLA"  => {}
                        "RLCA" => {}
                        "RRA"  => {}
                        "RRCA" => {}
                        _      => unreachable!()
                    }
                } else  {
                    // cbprefixed
                    match mnemonic.as_str() {
                        "BIT"  => {}
                        "RES"  => {}
                        "RL"   => {}
                        "RLC"  => {}
                        "RR"   => {}
                        "RRC"  => {}
                        "SET"  => {}
                        "SLA"  => {}
                        "SRA"  => {}
                        "SRL"  => {}
                        "SWAP" => {}
                        _      => unreachable!()
                    }
                }
            }
            "control/br" => {
                assert!(opcode < 0xcb00);
                match mnemonic.as_str() {
                    "CALL" => {}
                    "JP"   => {}
                    "JR"   => {}
                    "RET"  => {}
                    "RETI" => {}
                    "RST"  => {}
                    _      => unreachable!()
                }
            }
            "control/misc" => {
                assert!(opcode < 0xcb00);
                match mnemonic.as_str() {
                    "DI"     => {}
                    "EI"     => {}
                    "HALT"   => {}
                    "NOP"    => {}
                    "PREFIX" => {}
                    "STOP"   => {} 
                    _        => unreachable!()
                }
            }
            _ => unreachable!()
        }
        let name = format_ident!("op_{:0>4x}", opcode);
        functions.extend(quote! {
            #[allow(unused_variables)]
            fn #name(cpu: &mut CPU) -> (u8, u8) { 
                print!("{}", #mnemonic);
                #body
                println!();
                (#cycles, #len_of_inst) 
            }
        });
    }

    let mut branches = quote!();
    #[allow(nonstandard_style)]
    for (opcode, _) in opcodes.clone() {
        // TODO not sure what to do when there are 2 cycle values
        let func = format_ident!("op_{:0>4x}", opcode);
        branches.extend(quote! {
            #opcode => #func(cpu),
        });
    }

    let code = quote! {
        use super::CPU;
        // returns cycles consumed and the length of the instruction
        pub(super) fn decode(opcode: u16, cpu: &mut CPU) -> Result<(u8, u8), u16> {
            let res = match opcode {
                #branches
                _ => return Err(opcode),
            };
            Ok(res)
        }
        #functions
    };

    Ok(code.to_string())
}

fn getter8(s: &str) -> TokenStream {
    match s {
        "d8" => {}
        "A" | "B" | "C" | "D" | "E" | "H" | "L" => {}
        "()" => {}
    }
    quote!()
}
fn getter16(s: &str) -> TokenStream {
    quote!()
}