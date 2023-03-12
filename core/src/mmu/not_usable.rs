use super::busio::{BusIO, SResult};
use crate::util::Addr;

#[derive(Debug)]
pub(super) struct NotUsableHigh;
impl BusIO for NotUsableHigh {
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

    fn as_slice(&self, _addr: Addr, _len: usize) -> SResult<&[u8]> {
        Err(Box::from("u8 doesn't support as_slice"))
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}

#[derive(Debug)]
pub(super) struct NotUsableLow;
impl BusIO for NotUsableLow {
    fn readu8(&self, _addr: Addr) -> SResult<u8> {
        Ok(0x00)
    }

    fn writeu8(&mut self, _addr: Addr, _value: u8) -> SResult<()> {
        Ok(())
    }

    fn writeu16(&mut self, _addr: Addr, _value: u16) -> SResult<()> {
        Ok(())
    }

    fn readu16(&self, _addr: Addr) -> SResult<u16> {
        Ok(0x0000)
    }

    fn as_slice(&self, _addr: Addr, _len: usize) -> SResult<&[u8]> {
        Err(Box::from("u8 doesn't support as_slice"))
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}