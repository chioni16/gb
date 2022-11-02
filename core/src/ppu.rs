use crate::{mmu::MMU, Addr};

// https://forums.nesdev.org/viewtopic.php?f=20&t=17754&p=225009#p225009
// http://blog.kevtris.org/blogfiles/Nitty%20Gritty%20Gameboy%20VRAM%20Timing.txt
// http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings

// ===================================================================================
// Period	                    GPU mode number	            Time spent (clocks)
// ===================================================================================
// Scanline (accessing OAM)	        2	                            80
// Scanline (accessing VRAM)	    3	                            172
// Horizontal blank	                0	                            204
// One line (scan and blank)		                                456
// Vertical blank	                1	                        4560 (10 lines)
// Full frame (scans and vblank)		                            70224

#[repr(u8)]
enum PpuState {
    OAMSearch = 2,
    PixelTransfer = 3, 
    HBlank = 0,
    VBlank = 1,
}

pub (crate) struct PPU {
    state: PpuState,
    ticks: usize,
    line: u8,
}

impl PPU {
    pub(crate) fn new() -> Self {
        Self {
            state: PpuState::OAMSearch,
            ticks: 0,
            line: 0,
        }
    }

    pub(crate) fn tick(&mut self, _mmu: &mut MMU, cpu_ticks: usize) {
        self.ticks += cpu_ticks;

        match self.state {
            PpuState::OAMSearch => {
                if self.ticks >= 80 {
                    self.ticks -= 80;
                    self.state = PpuState::PixelTransfer;
                }
            }
            PpuState::PixelTransfer => {
                if self.ticks >= 172 {
                    self.ticks -= 172;
                    self.state = PpuState::HBlank;
                }
            }
            PpuState::HBlank => {
                if self.ticks >= 204 {
                    self.ticks -= 204;
                    self.line += 1;

                    if self.line < 144 {
                        self.state = PpuState::OAMSearch;
                    } else {
                        // TODO draw the line
                        self.state = PpuState::VBlank;
                    }
                    self.state = PpuState::HBlank;
                }
            }
            PpuState::VBlank => {
                if self.ticks >= 456 {
                    self.ticks -= 456;
                    self.line += 1;

                    if !(self.line < 154) {
                        self.state = PpuState::OAMSearch;
                        self.line = 0;
                    }
                }
            }
        }
    }
}

// Registers
// ==============================================
// Address	Register	            Status
// ==============================================
// 0xFF40	LCD and GPU control	    Read/write
// 0xFF42	Scroll-Y	            Read/write
// 0xFF43	Scroll-X	            Read/write
// 0xFF44	Current scan line	    Read only
// 0xFF47	Background palette	    Write only

const REG_LCDC           : Addr = Addr::from(0xff40);
const REG_SCROLL_Y       : Addr = Addr::from(0xff42);
const REG_SCROLL_X       : Addr = Addr::from(0xff43);
const REG_CURR_SCANLINE  : Addr = Addr::from(0xff44);
const REG_BG_PALETTE     : Addr = Addr::from(0xff47);

impl PPU {
    fn get_lcdc(&self, mmu: &MMU) -> LCDC {
        mmu.readu8(REG_LCDC).into()
    }
    fn set_lcdc(&self, mmu: &mut MMU, value: LCDC) {
        mmu.writeu8(REG_LCDC, value.into())
    }

    fn get_scroll_y(&self, mmu: &MMU) -> u8 {
        mmu.readu8(REG_SCROLL_Y)
    }
    fn set_scroll_y(&self, mmu: &mut MMU, value: u8) {
        mmu.writeu8(REG_SCROLL_Y, value)
    }

    fn get_scroll_x(&self, mmu: &MMU) -> u8 {
        mmu.readu8(REG_SCROLL_X)
    }
    fn set_scroll_x(&self, mmu: &mut MMU, value: u8) {
        mmu.writeu8(REG_SCROLL_X, value)
    }

    fn get_curr_scanline(&self, mmu: &MMU) -> u8 {
        mmu.readu8(REG_CURR_SCANLINE)
    }
    fn set_bg_palette(&self, mmu: &mut MMU, value: Palette) {
        mmu.writeu8(REG_BG_PALETTE, value.into())
    }
}

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

