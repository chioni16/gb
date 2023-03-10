use super::busio::{BusIO, SResult};
use crate::util::Addr;

#[derive(Debug)]
pub(super) struct Region {
    pub(super) device: Box<dyn BusIO + 'static>,
    pub(super) start: Addr,
    pub(super) end: Addr,
    pub(super) remap: bool,
}

impl Region {
    pub(super) fn new(device: impl BusIO + 'static, start: Addr, end: Addr, remap: bool) -> Self {
        Self {
            device: Box::new(device),
            start,
            end,
            remap,
        }
    }

    pub(super) fn remap_addr(&self, addr: Addr) -> Addr {
        if self.remap {
            addr - self.start
        } else {
            addr
        }
    }
    pub(super) fn readu8(&self, addr: Addr) -> SResult<u8> {
        let addr = self.remap_addr(addr);
        self.device.readu8(addr)
    }
    pub(super) fn readu16(&self, addr: Addr) -> SResult<u16> {
        let addr = self.remap_addr(addr);
        self.device.readu16(addr)
    }
    pub(super) fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        let addr = self.remap_addr(addr);
        self.device.writeu8(addr, value)
    }
    pub(super) fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        let addr = self.remap_addr(addr);
        self.device.writeu16(addr, value)
    }
}
