use crate::consts::*;
use crate::cartridge::{CartridgeType, Cartridge};
use std::io::prelude::Write;


const SAVE_PATH :&str = "roms/games/saves";

#[derive(PartialEq)]
enum SelectionExternal {
    ExtRAM,
    RTC
}

pub struct MBC3 {
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
    enable_rtc          : bool,
    reg_bank1           : u16,
    reg_ram_bank        : u8,
    reg_rtc             : u8,
    ext_selected        : SelectionExternal,

    //rtc_secs : u8,
    //rtc_mins : u8,
    //rtc_hour : u8,
    //rtc_day  : u8,
}

impl MBC3 {
    pub fn new(file :&str, rom :Vec<u8>) -> MBC3 {
        let cartridge_type :CartridgeType = rom[0x147].into();
        let rom_bank_n = 1 << (rom[0x148]+1);
        let ram_size = if cartridge_type.has_ram() { [0, 0, 8, 32, 128, 64][rom[0x149] as usize] } else {0};
        let ext_ram = vec![0;ram_size*1024];

        return MBC3 {
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
            reg_ram_bank        : 0,
            reg_rtc             : 0,
            ext_selected        : SelectionExternal::ExtRAM,
            enable_ext_ram      : false,
            enable_rtc          : false,

            //rtc_secs : 0,
            //rtc_mins : 0,
            //rtc_hour : 0,
            //rtc_day  : 0,
        }
    }

    pub fn map_bank1_addr(&self, addr :u16)   -> u32 {
        return (addr - BANK1_START) as u32 + 0x4000*self.reg_bank1 as u32;
    }
    pub fn map_ext_ram_addr(&self, addr :u16) -> u16 {
        return addr - EXT_RAM_START + 0x2000*self.reg_ram_bank as u16;
    }
}

impl Cartridge for MBC3 {
    fn is_test_cart(&self) -> bool { return false; }

    fn init(&mut self) {
        if self.cartridge_type.has_ram() && self.ram_size > 0 {
            self.load_ram();
        }

        // TODO: Disable on debug
        self.print_rom_data();
    }

    fn load_ram(&mut self) {
        let path = format!("{}/{}", SAVE_PATH, self.file);
        if std::path::Path::new(&path).exists() {
            let ram = std::fs::read(path).unwrap();

            for (i, byte) in ram.iter().enumerate() {
                self.ext_ram[i] = *byte;
            }
        }

    }

    fn save_ram(&self) {
        if self.cartridge_type.has_ram() && self.ram_size > 0 {
            let path = format!("{}/{}", SAVE_PATH, self.file);

            let mut file = std::fs::File::create(path).unwrap();
            file.write_all(&self.ext_ram).unwrap();
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

impl ComponentWithMemory for MBC3 {
    fn read(&self, addr :u16) -> u8 {
        return match addr {
            BANK0_START..=BANK0_END => self.rom[addr as usize],
            BANK1_START..=BANK1_END => self.rom[self.map_bank1_addr(addr) as usize],
            // TODO: Check that RAM is enabled
            EXT_RAM_START..=EXT_RAM_END => if self.cartridge_type.has_ram()
                                           && self.ram_size > 0
                                           && self.enable_ext_ram
                                           && self.ext_selected == SelectionExternal::ExtRAM {
                self.ext_ram[self.map_ext_ram_addr(addr) as usize]
            } else if self.enable_rtc && self.ext_selected == SelectionExternal::RTC {
                // TODO: Read RTC
                0xFF
            } else {
                0xFF
            }
            _ => panic!("read(): Invalid address: {:04X}", addr)
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            // External RAM enable/disable. RTC enable/disable
            // Lower 4 bits are A: Enable. Otherwise: Disable.
            0x0000..=0x1FFF => {
                self.enable_ext_ram = (val&0x0F) == 0x0A;
                self.enable_rtc = (val&0x0F) == 0x0A;
            }
            // ROM bank select
            0x2000..=0x3FFF => {
                let mut bank_n = ((val&0x7F) as u16) % self.rom_bank_n;
                if bank_n == 0x00 { bank_n = 0x01; }

                self.reg_bank1 = bank_n;
            },
            // RAM bank select / RTC register select
            0x4000..=0x5FFF => if self.enable_ext_ram {
                // RAM bank selection
                if val <= 0x03 {
                    self.reg_ram_bank = val;
                    self.ext_selected = SelectionExternal::ExtRAM;
                }
                else if val >= 0x08 && val <= 0x0C {
                    self.reg_rtc = val;
                    self.ext_selected = SelectionExternal::RTC;
                }
            },
            // RTC register latching
            0x6000..=0x7FFF => {
                // TODO: 0x01 -> 0x00
                if val == 0x01 {
                    // TODO
                }
            },
            // External RAM/RTC write
            0xA000..=0xBFFF => {
                // TODO: WRite RAM
                if self.enable_ext_ram && self.ext_selected == SelectionExternal::ExtRAM {
                    let _addr = self.map_ext_ram_addr(addr);
                    //println!("addr {:04X} len {:04X}", _addr, self.ext_ram.len());
                    self.ext_ram[_addr as usize] = val;
                }
                // TODO: Write RTC
                else if self.enable_rtc && self.ext_selected == SelectionExternal::RTC {
                    
                }
            },
            _ => panic!("write(): Invalid address: {:04X}", addr)
        }
    }

}