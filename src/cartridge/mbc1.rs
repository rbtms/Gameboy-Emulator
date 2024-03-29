use crate::consts::*;
use crate::cartridge::{CartridgeType, Cartridge};
use std::io::prelude::Write;


const SAVE_PATH :&str = "roms/games/saves";

pub struct MBC1 {
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
    enable_ext_ram      : bool,
    reg_bank1           : u16,
    reg_bank2           : u8,
    selected_mode       : u8
}

impl MBC1 {
    pub fn new(file :&str, rom :Vec<u8>) -> MBC1 {
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

        return MBC1 {
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
            reg_bank1           : 0x01,
            reg_bank2           : 0x00,
            enable_ext_ram      : false,
            selected_mode       : 0
        }
    }

    pub fn map_bank0_addr(&self, addr :u16)   -> u32 {
        return addr as u32;
    }
    pub fn map_bank1_addr(&self, addr :u16)   -> u32 {
        //let bank_n = if self.selected_mode == 0 {self.reg_bank1 as u16} else { ((self.reg_bank2 as u16) << 5) | self.reg_bank1 };
        let bank_n = self.reg_bank1;
        return (addr - BANK1_START) as u32 + 0x4000*bank_n as u32;
    }
    pub fn map_ext_ram_addr(&self, addr :u16) -> u16 {
        let bank_i = if self.selected_mode == 0 {0} else {self.reg_bank2};
        return addr - EXT_RAM_START + 0x2000*bank_i as u16;
    }
    
    /*
     * Debug
     */

    pub fn print_rom_data(&self) {
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

impl Cartridge for MBC1 {
    fn init(&mut self) {
        if self.cartridge_type.has_ram() && self.ram_size > 0 {
            self.load_ram();
        }

        self.print_rom_data(); // TODO: Disable on debug
    }

    fn read(&self, addr :u16) -> u8 {
        return match addr {
            BANK0_START..=BANK0_END => self.rom[self.map_bank0_addr(addr) as usize],
            BANK1_START..=BANK1_END => self.rom[self.map_bank1_addr(addr) as usize],
            // TODO: Check that RAM is enabled
            EXT_RAM_START..=EXT_RAM_END => if self.cartridge_type.has_ram()
                                           && self.ram_size > 0
                                           && self.enable_ext_ram {
                self.ext_ram[self.map_ext_ram_addr(addr) as usize]
            } else {
                0xFF
            }
            _ => panic!("read(): Invalid address: {:04X}", addr)
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            // External RAM enable/disable
            // Lower 4 bits are A: Enable. Otherwise: Disable.
            0x0000..=0x1FFF => self.enable_ext_ram = (val&0x0F) == 0x0A,
            // ROM bank select
            0x2000..=0x3FFF => {
                // Only the 5 lower bits are taken. If val > number of banks, its masked by the
                // corresponding number of bits from the bank number
                let mut bank_n = (val&0x1f) as u16 % self.rom_bank_n;
                if bank_n == 0x00 || bank_n == 0x20 || bank_n == 0x40 || bank_n == 0x60 { bank_n += 1; }

                self.reg_bank1 = bank_n as u16;
            },
            // RAM bank select / 2 upper bits of BANK1 select
            0x4000..=0x5FFF => if self.enable_ext_ram {
                if self.ram_size > 8 || self.rom_size >= 1024 {
                    if val > 3 { panic!("write(); Invalid ram bank: {}", val); }
                    self.reg_bank2 = val; // 00~11
                }
            },
            // Mode select
            0x6000..=0x7FFF => {
                // If the ROM <= 512 KiB or RAM <= 8 KiB, this register has no observable
                // effects
                if self.ram_size > 8 || self.rom_size > 512 {
                    if val > 1 { panic!("write(); Invalid mode") }
                    self.selected_mode = val;
                }
            }
            // External RAM write
            0xA000..=0xBFFF => if self.enable_ext_ram {
                if self.cartridge_type.has_ram() && self.ram_size > 0 {
                    let _addr = self.map_ext_ram_addr(addr);
                    self.ext_ram[_addr as usize] = val;
                }
            },
            _ => panic!("write(): Invalid address: {:04X}", addr)
        }
    }

    fn load_ram(&mut self) {
        let path = format!("{}/{}", SAVE_PATH, self.file);
        if std::path::Path::new(&path).exists() {
            let ram = std::fs::read(path).unwrap();
            self.ext_ram = ram;
        }
    }

    fn save_ram(&self) {
        if self.cartridge_type.has_ram() && self.ram_size > 0 {
            let path = format!("{}/{}", SAVE_PATH, self.file);

            let mut file = std::fs::File::create(path).unwrap();
            file.write_all(&self.ext_ram).unwrap();
        }
    }
}

