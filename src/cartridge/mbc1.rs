use crate::consts::*;
use crate::cartridge::{CartridgeType, Cartridge};
use std::io::prelude::Write;


const SAVE_PATH :&str = "roms/games/saves";
const ROM_BANK_SIZE :u16 = 0x4000;
const RAM_BANK_SIZE :u16 = 0x2000;

pub struct MBC1 {
    file                : String,
    rom                 : Vec<u8>,
    // 32 KiBs, or 4 KiB banks
    ext_ram             : Vec<u8>,
    cartridge_type      : CartridgeType,
    rom_size            : u16,
    rom_bank_n          : u16,
    ram_size            : u16,
    ram_bank_n          : u8,
    mask_rom_version_n  : u8,
    header_checksum     : u8,
    global_checksum     : u16,

    // MBC registers
    ramg          : bool, // RAM gate register / RAM enable/disable
    romb0         : u8,   // ROM bank 0
    romb1         : u8,   // ROM bank 1
    selected_mode : u8
}


impl MBC1 {
    pub fn new(file :&str, rom :Vec<u8>) -> MBC1 {
        let cartridge_type :CartridgeType = rom[CART_HEADER_CART_TYPE].into();
        let ram_size  = if cartridge_type.has_ram() { [0, 0, 8, 32, 128, 64][rom[CART_HEADER_RAM_SIZE] as usize] } else {0};

        return MBC1 {
            file: file.to_string(),
            cartridge_type,
            rom_size            : 32 * (1 << rom[CART_HEADER_ROM_SIZE]), // In KiB
            rom_bank_n          : 1 << (rom[CART_HEADER_ROM_SIZE]+1),
            ram_size            : ram_size as u16,
            ram_bank_n          : (ram_size as u8/8),
            mask_rom_version_n  : rom[CART_HEADER_ROM_VERSION],
            header_checksum     : rom[CART_HEADER_HEADER_CHECKSUM],
            global_checksum     : ((rom[CART_HEADER_CHECKSUM_START] as u16) << 8) | rom[CART_HEADER_CHECKSUM_END] as u16,
            rom,
            ext_ram             : vec![0;ram_size*1024],

            // MBC registers
            romb0         : 0x01,
            romb1         : 0x00,
            ramg          : false,
            selected_mode : 0
        }
    }

    /*
     Map the BANK0 address to the index of the rom array
     
     For the range 0000-3FFF the bank number depends mainly on the selected MODE.
     If the MODE == 0, then the bank number is always 0, but if it's 1 the bank_n
     equals to the romb1 register shifted to the left 5 places
     */
    pub fn map_bank0_addr(&self, addr :u16) -> usize {
        let bank_n = if self.selected_mode == 0 {0} else { (self.romb1 as u16) << 5 };
        let bank_n = bank_n % self.rom_bank_n;

        let offset = (ROM_BANK_SIZE as u32) * (bank_n as u32);
        
        return (addr as u32 + offset) as usize;
    }

    /*
     Map the BANK1 address to the index of the rom array

     For the range 0x4000-7FFF, it always uses the 5 last bits of romb0 and the last 2 of romb1
     to choose the bank number
    */
    pub fn map_bank1_addr(&self, addr :u16) -> usize {
        let bank_n = ((self.romb1 as u16) << 5) | (self.romb0 as u16);
        let bank_n = bank_n % self.rom_bank_n;

        let base_addr = (addr - BANK1_START) as u32; // Start at 0x0000 because bank_n is going to be at least 1
        let offset = (ROM_BANK_SIZE as u32) * (bank_n as u32);

        return (base_addr + offset) as usize;
    }

    // Map the ext RAM address to an index of the array where it's stored
    pub fn map_ext_ram_addr(&self, addr :u16) -> usize {
        let bank_n = if self.selected_mode == 0 {0} else {self.romb1};
        return (addr - EXT_RAM_START + RAM_BANK_SIZE*(bank_n%self.ram_bank_n) as u16) as usize;
    }
}

impl Cartridge for MBC1 {
    fn is_test_cart(&self) -> bool { return false; }

    fn init(&mut self) {
        if self.cartridge_type.has_ram() && self.ram_size > 0 {
            self.load_ram();
        }

        self.print_rom_data(); // TODO: Disable on debug
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

impl ComponentWithMemory for MBC1 {
    fn read(&self, addr :u16) -> u8 {
        return match addr {
            BANK0_START..=BANK0_END => self.rom[self.map_bank0_addr(addr)],
            BANK1_START..=BANK1_END => self.rom[self.map_bank1_addr(addr)],
            // It checks RAM is enabled. In case it isn't, the most frequent
            // thing is to return FF per pandocs.
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

    /*
     For writes, the range 0000~7FFF does not set values in the ROM, but the written values
     are used to set registers.
     */
    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            // External RAM enable/disable
            // Lower 4 bits are A: Enable. Otherwise: Disable.
            0x0000..=0x1FFF => self.ramg = (val&0x0F) == 0x0A,
            // ROM bank select
            0x2000..=0x3FFF => {
                // Only the 5 lower bits are taken. If val > number of banks, its masked by the
                // corresponding number of bits from the bank number
                self.romb0 = (val&0b11111).max(1); // If it were to write 0, write 1 instead
            },
            // RAM bank select / 2 upper bits of BANK1 select
            0x4000..=0x5FFF => if self.ramg {
                self.romb1 = val&0b11; // 00~11
            },
            // Mode select
            0x6000..=0x7FFF => {
                self.selected_mode = val&1;
            }
            // External RAM write
            0xA000..=0xBFFF => if self.ramg && self.cartridge_type.has_ram() && self.ram_size > 0 {
                    let _addr = self.map_ext_ram_addr(addr);
                    self.ext_ram[_addr] = val;
            },
            _ => panic!("write(): Invalid address: {:04X}", addr)
        }
    }
}