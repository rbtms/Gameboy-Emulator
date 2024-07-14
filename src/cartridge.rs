use crate::consts::CART_HEADER_CART_TYPE;
use crate::consts::ComponentWithMemory;

mod noMBC;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod mbcTest;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum CartridgeType {
    ROM                 = 0x00,
    ROM_RAM             = 0x08,
    ROM_RAM_BAT         = 0x09,
    MBC1                = 0x01,
    MBC1_RAM            = 0x02,
    MBC1_RAM_BAT        = 0x03,
    MBC2                = 0x05,
    MBC2_BAT            = 0x06,
    MBC3                = 0x11,
    MBC3_RAM            = 0x12,
    MBC3_RAM_BAT        = 0x13,
    MBC3_BAT_TIMER      = 0x0F,
    MBC3_RAM_BAT_TIMER  = 0x10,
    MBC5                = 0x19,
    MBC5_RAM            = 0x1A,
    MBC5_RAM_BAT        = 0x1B,
    MBC5_RUMBLE         = 0x1C,
    MBC5_RAM_RUMBLE     = 0x1D,
    MBC5_RAM_BAT_RUMBLE = 0x1E,
    MBC6                = 0x20,
    MBC7                = 0x22,
    MBCTest             = 0xFF,
    OTHER               = 0xF0
}

impl From<u8> for CartridgeType {
    fn from(val :u8) -> CartridgeType {
        match val {
            0x00 => CartridgeType::ROM,
            0x01 => CartridgeType::MBC1,
            0x02 => CartridgeType::MBC1_RAM,
            0x03 => CartridgeType::MBC1_RAM_BAT,
            0x05 => CartridgeType::MBC2,
            0x06 => CartridgeType::MBC2_BAT,
            0x08 => CartridgeType::ROM_RAM,
            0x09 => CartridgeType::ROM_RAM_BAT,
            0x10 => CartridgeType::MBC3_RAM_BAT_TIMER,
            0x11 => CartridgeType::MBC3,
            0x12 => CartridgeType::MBC3_RAM,
            0x13 => CartridgeType::MBC3_RAM_BAT,
            0x0F => CartridgeType::MBC3_BAT_TIMER,
            0x19 => CartridgeType::MBC5,
            0x1A => CartridgeType::MBC5_RAM,
            0x1B => CartridgeType::MBC5_RAM_BAT,
            0x1C => CartridgeType::MBC5_RUMBLE,
            0x1D => CartridgeType::MBC5_RAM_RUMBLE,
            0x1E => CartridgeType::MBC5_RAM_BAT_RUMBLE,
            0x20 => CartridgeType::MBC6,
            0x22 => CartridgeType::MBC7,
            0xFF => CartridgeType::MBCTest,
            _    => CartridgeType::OTHER
        }
    }
}

impl CartridgeType {
    pub fn has_ram(&self) -> bool {
        return format!("{:?}", self).contains("RAM");
    }
    pub fn has_battery(&self) -> bool {
        return format!("{:?}", self).contains("BAT");
    }
    pub fn has_rumble(&self) -> bool {
        return format!("{:?}", self).contains("RUMBLE");
    }
    pub fn has_timer(&self) -> bool {
        return format!("{:?}", self).contains("TIMER");
    }

    pub fn mbc_n(&self) -> u8 {
        let s = format!("{:?}", self);

        return if      s.contains("MBC1")    {1}
               else if s.contains("MBC2")    {2}
               else if s.contains("MBC3")    {3}
               else if s.contains("MBC5")    {5}
               else if s.contains("MBC6")    {6}
               else if s.contains("MBC7")    {7}
               else if s.contains("MBCTest") {255}
               else                          {0};
    }
}

pub trait Cartridge: ComponentWithMemory {
    fn init(&mut self);

    fn load_ram(&mut self);
    fn save_ram(&self);
    fn print_rom_data(&self);

    fn is_test_cart(&self) -> bool; // For tests. Remove.
}

pub fn load_cartridge(path :&str) -> Box<dyn Cartridge> {
    let rom = std::fs::read(path).unwrap();
    let file = path.split('/').last().unwrap();
    let cartridge_type :CartridgeType = rom[CART_HEADER_CART_TYPE].into();

    return match cartridge_type.mbc_n() {
        0 => Box::new(noMBC::NoMBC::new(path, rom)),
        1 => Box::new(mbc1::MBC1::new(file, rom)),
        2 => Box::new(mbc2::MBC2::new(file, rom)),
        3 => Box::new(mbc3::MBC3::new(file, rom)),
        5 => Box::new(mbc5::MBC5::new(file, rom)),
        255 => Box::new(mbcTest::MBCTest::new(file, rom)),
        _ => panic!("MBC type not supported: {:?}", cartridge_type)
    }
}
