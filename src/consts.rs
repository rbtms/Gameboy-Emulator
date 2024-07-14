pub type PCSIZE   = u16;
pub type SPSIZE   = u16;
pub type REG      = u8;
pub type REGINDEX = u8;
pub type RAMVAL   = u8;
pub type RAMINDEX = u16;

pub const REG_A :REGINDEX = 0;
pub const REG_B :REGINDEX = 2;
pub const REG_C :REGINDEX = 3;
pub const REG_D :REGINDEX = 4;
pub const REG_E :REGINDEX = 5;
pub const REG_F :REGINDEX = 6;
pub const REG_H :REGINDEX = 7;
pub const REG_L :REGINDEX = 1;

#[derive(Debug)]
pub struct Config {
    pub is_debug :bool,
    pub has_breakpoint :bool,
    pub breakpoint_addr :u16,
    pub rom_path :String,
    pub screen_mult: u8
}

pub enum JmpCond {
    NZ,
    Z,
    NC,
    C
}

#[derive(Debug)]
pub enum Interrupt {
    VBlank,
    STAT,
    Timer,  
    Serial, 
    Joypad 
}

pub trait ComponentWithMemory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

pub const REG_N :u8 = 8;

pub const CART_HEADER_TITLE_START         :usize = 0x134;
pub const CART_HEADER_TITLE_END           :usize = 0x143;
pub const CART_HEADER_MANUF_CODE_START    :usize = 0x13F;
pub const CART_HEADER_MANUF_CODE_END      :usize = 0x142;
pub const CART_HEADER_LICENSE_CODE_START  :usize = 0x144;
pub const CART_HEADER_LICENSE_CODE_END    :usize = 0x145;
pub const CART_HEADER_CART_TYPE           :usize = 0x147;
pub const CART_HEADER_ROM_SIZE            :usize = 0x148;
pub const CART_HEADER_RAM_SIZE            :usize = 0x149;
pub const CART_HEADER_ROM_VERSION         :usize = 0x14C;
pub const CART_HEADER_HEADER_CHECKSUM     :usize = 0x14D;
pub const CART_HEADER_CHECKSUM_START      :usize = 0x14E;
pub const CART_HEADER_CHECKSUM_END        :usize = 0x14F;

pub const ADDR_P1    :u16 = 0xFF00; pub const ADDR_SB    :u16 = 0xFF01; pub const ADDR_SC    :u16 = 0xFF02;
pub const ADDR_DIV   :u16 = 0xFF04; pub const ADDR_TIMA  :u16 = 0xFF05; pub const ADDR_TMA   :u16 = 0xFF06;
pub const ADDR_TAC   :u16 = 0xFF07; pub const ADDR_IF    :u16 = 0xFF0F; pub const ADDR_NR10  :u16 = 0xFF10;
pub const ADDR_NR11  :u16 = 0xFF11; pub const ADDR_NR12  :u16 = 0xFF12; pub const ADDR_NR13  :u16 = 0xFF13;
pub const ADDR_NR14  :u16 = 0xFF14; pub const ADDR_NR21  :u16 = 0xFF16; pub const ADDR_NR22  :u16 = 0xFF17;
pub const ADDR_NR23  :u16 = 0xFF18; pub const ADDR_NR24  :u16 = 0xFF19; pub const ADDR_NR30  :u16 = 0xFF1A;
pub const ADDR_NR31  :u16 = 0xFF1B; pub const ADDR_NR32  :u16 = 0xFF1C; pub const ADDR_NR33  :u16 = 0xFF1D;
pub const ADDR_NR34  :u16 = 0xFF1E; pub const ADDR_NR41  :u16 = 0xFF20; pub const ADDR_NR42  :u16 = 0xFF21;
pub const ADDR_NR43  :u16 = 0xFF22; pub const ADDR_NR44  :u16 = 0xFF23; pub const ADDR_NR50  :u16 = 0xFF24;
pub const ADDR_NR51  :u16 = 0xFF25; pub const ADDR_NR52  :u16 = 0xFF26; pub const ADDR_LCDC  :u16 = 0xFF40;
pub const ADDR_STAT  :u16 = 0xFF41; pub const ADDR_SCY   :u16 = 0xFF42; pub const ADDR_SCX   :u16 = 0xFF43;
pub const ADDR_LY    :u16 = 0xFF44; pub const ADDR_LYC   :u16 = 0xFF45; pub const ADDR_DMA   :u16 = 0xFF46; 
pub const ADDR_BGP   :u16 = 0xFF47; pub const ADDR_OBP0  :u16 = 0xFF48; pub const ADDR_OBP1  :u16 = 0xFF49;
pub const ADDR_WY    :u16 = 0xFF4A; pub const ADDR_WX    :u16 = 0xFF4B; pub const ADDR_KEY1  :u16 = 0xFF4D;
pub const ADDR_VBK   :u16 = 0xFF4F; pub const ADDR_HDMA1 :u16 = 0xFF51; pub const ADDR_HDMA2 :u16 = 0xFF52;
pub const ADDR_HDMA3 :u16 = 0xFF53; pub const ADDR_HDMA4 :u16 = 0xFF54; pub const ADDR_HDMA5 :u16 = 0xFF55;
pub const ADDR_RP    :u16 = 0xFF56; pub const ADDR_BCPS  :u16 = 0xFF68; pub const ADDR_BCPD  :u16 = 0xFF69;
pub const ADDR_OCPS  :u16 = 0xFF6A; pub const ADDR_OCPD  :u16 = 0xFF6B; pub const ADDR_SVBK  :u16 = 0xFF70;
pub const ADDR_IE    :u16 = 0xFFFF;

