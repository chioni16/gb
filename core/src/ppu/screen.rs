use std::fmt::{Debug, Display};

use super::Colour;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

pub struct Screen([Colour; SCREEN_WIDTH * SCREEN_HEIGHT]);

pub static mut screen_u32: [u32; SCREEN_WIDTH * SCREEN_HEIGHT] = [0; SCREEN_WIDTH * SCREEN_HEIGHT];

impl Screen {
    pub fn new() -> Self {
        Self([Colour::White; SCREEN_WIDTH * SCREEN_HEIGHT])
    }

    pub(super) fn set(&mut self, row: u8, col: u8, colour: Colour) {
        let index = row as usize * SCREEN_WIDTH + col as usize;
        self.0[index] = colour;
        unsafe {
            screen_u32[index] = colour.into();
        }
    }

    pub fn as_u32(&self) -> Vec<u32> {
        self.0.iter().map(|c| (*c).into()).collect::<Vec<_>>()
    }
}

impl Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..SCREEN_HEIGHT {
            for c in &self.0[(i * SCREEN_WIDTH)..((i + 1) * SCREEN_WIDTH)] {
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        write!(f, "\x1b[2J\x1b[H")?;
        Ok(())
    }
}

impl Debug for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..SCREEN_HEIGHT {
            for c in &self.0[(i * SCREEN_WIDTH)..((i + 1) * SCREEN_WIDTH)] {
                write!(f, "{}", *c as u8)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
