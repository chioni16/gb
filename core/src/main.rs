use machine::Machine;

use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    let bootrom = Some("/Users/ggd/projects/gb/roms/dmg_boot.bin");
    let cartridge = "/Users/ggd/projects/gb/roms/dmg_boot.bin";
    let mut m = Machine::new(cartridge, bootrom).unwrap();
    
    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X4,
            ..WindowOptions::default()
        }
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut i= 0usize;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        m.step();

        println!("{}", i);
        i = i.wrapping_add(1);

        if i % 10000 == 0 {
            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window
                .update_with_buffer(unsafe {&machine::ppu::screen_u32 } , WIDTH, HEIGHT)
                .unwrap();
        }
    }
    // loop {
    //     m.step();
        
    //     println!("{}", i);
    //     i += 1;
    // }

}
