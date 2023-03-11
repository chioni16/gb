use crate::util::Addr;
use super::get_nth_bit;
// LCDC
// ============================================================================
// Bit	Name	                        Usage notes
// ============================================================================
//  7	LCD and PPU enable	            0=Off, 1=On
//  6	Window tile map area	        0=9800-9BFF, 1=9C00-9FFF
//  5	Window enable	                0=Off, 1=On
//  4	BG and Window tile data area	0=8800-97FF, 1=8000-8FFF
//  3	BG tile map area	            0=9800-9BFF, 1=9C00-9FFF
//  2	OBJ size	                    0=8x8, 1=8x16
//  1	OBJ enable	                    0=Off, 1=On
//  0	BG and Window enable/priority	0=Off, 1=On

pub(super) enum TileMap {
    Low,  // 9800-9BFF
    High, // 9C00-9FFF
}

impl From<TileMap> for bool {
    fn from(value: TileMap) -> Self {
        match value {
            TileMap::Low => false,
            TileMap::High => true,
        }
    }
}

impl From<bool> for TileMap {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Low,
            true => Self::High,
        }
    }
}

impl From<TileMap> for Addr {
    fn from(value: TileMap) -> Self {
        match value {
            TileMap::Low => 0x9800.into(),
            TileMap::High => 0x9c00.into(),
        }
    }
}

pub(super) enum TileData {
    Low,  // 8000-8FFF, 0 - 255
    High, // 8800-97FF, -128 - 127
}

impl TileData {
    pub(super) fn get_tile_data_addr(&self, tile_index: u8) -> Addr {
        match self {
            Self::Low => {
                let base = 0x8000u16;
                let tile_index: i16 = 16 * tile_index as i16;
                base.wrapping_add_signed(tile_index).into()
            }
            Self::High => {
                let base = 0x9000u16;
                let tile_index: i16 = 16 * (tile_index as i8) as i16;
                base.wrapping_add_signed(tile_index).into()
            }
        }
    }
}

impl From<TileData> for bool {
    fn from(value: TileData) -> Self {
        match value {
            TileData::Low => true,
            TileData::High => false,
        }
    }
}

impl From<bool> for TileData {
    fn from(value: bool) -> Self {
        match value {
            false => Self::High,
            true => Self::Low,
        }
    }
}

pub(super) enum ObjectSize {
    Short, // 8x8
    Long,  // 8x16
}

impl From<ObjectSize> for bool {
    fn from(value: ObjectSize) -> Self {
        match value {
            ObjectSize::Short => false,
            ObjectSize::Long => true,
        }
    }
}

impl From<bool> for ObjectSize {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Short,
            true => Self::Long,
        }
    }
}

pub(super) struct LCDC {
    pub(super) ppu_enable: bool,
    pub(super) window_tile_map: TileMap,
    pub(super) window_enable: bool,
    pub(super) bg_window_tile_data: TileData,
    pub(super) bg_tile_map: TileMap,
    pub(super) object_size: ObjectSize,
    pub(super) object_enable: bool,
    pub(super) bg_window_enable: bool,
}

impl From<u8> for LCDC {
    fn from(value: u8) -> Self {
        Self {
            ppu_enable: get_nth_bit(value, 7),
            window_tile_map: get_nth_bit(value, 6).into(),
            window_enable: get_nth_bit(value, 5),
            bg_window_tile_data: get_nth_bit(value, 4).into(),
            bg_tile_map: get_nth_bit(value, 3).into(),
            object_size: get_nth_bit(value, 2).into(),
            object_enable: get_nth_bit(value, 1),
            bg_window_enable: get_nth_bit(value, 0),
        }
    }
}

impl From<LCDC> for u8 {
    fn from(value: LCDC) -> Self {
        (((((((value.ppu_enable as u8) << 1
            | <TileMap as Into<bool>>::into(value.window_tile_map) as u8)
            << 1
            | value.window_enable as u8)
            << 1
            | <TileData as Into<bool>>::into(value.bg_window_tile_data) as u8)
            << 1
            | <TileMap as Into<bool>>::into(value.bg_tile_map) as u8)
            << 1
            | <ObjectSize as Into<bool>>::into(value.object_size) as u8)
            << 1
            | value.object_enable as u8)
            << 1
            | value.bg_window_enable as u8
    }
}