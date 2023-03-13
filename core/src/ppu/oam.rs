use std::char::MAX;

use super::lcdc::ObjectSize;
use super::{get_nth_bit, OAM_SIZE, MAX_SPRITES_PER_ROW};
use crate::mmu::ram::RAM;
use crate::mmu::busio::{BusIO, SResult};
use crate::util::Addr;


#[derive(Debug)]
pub(crate) struct OAM(pub RAM);

impl OAM {
    pub(crate) fn row(&self, y: u8, obj_size: ObjectSize) -> impl Iterator<Item = Sprite> + '_ {
        let sprite_size = match obj_size {
            ObjectSize::Long  => 16,
            ObjectSize::Short => 8,
        };
        self
            .into_iter()
            .filter(move |s| s.y_pos <= y + 16 && y + 16 <= s.y_pos + sprite_size )
            .take(MAX_SPRITES_PER_ROW)
    }
}

impl BusIO for OAM {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        self.0.readu8(addr)
    }

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        self.0.writeu8(addr, value)
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        self.0.readu16(addr)
    }

    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        self.0.writeu16(addr, value)
    }

    fn print_dbg(&self, start: Addr, len: u16) -> String {
        self.0.print_dbg(start, len)
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        self.0.as_slice(addr, len)
    }
}

impl<'a> IntoIterator for &'a OAM {
    type Item = Sprite;
    type IntoIter = OAMIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        OAMIter {
            bytes: &self.0.buffer,
            index: 0,
        }
    }
}

pub(crate) struct OAMIter<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Iterator for OAMIter<'a> {
    type Item = Sprite;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < OAM_SIZE {
            let res = Some(Sprite::new(&self.bytes[self.index..][..4]));
            self.index += 4;
            res
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Sprite {
    pub(crate) y_pos: u8,
    pub(crate) x_pos: u8,
    pub(crate) ti: u8,
    pub(crate) attr: SpriteAttr,
}

impl Sprite {
    pub(crate) fn new(sprite: &[u8]) -> Self {
        Self {
            y_pos: sprite[0],
            x_pos: sprite[1],
            ti: sprite[2],
            attr: SpriteAttr::from(sprite[3]),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SpriteAttr {
    pub(crate) bg_win_over_obj: bool,
    pub(crate) y_flip: bool,
    pub(crate) x_flip: bool,
    pub(crate) palette: ObjPaletteType,
}

impl From<u8> for SpriteAttr {
    fn from(value: u8) -> Self {
        Self {
            bg_win_over_obj: get_nth_bit(value, 7),
            y_flip: get_nth_bit(value, 6),
            x_flip: get_nth_bit(value, 5),
            palette: get_nth_bit(value, 4).into(),
        }
    }
}

impl From<SpriteAttr> for u8 {
    fn from(value: SpriteAttr) -> Self {
        (value.bg_win_over_obj as u8) << 7
        | (value.y_flip as u8) << 6
        | (value.x_flip as u8) << 5
        | (value.palette as u8) << 4
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub(crate) enum ObjPaletteType {
    #[default]
    OBP0 = 0,
    OBP1 = 1,
}

impl From<bool> for ObjPaletteType {
    fn from(value: bool) -> Self {
        match value {
            false => Self::OBP0,
            true => Self::OBP1,
        }
    }
}
impl TryFrom<u8> for ObjPaletteType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ObjPaletteType::OBP0),
            1 => Ok(ObjPaletteType::OBP1),
            _ => Err(()),
        } 
    }
}