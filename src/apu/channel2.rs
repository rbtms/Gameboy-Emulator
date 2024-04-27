#![allow(dead_code)]
#![allow(unused_variables)]
use crate::consts::*;

pub struct Channel2 {
    nr21 :u8, nr22 :u8, nr23 :u8, nr24 :u8, 

    is_enabled: bool,
}

impl Channel2 {
    pub fn new() -> Channel2 {
        return Channel2 {
            nr21 :0x3F, nr22 :0x00, nr23 :0xFF, nr24 :0xBF,

            is_enabled: false
        }
    }
    
    pub fn init(&mut self) {
    }
    
    pub fn read(&self, addr :u16) -> u8 {
        return match addr {
            ADDR_NR21 => self.nr21 | 0x3F,
            ADDR_NR22 => self.nr22,
            ADDR_NR23 => 0xFF, // Write only,
            ADDR_NR24 => self.nr24 | 0xBF,
            _ => panic!()
        }
    }

    pub fn write(&mut self, addr :u16, val :u8) {
        match addr {
            ADDR_NR21 => self.nr21 = val,
            ADDR_NR22 => self.nr22 = val,
            ADDR_NR23 => self.nr23 = val,
            ADDR_NR24 => self.nr24 = val,
            _ => panic!()
        }
    }

    pub fn set_volume(&mut self, left: u8, right :u8) {
    }

    pub fn set_mix(&mut self, left :bool, right :bool) {
    }

    pub fn enable(&mut self) {
        self.is_enabled = true;
    }

    pub fn disable(&mut self) {
        self.is_enabled = false;
    }


    pub fn reset_regs(&mut self) {
        self.nr21 = 0x00; self.nr22 = 0x00; self.nr23 = 0x00; self.nr24 = 0x00; 
    }

    pub fn inc_length(&mut self) {
    }

    pub fn is_enabled(&self) -> bool { self.is_enabled }

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
    fn wave_duty(&self)           -> u8   { self.nr21 >> 6 }
    fn initial_length(&self)      -> u8   { self.nr21 & 0x1F }
    fn initial_env_volume(&self)  -> u8   { self.nr22 >> 4 }
    fn env_direction(&self)       -> bool { self.is_bit_set(self.nr22, 3)  }
    fn env_sweep_pace(&self)      -> u8   { self.nr22 & 3 }
    fn period(&self)              -> u16  { (self.nr23 as u16) | ((self.nr24 as u16 & 3) << 8) }
    fn sound_length_enable(&self) -> bool { self.is_bit_set(self.nr24, 6) }
}
