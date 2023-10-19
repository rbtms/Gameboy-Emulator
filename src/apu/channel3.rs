use sdl2::audio::{AudioCallback, AudioSpecDesired};

use crate::consts::*;

struct Wave {
    freq :u8,
    data :[u8;16],
    data_pos :u8,
}

pub struct Channel3 {
    nr30 :u8, nr31 :u8, nr32 :u8, nr33 :u8, nr34 :u8,
    wave_ram :[u8;16],
    is_enabled: bool
}

impl Channel3 {
    pub fn new() -> Channel3 {
        return Channel3 {
            nr30 :0x7F, nr31 :0xFF, nr32 :0x9F, nr33 :0xFF, nr34 :0xBF,
            wave_ram: [0;16],
            is_enabled: false
        }
    }
    
    pub fn init(&mut self) {
    }
    
    pub fn read(&self, addr :u16) -> u8 {
        return match addr {
            ADDR_NR30 => self.nr30 | 0x7F,
            ADDR_NR31 => 0xFF, // Write only,
            ADDR_NR32 => self.nr32 | 0x9F,
            ADDR_NR33 => 0xFF, // Write only,
            ADDR_NR34 => self.nr34 | 0xBF,

            WAVE_RAM_START..=WAVE_RAM_END
                => self.wave_ram[(addr-WAVE_RAM_START) as usize],
            _ => panic!()
        }
    }

    pub fn write(&mut self, addr :u16, val :u8) {
        match addr {
            // Enable/disable DAC
            ADDR_NR30 => self.nr30 = val,
            ADDR_NR31 => self.nr31 = val,
            ADDR_NR32 => self.nr32 = val,
            ADDR_NR33 => self.nr33 = val,
            // Trigger/length enable
            ADDR_NR34 => self.nr34 = val,

            WAVE_RAM_START..=WAVE_RAM_END => {
                self.wave_ram[(addr-WAVE_RAM_START) as usize] = val;
            },
            _ => panic!()
        }
    }

    pub fn inc_length(&mut self) {
    }

    pub fn set_volume(&mut self, left :u8, right :u8) {
    }

    pub fn reset_regs(&mut self) {
        self.nr30 = 0x00; self.nr31 = 0x00; self.nr32 = 0x00; self.nr33 = 0x00; self.nr34 = 0x00;
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn enable(&mut self) {
        self.is_enabled = true;
    }
    pub fn disable(&mut self) {
        self.is_enabled = false;
    }

    pub fn turn_off(&mut self) {
        self.reset_regs();
        self.disable();
    }

    pub fn tick (&mut self) {
    }

    /* --------------------------------------------------------------------------------- */

    fn is_bit_set(&self, n :u8, b_i :u8) -> bool {
        return (n >> b_i) & 1 == 1;
    }
    fn dac_enable(&self)          -> bool { self.is_bit_set(self.nr30, 7) }
    fn length_timer(&self)        -> u8   { self.nr31 }
    fn output_level(&self)        -> u8   { (self.nr32 >> 5) & 3 }
    fn period(&self)              -> u16  { (self.nr33 as u16) | ((self.nr34 as u16 & 3) << 8) }
    fn sound_length_enable(&self) -> bool { self.is_bit_set(self.nr34, 6) }
}

