use std::cell::RefCell;
use std::rc::Rc;

use crate::ppu::PPU;
use crate::apu::APU;
use crate::interruptManager::InterruptManager;
use crate::joypad::Joypad;
use crate::cartridge::{Cartridge, load_cartridge};
use crate::consts::*;

mod ram;
mod timer;

pub struct Bus {
    ram    : ram::RAM,
    timer  : timer::Timer,
    ppu    : PPU,
    apu    : APU,
    int    : Rc<RefCell<InterruptManager>>,
    joypad : Rc<RefCell<Joypad>>,
    cart   : Box<dyn Cartridge>,

    is_oam_dma: bool,
    wait_oam_dma :u8, // Wait for 4 cycles until the OAM DMA actually starts
    dma_src_addr: u16,
    dma_dst_addr: u16,
    schedule_oam_dma: bool,
    dma_until_next_m_cycle: u8,
}

impl Bus {
    pub fn new(ppu      :PPU,
               apu      :APU,
               int      :Rc<RefCell<InterruptManager>>,
               joypad   :Rc<RefCell<Joypad>>,
               path     :&str) -> Bus {
        return Bus {
            ram   : ram::RAM::new(),
            timer : timer::Timer::new(int.clone()),
            cart  : load_cartridge(path),
            ppu,
            apu,
            int,
            joypad,

            is_oam_dma: false,
            dma_src_addr: 0x0000,
            dma_dst_addr: 0x0000,
            schedule_oam_dma: false,
            wait_oam_dma: 0,
            dma_until_next_m_cycle: 0,
        };
    }

    pub fn init(&mut self) {
        self.ram.init();
        self.ppu.init();
        self.apu.init();
        self.cart.init();
    }

    // TODO: For debugger. Remove.
    pub fn timer_counter(&self) -> u16 { return self.timer.timer_counter(); }
    pub fn div_counter(&self)   -> u16 { return self.timer.div_counter(); }

    pub fn read_oam_dma(&self, addr :u16) -> u8 { 
        match addr {
            HRAM_START..=HRAM_END => self.ram.read(addr),
            // For mooneye oam_dma/reg_read, even though the CPU is supposed to
            // only be able to access High RAM while OAM-DMA
            ADDR_DMA => self.ram.read(addr),
            _ => 0x90
        }
    }

