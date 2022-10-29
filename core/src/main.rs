use machine::Machine;

fn main() {
    let cartridge = "/home/govardhan/projects/emulators/gb/roms/dmg_boot.bin";
    let mut machine = Machine::new(cartridge).unwrap();
    machine.run();
}
