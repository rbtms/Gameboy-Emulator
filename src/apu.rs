#![allow(dead_code)]
#![allow(unused_variables)]
use crate::consts::*;

mod channel1;
mod channel2;
mod channel3;
mod channel4;
mod audio;

// TODO: Turning the APU off resets the duty counters

pub trait Channel: ComponentWithMemory {
    fn is_enabled(&self) -> bool;
    fn inc_length(&mut self);
    fn turn_off(&mut self);
    fn sample(&self) -> u8;
}

#[allow(dead_code)]
pub struct APU {
    nr50 :u8, nr51 :u8, nr52 :u8,
    prev_div_bit: bool,
    div_apu: u8,

    ch1 :channel1::Channel1,
    ch2 :channel2::Channel2,
    ch3 :channel3::Channel3,
    ch4 :channel4::Channel4,

    audio: audio::Audio,
    sample_counter: u16
}

impl APU {
    pub fn new(subsystem :sdl2::AudioSubsystem) -> APU {
        return APU {
            nr50 :0x77, nr51 :0xF3, nr52 :0xF1,
            prev_div_bit: false,
            div_apu: 0,

            ch1: channel1::Channel1::new(),
            ch2: channel2::Channel2::new(),
            ch3: channel3::Channel3::new(),
            ch4: channel4::Channel4::new(),

            audio: audio::Audio::new(subsystem),
            sample_counter: 0
        }
    }

    pub fn init(&mut self) {
        self.ch1.init();
        self.ch2.init();
        self.ch3.init();
        self.ch4.init();

        self.audio.resume();
    }

    // Master control
    fn write_nr52(&mut self, val :u8) {
        let prev_enable = self.is_apu_enabled();
        
        // Only set the first bit
        self.nr52 = (val&0x80) | (self.nr52&0x7F);
        let enable = self.is_apu_enabled();
        
        // Enable the audio if the APU is turned on
        if !prev_enable && enable {
            self.audio.resume();
        }
        // Clear registers when APU is turned off
        else if prev_enable && !enable {
            self.audio.pause();

            self.ch1.turn_off();
            self.ch2.turn_off();
            self.ch3.turn_off();
            self.ch4.turn_off();

            self.nr50 = 0x00; self.nr51 = 0x00; // self.nr52 = 0x00;
            self.div_apu = 0;
            self.prev_div_bit = false;
        }
    }

    fn is_bit_set(&self, n :u8, b_i :u8) -> bool {
        return (n >> b_i) & 1 == 1;
    }

    // NR52: Sound on/off
    fn is_apu_enabled(&self) -> bool { self.is_bit_set(self.nr52, 7) }
    // NR51: Sound panning
    fn is_mix_ch4_left(&self)   -> bool { self.is_bit_set(self.nr51, 7) }
    fn is_mix_ch3_left(&self)   -> bool { self.is_bit_set(self.nr51, 6) }
    fn is_mix_ch2_left(&self)   -> bool { self.is_bit_set(self.nr51, 5) }
    fn is_mix_ch1_left(&self)   -> bool { self.is_bit_set(self.nr51, 4) }
    fn is_mix_ch4_right(&self)  -> bool { self.is_bit_set(self.nr51, 3) }
    fn is_mix_ch3_right(&self)  -> bool { self.is_bit_set(self.nr51, 2) }
    fn is_mix_ch2_right(&self)  -> bool { self.is_bit_set(self.nr51, 1) }
    fn is_mix_ch1_right(&self)  -> bool { self.is_bit_set(self.nr51, 0) }
    // NR50: Master volume
    fn left_volume(&self)    -> u8   { (self.nr50 >> 4)&7 }
    fn right_volume(&self)   -> u8   { self.nr50 & 3 }


    /* ------------------------------------------------------------------------------------------ */

    fn add_samples(&mut self, n_samples: u16) {
        // Left audio
        let left = if self.is_mix_ch1_left() {self.ch1.sample()} else {0}
            + if self.is_mix_ch2_left() {self.ch2.sample()} else {0}
            + if self.is_mix_ch3_left() {self.ch3.sample()} else {0}
            + if self.is_mix_ch4_left() {self.ch4.sample()} else {0};

        // Right audio
        let right = if self.is_mix_ch1_right() {self.ch1.sample()} else {0}
            + if self.is_mix_ch2_right() {self.ch2.sample()} else {0}
            + if self.is_mix_ch3_right() {self.ch3.sample()} else {0}
            + if self.is_mix_ch4_right() {self.ch4.sample()} else {0};


        let left = left.max(0).min(127);
        let right = right.max(0).min(127);

        self.audio.queue(left, right, n_samples);
    }

