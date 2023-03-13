// =====================
// Value	Colour
// =====================
//    0	    White
//    1	    Light gray
//    2	    Dark gray
//    3	    Black

use std::ops::{BitAnd, BitOr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Colour {
    White = 0,
    LightGrey = 1,
    DarkGrey = 2,
    Black = 3,
    Transparent = 4,
}

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Colour::White => '█',
            Colour::LightGrey => '▓',
            Colour::DarkGrey => '▒',
            Colour::Black => '░',
            Colour::Transparent => '⠀',
        };
        write!(f, "{}", c)
    }
}

impl Into<u32> for Colour {
    fn into(self) -> u32 {
        match self {
            Colour::White => (255 << 16) | (255 << 8) | 255,
            Colour::LightGrey => (192 << 16) | (192 << 8) | 192,
            Colour::DarkGrey => (96 << 16) | (96 << 8) | 96,
            Colour::Black | Colour::Transparent => 0,
        }
    }
}

impl TryFrom<u8> for Colour {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            0 => Self::White,
            1 => Self::LightGrey,
            2 => Self::DarkGrey,
            3 => Self::Black,
            4 => Self::Transparent,
            _ => return Err(()),
        };
        Ok(res)
    }
}

// favour the left side
impl BitOr for Colour {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Colour::Transparent, colour) => colour,
            (colour, _)                   => colour,             
        }
    }
}

// // favour the right side
// impl BitAnd for Colour {
//     type Output = Self;
//     fn bitand(self, rhs: Self) -> Self::Output {
//         match (self, rhs) {

//         }
//     }
// }