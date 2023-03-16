use machine::{Machine, joypad::{Key as JKey, Joypad}};

use minifb::{Key as MKey, Scale, Window, WindowOptions, KeyRepeat};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    let bootrom: Option<&str> = None;
    // let bootrom = Some("/Users/ggd/projects/gb/roms/dmg_boot.bin");
    let cartridge = "/Users/ggd/projects/gb/roms/Tetris.gb";
    // let cartridge = "/Users/ggd/projects/gb/roms/dmg-acid2.gb";
    // let cartridge = "/Users/ggd/projects/gb/gb-test-roms/cpu_instrs/individual/02-interrupts.gb";
    let mut m = Machine::new(cartridge, bootrom).unwrap();

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut i = 0usize;

    while window.is_open() && !window.is_key_down(MKey::Escape) {
        m.step();
        handle_key_press(&window, &mut m.mmu.joypad);

        // println!("{}", i);
        i = i.wrapping_add(1);

        if i % 10000 == 0 {
            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window
                .update_with_buffer(unsafe { &machine::ppu::screen_u32 }, WIDTH, HEIGHT)
                .unwrap();
        }
    }
}

fn handle_key_press(window: &Window, joypad: &mut Joypad) {
        joypad.reset_all();
        if window.is_key_down(MKey::Up) {
            println!("UP");
            joypad.set(JKey::Up);
        }
        if window.is_key_down(MKey::Left) {
            println!("LEFT");
            joypad.set(JKey::Left);
        }
        if window.is_key_down(MKey::Down) {
            println!("DOWN");
            joypad.set(JKey::Down);
        }
        if window.is_key_down(MKey::Right) {
            println!("RIGHT");
            joypad.set(JKey::Right);
        }
        if window.is_key_down(MKey::Enter) {
            println!("START");
            joypad.set(JKey::Start);
        }
        if window.is_key_down(MKey::Space) {
            println!("SELECT");
            joypad.set(JKey::Select);
        }
        if window.is_key_down(MKey::Z) {
            println!("A");
            joypad.set(JKey::ButtonA);
        }
        if window.is_key_down(MKey::X) {
            println!("B");
            joypad.set(JKey::ButtonB);
        }
}

// fn handle_key_press(window: &Window, joypad: &mut Joypad) {
//         if window.is_key_pressed(MKey::Up, KeyRepeat::Yes) {
//             println!("UP");
//             joypad.set(JKey::Up);
//         }
//         if window.is_key_pressed(MKey::Left, KeyRepeat::Yes) {
//             println!("LEFT");
//             joypad.set(JKey::Left);
//         }
//         if window.is_key_pressed(MKey::Down, KeyRepeat::Yes) {
//             println!("DOWN");
//             joypad.set(JKey::Down);
//         }
//         if window.is_key_pressed(MKey::Right, KeyRepeat::Yes) {
//             println!("RIGHT");
//             joypad.set(JKey::Right);
//         }
//         if window.is_key_pressed(MKey::Enter, KeyRepeat::Yes) {
//             println!("START");
//             joypad.set(JKey::Start);
//         }
//         if window.is_key_pressed(MKey::Space, KeyRepeat::Yes) {
//             println!("SELECT");
//             joypad.set(JKey::Select);
//         }
//         if window.is_key_pressed(MKey::Z, KeyRepeat::Yes) {
//             println!("A");
//             joypad.set(JKey::ButtonA);
//         }
//         if window.is_key_pressed(MKey::X, KeyRepeat::Yes) {
//             println!("B");
//             joypad.set(JKey::ButtonB);
//         }
// }