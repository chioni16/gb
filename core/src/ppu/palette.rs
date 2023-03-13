use super::get_nth_bit;
use super::Colour;

#[derive(Debug, Clone, Copy)]
pub(crate) struct BgWinPalette {
    // each field can hold one of four values: 0,1,2,3
    colour0: Colour, // bits 0-1
    colour1: Colour, // bits 2-3
    colour2: Colour, // bits 4-5
    colour3: Colour, // bits 6-7
}

impl BgWinPalette {
    pub(crate) fn map(&self, v: u8) -> Result<Colour, u8> {
        let colour = match v {
            0 => self.colour0,
            1 => self.colour1,
            2 => self.colour2,
            3 => self.colour3,
            _ => return Err(v)
        };
        Ok(colour)
    }
}

impl From<u8> for BgWinPalette {
    fn from(value: u8) -> Self {
        Self {
            colour0: Colour::try_from(
                ((get_nth_bit(value, 1) as u8) << 1) | get_nth_bit(value, 0) as u8,
            )
            .unwrap(),
            colour1: Colour::try_from(
                ((get_nth_bit(value, 3) as u8) << 1) | get_nth_bit(value, 2) as u8,
            )
            .unwrap(),
            colour2: Colour::try_from(
                ((get_nth_bit(value, 5) as u8) << 1) | get_nth_bit(value, 4) as u8,
            )
            .unwrap(),
            colour3: Colour::try_from(
                ((get_nth_bit(value, 7) as u8) << 1) | get_nth_bit(value, 6) as u8,
            )
            .unwrap(),
        }
    }
}

impl From<BgWinPalette> for u8 {
    fn from(value: BgWinPalette) -> Self {
        ((((value.colour3 as u8) << 2) | (value.colour2 as u8) << 2) | (value.colour1 as u8) << 2)
            | value.colour0 as u8
    }
}

impl Default for BgWinPalette {
    fn default() -> Self {
        Self::from(0)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ObjPalette {
    // each field can hold one of four values: 0,1,2,3
    colour1: Colour, // bits 2-3
    colour2: Colour, // bits 4-5
    colour3: Colour, // bits 6-7
}
impl From<u8> for ObjPalette {
    fn from(value: u8) -> Self {
        Self {
            colour1: Colour::try_from(
                ((get_nth_bit(value, 3) as u8) << 1) | get_nth_bit(value, 2) as u8,
            )
            .unwrap(),
            colour2: Colour::try_from(
                ((get_nth_bit(value, 5) as u8) << 1) | get_nth_bit(value, 4) as u8,
            )
            .unwrap(),
            colour3: Colour::try_from(
                ((get_nth_bit(value, 7) as u8) << 1) | get_nth_bit(value, 6) as u8,
            )
            .unwrap(),
        }
    }
}

impl ObjPalette {
    pub(crate) fn map(&self, v: u8) -> Result<Colour, u8> {
        let colour = match v {
            0 => Colour::Transparent,
            1 => self.colour1,
            2 => self.colour2,
            3 => self.colour3,
            _ => return Err(v)
        };
        Ok(colour)
    }
}

impl From<ObjPalette> for u8 {
    fn from(value: ObjPalette) -> Self {
        (((value.colour3 as u8) << 2) | (value.colour2 as u8) << 2) | (value.colour1 as u8) << 2
    }
}

impl Default for ObjPalette {
    fn default() -> Self {
        Self::from(0)
    }
}