#![allow(dead_code)]
#![allow(unused_variables)]

use crate::consts::*;
use crate::apu::Channel as Channel;

pub struct Channel2 {
    nr21 :u8, nr22 :u8, nr23 :u8, nr24 :u8,

    is_enabled :bool,
    volume :u8,
    env_counter :u8,
    length_timer :u8,
    duty_counter :u8,
    counter: u8,
    period_timer :u32
}

impl Channel2 {
    pub fn new() -> Channel2 {
        return Channel2 {
            nr21 :0xBF, nr22 :0xF3, nr23 :0xFF, nr24 :0xBF,

            is_enabled: false,
            volume: 0,
            counter: 0,
            env_counter: 0,   // 64KHz ticks until the envelope is modified
            length_timer: 0,  // Counts up to 64 and turns the channel off
            duty_counter: 0,  // Actual duty wave position in 0~8
            period_timer: 0   // Number of ticks until duty_counter is modified
        }
    }


    pub fn init(&mut self) {}

    // Triggers the channel
    fn trigger(&mut self) {
        self.is_enabled = true;

        self.volume = self.initial_volume();
        self.length_timer = self.initial_length();
        self.env_counter = self.env_sweep_pace();
    }

    // Set the period registers
    fn set_period(&mut self, val :u16) {
        self.nr23 = (val&0xFF) as u8; // nr13 = Lowest 8 bits
        self.nr24 = (self.nr24&0xF8) | (((val>>8)&3)) as u8;
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


    // TODO: Reset to default values?
    fn reset_regs(&mut self) {
        self.nr21 = 0; self.nr22 = 0;
        self.nr23 = 0; self.nr24 = 0;
    }

    fn duty_cicle(&self) -> u8 {
        let duties :[[u8;8]; 4] = [
            [1,1,1,1,1,1,1,0], // 12.5%
            [0,1,1,1,1,1,1,0], // 25%
            [0,1,1,1,1,0,0,0], // 50%
            [1,0,0,0,0,0,0,1]  // 75%
        ];

        return duties[self.wave_duty() as usize][self.duty_counter as usize];
    }

    pub fn tick(&mut self) {
        //if self.counter == 0 {
            if self.period_timer == 0 {
                self.period_timer = 2 * (0x800 - self.period() as u32);
                self.duty_counter = (self.duty_counter+1)%8;
            } else {
                self.period_timer -= 1;
            }
        //}

        // Every 4 ticks
        //self.counter = (self.counter+1)%4;
    }


    /* ---------------------------------------------------------------------- */

    fn is_bit_set(&self, n :u8, b_i :u8) -> bool {
        return (n >> b_i) & 1 == 1;
    }

    /*
        Channel x’s DAC is enabled if and only if [NRx2] & $F8 != 0; the exception is CH3, whose DAC is
        directly controlled by bit 7 of NR30 instead. Note that the envelope functionality changes the volume,
        but not the value stored in NRx2, and thus doesn’t disable the DACs.
        
        If a DAC is enabled, the digital range $0 to $F is linearly translated to the analog range -1 to 1,
        in arbitrary units. Importantly, the slope is negative: “digital 0” maps to “analog 1”, not “analog -1”.
        
        If a DAC is disabled, it fades to an analog value of 0, which corresponds to “digital 7.5”. The nature of
        this fade is not entirely deterministic and varies between models.
     */
    fn is_dac_enabled(&self) -> bool { self.nr22 & 0xF8 != 0 }

    // Register getters
    // NR21
    fn wave_duty(&self)           -> u8   { self.nr21 >> 6 }
    fn initial_length(&self)      -> u8   { self.nr21 & 0x3F }

    // NR22
    fn initial_volume(&self)      -> u8   { self.nr22 >> 4 }
    // 1: Add, 0: Sustract
    fn env_direction(&self)       -> bool { self.is_bit_set(self.nr22, 3) }
    fn env_sweep_pace(&self)      -> u8   { self.nr22 & 7 }

    // NR23 - NR24
    fn period(&self)              -> u16  { ((self.nr24 as u16 & 7) << 8) | (self.nr23 as u16) }

    // NR24
    fn sound_length_enable(&self) -> bool { self.is_bit_set(self.nr24, 6)  }
}

impl Channel for Channel2 {
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
        return if self.is_enabled {
             self.volume * self.duty_cicle()
        } else {
            0
        }
    }
}

impl ComponentWithMemory for Channel2 {
    fn read(&self, addr :u16) -> u8 {
        return match addr {
            ADDR_NR21 => self.nr21 | 0x3F,
            ADDR_NR22 => self.nr22,
            ADDR_NR23 => 0xFF, // Write only
            ADDR_NR24 => self.nr24 | 0xBF,
            _ => panic!()
        }
    }

    /*
        TODO
        
        The channel is turned off when:

        The channel’s length timer is enabled in NRx4 and expires, or
        For CH1 only: when the period sweep overflows, or
        The channel’s DAC is turned off. The envelope reaching a volume of 0 does NOT turn the channel off!
     */
    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            /*
                Length timer and duty cycle

                b7-6 Wave duty (read/write)
                b5-0 Initial length timer (write only)
             */
            ADDR_NR21 => self.nr21 = val,
            /*
                Volume & envelope

                b7-4 Initial volume
                b3   Env dir
                b2-0 Sweep pace. A setting of 0 disables the envelope
             */
            ADDR_NR22 => {
                // Turn DAC off if bits 7-3 are 0
                if (val & 0xF0) == 0 {
                    self.is_enabled = false; // TODO: Check
                }                

                self.nr22 = val;
            }
            ADDR_NR23 => self.nr23 = val,
            /*                
                Period high and control

                * If the channel’s DAC is off, then the write to NRx4 will be ineffective and won’t turn the channel on.
                
                b7   Trigger (Write-only)
                b6   Length enable (Read/Write)
                b5-3 Unused
                b2-0 Period, upper 3 bits (Write-only)
             */
            ADDR_NR24 => {
                // Trigger the channel
                if self.is_bit_set(val, 7) && self.is_dac_enabled() {
                    self.trigger();
                }

                self.nr24 = val;
            }
            _ => panic!("write(): Invalid address: {:04X}", addr)
        }
    }
}