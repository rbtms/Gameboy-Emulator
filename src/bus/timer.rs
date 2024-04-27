use std::cell::RefCell;
use std::rc::Rc;

use crate::interruptManager::InterruptManager;
use crate::consts::*;

pub struct Timer {
    timer_counter   :u16,

    tma: u8,
    tima: u8,
    tac: u8,
    old_tma: u8,

    // Request to set TIMA
    is_wait_set_tima :bool,
    wait_tima        :u8,
    int :Rc<RefCell<InterruptManager>>,
}

impl Timer {
    pub fn new(int :Rc<RefCell<InterruptManager>>) -> Timer {
        return Timer {
            timer_counter: 0xABCC,

            // Boot values
            tima    : 0,
            tma     : 0,
            tac     : 0xF8,
            old_tma : 0,

            wait_tima: 0,
            is_wait_set_tima :false,

            int,
        }
    }

    // TODO: For debugger. Remove.
    pub fn timer_counter(&self) -> u16 { return self.timer_counter; }
    pub fn div_counter(&self)   -> u16 { return self.timer_counter; }

    pub fn read(&self, addr :u16) -> u8 {
        return match addr {
            ADDR_DIV  => { (self.timer_counter>>8) as u8 },
            ADDR_TMA  => self.tma,
            ADDR_TIMA => self.tima,
            ADDR_TAC  => self.tac | 0b11111000, // bits 3-7 are always 1
            _ => panic!("read(): Invalid address: {:04X}", addr)
        }
    }

    pub fn write(&mut self, addr :u16, val :u8) {
        match addr {
            ADDR_TAC => {
                let prev_mux_output = self.div_mux_output();
                let prev_enable = self.is_timer_enabled();
                
                // TAC value changes
                if (self.tac&0x03) != (val & 0x03) {
                    //println!("[TAC Change] {} -> {}", self.tac&3, val & 3); // TODO: Reenable
                }

                // Falling edge detector
                self.tac = val;
                let mux_output = self.div_mux_output();
                let enable = self.is_timer_enabled();

                if self.is_falling_edge(prev_mux_output, mux_output, prev_enable, enable) {
                    self.inc_tima_falling();
                }

                /*
                else if enable && !prev_mux_output && mux_output {
                    self.inc_tima();
                }*/
            },
            ADDR_TMA => {
                self.old_tma = self.tma;
                self.tma = val;
            },
            ADDR_DIV => {
                let enable = self.is_timer_enabled();
                let prev_mux_output = self.div_mux_output();
                self.timer_counter = 0;
                let new_mux_output = self.div_mux_output();

                //self.has_div_been_reset = true;

                // Detect a falling edge
                if self.is_falling_edge(prev_mux_output, new_mux_output, enable, enable) {
                    self.inc_tima_falling();
                }
            },
            ADDR_TIMA => {
                self.tima = val;
            },
            _ => panic!("write(): Invalid address: {:04X}", addr)
        }
    }

    pub fn is_timer_enabled(&self) -> bool {
        return (self.tac >> 2) & 1 == 1;
    }

    pub fn div_mux_output(&self) -> bool {
        return match self.tac & 3 {
            0 => (self.timer_counter >> 9) & 1 == 1,
            1 => (self.timer_counter >> 3) & 1 == 1,
            2 => (self.timer_counter >> 5) & 1 == 1,
            3 => (self.timer_counter >> 7) & 1 == 1,
            _ => panic!()
        }
    }

    /* Detect if the DIV/TAC mux output goes from 1 to 0 */
    pub fn is_falling_edge(&self,
        prev_mux_out :bool, mux_out :bool,
        prev_enable  :bool, enable  :bool) -> bool {
            return (prev_enable && prev_mux_out) && !(enable && mux_out);
    }

    /* Incrementing tima through natural incrementation triggers strange cycles */
    pub fn inc_tima(&mut self) {
        if self.is_timer_enabled() {
            if self.tima == 0xFF {
                self.is_wait_set_tima = true;
                self.wait_tima = 4;
                self.tima = 0x00; // Strange cycle
            }
            else {
                self.tima = self.tima.wrapping_add(1);
            }
        }
    }

    /* Incrementing tima through the modification of other registers does not trigger strange
     * cycles */
    pub fn inc_tima_falling(&mut self) {
        if self.tima == 0xFF {
            self.tima = self.tma;
            self.int.borrow_mut().request_interrupt(Interrupt::Timer);
        }
        else {
            self.tima = self.tima.wrapping_add(1);
        }
    }

    pub fn tick(&mut self) {
        if self.old_tma == 0 {
            self.old_tma = self.tma;
        }

        if self.is_wait_set_tima {
            self.wait_tima -= 1;

            if self.wait_tima == 0 {
                self.is_wait_set_tima = false;

                self.tima = self.tma;
                self.int.borrow_mut().request_interrupt(Interrupt::Timer);
            }
        }

        // Update div at 16384hz = once every 256 cycles
        // Dont update it if it has been reset this cycle.
        let old_mux_bit = self.div_mux_output();
        self.timer_counter = self.timer_counter.wrapping_add(1);
        let new_mux_bit = self.div_mux_output();

        let enable = self.is_timer_enabled();

        if self.is_falling_edge(old_mux_bit, new_mux_bit, enable, enable) {
            self.inc_tima();
        }
    }
}
