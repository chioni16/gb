use crate::util::get_nth_bit;

// http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-Input

pub(crate) const REG_JOYPAD: u16 = 0xff00;

#[derive(Debug, Clone, Copy)]
pub struct Joypad {
    // false -> on, true -> off

    select_button: bool,
    select_dir: bool,

    b3: bool,
    b2: bool,
    b1: bool,
    b0: bool,

    pub(crate) interrupt: bool,
}

impl Default for Joypad {
    fn default() -> Self {
        Self { 
            // key_type: KeyType::Direction, 
            select_button: true,
            select_dir: true,

            b3: true, 
            b2: true, 
            b1: true, 
            b0: true,
            interrupt: false,
        }  
    }
}

impl Joypad {
    pub fn reset_all(&mut self) {
        self.b3 = true;
        self.b2 = true;
        self.b1 = true;
        self.b0 = true;
    }

    pub fn set(&mut self, key: Key) {
        match key {
            Key::Down  => if self.is_dir() {self.b3 = false; self.interrupt = true;},
            Key::Up    => if self.is_dir() {self.b2 = false; self.interrupt = true;},
            Key::Left  => if self.is_dir() {self.b1 = false; self.interrupt = true;},
            Key::Right => if self.is_dir() {self.b0 = false; self.interrupt = true;},

            Key::Start   => if self.is_button() {self.b3 = false; self.interrupt = true;},
            Key::Select  => if self.is_button() {self.b2 = false; self.interrupt = true;},
            Key::ButtonB => if self.is_button() {self.b1 = false; self.interrupt = true;},
            Key::ButtonA => if self.is_button() {self.b0 = false; self.interrupt = true;},
        }
    }

    fn is_dir(&self) -> bool {
        !self.select_dir
    }

    fn is_button(&self) -> bool {
        !self.select_button
    }

    pub(crate) fn write_reg(&mut self, value: u8) {
        self.select_button = get_nth_bit(value, 5);
        self.select_dir = get_nth_bit(value, 4);
    }

    pub(crate) fn read(&self) -> u8 {
        0 << 7
        | 0 << 6
        | (!self.is_button() as u8) << 5
        | (!self.is_dir() as u8) << 4
        | (self.b3 as u8) << 3
        | (self.b2 as u8) << 2
        | (self.b1 as u8) << 1
        | (self.b0 as u8)
    } 


}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    // direction keys
    Down,
    Up,
    Left,
    Right,

    // buttons
    Start, 
    Select,
    ButtonA, 
    ButtonB,
}