    pub fn tick(&mut self, div :u8) {
        if self.is_apu_enabled() {
            /*
                A “DIV-APU” counter is increased every time DIV’s bit 4 (5 in double-speed mode) goes from 1 to 0,
                therefore at a frequency of 512 Hz.
                Thus, the counter can be made to increase faster by writing to DIV while its relevant bit is set
                (which clears DIV, and triggers the falling edge).
             */
            let div_bit = self.is_bit_set(div, 4);

            if self.prev_div_bit && !div_bit {
                self.div_apu = (self.div_apu + 1) % 8;

                // Sound length event every 2 DIV-APU ticks (256hz)
                if self.div_apu%2 == 0 {
                    self.ch1.inc_length();
                    self.ch2.inc_length();
                    self.ch3.inc_length();
                    self.ch4.inc_length();
                }
                // ch1 frequency sweep event every 4 DIV-APU ticks (128hz)
                if self.div_apu%4 == 0 {
                    self.ch1.change_sweep();
                }
                // Envelope length event every 8 DIV-APU ticks (64hz)
                if self.div_apu%8 == 0 {
                    self.ch1.change_envelope();
                    self.ch2.change_envelope();
                    self.ch4.change_envelope();
                }
            }

            self.prev_div_bit = div_bit;

            self.ch1.tick();
            self.ch2.tick();
            self.ch3.tick();
            self.ch4.tick();
        }

        
        // FREQ_CPU/FREQ_AUDIO = 87.38, accounting for both channels = 174.76
        if self.sample_counter == 88 {
            self.add_samples(4);
            self.sample_counter = 0;
        } else {
            self.sample_counter += 1;
        }
    }
}

impl ComponentWithMemory for APU {
    fn read(&self, addr :u16) -> u8 {
        return match addr {
            ADDR_NR10..=ADDR_NR14 => self.ch1.read(addr),
            ADDR_NR21..=ADDR_NR24 => self.ch2.read(addr),
            ADDR_NR30..=ADDR_NR34 => self.ch3.read(addr),
            ADDR_NR41..=ADDR_NR44 => self.ch4.read(addr),
            WAVE_RAM_START..=WAVE_RAM_END => self.ch3.read(addr),

            ADDR_NR50 => self.nr50,
            ADDR_NR51 => self.nr51,
            ADDR_NR52 => {
                return ((self.is_apu_enabled() as u8) << 7)
                     | ((self.ch4.is_enabled() as u8) << 3)
                     | ((self.ch3.is_enabled() as u8) << 2)
                     | ((self.ch2.is_enabled() as u8) << 1)
                     | (self.ch1.is_enabled() as u8)
                     | 0x70;
            }

            _ => panic!("read(): Invalid address: {:04X}", addr)
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        if self.is_apu_enabled() {
            // If the APU is enabled, allow write of all registers
            match addr {
                ADDR_NR10..=ADDR_NR14 => self.ch1.write(addr, val),
                ADDR_NR21..=ADDR_NR24 => self.ch2.write(addr, val),
                ADDR_NR30..=ADDR_NR34 => self.ch3.write(addr, val),
                ADDR_NR41..=ADDR_NR44 => self.ch4.write(addr, val),
                WAVE_RAM_START..=WAVE_RAM_END => self.ch3.write(addr, val),

                ADDR_NR50 => self.nr50 = val,
                ADDR_NR51 => self.nr51 = val,
                ADDR_NR52 => self.write_nr52(val),
                
                _ => panic!("write(): Invalid address: {:04X}", addr)
            }
        } else {
            // Otherwise only allow R/W of wave ram and NR52
            match addr {
                /*ADDR_NR11 => self.ch1.write(addr, val),
                ADDR_NR21 => self.ch2.write(addr, val),
                ADDR_NR31 => self.ch3.write(addr, val),
                ADDR_NR41 => self.ch4.write(addr, val),*/
                ADDR_NR52 => self.write_nr52(val),
                WAVE_RAM_START..=WAVE_RAM_END => self.ch3.write(addr, val),
                _ => {}
            }
        }
    }
}
