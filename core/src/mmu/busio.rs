
use std::fmt::Debug;
use crate::util::Addr;

pub(crate) type SResult<T> = Result<T, Box<dyn std::error::Error>>;

// common interface to read and write from/to addresses
pub(crate) trait BusIO: Debug {
    fn readu8(&self, addr: Addr) -> SResult<u8>;
    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()>;

    fn readu16(&self, addr: Addr) -> SResult<u16>;
    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()>;

    fn print_dbg(&self, start: Addr, len: u16) -> String;
}

impl BusIO for u8 {
    fn readu8(&self, _addr: Addr) -> SResult<u8> {
        Ok(*self)
    }

    fn writeu8(&mut self, _addr: Addr, value: u8) -> SResult<()> {
        *self = value;
        Ok(())
    }

    fn writeu16(&mut self, _addr: Addr, _value: u16) -> SResult<()> {
        unimplemented!()
    }

    fn readu16(&self, _addr: Addr) -> SResult<u16> {
        unimplemented!()        
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
       unimplemented!() 
    }
}
