mod screen;

use std::fmt::Display;

use crate::{mmu::MMU, Addr};
use screen::Screen;

// https://forums.nesdev.org/viewtopic.php?f=20&t=17754&p=225009#p225009
// http://blog.kevtris.org/blogfiles/Nitty%20Gritty%20Gameboy%20VRAM%20Timing.txt
// http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings
// https://pixelbits.16-b.it/GBEDG/ppu/

// ===================================================================================
// Period	                    GPU mode number	            Time spent (clocks)
// ===================================================================================
// Scanline (accessing OAM)	        2	                            80
// Scanline (accessing VRAM)	    3	                            172
// Horizontal blank	                0	                            204
// One line (scan and blank)		                                456
// Vertical blank	                1	                        4560 (10 lines)
// Full frame (scans and vblank)		                            70224

const TILE_WIDTH_PIXELS:    u8 = 8;
const TILE_HEIGHT_PIXELS:   u8 = 8;

const SCREEN_HEIGHT_PIXELS: u8 = 144;
const SCREEN_WIDTH_PIXELS:  u8 = 160;
const SCREEN_HEIGHT_TILES:  u8 = SCREEN_HEIGHT_PIXELS / TILE_HEIGHT_PIXELS;
const SCREEN_WIDTH_TILES:   u8 = SCREEN_WIDTH_PIXELS  / TILE_WIDTH_PIXELS;

const TILEMAP_HEIGHT_PIXELS: u16 = 256;
const TILEMAP_WIDTH_PIXELS:  u16 = 256;
const TILEMAP_HEIGHT_TILES:  u16 = TILEMAP_HEIGHT_PIXELS / TILE_HEIGHT_PIXELS as u16;
const TILEMAP_WIDTH_TILES:   u16 = TILEMAP_WIDTH_PIXELS  / TILE_WIDTH_PIXELS as u16;

// pseudo-scanlines
const VBLANK_LINES:         u8 = 10;

const TICKS_OAM_SEARCH:     u64 = 80;
const TICKS_PIXEL_TRANSFER: u64 = 172;
const TICKS_HBLANK:         u64 = 204;
const TICKS_ONE_LINE:       u64 = TICKS_OAM_SEARCH + TICKS_PIXEL_TRANSFER + TICKS_HBLANK;
const TICKS_VBLANK:         u64 = VBLANK_LINES as u64 * TICKS_ONE_LINE;              
const TICKS_ONE_FRAME:      u64 = SCREEN_HEIGHT_PIXELS as u64 * TICKS_ONE_LINE + TICKS_VBLANK;

type ColourScreenRow = [Colour; SCREEN_WIDTH_PIXELS as usize];
type ColourTileRow   = [Colour; TILE_WIDTH_PIXELS as usize];

#[repr(u8)]
enum PpuState {
    OAMSearch = 2,
    PixelTransfer = 3, 
    HBlank = 0,
    VBlank = 1,
}

pub (crate) struct PPU {
    state: PpuState,
    ticks: u64,
    screen: Screen,
}

impl PPU {
    pub(crate) fn new(mmu: &mut MMU) -> Self {
        let ppu = Self {
            state: PpuState::OAMSearch,
            ticks: 0,
            screen: Screen::new(),
        };
        // ppu.set_curr_scanline(mmu, 0);
        ppu
    }

    pub(crate) fn tick(&mut self, mmu: &mut MMU, cpu_ticks: u64) {
        self.ticks += cpu_ticks;

        match self.state {
            PpuState::OAMSearch => {
                if self.ticks >= TICKS_OAM_SEARCH {
                    self.ticks -= TICKS_OAM_SEARCH;
                    self.state = PpuState::PixelTransfer;
                }
            }
            PpuState::PixelTransfer => {
                if self.ticks >= TICKS_PIXEL_TRANSFER {
                    self.ticks -= TICKS_PIXEL_TRANSFER;
                    self.state = PpuState::HBlank;

                    self.renderscan(mmu);
                }
            }
            PpuState::HBlank => {
                if self.ticks >= TICKS_HBLANK {
                    self.ticks -= TICKS_HBLANK;
                    self.incr_curr_scanline(mmu);

                    if self.get_curr_scanline(mmu) < SCREEN_HEIGHT_PIXELS {
                        self.state = PpuState::OAMSearch;
                    } else {
                        println!("{}", self.screen);
                        self.state = PpuState::VBlank;
                    }
                }
            }
            PpuState::VBlank => {
                if self.ticks >= TICKS_ONE_LINE {
                    self.ticks -= TICKS_ONE_LINE;
                    self.incr_curr_scanline(mmu);

                    if !(self.get_curr_scanline(mmu) < SCREEN_HEIGHT_PIXELS + VBLANK_LINES) {
                        self.state = PpuState::OAMSearch;
                        self.set_curr_scanline(mmu, 0);
                    }
                }
            }
        }
    }