    fn write_oam_dma(&mut self, addr :u16, val :u8) {
        match addr {
            HRAM_START..=HRAM_END => self.ram.write(addr, val),
            ADDR_DMA => self.ram.write(addr, val),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.ppu.tick();    // TODO: Possible delay of 1 cycle on OAM DMA
        self.timer.tick();
        self.apu.tick(self.read(ADDR_DIV));

        // Wait for 5 cycles (actual cycle + 1 M-cycle to start OAM-DMA
        if self.schedule_oam_dma {
            self.wait_oam_dma -= 1;

            if self.wait_oam_dma == 0 {
                self.schedule_oam_dma = false;
                self.is_oam_dma = true;
                self.ppu.set_oam_dma(true);
                self.dma_until_next_m_cycle = 1; // Wait until next M-cycle
            }
        } else if self.is_oam_dma {
            self.dma_until_next_m_cycle -= 1;

            if self.dma_until_next_m_cycle == 0 {
                //println!("dma move");

                self.is_oam_dma = false; // Deactivate momentarily to be able to read
                let val = self.read(self.dma_src_addr);
                self.is_oam_dma = true; // Enable it again so that the cpu can only access HRAM

                self.ppu.write_oam_dma(self.dma_dst_addr, val);
                self.dma_src_addr += 1;
                self.dma_dst_addr += 1;
                self.dma_until_next_m_cycle = 4;

                // The last OAM address is 
                if self.dma_dst_addr == OAM_END+1 {
                    //println!("dma end");
                    self.is_oam_dma = false;
                    self.ppu.set_oam_dma(false);
                }
            }
        }
    }

    pub fn save_ram(&self) {
        self.cart.save_ram();
    }
}

impl ComponentWithMemory for Bus {
    fn read(&self, addr :u16) -> u8 {
        if self.cart.is_test_cart() {
            return self.cart.read(addr); // For tests. Remove.
        }

        if self.is_oam_dma {
            return self.read_oam_dma(addr);
        }
        else {
            return match addr {
                BANK0_START..=BANK0_END       => self.cart.read(addr),
                BANK1_START..=BANK1_END       => self.cart.read(addr),
                VRAM_START..=VRAM_END         => self.ppu.read(addr),
                EXT_RAM_START..=EXT_RAM_END   => self.cart.read(addr),
                WORK_RAM_START..=WORK_RAM_END => self.ram.read(addr),
                ECHO_RAM_START..=ECHO_RAM_END => self.ram.read(addr),
                OAM_START..=OAM_END           => self.ppu.read(addr),
                // Not usable
                0xFEA0..=0xFEFF               => 0x00,
                HRAM_START..=HRAM_END         => self.ram.read(addr),
                // PPU
                ADDR_LY   | ADDR_LYC  | ADDR_WY   | ADDR_WX   |
                ADDR_SCY  | ADDR_SCX  | ADDR_BGP  | ADDR_OBP0 |
                ADDR_OBP1 | ADDR_LCDC | ADDR_STAT
                    => self.ppu.read(addr),
                // APU
                ADDR_NR10 | ADDR_NR11 | ADDR_NR12 | ADDR_NR13 | ADDR_NR14 |
                ADDR_NR21 | ADDR_NR22 | ADDR_NR23 | ADDR_NR24 |
                ADDR_NR30 | ADDR_NR31 | ADDR_NR32 | ADDR_NR33 | ADDR_NR34 |
                ADDR_NR41 | ADDR_NR42 | ADDR_NR43 | ADDR_NR44 |
                ADDR_NR50 | ADDR_NR51 | ADDR_NR52 | WAVE_RAM_START..=WAVE_RAM_END
                    => self.apu.read(addr),
                // Joypad
                ADDR_P1
                    => self.joypad.borrow_mut().read(addr),
                // Timer
                ADDR_DIV | ADDR_TIMA | ADDR_TMA | ADDR_TAC
                    => self.timer.read(addr),
                // Interrupts
                ADDR_IE | ADDR_IF
                    => self.int.borrow().read(addr),
                // General RAM
                _ => self.ram.read(addr)
            }
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        if self.cart.is_test_cart() {
            return self.cart.write(addr, val); // For tests. Remove.
        }

        // Intercept DMA address to start OAM DMA
        if addr == ADDR_DMA {
            //println!("DMA {:04X}", (val as u16)<<8);
            self.is_oam_dma = false;
            self.schedule_oam_dma = true;
            self.wait_oam_dma = 8; // Supposing the bus executes
                                   // right after the CPU

            self.dma_src_addr = (val as u16) << 8;
            self.dma_dst_addr = 0xFE00;
        }

        if self.is_oam_dma {
            self.write_oam_dma(addr, val);
        } else {
            match addr {
                BANK0_START..=BANK0_END       => self.cart.write(addr, val),
                BANK1_START..=BANK1_END       => self.cart.write(addr, val),
                VRAM_START..=VRAM_END         => self.ppu.write(addr, val),
                EXT_RAM_START..=EXT_RAM_END   => self.cart.write(addr, val),
                WORK_RAM_START..=WORK_RAM_END => self.ram.write(addr, val),
                ECHO_RAM_START..=ECHO_RAM_END => self.ram.write(addr, val),
                OAM_START..=OAM_END           => self.ppu.write(addr, val),
                // Not usable
                0xFEA0..=0xFEFF               => {},
                HRAM_START..=HRAM_END         => self.ram.write(addr, val),
                // PPU
                ADDR_LY   | ADDR_LYC  | ADDR_WY   | ADDR_WX   |
                ADDR_SCY  | ADDR_SCX  | ADDR_BGP  | ADDR_OBP0 |
                ADDR_OBP1 | ADDR_LCDC | ADDR_STAT
                    => self.ppu.write(addr, val),
                // APU
                ADDR_NR10 | ADDR_NR11 | ADDR_NR12 | ADDR_NR13 | ADDR_NR14 |
                ADDR_NR21 | ADDR_NR22 | ADDR_NR23 | ADDR_NR24 |
                ADDR_NR30 | ADDR_NR31 | ADDR_NR32 | ADDR_NR33 | ADDR_NR34 |
                ADDR_NR41 | ADDR_NR42 | ADDR_NR43 | ADDR_NR44 |
                ADDR_NR50 | ADDR_NR51 | ADDR_NR52 | WAVE_RAM_START..=WAVE_RAM_END
                    => self.apu.write(addr, val),
                // Joypad
                ADDR_P1 =>
                    self.joypad.borrow_mut().write(addr, val),
                // Timer
                ADDR_DIV | ADDR_TIMA | ADDR_TMA | ADDR_TAC
                    => self.timer.write(addr, val),
                    // Interrupots
                ADDR_IE | ADDR_IF
                    => self.int.borrow_mut().write(addr, val),
                // General RAM
                _ => {
                    self.ram.write(addr, val);
                }
            }
        }
    }
}