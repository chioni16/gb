use crate::util::Addr;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub(crate) type SResult<T> = Result<T, Box<dyn std::error::Error>>;

// common interface to read and write from/to addresses
pub(crate) trait BusIO: Debug {
    fn readu8(&self, addr: Addr) -> SResult<u8>;
    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()>;

    fn readu16(&self, addr: Addr) -> SResult<u16>;
    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()>;

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]>;

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

    fn as_slice(&self, _addr: Addr, _len: usize) -> SResult<&[u8]> {
        Err(Box::from("u8 doesn't support as_slice"))
    }

    fn print_dbg(&self, _start: Addr, _len: u16) -> String {
        unimplemented!()
    }
}

impl<T: BusIO> BusIO for std::cell::Ref<'_, T> {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        self.deref().readu8(addr)
    }

    fn writeu8(&mut self, _addr: Addr, _value: u8) -> SResult<()> {
        unreachable!()
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        self.deref().readu16(addr)
    }

    fn writeu16(&mut self, _addr: Addr, _value: u16) -> SResult<()> {
        unreachable!()
    }

    fn print_dbg(&self, start: Addr, len: u16) -> String {
        self.deref().print_dbg(start, len)
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        self.deref().as_slice(addr, len)
    }
}

impl<T: BusIO> BusIO for std::cell::RefMut<'_, T> {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        self.deref().readu8(addr)
    }

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        self.deref_mut().writeu8(addr, value)
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        self.deref().readu16(addr)
    }

    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        self.deref_mut().writeu16(addr, value)
    }

    fn print_dbg(&self, start: Addr, len: u16) -> String {
        self.deref().print_dbg(start, len)
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        self.deref().as_slice(addr, len)
    }
}


impl<T: BusIO> BusIO for Box<T> {
    fn readu8(&self, addr: Addr) -> SResult<u8> {
        self.deref().readu8(addr)
    }

    fn writeu8(&mut self, addr: Addr, value: u8) -> SResult<()> {
        self.deref_mut().writeu8(addr, value)
    }

    fn readu16(&self, addr: Addr) -> SResult<u16> {
        self.deref().readu16(addr)
    }

    fn writeu16(&mut self, addr: Addr, value: u16) -> SResult<()> {
        self.deref_mut().writeu16(addr, value)
    }

    fn print_dbg(&self, start: Addr, len: u16) -> String {
        self.deref().print_dbg(start, len)
    }

    fn as_slice(&self, addr: Addr, len: usize) -> SResult<&[u8]> {
        self.deref().as_slice(addr, len)
    }
}
