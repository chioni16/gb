mod colour;
pub(crate) mod lcdc;
mod oam;
pub(crate) mod palette;
mod screen;
pub(crate) mod status;

use crate::{mmu::{ram::RAM, busio::BusIO}, util::Addr};
use colour::Colour;
use lcdc::LCDC;
use oam::{OAM, Sprite, SpriteAttr, ObjPaletteType};  
use palette::{BgWinPalette, ObjPalette};
pub use screen::screen_u32;
use screen::Screen;
use status::{PpuMode, Status};

use self::lcdc::TileData;

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

const TILE_WIDTH_PIXELS: u8 = 8;
const TILE_HEIGHT_PIXELS: u8 = 8;

const SCREEN_HEIGHT_PIXELS: u8 = 144;
const SCREEN_WIDTH_PIXELS: u8 = 160;
const SCREEN_HEIGHT_TILES: u8 = SCREEN_HEIGHT_PIXELS / TILE_HEIGHT_PIXELS;
const SCREEN_WIDTH_TILES: u8 = SCREEN_WIDTH_PIXELS / TILE_WIDTH_PIXELS;

const TILEMAP_HEIGHT_PIXELS: u16 = 256;
const TILEMAP_WIDTH_PIXELS: u16 = 256;
const TILEMAP_HEIGHT_TILES: u16 = TILEMAP_HEIGHT_PIXELS / TILE_HEIGHT_PIXELS as u16;
const TILEMAP_WIDTH_TILES: u16 = TILEMAP_WIDTH_PIXELS / TILE_WIDTH_PIXELS as u16;

// pseudo-scanlines
const VBLANK_LINES: u8 = 10;

const TICKS_OAM_SEARCH: u64 = 80;
const TICKS_PIXEL_TRANSFER: u64 = 172;
const TICKS_HBLANK: u64 = 204;
const TICKS_ONE_LINE: u64 = TICKS_OAM_SEARCH + TICKS_PIXEL_TRANSFER + TICKS_HBLANK;
const TICKS_VBLANK: u64 = VBLANK_LINES as u64 * TICKS_ONE_LINE;
const TICKS_ONE_FRAME: u64 = SCREEN_HEIGHT_PIXELS as u64 * TICKS_ONE_LINE + TICKS_VBLANK;

const OAM_SIZE: usize = 0xa0;
const MAX_SPRITES_PER_ROW: usize = 10;

type ColourScreenRow = [Colour; SCREEN_WIDTH_PIXELS as usize];
type ColourTileRow = [Colour; TILE_WIDTH_PIXELS as usize];


#[derive(Debug)]
pub struct PPU {
    ticks: u64,
    pub screen: Screen,

    // registers
    pub(crate) lcdc: LCDC,
    pub(crate) status: Status,

    pub(crate) scx: u8,
    pub(crate) scy: u8,
    pub(crate) curr_scanline: u8,

    pub(crate) wy: u8,
    pub(crate) wx: u8,

    pub(crate) bgp: BgWinPalette,
    pub(crate) obp0: ObjPalette,
    pub(crate) obp1: ObjPalette,

    pub(crate) vblank_interrupt: bool,

    pub(crate) vram: RAM,
    pub(crate) oam: OAM,

}

impl PPU {
    pub(crate) fn new() -> Self {
        let ppu = Self {
            ticks: 0,
            screen: Screen::new(),

            lcdc: Default::default(),
            status: Default::default(),

            scx: 0,
            scy: 0,
            curr_scanline: 0,
            
            wx: 0,
            wy: 0,

            bgp: Default::default(),
            obp0: Default::default(),
            obp1: Default::default(),

            vblank_interrupt: false,

            vram: RAM::new(8 * 1024, Box::new(|addr: Addr| addr - 0x8000.into()), 0xda),
            oam: OAM(RAM::new(OAM_SIZE, Box::new(|addr: Addr| addr - 0xfe00.into()), 0)),
        };
        ppu
    }

