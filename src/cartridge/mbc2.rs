use crate::consts::*;
use crate::cartridge::{CartridgeType, Cartridge};


const ROM_BANK_SIZE :u16 = 0x4000;


pub struct MBC2 {
    file                : String,
    rom                 : Vec<u8>,
    // 32 KiBs, or 4 KiB banks
    builtin_ram         : Vec<u8>,
    cgb_flag            : bool,
    sgb_flag            : bool,
    cartridge_type      : CartridgeType,
    rom_size            : u16,
    rom_bank_n          : u8,
    ram_size            : u16,
    ram_bank_n          : u16,
    mask_rom_version_n  : u8,
    header_checksum     : u8,
    global_checksum     : u16,

    // MBC registers
    ramg : bool,
    romb : u8,
}

impl MBC2 {
    pub fn new(file :&str, rom :Vec<u8>) -> MBC2 {
        let cartridge_type :CartridgeType = rom[0x147].into();
        let ram_size = if cartridge_type.has_ram() { [0, 0, 8, 32, 128, 64][rom[0x149] as usize] } else {0};
        let builtin_ram = vec![0;512]; // 512 bytes regardless of what the cartridge indicates

        /*
         * Testing
         */
        //let mut rom = rom.clone();
        //let mut v :Vec<u8> = vec![0;16*1024 - rom.len()];
        //rom.append(&mut v);
        //let rom_bank_n = 16;

        return MBC2 {
            file: file.to_string(),
            cgb_flag            : rom[0x143] == 0xC0,
            sgb_flag            : rom[0x146] == 0x03,
            cartridge_type,
            rom_size            : 32 * (1 << rom[0x148]), // In KiB
            rom_bank_n          : (1 << (rom[0x148]+1)) as u8,
            ram_size            : ram_size as u16,
            ram_bank_n          : (ram_size as u16/8),
            mask_rom_version_n  : rom[0x14c],
            header_checksum     : rom[0x14d],
            global_checksum     : ((rom[0x14e] as u16) << 8) | rom[0x14f] as u16,
            rom,
            builtin_ram,

            // MBC registers
            romb : 0x01,  // ROM bank register
            ramg : false, // RAM gate register / RAM enable/disable
        }
    }

    pub fn map_bank1_addr(&self, addr :u16) -> usize {
        let base_addr = addr - BANK1_START;
        let offset = ROM_BANK_SIZE as u32 * self.romb as u32;

        return base_addr as usize + offset as usize;
    }
}

impl Cartridge for MBC2 {
    fn init(&mut self) {
        // TODO: load RAM
        // TODO: Disable on debug
        //self.print_rom_data();
    }

    fn read(&self, addr :u16) -> u8 {
        return match addr {
            BANK0_START..=BANK0_END => self.rom[addr as usize],
            BANK1_START..=BANK1_END => self.rom[self.map_bank1_addr(addr)],
            EXT_RAM_START..=0xA1FF => if self.ramg {
                self.builtin_ram[(addr-EXT_RAM_START) as usize]
            } else {
                0xFF
            },
            // ECHO of A000-A1FF 15 times. Sustract 512 (0x200) until it's on normal RAM range.
            0xA200..=0xBFFF => {
                let mut _addr = addr;
                while addr > 0xA1FF { _addr -= 512; }

                return self.read(_addr);
            },
            _ => panic!("read(): Invalid address: {:04X}", addr)
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            // Enable builtin RAM / Select ROM bank. Highest bit decides which.
            BANK0_START..=BANK0_END => {
                // RAM enable/disable. RAMG is enabled when the last nibble is 0xA
                if (val>>7)&1 == 0 {
                    self.ramg = val&0x0F == 0x0A;
                }
                // ROM bank select
                else {
                    let bank_n = (val&0x0F).max(1); // If it's going to write 0, write 1 instead.

                    self.romb = (bank_n & 0b1111) % self.rom_bank_n as u8; // Only the last 4 bits are used
                }
            },
            BANK1_START..=BANK1_END => {},
            // Builtin RAM write
            EXT_RAM_START..=0xA1FF => if self.ramg {
                self.builtin_ram[(addr-EXT_RAM_START) as usize] = val & 0x0F; // Keep only the lower half-byte
            },
            // ECHO of A000-A1FF 15 times. Sustract 512 (0x200) until it's on normal RAM range.
            0xA200..=EXT_RAM_END => {
                let mut _addr = addr;
                while addr > 0xA1FF { _addr -= 512; }

                self.write(_addr, val);
            }
            _ => panic!("write(): Invalid address: {:04X}", addr)
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
