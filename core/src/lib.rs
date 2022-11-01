#![feature(bigint_helper_methods)]

mod cpu;

use std::{path::Path, fs, io::Read};

use cpu::CPU;

// const DEBUG: bool = true;

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
        // buf.resize(32*1024, 0);
        // let logo: [u8; 48] = 
        //     [0xce,0xed,0x66,0x66,0xcc,0x0d,0x00,0x0b,0x03,0x73,0x00,0x83,
        //      0x00,0x0c,0x00,0x0d,0x00,0x08,0x11,0x1f,0x88,0x89,0x00,0x0e,
        //      0xdc,0xcc,0x6e,0xe6,0xdd,0xdd,0xd9,0x99, 0xbb,0xbb,0x67,0x63,
        //      0x6e,0x0e,0xec,0xcc,0xdd,0xdc,0x99,0x9f,0xbb,0xb9,0x33,0x3e];
        // logo.into_iter().enumerate().for_each(|(i, v)| {
        //     buf[0x104 + i] = v;
        // });
        // let checksum = [0x42,0x47,0x42,0x57,0x45,0x4C,0x43,0x4F,0x4D, 0x45,0x10,00,00,00,00,00,00,00,00,00,00,00,00,00,00];
        // checksum.into_iter().enumerate().for_each(|(i, v)| {
        //     buf[0x134 + i] = v;
        // });
        let mut m = Self {
            cpu: CPU::new(buf)
        };
        m.cpu.no_boot();
        Ok(m)
    }
    pub fn run(&mut self) {
        loop {
            if cfg!(feature="debug") {
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
