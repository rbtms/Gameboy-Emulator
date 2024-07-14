use crate::consts::*;
use crate::cartridge::{CartridgeType, Cartridge};
use std::io::prelude::Write;


const SAVE_PATH :&str = "roms/games/saves";


pub struct NoMBC {
    file                : String,
    rom                 : Vec<u8>,
    ext_ram             : Vec<u8>,
    cgb_flag            : bool,
    sgb_flag            : bool,
    cartridge_type      : CartridgeType,
    rom_size            : u16,
    rom_bank_n          : u16,
    ram_size            : u16,
    ram_bank_n          : u16,
    mask_rom_version_n  : u8,
    header_checksum     : u8,
    global_checksum     : u16,
}

impl NoMBC {
    pub fn new(file :&str, rom :Vec<u8>) -> NoMBC {
        let cartridge_type :CartridgeType = rom[0x147].into();
        let ram_size = if cartridge_type.has_ram() { [0, 0, 8, 32, 128, 64][rom[0x149] as usize] } else {0};

        return NoMBC {
            file                : file.to_string(),
            cgb_flag            : rom[0x143] == 0xC0,
            sgb_flag            : rom[0x146] == 0x03,
            cartridge_type,
            rom_size            : 32 * (1 << rom[0x148]), // In KiB
            rom_bank_n          : 1 << (rom[0x148]+1),
            ram_size            : ram_size as u16,
            ram_bank_n          : (ram_size as u16/8),
            mask_rom_version_n  : rom[0x14c],
            header_checksum     : rom[0x14d],
            global_checksum     : ((rom[0x14e] as u16) << 8) | rom[0x14f] as u16,
            rom,
            ext_ram             : vec![0;ram_size*1024],
        }
    }

    // Map the ext RAM address to an index of the array where it's stored
    fn map_ext_ram_addr(&self, addr :u16) -> usize {
        return (addr - 0xA000) as usize;
    }
}

impl Cartridge for NoMBC {
    fn is_test_cart(&self) -> bool { return false; }

    fn init(&mut self) {
        // TODO: Disable on debug
        self.print_rom_data();
    }

    fn load_ram(&mut self) {
        if self.cartridge_type.has_ram() && self.ram_size > 0 {
            let path = format!("{}/{}", SAVE_PATH, self.file);
            if std::path::Path::new(&path).exists() {
                let ram = std::fs::read(path).unwrap();
                self.ext_ram = ram;
            }
        }
    }

    fn save_ram(&self) {
        if self.cartridge_type.has_ram() && self.ram_size > 0 {
            if self.cartridge_type.has_ram() && self.ram_size > 0 {
                let path = format!("{}/{}", SAVE_PATH, self.file);

                let mut file = std::fs::File::create(path).unwrap();
                file.write_all(&self.ext_ram).unwrap();
            }
        }
    }

    fn print_rom_data(&self) {
        println!("\nFile:\n{}", self.file);
        
        println!("\nTitle:");
        for n in self.rom[0x134..=0x143].iter() {
            if *n >= 60 && *n <= 120 { // Pritable ascii
                print!("{}", *n as char);
            }
        }
        println!();

        println!("\nCGB Flag\t\t: {}", self.cgb_flag);
        println!("SGB Flag\t\t: {}", self.sgb_flag);
        println!("Cartridge type\t\t: {:?}", self.cartridge_type);
        println!("ROM size\t\t: {} KiB", self.rom_size);
        println!("ROM Banks \t\t: {}", self.rom_bank_n);
        println!("RAM size\t\t: {} KiB", self.ram_size);
        println!("RAM Banks \t\t: {}", self.ram_bank_n);
        println!("Mask ROM version number\t: 0x{:02X}", self.mask_rom_version_n);
        println!("Header checksum\t\t: 0x{:02X}", self.header_checksum);
        println!("Global checksum\t\t: 0x{:04X}", self.global_checksum);
        println!();
        println!("ROM loaded");
        println!("--------------------------------------\n");
    }
}

impl ComponentWithMemory for NoMBC {
    fn read(&self, addr :u16) -> u8 {
        return match addr {
            BANK0_START..=BANK0_END => self.rom[addr as usize],
            BANK1_START..=BANK1_END => self.rom[addr as usize],
            0xA000..=0xBFFF => if self.cartridge_type.has_ram()
                            && self.ram_size > 0 {
                self.ext_ram[self.map_ext_ram_addr(addr)]
            } else {
                0xFF
            },
            _ => panic!("read(): Invalid address: {:04X}", addr)
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            0xA000..=0xBFFF => if self.cartridge_type.has_ram()
                            && self.ram_size > 0 {
                let _addr = self.map_ext_ram_addr(addr);
                self.ext_ram[_addr] = val;
            },
            // TODO: Log
            _ => {}
        }
    }
}