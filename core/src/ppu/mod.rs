mod colour;
pub(crate) mod lcdc;
pub(crate) mod palette;
mod screen;
pub(crate) mod status;

use crate::{mmu::{ram::RAM, busio::BusIO}, util::Addr};
use colour::Colour;
use lcdc::LCDC;
use palette::Palette;
pub use screen::screen_u32;
use screen::Screen;
use status::{PpuMode, Status};

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

    pub(crate) bgp: Palette,
    pub(crate) obp0: Palette,
    pub(crate) obp1: Palette,

    pub(crate) vblank_interrupt: bool,

    pub(crate) vram: RAM,
    pub(crate) oam: RAM,

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
            oam: RAM::new(0xa0, Box::new(|addr: Addr| addr - 0xfe00.into()), 0),
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
        self.oam.copy_from_slice(src);
    }

    fn renderscan(&mut self) {
        let py = self.get_curr_scanline();
        let row_data: [ColourTileRow; SCREEN_WIDTH_TILES as usize] = (0..SCREEN_WIDTH_TILES)
            .map(|tx| self.get_bg_tile_row(TILE_WIDTH_PIXELS * tx, py))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let row_data: ColourScreenRow = unsafe { std::mem::transmute(row_data) };
        self.update_row(py, row_data);
    }

    fn get_bg_tile_row(&self, px: u8, py: u8) -> ColourTileRow {
        let (spx, spy) = self.adjust_viewport_scroll(px, py);
        let tra = self.get_tile_row_data_addr(spx, spy);
        self.get_tile_row_data(tra)
    }

    fn adjust_viewport_scroll(&self, px: u8, py: u8) -> (u8, u8) {
        // adjusting for viewport offsets
        let px = px + self.get_scroll_x();
        let py = py + self.get_scroll_y();
        (px, py)
    }

    fn get_tile_data_addr(&self, spx: u8, spy: u8) -> Addr {
        // get tile number
        let ty = spy >> 3; // py / 8
        let tx = spx >> 3; // px / 8
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
        let ro = 2 * (spy % TILE_HEIGHT_PIXELS) as u16; // 2 bytes per row * (py % 8)
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
                Colour::try_from(((get_nth_bit(hb, n) as u8) << 1) | get_nth_bit(lb, n) as u8)
                    .unwrap()
            })
            .collect::<Vec<Colour>>()
            .try_into()
            .unwrap()
    }

    fn update_row(&mut self, row: u8, row_data: ColourScreenRow) {
        for (i, c) in row_data.iter().enumerate() {
            self.screen.set(row, i as u8, *c);
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
    fn set_bg_palette(&mut self, palette: Palette) {
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
