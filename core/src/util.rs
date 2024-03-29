use derive_more::{Add, AddAssign, Display, From, Into, Sub, SubAssign};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    From,
    Into,
    Display,
)] // derive_more
pub(crate) struct Addr(pub u16);
impl Addr {
    pub(crate) const fn new() -> Self {
        Self(0)
    }

    pub(crate) const fn from(value: u16) -> Self {
        Self(value)
    }
}

pub(crate) fn pause() {
    use std::io;
    use std::io::prelude::*;

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

pub(crate) fn get_nth_bit(value: u8, n: u8) -> bool {
    assert!(n < 8);
    match (value >> n) & 1 {
        0 => false,
        1 => true,
        _ => unreachable!(),
    }
}