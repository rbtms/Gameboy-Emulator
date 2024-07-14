#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use std::path::Component;

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use crate::consts::*;
use crate::apu::Channel as Channel;


pub struct Channel3 {
    nr30 :u8, nr31 :u8, nr32 :u8, nr33 :u8, nr34 :u8,
    wave_ram :[u8;16],

    is_enabled: bool,
    length_timer: u8,
    counter: u8,
    period_timer :u32,
    waveram_pos: u8
}

impl Channel3 {
    pub fn new() -> Channel3 {
        return Channel3 {
            nr30 :0x7F, nr31 :0xFF, nr32 :0x9F, nr33 :0xFF, nr34 :0xBF,
            wave_ram: [0;16],

            is_enabled: false,
            length_timer: 0,
            counter: 0,
            period_timer: 0,
            waveram_pos: 1
        }
    }
    
    pub fn init(&mut self) {}

    // Triggers the channel
    fn trigger(&mut self) {
        self.is_enabled = true;
        self.length_timer = self.initial_length_timer();
    }

    pub fn reset_regs(&mut self) {
        self.nr30 = 0x00; self.nr31 = 0x00; self.nr32 = 0x00;
        self.nr33 = 0x00; self.nr34 = 0x00;
    }

    pub fn tick (&mut self) {
        //if self.counter == 0 {
            // Don't overflow
            if self.period_timer > 0 {
                self.period_timer -= 1;
            }

            if self.period_timer == 0 {
                self.period_timer = 4 * (0x800 - self.period()) as u32;
                self.waveram_pos = (self.waveram_pos+1)%32;
            }
        //}

        // Every 2 ticks
        //self.counter = (self.counter+1)%2;
    }

    /* --------------------------------------------------------------------------------- */

    fn is_bit_set(&self, n :u8, b_i :u8) -> bool {
        return (n >> b_i) & 1 == 1;
    }
    
    // NR30
    fn dac_enable(&self)           -> bool { self.is_bit_set(self.nr30, 7) }
    // NR31
    fn initial_length_timer(&self) -> u8   { self.nr31 }
    // NR32
    fn output_level(&self)         -> u8   { (self.nr32 >> 5) & 3 }
    // NR33 - NR34
    fn period(&self)               -> u16  { ((self.nr34 as u16 & 7) << 8) | (self.nr33 as u16) }
    // NR34
    fn sound_length_enable(&self)  -> bool { self.is_bit_set(self.nr34, 6) }
}

impl Channel for Channel3 {
    fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    // Increase length timer. When the timer reaches 255, the timer is turned off
    fn inc_length(&mut self) {
        if self.sound_length_enable() {
            if self.length_timer < 255 {
                self.length_timer += 1;
            } else {
                // TODO: Call turn_off instead?
                self.is_enabled = false;
            }
        }
    }

    fn turn_off(&mut self) {
        self.reset_regs();
        self.is_enabled = false;
    }

    fn sample(&self) -> u8 {
        let wave_nibble = if self.waveram_pos%2 == 0 {
            // Upper nibble
            self.wave_ram[(self.waveram_pos / 2) as usize] >> 4
        } else {
            // Lower nibble
            self.wave_ram[((self.waveram_pos - 1) / 2) as usize]&0x0F
        };

        return match self.output_level() {
            0 => 0,              // Mute
            1 => wave_nibble,    // 100%
            2 => wave_nibble>>1, // 50%
            3 => wave_nibble>>2, // 25&
            _ => panic!("Invalid output level: {}", self.output_level())
        };
    }
}

impl ComponentWithMemory for Channel3 {
    fn read(&self, addr :u16) -> u8 {
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

    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            // Enable/disable DAC
            //
            // b7   DAC On/off
            // b6-0 Unused
            ADDR_NR30 => {
                self.nr30 = val;

                // Turning off the DAC also disables the channel
                self.is_enabled = false;
            },
            // Length timer
            //
            // b7-0 Initial length timer
            ADDR_NR31 => self.nr31 = val,
            // Output level
            //
            // b7   Unused
            // b6-5 Output level
            // b4-0 Unused
            ADDR_NR32 => self.nr32 = val,
            // Period low
            ADDR_NR33 => self.nr33 = val,
            // Trigger/length enable/period high
            //
            // b7   Trigger
            // b6   Length enable 
            // b2-0 Period high
            ADDR_NR34 => {
                if self.is_bit_set(val, 7) {
                    self.trigger();
                }
 
                self.nr34 = val;
            },

            WAVE_RAM_START..=WAVE_RAM_END => {
                self.wave_ram[(addr-WAVE_RAM_START) as usize] = val;
            },
            _ => panic!()
        }
    }
}