    pub(crate) fn tick(&mut self, cpu_ticks: u64) {
        self.ticks += cpu_ticks;

        match self.status.mode {
            PpuMode::OAMSearch => {
                if self.ticks >= TICKS_OAM_SEARCH {
                    self.ticks -= TICKS_OAM_SEARCH;
                    self.status.mode = PpuMode::PixelTransfer;
                }
            }
            PpuMode::PixelTransfer => {
                if self.ticks >= TICKS_PIXEL_TRANSFER {
                    self.ticks -= TICKS_PIXEL_TRANSFER;
                    self.status.mode = PpuMode::HBlank;

                    self.renderscan();
                }
            }
            PpuMode::HBlank => {
                if self.ticks >= TICKS_HBLANK {
                    self.ticks -= TICKS_HBLANK;
                    self.incr_curr_scanline();

                    if self.get_curr_scanline() < SCREEN_HEIGHT_PIXELS {
                        self.status.mode = PpuMode::OAMSearch;
                    } else {
                        // println!("{}", self.screen);
                        // println!("{:?}", self.oam.into_iter().collect::<Vec<_>>());
                        // println!("{:?}", self.lcdc);
                        self.status.mode = PpuMode::VBlank;
                    }
                }
            }
            PpuMode::VBlank => {
                self.vblank_interrupt = true;

                if self.ticks >= TICKS_ONE_LINE {
                    self.ticks -= TICKS_ONE_LINE;
                    self.incr_curr_scanline();

                    if !(self.get_curr_scanline() < SCREEN_HEIGHT_PIXELS + VBLANK_LINES) {
                        self.status.mode = PpuMode::OAMSearch;
                        self.set_curr_scanline(0);
                    }
                }
            }
        }
    }

    pub(crate) fn dma(&mut self, src: &[u8]) {
        self.oam.0.copy_from_slice(src);
    }

    fn renderscan(&mut self) {
        let py = self.get_curr_scanline();

        let bg_win_data: ColourScreenRow = if self.lcdc.bg_window_enable {
            let bg_win_data: [ColourTileRow; SCREEN_WIDTH_TILES as usize] = (0..SCREEN_WIDTH_TILES)
                .map(|tx| self.get_bg_tile_row(TILE_WIDTH_PIXELS * tx, py))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            unsafe { std::mem::transmute(bg_win_data) }
        } else {
            [Colour::White; SCREEN_WIDTH_PIXELS as usize]
        };

        let row_data: Box<dyn Iterator<Item = Colour>> = if self.lcdc.object_enable {
            let sprite_data: ColourScreenRow = self.get_sprite_tile_data(py);
            Box::from(sprite_data.into_iter().zip(bg_win_data.into_iter()).map(|(s, bw)| s | bw))
        } else {
            Box::from(bg_win_data.into_iter())
        };

        self.update_row(py, row_data);
    }

    fn get_sprite_tile_data(&mut self, py: u8) -> ColourScreenRow {
        let mut row: ColourScreenRow = [Colour::Transparent; SCREEN_WIDTH_PIXELS as usize];
        for s in self.oam.row(py, self.lcdc.object_size) {
            let mut distance_from_top_of_sprite = py - (s.y_pos - 16);
            if s.attr.y_flip {
                let sprite_height = self.lcdc.object_size as u8;
                distance_from_top_of_sprite = sprite_height - distance_from_top_of_sprite;
            }
            
            let ta = TileData::Low.get_tile_data_addr(s.ti | (distance_from_top_of_sprite > TILE_HEIGHT_PIXELS) as u8);
            
            let ro = 2 * (distance_from_top_of_sprite % TILE_HEIGHT_PIXELS) as u16; // 2 bytes per row * (spy % 8)
            let tra = ta + ro.into();

            let lb = self.read_vram(tra);
            let hb = self.read_vram(tra + 1.into());
            (0..TILE_WIDTH_PIXELS)
                .rev()
                .map(|n| self.get_obj_palette(&s).map(((get_nth_bit(hb, n) as u8) << 1) | get_nth_bit(lb, n) as u8).unwrap())
                .enumerate()
                .for_each(|(i, c)| {
                    let i = if s.attr.x_flip { TILE_WIDTH_PIXELS as usize - i } else { i };
                    if (s.x_pos + i as u8) > 8 && (s.x_pos + i as u8) < (SCREEN_WIDTH_PIXELS + 8) { 
                        row[s.x_pos as usize + i - 8] = c;
                    }
                });
        };
        row
    }

    fn get_obj_palette(&self, s: &Sprite) -> &ObjPalette {
        match s.attr.palette {
            ObjPaletteType::OBP0 => &self.obp0,
            ObjPaletteType::OBP1 => &self.obp1,
        }
    }

    fn get_bg_tile_row(&self, px: u8, py: u8) -> ColourTileRow {
        let (spx, spy) = self.adjust_viewport_scroll(px, py);
        let tra = self.get_tile_row_data_addr(spx, spy);
        self.get_tile_row_data(tra)
    }

