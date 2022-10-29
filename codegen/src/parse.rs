use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::{ Map, Value };
use serde::Deserialize;

// {
//  "mnemonic": "NOP",
//  "bytes": 1,
//  "cycles": [
//    4
//  ],
//  "operands": [],
//  "immediate": true,
//  "flags": {
//    "Z": "-",
//    "N": "-",
//    "H": "-",
//    "C": "-"
//  }
//}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Operand {
    pub name: String,
    pub bytes: Option<u8>,
    pub immediate: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Flags {
    #[serde(rename = "Z")]
    z: String, 
    #[serde(rename = "N")]
    n: String,
    #[serde(rename = "H")]
    h: String,
    #[serde(rename = "C")]
    c: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Instruction {
    pub mnemonic: String, 
    pub bytes: u8,
    pub cycles: Vec<u8>,
    pub operands: Vec<Operand>,
    pub immediate: bool, 
    // pub group: String,
    pub flags: Flags,
    // pub operand1: Option<String>,
    // pub operand2: Option<String>,
}

pub(crate) type Opcode = u16;

fn parse_inner(opcodes: String, prefix: u8) -> Vec<(u16, Instruction)> {
    let opcodes: Map<String, Value> = serde_json::from_str(&opcodes).unwrap(); 
    let mut opcodes = opcodes.into_iter()
        .map(|(k, v)| (
            // ignoring the "0x" prefix
            (prefix as u16) << 8 | u16::from_str_radix(&k[2..], 16).unwrap(),
            serde_json::from_value::<Instruction>(v).unwrap()
        ))
        .collect::<Vec<(Opcode, Instruction)>>();
    opcodes.sort_by_key(|(k, _)| *k );
    opcodes
}

pub(crate) fn parse_opcodes(filepath: impl AsRef<Path>) -> Vec<(Opcode, Instruction)> {
    // all the errors are irrecoverable
    // so panicing when things go wrong is a valid strategy

    let file = File::open(filepath).unwrap();
    let reader = BufReader::new(file);

    let opcodes: Map<String, Value> = serde_json::from_reader(reader).unwrap();

    let mut unprefixed = parse_inner(opcodes["unprefixed"].to_string(), 0);
    let cbprefixed = parse_inner(opcodes["cbprefixed"].to_string(), 0xcb);
    
    unprefixed.extend(cbprefixed);

    unprefixed
}