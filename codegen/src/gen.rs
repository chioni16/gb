use crate::parse::{ Instruction, Opcode };

use quote::{quote, format_ident};

pub(crate) fn execute(opcodes: Vec<(Opcode, Instruction)>) -> Result<String, Box<dyn std::error::Error>> {
    let mut functions = quote!();
    #[allow(nonstandard_style)]
    for (opcode, i) in opcodes.clone() {
        // TODO not sure what to do when there are 2 cycle values
        let cycles = i.cycles[0];
        let len_of_inst = i.bytes;
        let mnemonic = i.mnemonic;

        let mut body = quote!();
        if opcode < 0xcb00 && i.bytes > 1 {
            for op in i.operands {
                if let Some(length) = op.bytes {
                    let name = op.name;
                    let method = format_ident!("readu{}", length*8);
                    let op = quote!({
                        print!(" {} ", #name);
                        print!(" {:#x} ", cpu.#method());
                    });
                    body.extend(op);
                }
            }
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
