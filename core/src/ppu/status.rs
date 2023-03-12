use super::get_nth_bit;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum PpuMode {
    OAMSearch = 2,
    PixelTransfer = 3,
    HBlank = 0,
    VBlank = 1,
}

impl TryFrom<u8> for PpuMode {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(PpuMode::OAMSearch),
            3 => Ok(PpuMode::PixelTransfer),
            0 => Ok(PpuMode::HBlank),
            1 => Ok(PpuMode::VBlank),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Status {
    pub(crate) lyc_int: bool,
    pub(crate) oam_int: bool,
    pub(crate) vblank_int: bool,
    pub(crate) hblank_int: bool,
    pub(crate) lyc_equal: bool,
    pub(crate) mode: PpuMode,
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Self {
            lyc_int: get_nth_bit(value, 6),
            oam_int: get_nth_bit(value, 5),
            vblank_int: get_nth_bit(value, 4),
            hblank_int: get_nth_bit(value, 3),
            lyc_equal: get_nth_bit(value, 2),
            mode: PpuMode::try_from(value & 0b11).unwrap(),
        }
    }
}

impl From<Status> for u8 {
    fn from(value: Status) -> Self {
        (value.lyc_int as u8) << 6
        | (value.oam_int as u8) << 5
        | (value.vblank_int as u8) << 4
        | (value.hblank_int as u8) << 3
        | (value.lyc_equal as u8) << 2
        | (value.mode as u8)
    }
}

impl Default for Status {
    fn default() -> Self {
        Self {
            lyc_int: false,
            oam_int: false,
            vblank_int: false,
            hblank_int: false,
            lyc_equal: false,
            mode: PpuMode::OAMSearch,
        }
    }
}
