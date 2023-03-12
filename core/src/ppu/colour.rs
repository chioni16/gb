// =====================
// Value	Colour
// =====================
//    0	    White
//    1	    Light gray
//    2	    Dark gray
//    3	    Black

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum Colour {
    White = 0,
    LightGrey = 1,
    DarkGrey = 2,
    Black = 3,
}

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Colour::White => '█',
            Colour::LightGrey => '▓',
            Colour::DarkGrey => '▒',
            Colour::Black => '░',
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
            Colour::Black => 0,
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
            _ => return Err(()),
        };
        Ok(res)
    }
}
