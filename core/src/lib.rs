#![feature(bigint_helper_methods)]

mod cpu;

use std::{path::Path, fs, io::Read};

use cpu::CPU;

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
            self.cpu.step();
        }
    }
}
