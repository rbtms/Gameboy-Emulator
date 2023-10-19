use crate::consts::*;

mod channel1;
mod channel2;
mod channel3;
mod channel4;

pub struct APU {
    nr50 :u8, nr51 :u8, nr52 :u8,
    prev_div_bit: bool,
    div_apu: u8,

    ch1 :channel1::Channel1,
    ch2 :channel2::Channel2,
    ch3 :channel3::Channel3,
    ch4 :channel4::Channel4,
}

impl APU {
    pub fn new(audio :sdl2::AudioSubsystem) -> APU {
        return APU {
            nr50 :0x77, nr51 :0xF3, nr52 :0xF1,
            prev_div_bit: false,
            div_apu: 0,

            ch1: channel1::Channel1::new(),
            ch2: channel2::Channel2::new(),
            ch3: channel3::Channel3::new(),
            ch4: channel4::Channel4::new(),
        }
    }

    pub fn init(&mut self) {
        self.ch1.init();
        self.ch2.init();
        self.ch3.init();
        self.ch4.init();
    }

    pub fn read(&self, addr :u16) -> u8 {
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

    pub fn write(&mut self, addr :u16, val :u8) {
        if self.is_apu_enabled() {
            // If the APU is enabled, allow write of all registers
            match addr {
                ADDR_NR10..=ADDR_NR14 => self.ch1.write(addr, val),
                ADDR_NR21..=ADDR_NR24 => self.ch2.write(addr, val),
                ADDR_NR30..=ADDR_NR34 => self.ch3.write(addr, val),
                ADDR_NR41..=ADDR_NR44 => self.ch4.write(addr, val),
                WAVE_RAM_START..=WAVE_RAM_END => self.ch3.write(addr, val),

                ADDR_NR50 => {
                    self.ch1.set_mix(
                        self.is_bit_set(val, 7), // left
                        self.is_bit_set(val, 0), // right
                    );

                    self.ch2.set_mix(
                        self.is_bit_set(val, 7), // left
                        self.is_bit_set(val, 0), // right
                    );
                    self.nr50 = val;
                }
                ADDR_NR51 => {
                    self.ch1.set_volume((val>>4)&3, val&3); // left, right
                    self.ch2.set_volume((val>>4)&3, val&3); // left, right
                    self.ch3.set_volume((val>>4)&3, val&3); // left, right
                    self.ch4.set_volume((val>>4)&3, val&3); // left, right
                    self.nr51 = val;
                },
                ADDR_NR52 => self.write_nr52(val), // Bits other than 7 are read-only

                _ => panic!("write(): Invalid address: {:04X}", addr)
            }
        } else {
            // Otherwise only allow R/W of wave ram and NR52
            match addr {
                ADDR_NR11 => self.ch1.write(addr, val),
                ADDR_NR21 => self.ch2.write(addr, val),
                ADDR_NR31 => self.ch3.write(addr, val),
                ADDR_NR41 => self.ch4.write(addr, val),
                ADDR_NR52 => self.write_nr52(val),
                WAVE_RAM_START..=WAVE_RAM_END => self.ch3.write(addr, val),
                _ => {}
            }
        }
    }

    // Master control
    fn write_nr52(&mut self, val :u8) {
        let prev_enable = self.is_apu_enabled();
        self.nr52 = (val&0x80) | (self.nr52&0x7F); // Only set the first bit
        let enable = self.is_apu_enabled();

        // Clear registers when PPU is turned off
        if prev_enable && !enable {
            self.ch1.turn_off();
            self.ch2.turn_off();
            self.ch3.turn_off();
            self.ch4.turn_off();

            self.nr50 = 0x00; self.nr51 = 0x00; self.nr52 = 0x00;
            self.div_apu = 0;
        }
    }

    fn is_bit_set(&self, n :u8, b_i :u8) -> bool {
        return (n >> b_i) & 1 == 1;
    }

    // NR52: Sound on/off
    fn is_apu_enabled(&self) -> bool { self.is_bit_set(self.nr52, 7) }
    fn is_ch4_enabled(&self) -> bool { self.is_bit_set(self.nr52, 3) }
    fn is_ch3_enabled(&self) -> bool { self.is_bit_set(self.nr52, 2) }
    fn is_ch2_enabled(&self) -> bool { self.is_bit_set(self.nr52, 1) }
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

    fn inc_ch_length(&mut self) {
        self.ch1.inc_length();
        self.ch2.inc_length();
        self.ch3.inc_length();
        self.ch4.inc_length();
    }

    pub fn tick(&mut self, div :u8) {
        self.ch1.tick();
        self.ch2.tick();
        self.ch3.tick();
        self.ch4.tick();

        let div_bit = self.is_bit_set(div, 4);

        if self.prev_div_bit && !div_bit {
            self.div_apu = self.div_apu.wrapping_add(1);

            // Sound length event every 2 DIV-APU ticks (256hz)
            if self.div_apu%2 == 0 { self.inc_ch_length(); }
            // ch1 frequency sweep event every 4 DIV-APU ticks (128hz)
            if self.div_apu%4 == 0 { self.ch1.change_sweep(); }
            // Envelope length event every 8 DIV-APU ticks (64hz)
            if self.div_apu%8 == 0 { self.ch1.change_envelope(); }
        }

        self.prev_div_bit = div_bit;
    }
}