    fn renderscan(&mut self, mmu:&MMU) {
        let py = self.get_curr_scanline(mmu);
        let row_data: [ColourTileRow; SCREEN_WIDTH_TILES as usize] 
            = (0..SCREEN_WIDTH_TILES)
            .map(|tx| self.get_bg_tile_row(mmu, TILE_WIDTH_PIXELS*tx, py))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let row_data: ColourScreenRow = unsafe { std::mem::transmute(row_data) }; 
        self.update_row(py, row_data);
    }

    fn get_bg_tile_row(&self, mmu: &MMU, px: u8, py: u8) -> ColourTileRow {
        let (spx, spy) = self.adjust_viewport_scroll(mmu, px, py);
        let tra = self.get_tile_row_data_addr(mmu, spx, spy);
        self.get_tile_row_data(mmu, tra)
    }

    fn adjust_viewport_scroll(&self, mmu: &MMU, px: u8, py: u8) -> (u8, u8) {
        // adjusting for viewport offsets
        let px = px + self.get_scroll_x(mmu);
        let py = py + self.get_scroll_y(mmu);
        (px, py)
    }

    fn get_tile_data_addr(&self, mmu: &MMU, spx: u8, spy: u8) -> Addr {
        // get tile number
        let ty = spy >> 3; // py / 8
        let tx = spx >> 3; // px / 8
        // type conversion for individual variables instead of type conversion at the end - in order to avoid overflow
        let to = (ty as u16) * TILEMAP_WIDTH_TILES + tx as u16; // 32 = Number of tiles in a row = 256(number of pixels per row)/8(number of pixels per tile row)

        // get tile index from the right tile map
        let map: Addr = self.get_lcdc(mmu).bg_tile_map.into();
        let ti = mmu.readu8(map + to.into());

        // get the address where tile data is stored 
        let ta = self.get_lcdc(mmu).bg_window_tile_data.get_tile_data_addr(ti);
        ta
    }

    fn get_tile_row_data_addr(&self, mmu: &MMU, spx: u8, spy: u8) -> Addr {
        let ta = self.get_tile_data_addr(mmu, spx, spy);

        // get the address where the row data is stored  
        let ro = 2 * (spy % TILE_HEIGHT_PIXELS) as u16; // 2 bytes per row * (py % 8)
        let tra = ta + ro.into();
        tra
    }

    fn get_tile_row_data(&self, mmu: &MMU, tra: Addr) -> ColourTileRow {
        // read the row colour data
        let lb = mmu.readu8(tra);
        let hb = mmu.readu8(tra + 1.into());
        (0..TILE_WIDTH_PIXELS)
        .rev()
        .map(|n| 
            Colour::try_from(
                ((get_nth_bit(hb, n) as u8) << 1) 
                | get_nth_bit(lb, n) as u8
            ).unwrap()
        ).collect::<Vec<Colour>>()
        .try_into()
        .unwrap()
    }

    fn update_row(&mut self, row: u8, row_data: ColourScreenRow) {
        for (i, c) in row_data.iter().enumerate() {
            self.screen.set(row, i as u8, *c);
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
    fn set_curr_scanline(&self, mmu: &mut MMU, value: u8) {
        mmu.writeu8(REG_CURR_SCANLINE, value)
    }
    fn incr_curr_scanline(&self, mmu: &mut MMU) {
        let curr_scanline = self.get_curr_scanline(mmu);
        self.set_curr_scanline(mmu, curr_scanline + 1);
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

impl From<TileMap> for Addr {
    fn from(value: TileMap) -> Self {
        match value {
            TileMap::Low  => 0x9800.into(),
            TileMap::High => 0x9c00.into(),
        }
    }
}

enum TileData {
    Low,  // 8000-8FFF, 0 - 255
    High, // 8800-97FF, -128 - 127
}

impl TileData {
    fn get_tile_data_addr(&self, tile_index: u8) -> Addr {
        match self {
            Self::Low  => {
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
            TileData::Low  => true, 
            TileData::High => false,
        }
    }
}

impl From<bool> for TileData {
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
    bg_window_tile_data: TileData,
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
        | <TileMap as Into<bool>>::into(value.window_tile_map) as u8) << 1
        | value.window_enable as u8) << 1
        | <TileData as Into<bool>>::into(value.bg_window_tile_data) as u8) << 1
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

impl Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Colour::White     => '█',
            Colour::LightGrey => '▓',
            Colour::DarkGrey  => '▒',
            Colour::Black     => '░',
        };
        write!(f, "{}", c)
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