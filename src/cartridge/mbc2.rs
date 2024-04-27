use crate::consts::*;
use crate::cartridge::{CartridgeType, Cartridge};

pub struct MBC2 {
    file                : String,
    rom                 : Vec<u8>,
    // 32 KiBs, or 4 KiB banks
    builtin_ram         : Vec<u8>,
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
    enable_builtin_ram  : bool,
    reg_bank1           : u16,
}

impl MBC2 {
    pub fn new(file :&str, rom :Vec<u8>) -> MBC2 {
        let cartridge_type :CartridgeType = rom[0x147].into();
        let rom_bank_n = 1 << (rom[0x148]+1);
        let ram_size = if cartridge_type.has_ram() { [0, 0, 8, 32, 128, 64][rom[0x149] as usize] } else {0};
        let builtin_ram = vec![0;512];

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
            rom_bank_n,
            ram_size            : ram_size as u16,
            ram_bank_n          : (ram_size as u16/8),
            mask_rom_version_n  : rom[0x14c],
            header_checksum     : rom[0x14d],
            global_checksum     : ((rom[0x14e] as u16) << 8) | rom[0x14f] as u16,
            rom,
            builtin_ram,

            // MBC registers
            reg_bank1           : 0x01,
            enable_builtin_ram  : false,
        }
    }

    pub fn map_bank1_addr(&self, addr :u16)   -> u32 {
        return (addr - BANK1_START) as u32 + 0x4000*self.reg_bank1 as u32;
    }
}

impl Cartridge for MBC2 {
    fn init(&mut self) {
        // TODO: load RAM
        // TODO: Disable on debug
        self.print_rom_data();
    }

    fn read(&self, addr :u16) -> u8 {
        return match addr {
            BANK0_START..=BANK0_END => self.rom[addr as usize],
            BANK1_START..=BANK1_END => self.rom[self.map_bank1_addr(addr) as usize],
            EXT_RAM_START..=0xA1FF => if self.enable_builtin_ram {
                //println!("[RAM READ] addr {:04X}", addr);
                self.builtin_ram[(addr-EXT_RAM_START) as usize]
            } else {
                0xFF
            },
            // ECHO (only 9 lower bits are used)
            0xA200..=0xBFFF => {
                self.read(EXT_RAM_START + (addr&0x01FF))
            },
            _ => panic!("read(): Invalid address: {:04X}", addr)
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            // Enable builtin RAM / Select ROM bank. Bit 8 decides which.
            0x0000..=0x3FFF => {
                // RAM enable/disable
                if (val>>7)&1 == 0 {
                    self.enable_builtin_ram = val&0x0F == 0x0A;
                    //println!("enable ram {}", self.enable_builtin_ram);
                }
                // ROM bank select
                else {
                    let bank_n = if (val&0x0F) == 0 {1} else {val&0x0F};
                    self.reg_bank1 = (bank_n as u16) % self.rom_bank_n;
                    //println!("set ROM bank {}", self.reg_bank1);
                }
            },
            0x4000..=0x7FFF => {},
            // Builtin RAM write
            0xA000..=0xA1FF => if self.enable_builtin_ram {
                //println!("[RAM WRITE] addr {:04X} val {:02X}", addr, val);
                self.builtin_ram[(addr-EXT_RAM_START) as usize] = val|0xf0;
            },
            // ECHO (only 9 lower bits are used)
            0xA200..=0xBFFF => self.write(EXT_RAM_START + (addr&0x01FF), val),
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
