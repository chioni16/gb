use machine::Machine;

fn main() {
    let cartridge = "/home/govardhan/projects/emulators/gb/roms/dmg_boot.bin";
    // let cartridge = "/home/govardhan/projects/emulators/gb/roms/bgbtest.gb";
    let mut machine = Machine::new(cartridge).unwrap();
    machine.run();
}
