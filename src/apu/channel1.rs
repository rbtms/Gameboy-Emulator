#![allow(dead_code)]
#![allow(unused_variables)]
use crate::consts::*;

pub struct Channel1 {
    nr10 :u8, nr11 :u8, nr12 :u8, nr13 :u8, nr14 :u8,

    is_enabled :bool,
}

impl Channel1 {
    pub fn new() -> Channel1 {
        return Channel1 {
            nr10 :0x80, nr11 :0xBF, nr12 :0xF3, nr13 :0xFF, nr14 :0xBF,

            is_enabled: false,
        }
    }

    pub fn is_enabled(&self) -> bool { self.is_enabled }

    pub fn init(&mut self) {
    }

    pub fn set_volume(&mut self, left: u8, right :u8) {
    }

    pub fn set_mix(&mut self, left :bool, right :bool) {
    }

    pub fn read(&self, addr :u16) -> u8 {
        return match addr {
            ADDR_NR10 => self.nr10 | 0x80,
            ADDR_NR11 => self.nr11 | 0x3F,
            ADDR_NR12 => self.nr12,
            ADDR_NR13 => 0xFF, // Write only
            ADDR_NR14 => self.nr14 | 0xBF,
            _ => panic!()
        }
    }

    pub fn write(&mut self, addr :u16, val :u8) {
        match addr {
            ADDR_NR10 => self.nr10 = val,
            ADDR_NR11 => self.nr11 = val,
            ADDR_NR12 => self.nr12 = val,
            ADDR_NR13 => self.nr13 = val,
            ADDR_NR14 => self.nr14 = val,
            _ => panic!()
        }
    }

    pub fn inc_length(&mut self) {
    }

    pub fn change_sweep(&mut self) {
    }

    pub fn change_envelope(&mut self) {
    }

    pub fn enable(&mut self) {
        self.is_enabled = true;
    }

    pub fn disable(&mut self) {
        self.is_enabled = false;
    }

    pub fn reset_regs(&mut self) {
        self.nr10 = 0; self.nr11 = 0; self.nr12 = 0;
        self.nr13 = 0; self.nr14 = 0;
    }

    pub fn turn_off(&mut self) {
        self.reset_regs();
        self.disable();
    }

    pub fn tick(&mut self) {
    }

    /* ---------------------------------------------------------------------- */

    fn is_bit_set(&self, n :u8, b_i :u8) -> bool {
        return (n >> b_i) & 1 == 1;
    }
    fn sweep_pace(&self)          -> u8   { self.nr10 >> 4 }
    fn sweep_slope(&self)         -> u8   { self.nr10 & 3 }
    fn sweep_inc_dec(&self)       -> bool { self.is_bit_set(self.nr10, 3) }
    fn wave_duty(&self)           -> u8   { self.nr11 >> 6 }
    fn initial_length(&self)      -> u8   { self.nr11 & 0x1F }
    fn initial_env_volume(&self)  -> u8   { self.nr12 >> 4 }
    fn env_direction(&self)       -> bool { self.is_bit_set(self.nr12, 3) }
    fn env_sweep_pace(&self)      -> u8   { self.nr12 & 3 }
    fn period(&self)              -> u16  { (self.nr13 as u16) | ((self.nr14 as u16 & 3) << 8) }
    fn sound_length_enable(&self) -> bool { self.is_bit_set(self.nr14, 6)  }
}

