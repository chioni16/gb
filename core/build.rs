use codegen;

use codegen::prepare_execute;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../codegen/");

    let opcodes = "/home/govardhan/projects/emulators/gb/codegen/Opcodes.json";
    let target_path = "/home/govardhan/projects/emulators/gb/core/src/cpu/instruction.rs";

    prepare_execute(opcodes, &target_path);
    Command::new("rustfmt")
        .arg(target_path).status()
        .unwrap_or_else(|err| panic!("rustfmt failed: {err}"));
}