use super::busio::{BusIO, SResult};
use crate::util::Addr;

#[derive(Debug)]
pub(super) struct NotUsable;
impl BusIO for NotUsable {
    fn readu8(&self, _addr: Addr) -> SResult<u8> {
        Ok(0xff)
    }

    fn writeu8(&mut self, _addr: Addr, _value: u8) -> SResult<()> {
        Ok(())
    }

    fn writeu16(&mut self, _addr: Addr, _value: u16) -> SResult<()> {
        Ok(())
    }

    fn readu16(&self, _addr: Addr) -> SResult<u16> {
        Ok(0xffff)
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
       unimplemented!() 
    }
}