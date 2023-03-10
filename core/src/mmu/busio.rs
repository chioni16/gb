
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
