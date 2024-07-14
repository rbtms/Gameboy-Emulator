use crate::consts::*;

pub struct RAM {
    wram : [RAMVAL;(WORK_RAM_END-WORK_RAM_START+1) as usize],
    hram     : [RAMVAL;(HRAM_END-HRAM_START+1) as usize],
    io_reg   : [RAMVAL;(IO_REG_END-IO_REG_START+1) as usize]
}

impl RAM {
    pub fn new() -> RAM {
        return RAM {
            wram   : [0;(WORK_RAM_END-WORK_RAM_START+1) as usize],
            hram   : [0;(HRAM_END-HRAM_START+1) as usize],
            io_reg : [0;(IO_REG_END-IO_REG_START+1) as usize]
        }
    }
    
    pub fn init(&mut self) {
        let hardware_regs = [
            (0xFF01, 0x00), (0xFF02, 0x7E), // SB, SC

            (0xFF2A, 0xFF), (0xFF2B, 0xFF), (0xFF2C, 0xFF), (0xFF2D, 0xFF), // Other registers
            (0xFF2E, 0xFF), (0xFF2F, 0xFF),

            (0xFF46, 0xFF),                                                 // DMA
            (0xFF4D, 0xFF), (0xFF4F, 0xFF),                                 // KEY1, VBK

            (0xFF51, 0xFF), (0xFF52, 0xFF), (0xFF53, 0xFF), (0xFF54, 0xFF), // HDMA1-4
            (0xFF55, 0xFF), (0xFF56, 0xFF),                                 // HDMA5, RP

            (0xFF68, 0xFF), (0xFF69, 0xFF), (0xFF6A, 0xFF), (0xFF6B, 0xFF), // BCPS, BCPD, OCPS, OCPD,
            (0xFF70, 0xFF)                                                  // SVBK
        ];

        for (addr, val) in hardware_regs.iter() {
            self.write(*addr, *val);
        }
    }

    pub fn read_io(&self, addr :RAMINDEX) -> RAMVAL {
        // Mask unused bits
        return self.io_reg[(addr-IO_REG_START) as usize] | match addr {    // OR mask
            ADDR_SC   => 0b01111110,
            ADDR_KEY1  | ADDR_VBK   | ADDR_HDMA1 | ADDR_HDMA2 |
            ADDR_HDMA3 | ADDR_HDMA4 | ADDR_HDMA5 | ADDR_RP    |
            ADDR_BCPS  | ADDR_BCPD  | ADDR_OCPS  | ADDR_OCPD  |
            ADDR_SVBK  |
            // Unused registers
            0xFF03 | 0xFF08 | 0xFF09 | 0xFF0A | 0xFF0B | 0xFF0C |
            0xFF0D | 0xFF0E | 0xFF15 | 0xFF1F | 0xFF27 | 0xFF28 |
            0xFF29 | 0xFF4C | 0xFF4E | 0xFF50 | 0xFF57 | 0xFF58 |
            0xFF59 | 0xFF5A | 0xFF5B | 0xFF5C | 0xFF5D | 0xFF5E |
            0xFF5F | 0xFF60 | 0xFF61 | 0xFF62 | 0xFF63 | 0xFF64 |
            0xFF65 | 0xFF66 | 0xFF67 | 0xFF6C | 0xFF6D | 0xFF6E |
            0xFF6F | 0xFF71 | 0xFF72 | 0xFF73 | 0xFF74 | 0xFF75 |
            0xFF76 | 0xFF77 | 0xFF78 | 0xFF79 | 0xFF7A | 0xFF7B |
            0xFF7C | 0xFF7D | 0xFF7E | 0xFF7F
            
            // Other registers
            | 0xFF2A  | 0xFF2B | 0xFF2C  | 0xFF2D  | 0xFF2E | 0xFF2F
              => 0xFF, // 11111111
            _ => 0x00 // Otherwise dont OR mask anything
        }
    }
}

impl ComponentWithMemory for RAM {
    fn read(&self, addr :RAMINDEX) -> RAMVAL {
        return match addr {
            WORK_RAM_START..=WORK_RAM_END => self.wram[(addr-WORK_RAM_START) as usize],
            // Its mapped to work ram
            ECHO_RAM_START..=ECHO_RAM_END => self.wram[(addr-0x2000-WORK_RAM_START) as usize],
            IO_REG_START..=IO_REG_END     => self.read_io(addr),
            HRAM_START..=HRAM_END         => self.hram[(addr-HRAM_START) as usize],
            _ => panic!("read(): Invalid address: 0x{:04X}", addr)
        }
    }
    
    fn write(&mut self, addr :RAMINDEX, val :RAMVAL) {
        return match addr {
            WORK_RAM_START..=WORK_RAM_END => self.wram[(addr-WORK_RAM_START) as usize] = val,
            // Its mapped to work ram
            ECHO_RAM_START..=ECHO_RAM_END => self.wram[(addr-0x2000-WORK_RAM_START) as usize] = val,
            IO_REG_START..=IO_REG_END     => self.io_reg[(addr-IO_REG_START) as usize] = val,
            HRAM_START..=HRAM_END         => self.hram[(addr-HRAM_START) as usize] = val,
            _ => panic!("write(): Invalid address: 0x{:04X}", addr)
        }
    }
}