enum TileMap {
    Low,   // 9800-9BFF
    High,  // 9C00-9FFF
}

impl From<TileMap> for bool {
    fn from(value: TileMap) -> Self {
        match value {
            TileMap::Low   => false,
            TileMap::High  => true,
        }
    }
}

impl From<bool> for TileMap {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Low,
            true  => Self::High,
        }
    }
}

enum TileAddressing {
    Low,  // 8000-8FFF, 0 - 255
    High, // 8800-97FF, -128 - 127
}

impl From<TileAddressing> for bool {
    fn from(value: TileAddressing) -> Self {
        match value {
            TileAddressing::Low  => true, 
            TileAddressing::High => false,
        }
    }
}

impl From<bool> for TileAddressing {
    fn from(value: bool) -> Self {
        match value {
            false => Self::High,
            true  => Self::Low,
        }
    }
}

enum ObjectSize {
    Short, // 8x8
    Long,  // 8x16
}

impl From<ObjectSize> for bool {
    fn from(value: ObjectSize) -> Self {
        match value {
            ObjectSize::Short => false,
            ObjectSize::Long  => true,
        }
    }
}

impl From<bool> for ObjectSize {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Short,
            true  => Self::Long,
        }
    }
}

struct LCDC {
    ppu_enable: bool,
    window_tile_map: TileMap,
    window_enable: bool,
    bg_window_tile_area: TileAddressing,
    bg_tile_map: TileMap,
    object_size: ObjectSize,
    object_enable: bool,
    bg_window_enable: bool,
}

impl From<u8> for LCDC {
    fn from(value: u8) -> Self {
        Self {
            ppu_enable: get_nth_bit(value, 7),
            window_tile_map: get_nth_bit(value, 6).into(),
            window_enable: get_nth_bit(value, 5),
            bg_window_tile_area: get_nth_bit(value, 4).into(),
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
        | <TileMap as Into<bool>>::into(value.window_tile_map) as u8) << 1
        | value.window_enable as u8) << 1
        | <TileAddressing as Into<bool>>::into(value.bg_window_tile_area) as u8) << 1
        | <TileMap as Into<bool>>::into(value.bg_tile_map) as u8) << 1
        | <ObjectSize as Into<bool>>::into(value.object_size) as u8) << 1
        | value.object_enable as u8) << 1
        | value.bg_window_enable as u8
    }
}

// =====================
// Value	Colour
// =====================
//    0	    White
//    1	    Light gray
//    2	    Dark gray
//    3	    Black

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Colour {
    White     = 0, 
    LightGrey = 1,
    DarkGrey  = 2,
    Black     = 3,
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

struct Palette {
    // each field can hold one of four values: 0,1,2,3
    colour0: Colour, // bits 0-1
    colour1: Colour, // bits 2-3
    colour2: Colour, // bits 4-5
    colour3: Colour, // bits 6-7
}

impl From<u8> for Palette {
    fn from(value: u8) -> Self {
        Self { 
            colour0: Colour::try_from(((get_nth_bit(value, 1) as u8)<< 1) | get_nth_bit(value, 0) as u8).unwrap(),
            colour1: Colour::try_from(((get_nth_bit(value, 3) as u8)<< 1) | get_nth_bit(value, 2) as u8).unwrap(),
            colour2: Colour::try_from(((get_nth_bit(value, 5) as u8)<< 1) | get_nth_bit(value, 4) as u8).unwrap(),
            colour3: Colour::try_from(((get_nth_bit(value, 7) as u8)<< 1) | get_nth_bit(value, 6) as u8).unwrap(),
        }
    }
}

impl From<Palette> for u8 {
    fn from(value: Palette) -> Self {
        ((((value.colour3 as u8) << 2)
        | (value.colour2 as u8) << 2)
        | (value.colour1 as u8) << 2)
        | value.colour0 as u8
    }
}


fn get_nth_bit(value: u8, n: u8) -> bool {
    assert!(n < 8);
    match (value >> n) & 1 {
        0 => false, 
        1 => true, 
        _ => unreachable!()
    }
}