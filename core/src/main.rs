use machine::Machine;

fn main() {
    // let cartridge = "/home/govardhan/projects/emulators/gb/roms/dmg_boot.bin";
    let cartridge = "/home/govardhan/projects/emulators/gb/gb-test-roms/cpu_instrs/individual/09-op r,r.gb";
    // let cartridge = "/home/govardhan/projects/emulators/gb/roms/bgbtest.gb";
    let mut machine = Machine::new(cartridge).unwrap();
    machine.run();
}
