#![feature(bigint_helper_methods)]

mod cpu;

use std::{path::Path, fs, io::Read};

use cpu::CPU;

const DEBUG: bool = false;

pub struct Machine {
    cpu: CPU,
}

impl Machine {
    pub fn new(cartridge: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = fs::File::open(cartridge.as_ref())?;
        // TODO change this / find a suitable function
        // let mut buf = vec![0u8; 32*1024];
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        Ok(Self {
            cpu: CPU::new(buf)
        })
    }
    pub fn run(&mut self) {
        loop {
            if DEBUG {
                pause();
            }
            self.cpu.step();
        }
    }
}

use std::io;
use std::io::prelude::*;

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}
