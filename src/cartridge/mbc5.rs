#![allow(dead_code)]

use crate::consts::*;
use crate::cartridge::{CartridgeType, Cartridge};


const ROM_BANK_SIZE :u16 = 0x4000;
const RAM_BANK_SIZE :u16 = 0x2000;


pub struct MBC5 {
    file                : String,
    rom                 : Vec<u8>,
    // 32 KiBs, or 4 KiB banks
    ext_ram             : Vec<u8>,
    cartridge_type      : CartridgeType,
    rom_size            : u16,
    rom_bank_n          : u16,
    ram_size            : u16,
    ram_bank_n          : u16,
    mask_rom_version_n  : u8,
    header_checksum     : u8,
    global_checksum     : u16,

    // MBC registers
    ramg  : bool, // RAM gate register / RAM enable/disable
    romb  : u16,
    ramb  : u8, // RAM Bank
}

impl MBC5 {
    pub fn new(file :&str, rom :Vec<u8>) -> MBC5 {
        let cartridge_type :CartridgeType = rom[0x147].into();
        let ram_size = if cartridge_type.has_ram() { [0, 0, 8, 32, 128, 64][rom[0x149] as usize] } else {0};
        
        return MBC5 {
            file: file.to_string(),
            cartridge_type,
            rom_size            : 32 * (1 << rom[CART_HEADER_ROM_SIZE]), // In KiB
            rom_bank_n          : 1 << (rom[CART_HEADER_ROM_SIZE]+1),
            ram_size            : ram_size as u16,
            ram_bank_n          : (ram_size as u16/8),
            mask_rom_version_n  : rom[CART_HEADER_ROM_VERSION],
            header_checksum     : rom[CART_HEADER_HEADER_CHECKSUM],
            global_checksum     : ((rom[CART_HEADER_CHECKSUM_START] as u16) << 8) | rom[CART_HEADER_CHECKSUM_END] as u16,
            rom,
            ext_ram: vec![0;ram_size*1024],

            // MBC registers
            romb  : 0x0001,
            ramg  : false,
            ramb  : 0,
        }
    }

    pub fn map_bank1_addr(&self, addr :u16) -> usize {
        return ((addr - BANK1_START) as u32 + (ROM_BANK_SIZE as u32 * self.romb as u32)) as usize;
    }
    pub fn map_ext_ram_addr(&self, addr :u16) -> usize {
        return ((addr - EXT_RAM_START) + (RAM_BANK_SIZE * self.ramb as u16)) as usize;
    }
}

impl Cartridge for MBC5 {
    fn is_test_cart(&self) -> bool { return false; }

    fn init(&mut self) {
        // TODO: load RAM
        // TODO: Disable on debug
        self.print_rom_data();
        
        //self.romb = 2;
        //println!("addr: {:04X}", self.map_bank1_addr(BANK1_START));
    }

    fn load_ram(&mut self) {}
    fn save_ram(&self) {}

    fn print_rom_data(&self) {
        println!("\nFile:\n{}", self.file);
        
        println!("\nTitle:");
        for n in self.rom[0x134..=0x143].iter() {
            if *n >= 60 && *n <= 120 { // Pritable ascii
                print!("{}", *n as char);
            }
        }
        println!();

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

impl ComponentWithMemory for MBC5 {
    fn read(&self, addr :u16) -> u8 {
        return match addr {
            BANK0_START..=BANK0_END => self.rom[addr as usize],
            BANK1_START..=BANK1_END => self.rom[self.map_bank1_addr(addr)],
            // TODO: Check that RAM is enabled
            EXT_RAM_START..=EXT_RAM_END => if self.cartridge_type.has_ram()
                                           && self.ram_size > 0
                                           && self.ramg {
                self.ext_ram[self.map_ext_ram_addr(addr)]
            } else {
                0xFF
            }
            _ => panic!("read(): Invalid address: {:04X}", addr)
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            // External RAM enable/disable
            // 0A: Enable. Otherwise: Disable.
            BANK0_START..=0x1FFF => self.ramg = val == 0x0A,
            // ROM bank select
            0x2000..=0x2FFF => {
                self.romb = (self.romb&0x100) | val as u16;
                self.romb %= self.rom_bank_n;
            },
            // 9th bit of the ROM bank number
            0x3000..=0x3FFF => {
                self.romb = (self.romb&0xFF) | (((val&1) as u16) << 8);
                self.romb %= self.rom_bank_n;
            },
            // RAM bank select
            0x4000..=0x5FFF => self.ramb = (val&0x0F) % self.ram_bank_n as u8,
            // External RAM write
            EXT_RAM_START..=EXT_RAM_END => if self.ramg {
                let _addr = self.map_ext_ram_addr(addr);
                self.ext_ram[_addr] = val;
            },
            _ => panic!("write(): Invalid address: {:04X}", addr)
        }
    }
}