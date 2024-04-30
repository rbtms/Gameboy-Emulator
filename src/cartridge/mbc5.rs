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

    // MBC registers
    ramg  : bool, // RAM gate register / RAM enable/disable
    romb0 : u8,
    romb1 : u8,
    ramb  : u8, // RAM Bank
}

impl MBC5 {
    pub fn new(file :&str, rom :Vec<u8>) -> MBC5 {
        let cartridge_type :CartridgeType = rom[0x147].into();
        let rom_bank_n = 1 << (rom[0x148]+1);
        let ram_size = if cartridge_type.has_ram() { [0, 0, 8, 32, 128, 64][rom[0x149] as usize] } else {0};
        let ext_ram = vec![0;ram_size*1024];

        /*
         * Testing
         */
        //let mut rom = rom.clone();
        //let mut v :Vec<u8> = vec![0;(512-64)*1024];
        //rom.append(&mut v);
        //let rom_bank_n = 32;

        return MBC5 {
            file: file.to_string(),
            cgb_flag            : rom[0x143] == 0xC0,
            sgb_flag            : rom[0x146] == 0x03,
            cartridge_type,
            rom_size            : 32 * (1 << rom[0x148]), // In KiB
            rom_bank_n,
            ram_size            : ram_size as u16,
            ram_bank_n          : (ram_size as u16/8),
            mask_rom_version_n  : rom[0x14c],
            header_checksum     : rom[0x14d],
            global_checksum     : ((rom[0x14e] as u16) << 8) | rom[0x14f] as u16,
            rom,
            ext_ram,

            // MBC registers
            romb0 : 0x01,
            romb1 : 0x00,
            ramg  : false,
            ramb  : 0,
        }
    }

    pub fn map_bank1_addr(&self, addr :u16) -> usize {
        let bank_n = ((self.romb1 as u16) << 8) | (self.romb0 as u16);
        let bank_n = bank_n % self.rom_bank_n;

        let base_addr = (addr - BANK1_START) as u32;
        let offset = (ROM_BANK_SIZE as u32) * (bank_n as u32);

        return (base_addr + offset) as usize;
    }
    pub fn map_ext_ram_addr(&self, addr :u16) -> usize {
        let base_addr = addr - EXT_RAM_START;
        let offset    = RAM_BANK_SIZE * self.ramb as u16;

        return (base_addr + offset) as usize;
    }
}

impl Cartridge for MBC5 {
    fn init(&mut self) {
        // TODO: load RAM
        // TODO: Disable on debug
        self.print_rom_data();
    }

    fn read(&self, addr :u16) -> u8 {
        return match addr {
            BANK0_START..=BANK0_END => self.rom[addr as usize],
            BANK1_START..=BANK1_END => self.rom[self.map_bank1_addr(addr) as usize],
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
        if self.cartridge_type.mbc_n() == 1 {
            match addr {
                // External RAM enable/disable
                // 0A: Enable. Otherwise: Disable.
                BANK0_START..=0x1FFF => self.ramg = val == 0x0A,
                // ROM bank select
                0x2000..=0x2FFF => self.romb0 = val,
                // 9th bit of the ROM bank number
                0x3000..=0x3FFF => self.romb1 = val&1,
                // RAM bank select
                0x4000..=0x5FFF => self.ramb = val&0x0F,
                // External RAM write
                EXT_RAM_START..=EXT_RAM_END => if self.ramg {
                    let _addr = self.map_ext_ram_addr(addr);
                    self.ext_ram[_addr as usize] = val;
                },
                _ => panic!("write(): Invalid address: {:04X}", addr)
            }
        }
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
