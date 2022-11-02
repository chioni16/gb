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
