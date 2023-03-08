use eframe::egui;
use machine::Machine;

fn main() -> Result<(), eframe::Error> {
    // let cartridge = "/home/govardhan/projects/emulators/gb/roms/dmg_boot.bin";
    // let cartridge = "/home/govardhan/projects/emulators/gb/gb-test-roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb";
    let cartridge = "/home/govardhan/projects/emulators/gb/gb-test-roms/cpu_instrs/individual/02-interrupts.gb";
    // let cartridge = "/home/govardhan/projects/emulators/gb/gb-test-roms/cpu_instrs/individual/01-special.gb";
    // let cartridge = "/home/govardhan/projects/emulators/gb/gb-test-roms/cpu_instrs/individual/08-misc instrs.gb";
    // let cartridge = "/home/govardhan/projects/emulators/gb/roms/bgbtest.gb";
    let mut machine = Machine::new(cartridge).unwrap();
    // machine.run();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(160.0, 144.0)),
        ..Default::default()
    };
    eframe::run_native(
        "GB",
        options,
        Box::new(|_cc| Box::new(machine)),
    )
}
