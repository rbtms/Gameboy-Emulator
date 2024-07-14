#![allow(dead_code)]
#![allow(unused_variables)]
use crate::consts::*;
use crate::apu::Channel as Channel;


pub struct Channel4 {
    nr41 :u8, nr42 :u8, nr43 :u8, nr44 :u8,
    is_enabled: bool,

    volume :u8,
    period_timer :u32,
    length_timer :u8,
    env_counter :u8,
    lsfr :[bool;16]
}

impl Channel4 {
    pub fn new() -> Channel4 {
        return Channel4 {
            nr41 :0xFF, nr42 :0x00, nr43 :0x00, nr44 :0xBF,
            is_enabled: false,

            volume: 0,
            period_timer: 0,
            lsfr: [false;16],
            length_timer: 0,  // Counts up to 64 and turns the channel off
            env_counter: 0    // 64KHz ticks until the envelope is modified
        }
    }
    
    pub fn init(&mut self) {}

    pub fn trigger(&mut self) {
        self.is_enabled = true;

        self.env_counter = self.env_sweep_pace();
        self.volume = self.initial_env_volume();
        self.length_timer = self.initial_length_timer();

        for i in 0..15 {
            self.lsfr[i] = false;
        }
    }

    pub fn change_envelope(&mut self) {
        // Each env sweep tick is 64KHz
        if self.env_sweep_pace() == 0 {
            return;
        }

        // Each env tick is 64KHz
        if self.env_counter > 0 {
            self.env_counter -= 1;
        } else {
            // Add
            if self.env_direction() {
                // The digital value produced by the channels is on the range [0, 15]
                self.volume = if self.volume == 0x0F {0x0F} else {self.volume+1};
            // Sustract
            } else {
                // Don't underflow
                if self.volume > 0 {
                    self.volume -= 1;
                }
            }

            self.env_counter = self.env_sweep_pace();
        }
    }

    pub fn reset_regs(&mut self) {
        self.nr41 = 0x00; self.nr42 = 0x00;
        self.nr43 = 0x00; self.nr44 = 0x00;
    }
    
    pub fn tick(&mut self) {
        // Don't overflow
        if self.period_timer > 0 {
            self.period_timer -= 1;
        }

        if self.period_timer == 0 {
            let new_val = !(self.lsfr[0] ^ self.lsfr[1]);

            // Shift LSFR to the right
            for i in 0..14 {
                self.lsfr[i] = self.lsfr[i+1];
            }

            // Add new value to the left
            self.lsfr[15] = new_val;
            
            // If short-mode is selected, copy the bit to b7 as well
            if self.lfsr_width() {
                self.lsfr[7] = new_val;
            }

            // TODO: 0.5 instead of 1
            let divider :u32 = if self.clock_divider() == 0 {1} else {self.clock_divider() as u32};
            self.period_timer = 262144/(divider * 2_u32.pow(self.clock_shift_sec() as u32));
        }
    }

    /* --------------------------------------------------------------------------------- */

    fn is_bit_set(&self, n :u8, b_i :u8) -> bool {
        return (n >> b_i) & 1 == 1;
    }
    fn initial_length_timer(&self) -> u8   { self.nr41 & 0x3F }
    fn initial_env_volume(&self)   -> u8   { self.nr42 >> 4 }
    fn env_direction(&self)        -> bool { self.is_bit_set(self.nr42, 3) }
    fn env_sweep_pace(&self)       -> u8   { self.nr42 & 7 }
    fn clock_shift_sec(&self)      -> u8   { self.nr43 >> 4 }
    fn lfsr_width(&self)           -> bool { self.is_bit_set(self.nr43, 3) }
    fn clock_divider(&self)        -> u8   { self.nr43 & 7 }
    fn sound_length_enable(&self)  -> bool { self.is_bit_set(self.nr44, 6) }
}

impl Channel for Channel4 {
    fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    // Increase length timer. When the timer reaches 64, the timer is turned off
    fn inc_length(&mut self) {
        if self.sound_length_enable() {
            if self.length_timer < 64 {
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
        return if self.lsfr[0] {
            if self.volume != 0 {
                println!("vol: {}", self.volume);
            }
            self.volume*5
        } else {
            //println!("vol: None");
            0
        }
    }
}

impl ComponentWithMemory for Channel4 {
    fn read(&self, addr :u16) -> u8 {
        return match addr {
            ADDR_NR41 => 0xFF, // Write only,
            ADDR_NR42 => self.nr42,
            ADDR_NR43 => self.nr43,
            ADDR_NR44 => self.nr44 | 0xBF,
            _ => panic!()
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            // Length timer
            //
            // b7-6 Unused
            // b5-0 Initial length timer
            ADDR_NR41 => self.nr41 = val,
            // Volume & envelope
            //
            // b7-6 Initial volume
            // b3   Env direction
            // b2-0 Env sweep pace
            ADDR_NR42 => self.nr42 = val,
            // Frecuency & randomness
            //
            // b7-4 Clock shift
            // b3   LSFR width
            // b2-0 Clock divider
            ADDR_NR43 => self.nr43 = val,
            // Control
            //
            // b7   Trigger
            // b6   Length enable
            // b5-0 Unused
            ADDR_NR44 => self.nr44 = val,
            _ => panic!()
        }
    }
}