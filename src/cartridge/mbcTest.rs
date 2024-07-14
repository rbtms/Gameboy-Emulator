use crate::{cartridge::Cartridge, consts::ComponentWithMemory};

/*
 Type of cartridge which is just an array for tests
 */

pub struct MBCTest {
    file                : String,
    rom                 : Vec<u8>
}

impl MBCTest {
    pub fn new(file :&str, rom :Vec<u8>) -> MBCTest {
        return MBCTest {
            file: file.to_string(),
            rom: rom
        }
    }
}

impl Cartridge for MBCTest {
    fn init(&mut self) {
        // TODO: Disable on debug
        self.print_rom_data();
    }

    fn is_test_cart(&self) -> bool { return true; }

    fn load_ram(&mut self) {}
    fn save_ram(&self) {}

    fn print_rom_data(&self) {
        println!("\nFile:\n{}", self.file);
        println!();
        println!("MBCTest");
        println!();
        println!("--------------------------------------\n");
    }
}

impl ComponentWithMemory for MBCTest {
    fn read(&self, addr :u16) -> u8 {
        return self.rom[addr as usize];
    }

    fn write(&mut self, addr :u16, val :u8) {
        self.rom[addr as usize] = val;
    }

}