// Wave ram registers
pub const WAVE_RAM_START :u16 = 0xFF30;
pub const WAVE_RAM_END   :u16 = 0xFF3F;

// MBC
pub const BANK0_START       :u16 = 0x0000;
pub const BANK0_END         :u16 = 0x3FFF;
pub const BANK1_START       :u16 = 0x4000;
pub const BANK1_END         :u16 = 0x7FFF;
pub const VRAM_START        :u16 = 0x8000;
pub const VRAM_END          :u16 = 0x9FFF;
pub const EXT_RAM_START     :u16 = 0xA000;
pub const EXT_RAM_END       :u16 = 0xBFFF;
pub const WORK_RAM_START    :u16 = 0xC000;
pub const WORK_RAM_END      :u16 = 0xDFFF;
pub const ECHO_RAM_START    :u16 = 0xE000;
pub const ECHO_RAM_END      :u16 = 0xFDFF;
pub const OAM_START         :u16 = 0xFE00;
pub const OAM_END           :u16 = 0xFE9F;
pub const IO_REG_START      :u16 = 0xFEA0;
pub const IO_REG_END        :u16 = 0xFF7F;
pub const HRAM_START        :u16 = 0xFF80;
pub const HRAM_END          :u16 = 0xFFFE;

pub const SCREEN_WIDTH  :u16 = 160;
pub const SCREEN_HEIGHT :u16 = 144;

// Instruction cycles
pub const WAIT :[u8;256] = [
//  0   1   2   3   4   5   6   7   8   9   a   b   c   d   e   f
    4, 12,  8,  8,  4,  4,  8,  4, 20,  8,  8,  8,  4,  4,  8,  4, // 0
    4, 12,  8,  8,  4,  4,  8,  4, 12,  8,  8,  8,  4,  4,  8,  4, // 1
    8, 12,  8,  8,  4,  4,  8,  4,  8,  8,  8,  8,  4,  4,  8,  4, // 2
    8, 12,  8,  8, 12, 12, 12,  4,  8,  8,  8,  8,  4,  4,  8,  4, // 3
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 4
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 5
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 6
    8,  8,  8,  8,  8,  8,  4,  8,  4,  4,  4,  4,  4,  4,  8,  4, // 7
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 8
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 9
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // a
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // b
    8, 12, 12, 16, 12, 16,  8, 16,  8, 16, 12,  4, 12, 24,  8, 16, // c
    8, 12, 12,  0, 12, 16,  8, 16,  8, 16, 12,  0, 12,  0,  8, 16, // d
   12, 12,  8,  0,  0, 16,  8, 16, 16,  4, 16,  0,  0,  0,  8, 16, // e
   12, 12,  8,  4,  0, 16,  8, 16, 12,  8, 16,  4,  0,  0,  8, 16  // f
];

// CB-prefixed instruction cycles
pub const WAIT_CB :[u8;256] = [
    //  0   1   2   3   4   5   6   7   8   9   a   b   c   d   e   f
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 0
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 1
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 2
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 3
    8,  8,  8,  8,  8,  8, 12,  8,  8,  8,  8,  8,  8,  8, 12,  8, // 4
    8,  8,  8,  8,  8,  8, 12,  8,  8,  8,  8,  8,  8,  8, 12,  8, // 5
    8,  8,  8,  8,  8,  8, 12,  8,  8,  8,  8,  8,  8,  8, 12,  8, // 6
    8,  8,  8,  8,  8,  8, 12,  8,  8,  8,  8,  8,  8,  8, 12,  8, // 7
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 8
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 9
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // a
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // b
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // c
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // d
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // e
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // f
];

pub const OP_BYTE_LEN :[u8;256] = [
//  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1, // 0
    1, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, // 1
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, // 2
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6 
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 8
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 9
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // a
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // b
    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, // c
    1, 1, 3, 0, 3, 1, 2, 1, 1, 1, 3, 0, 3, 0, 2, 1, // d
    2, 1, 1, 0, 0, 1, 2, 1, 2, 1, 3, 0, 0, 0, 2, 1, // e
    2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 3, 1, 0, 0, 2, 1, // f
];
