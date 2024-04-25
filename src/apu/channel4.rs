#![allow(dead_code)]
#![allow(unused_variables)]
use crate::consts::*;

pub struct Channel4 {
    nr41 :u8, nr42 :u8, nr43 :u8, nr44 :u8,
    is_enabled: bool
}

impl Channel4 {
    pub fn new() -> Channel4 {
        return Channel4 {
            nr41 :0xFF, nr42 :0x00, nr43 :0x00, nr44 :0xBF,
            is_enabled: false
        }
    }
    
    pub fn init(&mut self) {
    }
    
    pub fn read(&self, addr :u16) -> u8 {
        return match addr {
            ADDR_NR41 => 0xFF, // Write only,
            ADDR_NR42 => self.nr42,
            ADDR_NR43 => self.nr43,
            ADDR_NR44 => self.nr44 | 0xBF,
            _ => panic!()
        }
    }

    pub fn write(&mut self, addr :u16, val :u8) {
        match addr {
            ADDR_NR41 => self.nr41 = val,
            ADDR_NR42 => self.nr42 = val,
            ADDR_NR43 => self.nr43 = val,
            ADDR_NR44 => self.nr44 = val,
            _ => panic!()
        }
    }

    pub fn reset_regs(&mut self) {
        self.nr41 = 0x00; self.nr42 = 0x00; self.nr43 = 0x00; self.nr44 = 0x00;
    }

    pub fn inc_length(&mut self) {
    }

    pub fn set_volume(&mut self, left :u8, right :u8) {
    }
    
    pub fn is_enabled(&self) -> bool { self.is_enabled }
    pub fn disable(&mut self) { self.is_enabled = false; }

    pub fn turn_off(&mut self) {
        self.reset_regs();
        self.disable();
    }

    pub fn tick(&mut self) {
    }

    /* --------------------------------------------------------------------------------- */

    fn is_bit_set(&self, n :u8, b_i :u8) -> bool {
        return (n >> b_i) & 1 == 1;
    }
    fn length_timer(&self)        -> u8   { self.nr41 & 0x1F }
    fn initial_env_volume(&self)  -> u8   { self.nr42 >> 4 }
    fn env_direction(&self)       -> bool { self.is_bit_set(self.nr42, 3) }
    fn env_sweep_pace(&self)      -> u8   { self.nr42 & 3 }
    fn clock_shift_sec(&self)     -> u8   { self.nr43 >> 4 }
    fn lfsr_width(&self)          -> bool { self.is_bit_set(self.nr43, 3) }
    fn clock_divider(&self)       -> u8   { self.nr43 & 3 }
    fn sound_length_enable(&self) -> bool { self.is_bit_set(self.nr44, 6) }
}