    fn adjust_viewport_scroll(&self, px: u8, py: u8) -> (u8, u8) {
        // adjusting for viewport offsets
        let spx = px + self.get_scroll_x();
        let spy = py + self.get_scroll_y();
        (spx, spy)
    }

    fn get_tile_data_addr(&self, spx: u8, spy: u8) -> Addr {
        // get tile number
        let ty = spy >> 3; // spy / 8
        let tx = spx >> 3; // spx / 8
                           // type conversion for individual variables instead of type conversion at the end - in order to avoid overflow
        let to = (ty as u16) * TILEMAP_WIDTH_TILES + tx as u16; // 32 = Number of tiles in a row = 256(number of pixels per row)/8(number of pixels per tile row)

        // get tile index from the right tile map
        let map: Addr = self.get_lcdc().bg_tile_map.into();
        let ti = self.read_vram(map + to.into());

        // get the address where tile data is stored
        let ta = self.get_lcdc().bg_window_tile_data.get_tile_data_addr(ti);
        ta
    }

    fn get_tile_row_data_addr(&self, spx: u8, spy: u8) -> Addr {
        let ta = self.get_tile_data_addr(spx, spy);

        // get the address where the row data is stored
        let ro = 2 * (spy % TILE_HEIGHT_PIXELS) as u16; // 2 bytes per row * (spy % 8)
        let tra = ta + ro.into();
        tra
    }

    fn get_tile_row_data(&self, tra: Addr) -> ColourTileRow {
        // read the row colour data
        let lb = self.read_vram(tra);
        let hb = self.read_vram(tra + 1.into());
        (0..TILE_WIDTH_PIXELS)
            .rev()
            .map(|n| {
                self.bgp.map(((get_nth_bit(hb, n) as u8) << 1) | get_nth_bit(lb, n) as u8)
                    .unwrap()
            })
            .collect::<Vec<Colour>>()
            .try_into()
            .unwrap()
    }

    // fn update_row<T: Iterator<Item = Colour>>(&mut self, row: u8, row_data: T) {
    fn update_row(&mut self, row: u8, row_data: Box<dyn Iterator<Item = Colour>>) {
        let mut i = 0;
        for c in row_data {
            self.screen.set(row, i as u8, c);
            i += 1;
        }
    }

    fn read_vram(&self, addr: Addr) -> u8 {
        self.vram.readu8(addr).unwrap()
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

pub(crate) const REG_LCDC: u16 = 0xff40;
pub(crate) const REG_STAT: u16 = 0xff41;
pub(crate) const REG_SCROLL_Y: u16 = 0xff42;
pub(crate) const REG_SCROLL_X: u16 = 0xff43;
pub(crate) const REG_WIN_Y: u16 = 0xff4a;
pub(crate) const REG_WIN_X: u16 = 0xff4b;
pub(crate) const REG_CURR_SCANLINE: u16 = 0xff44;
pub(crate) const REG_BG_PALETTE: u16 = 0xff47;
pub(crate) const REG_OBJ_PALETTE_0: u16 = 0xff48;
pub(crate) const REG_OBJ_PALETTE_1: u16 = 0xff49;

impl PPU {
    fn get_lcdc(&self) -> LCDC {
        self.lcdc
    }
    fn set_lcdc(&mut self, lcdc: LCDC) {
        self.lcdc = lcdc;
    }
    fn get_scroll_y(&self) -> u8 {
        self.scy
    }
    fn set_scroll_y(&mut self, value: u8) {
        self.scy = value;
    }
    fn get_scroll_x(&self) -> u8 {
        self.scx
    }
    fn set_scroll_x(&mut self, value: u8) {
        self.scx = value;
    }
    fn get_curr_scanline(&self) -> u8 {
        self.curr_scanline
    }
    fn set_curr_scanline(&mut self, value: u8) {
        self.curr_scanline = value;
    }
    fn incr_curr_scanline(&mut self) {
        self.curr_scanline += 1;
    }
    fn set_bg_palette(&mut self, palette: BgWinPalette) {
        self.bgp = palette;
    }
}

fn get_nth_bit(value: u8, n: u8) -> bool {
    assert!(n < 8);
    match (value >> n) & 1 {
        0 => false,
        1 => true,
        _ => unreachable!(),
    }
}
