use std::cell::RefCell;
use std::rc::Rc;

use crate::cpu::CPU;
use crate::ppu::PPU;
use crate::apu::APU;
use crate::bus::Bus;
use crate::joypad::Joypad;
use crate::interruptManager::InterruptManager;
use crate::screen::Screen;

extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod clock;
use clock::Clock;

// Global constants
const TICKS_PER_FRAME :u32 = 69905; // 4194304 hz / 60fps/s

pub struct GBEmulator {
//    path            : String,
    cpu             : CPU,
    bus             : Rc<RefCell<Bus>>,
    joypad          : Rc<RefCell<Joypad>>,
    clock           : Clock,

    screen          : Rc<RefCell<Screen>>,

    #[allow(dead_code)]
    controller      : sdl2::controller::GameController,

    // Input
    events :sdl2::EventPump,
    is_quit :bool,
}

impl GBEmulator {
    pub fn new(rom_path: &str, screen_mult: u8) -> GBEmulator {
        let sdl_context = sdl2::init().unwrap();

        let screen = Rc::new(RefCell::new(
            Screen::new(&sdl_context, rom_path.to_string(), screen_mult)
        ));

        let audio = sdl_context.audio().unwrap();
        let apu   = APU::new(audio);

        let int: Rc<RefCell<InterruptManager>> = Rc::new(RefCell::new(
            InterruptManager::new()
        ));

        let ppu = PPU::new(screen.clone(), int.clone());

        let joypad = Rc::new(RefCell::new(
            Joypad::new(int.clone())
        ));

        let bus  = Rc::new(RefCell::new(
            Bus::new(ppu, apu, int.clone(), joypad.clone(), rom_path)
        ));

        return GBEmulator {
//            path            : rom_path.to_string(),
            cpu             : CPU::new(bus.clone(), int.clone()),
            bus             : bus.clone(),
            joypad          : joypad.clone(),
            clock           : Clock::new(),

            events: sdl_context.event_pump().unwrap(),
            screen: screen.clone(),

            // While the controller variable is not used, it works due to existing and being owned by
            // an existing object.
            controller: sdl_context.game_controller().unwrap().open(0).unwrap(),

            is_quit: false,
        }
    }

    pub fn get_cpu(&self) -> &CPU { return &self.cpu; }
    pub fn get_cpu_mut(&mut self) -> &mut CPU { return &mut self.cpu; }
    pub fn get_bus(&self) -> Rc<RefCell<Bus>> { return self.bus.clone(); }
    pub fn get_screen(&self) -> Rc<RefCell<Screen>> { return self.screen.clone(); }
    pub fn is_quit(&self) -> bool { return self.is_quit; }

    pub fn init(&mut self) {
        self.cpu.init();
        self.bus.borrow_mut().init();
        self.screen.borrow_mut().init();
    }

    pub fn run(&mut self) {
        // Initialize stuff
        self.screen.borrow_mut().clear();

        // Main loop
        while !self.is_quit {
            self.run_frame();
        }

        // Save RAM on quit
        self.bus.borrow().save_ram();
    }

    pub fn run_frame(&mut self) {
        // Time adjustment for 4.19MHz / 60 fps
        self.clock.wait_next_frame();

        // Set title FPS
        let fps = self.clock.get_fps();
        self.screen.borrow_mut().set_title_fps(fps);

        for _ in 0..TICKS_PER_FRAME {
            self.bus.borrow_mut().tick();
            self.cpu.tick();
        }

        // Process input
        self.event_loop();
    }

    fn event_loop(&mut self) {
        for event in self.events.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                    => self.is_quit = true,
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    //self.handle_reload()
                },
                Event::KeyDown { keycode: Some(Keycode::F), .. } => self.joypad.borrow_mut().press_start(),
                Event::KeyUp   { keycode: Some(Keycode::F), .. } => self.joypad.borrow_mut().release_start(),
                Event::KeyDown { keycode: Some(Keycode::G), .. } => self.joypad.borrow_mut().press_select(),
                Event::KeyUp   { keycode: Some(Keycode::G), .. } => self.joypad.borrow_mut().release_select(),
                Event::KeyDown { keycode: Some(Keycode::A), .. } => self.joypad.borrow_mut().press_left(),
                Event::KeyUp   { keycode: Some(Keycode::A), .. } => self.joypad.borrow_mut().release_left(),
                Event::KeyDown { keycode: Some(Keycode::S), .. } => self.joypad.borrow_mut().press_down(),
                Event::KeyUp   { keycode: Some(Keycode::S), .. } => self.joypad.borrow_mut().release_down(),
                Event::KeyDown { keycode: Some(Keycode::W), .. } => self.joypad.borrow_mut().press_up(),
                Event::KeyUp   { keycode: Some(Keycode::W), .. } => self.joypad.borrow_mut().release_up(),
                Event::KeyDown { keycode: Some(Keycode::D), .. } => self.joypad.borrow_mut().press_right(),
                Event::KeyUp   { keycode: Some(Keycode::D), .. } => self.joypad.borrow_mut().release_right(),
                Event::KeyDown { keycode: Some(Keycode::K), .. } => self.joypad.borrow_mut().press_b(),
                Event::KeyUp   { keycode: Some(Keycode::K), .. } => self.joypad.borrow_mut().release_b(),
                Event::KeyDown { keycode: Some(Keycode::L), .. } => self.joypad.borrow_mut().press_a(),
                Event::KeyUp   { keycode: Some(Keycode::L), .. } => self.joypad.borrow_mut().release_a(),
                Event::ControllerButtonDown { button, .. } => {
                    match button {
                        sdl2::controller::Button::A => self.joypad.borrow_mut().press_a(),
                        sdl2::controller::Button::X => self.joypad.borrow_mut().press_b(),
                        sdl2::controller::Button::Start => self.joypad.borrow_mut().press_start(),
                        sdl2::controller::Button::Back => self.joypad.borrow_mut().press_select(),
                        _ => {}
                    }
                },
                Event::ControllerButtonUp { button, .. } => {
                    match button {
                        sdl2::controller::Button::A => self.joypad.borrow_mut().release_a(),
                        sdl2::controller::Button::X => self.joypad.borrow_mut().release_b(),
                        sdl2::controller::Button::Start => self.joypad.borrow_mut().release_start(),
                        sdl2::controller::Button::Back => self.joypad.borrow_mut().release_select(),
                        _ => {}
                    }
                },
                Event::ControllerAxisMotion { axis, value, .. } => {
                    match axis {
                        // Left: Negative (0..-32000), Down: The opposite
                        sdl2::controller::Axis::LeftX =>  {
                            if      value < -16000 { self.joypad.borrow_mut().controller_left(); }
                            else if value >  16000 { self.joypad.borrow_mut().controller_right(); }
                            else { self.joypad.borrow_mut().controller_no_x(); }
                        },
                        // Up: Negative (0..-32000), Down: The opposite
                        sdl2::controller::Axis::LeftY => {
                            if      value < -16000 { self.joypad.borrow_mut().controller_up(); }
                            else if value >  16000 { self.joypad.borrow_mut().controller_down(); }
                            else { self.joypad.borrow_mut().controller_no_y(); }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
}
