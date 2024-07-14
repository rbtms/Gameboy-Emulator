use std::cell::RefCell;
use std::rc::Rc;

use crate::interruptManager::InterruptManager;
use crate::consts::*;

pub struct Joypad {
    int: Rc<RefCell<InterruptManager>>,
    p1: u8,
    mask: u8,
    direction: u8,
    action: u8,

    is_controller_up: bool,
    is_controller_down: bool,
    is_controller_left: bool,
    is_controller_right: bool,
}
impl Joypad {
    pub fn new(int : Rc<RefCell<InterruptManager>>) -> Joypad {
        return Joypad {
            p1: 0b11001111,
            mask: 0,
            int,
            direction: 0b11111111,
            action: 0b11111111,

            is_controller_up: false,
            is_controller_down: false,
            is_controller_left: false,
            is_controller_right: false,
        };
    }

    pub fn init(&mut self) {
        self.p1 = 0b11001111; // Initial boot value
        self.mask = 0b11111111;
        self.direction = 0b11111111;
        self.action = 0b11111111;
    }

    /*
     * Bit 7 - Not used
     * Bit 6 - Not used
     * Bit 5 - P15 Select Action buttons    (0=Select)
     * Bit 4 - P14 Select Direction buttons (0=Select)
     * Bit 3 - P13 Input: Down  or Start    (0=Pressed)
     * Bit 2 - P12 Input: Up    or Select   (0=Pressed)
     * Bit 1 - P11 Input: Left  or B        (0=Pressed)
     * Bit 0 - P10 Input: Right or A        (0=Pressed)
     */
    pub fn press_start(&mut self)  { self.press(0b11010111); }
    pub fn press_select(&mut self) { self.press(0b11011011); }
    pub fn press_b(&mut self)      { self.press(0b11011101); }
    pub fn press_a(&mut self)      { self.press(0b11011110); }
    pub fn press_down(&mut self)   { self.press(0b11100111); }
    pub fn press_up(&mut self)     { self.press(0b11101011); }
    pub fn press_left(&mut self)   { self.press(0b11101101); }
    pub fn press_right(&mut self)  { self.press(0b11101110); }

    pub fn release_start(&mut self)  { self.release(0b11010111); }
    pub fn release_select(&mut self) { self.release(0b11011011); }
    pub fn release_b(&mut self)      { self.release(0b11011101); }
    pub fn release_a(&mut self)      { self.release(0b11011110); }
    pub fn release_down(&mut self)   { self.release(0b11100111); }
    pub fn release_up(&mut self)     { self.release(0b11101011); }
    pub fn release_left(&mut self)   { self.release(0b11101101); }
    pub fn release_right(&mut self)  { self.release(0b11101110); }

    pub fn controller_left(&mut self) {
        if self.is_controller_right {
            self.is_controller_right = false;
            self.release_right();
        }

        if !self.is_controller_left {
            self.is_controller_left = true;
            self.press_left();
        }
    }

    pub fn controller_right(&mut self) {
        if self.is_controller_left {
            self.is_controller_left = false;
            self.release_left();
        }

        if !self.is_controller_right {
            self.is_controller_right = true;
            self.press_right();
        }
    }

    pub fn controller_up(&mut self) {
        if self.is_controller_down {
            self.is_controller_down = false;
            self.release_down();
        }

        if !self.is_controller_up {
            self.is_controller_up = true;
            self.press_up();
        }
    }

    pub fn controller_down(&mut self) {
        if self.is_controller_up {
            self.is_controller_up = false;
            self.release_up();
        }

        if !self.is_controller_down {
            self.is_controller_down = true;
            self.press_down();
        }
    }

    pub fn controller_no_x(&mut self) {
        if self.is_controller_left { self.release_left(); self.is_controller_left = false; }
        if self.is_controller_right { self.release_right(); self.is_controller_right = false; }
    }

    pub fn controller_no_y(&mut self) {
        if self.is_controller_up { self.release_up(); self.is_controller_up = false; }
        if self.is_controller_down { self.release_down(); self.is_controller_down = false; }
    }

    fn press(&mut self, val :u8) {
        let is_direction = (val>>4)&1 == 0;
        let is_action    = (val>>5)&1 == 0;

        // Keep in mind that the default value is 1, not 0
        if is_direction && self.direction == 0b11111111 { self.direction = val; }
        if is_action && self.action == 0b11111111 { self.action = val; }

        if self.p1 == 0b11111111 {
            self.int.borrow_mut().request_interrupt(Interrupt::Joypad);
        }
    }

    fn release(&mut self, val :u8) {
        let is_direction = (val>>4)&1 == 0;
        let is_action    = (val>>5)&1 == 0;

        if is_direction { self.direction = 0b11111111; }
        if is_action { self.action = 0b11111111; }
    }
}

impl ComponentWithMemory for Joypad {
    fn read(&self, addr: u16) -> u8 {
        if addr == ADDR_P1 {
            let is_direction = (self.mask>>4)&1 == 0;
            let is_action    = (self.mask>>5)&1 == 0;

            if is_direction && is_action { return self.action & self.direction; }
            else if is_direction { return self.direction; }
            else if is_action { return self.action; }

            return 0b11111111;
        } else {
            panic!("Invalid register address: {}", addr)
        }
    }

    fn write(&mut self, addr: u16, val :u8) {
        if addr == ADDR_P1 {
            self.mask = val;
        }
    }
}
