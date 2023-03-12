use super::get_nth_bit;
use super::Colour;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Palette {
    // each field can hold one of four values: 0,1,2,3
    colour0: Colour, // bits 0-1
    colour1: Colour, // bits 2-3
    colour2: Colour, // bits 4-5
    colour3: Colour, // bits 6-7
}

impl From<u8> for Palette {
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

impl From<Palette> for u8 {
    fn from(value: Palette) -> Self {
        ((((value.colour3 as u8) << 2) | (value.colour2 as u8) << 2) | (value.colour1 as u8) << 2)
            | value.colour0 as u8
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::from(0)
    }
}
