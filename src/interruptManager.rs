use crate::consts::*;

pub struct InterruptManager {
    IE :u8,
    IF :u8,
    IME :bool
}

impl InterruptManager {
    pub fn new() -> InterruptManager {
        return InterruptManager {
            IE: 0,
            IF: 0xE1, // Boot value
            IME :true
        }
    }

    // TODO: Remove. For debugging.
    pub fn get_ime(&self) -> bool {
        return self.IME;
    }

    pub fn set_ime(&mut self, val :bool) {
        self.IME = val;
    }

    pub fn request_interrupt(&mut self, int :Interrupt) {
        self.IF |= match int {
            Interrupt::VBlank => 0b00000001,
            Interrupt::STAT   => 0b00000010,
            Interrupt::Timer  => 0b00000100,
            Interrupt::Serial => 0b00001000,
            Interrupt::Joypad => 0b00010000
        };
    }

    pub fn disable_interrupt_request(&mut self, int :&Interrupt) {
        self.IF &= match int {
            Interrupt::VBlank => 0b11111110,
            Interrupt::STAT   => 0b11111101,
            Interrupt::Timer  => 0b11111011,
            Interrupt::Serial => 0b11110111,
            Interrupt::Joypad => 0b11101111
        };

        self.IME = false; // Disable IME after serving an interruption
    }

    pub fn get_jmp_address(&self, int :&Interrupt) -> u16 {
        return match int {
            Interrupt::VBlank => 0x0040,
            Interrupt::STAT   => 0x0048,
            Interrupt::Timer  => 0x0050,
            Interrupt::Serial => 0x0058,
            Interrupt::Joypad => 0x0060
        };
    }

    pub fn get_interrupt(&self) -> Option<Interrupt> {
        let i_and = self.IF & self.IE;

        if self.IME {
            if      i_and&1      == 1 { return Some(Interrupt::VBlank); }
            else if (i_and>>1)&1 == 1 { return Some(Interrupt::STAT); }
            else if (i_and>>2)&1 == 1 { return Some(Interrupt::Timer); }
            else if (i_and>>3)&1 == 1 { return Some(Interrupt::Serial); }
            else if (i_and>>4)&1 == 1 { return Some(Interrupt::Joypad); }
        }

        return None;
    }

    pub fn has_interrupts(&self) -> bool {
        return self.IME && ((self.IF & self.IE & 0x1f) != 0); // 0x1f to ignore the first 3 bits
    }
}

impl ComponentWithMemory for InterruptManager {
    fn read(&self, addr :u16) -> u8 {
        return match addr {
            ADDR_IE => self.IE, // A mask of 0b11100000 makes some tests fail
            // Test 2 of blargg fails with this mask
            ADDR_IF => self.IF | 0b11100000, // bits 5-7 are always 1
            _ => panic!("read(): Invalid address: {:04X}", addr)
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        match addr {
            ADDR_IE => self.IE = val,
            ADDR_IF => self.IF = val,
            _ => panic!("write(): Invalid address: {:04X}", addr)
        }
    }
}
