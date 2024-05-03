extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::pixels::PixelFormatEnum;

use crate::consts::*;
use crate::ppu::{Pixel, Palette::*};

const DEFAULT_SCREEN_MULT :u16  = 2;

fn parse_arg_mult() -> u16 {
    let args :Vec<String> = std::env::args().collect();

    for arg in args.iter() {
        if arg.contains("--mult") {
            return arg.split_once('=').unwrap().1.parse().unwrap();
        }
    }

    return DEFAULT_SCREEN_MULT;
}

pub struct Screen {
    canvas :sdl2::render::Canvas<sdl2::video::Window>,
    rom_path :String,
    screen_mult :u16,
    framebuffer :[u8;(SCREEN_WIDTH*SCREEN_HEIGHT) as usize],
    rects: Vec<Vec<Rect>>
}

impl Screen {
    pub fn new(sdl_context :&sdl2::Sdl, rom_path :String) -> Screen {
        let video_subsystem = sdl_context.video().unwrap();
        let screen_mult = parse_arg_mult();
        let window = video_subsystem.window(
            &rom_path,
            screen_mult as u32 *SCREEN_WIDTH as u32,
            screen_mult as u32 *SCREEN_HEIGHT as u32
        ).build().unwrap();
        let canvas = window.into_canvas().build().unwrap();

        return Screen {
            canvas,
            rom_path,
            screen_mult,
            // Initialize as 4 to force a first draw (id values go from 0 to 3)
            framebuffer: [4;(SCREEN_WIDTH*SCREEN_HEIGHT) as usize],
            rects: vec![vec![], vec![], vec![], vec![]]
        }
    }

    pub fn init(&mut self) {
        self.canvas.set_draw_color(Color::WHITE);
        self.canvas.clear();
        self.canvas.present();
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::WHITE);
        self.canvas.clear();
        self.canvas.present();
    }

    pub fn get_pixels(&self) -> Vec<u8> {
        return self.canvas.read_pixels(
            Rect::new(0, 0,
                (SCREEN_WIDTH*self.screen_mult) as u32, (SCREEN_HEIGHT*self.screen_mult) as u32
            ),
            PixelFormatEnum::RGBA32
        ).unwrap()
    }

    pub fn get_pixel_color_index(&self, pixel :Pixel, bgp :u8, obp0 :u8, obp1 :u8) -> u8 {
        return match pixel.get_palette() {
            BGP  => (bgp  >> (2*(pixel.get_id())))&3,
            OBP0 => (obp0 >> (2*(pixel.get_id())))&3,
            OBP1 => (obp1 >> (2*(pixel.get_id())))&3
        }
    }

    pub fn has_line_changed(&self, linebuffer :&[u8], y :u8) -> bool {
        for x in 0..SCREEN_WIDTH as usize {
            if linebuffer[x] != self.framebuffer[y as usize * SCREEN_WIDTH as usize + x] {
                return true;
            }
        }

        return false;
    }

    pub fn update_framebuffer(&mut self, linebuffer :&[u8], y :u8) {
        for x in 0..SCREEN_WIDTH as usize {
            self.framebuffer[y as usize * SCREEN_WIDTH as usize + x] = linebuffer[x];
        }
    }

    pub fn draw_linebuffer(&mut self, linebuffer :&Vec<Pixel>, y :u8, bgp :u8, obp0 :u8, obp1 :u8) {
        // Temporary linebuffer to prevent drawing if there is no change
        let mut tmp_linebuffer :[u8;SCREEN_WIDTH as usize] = [0;SCREEN_WIDTH as usize];

        for x in 0..SCREEN_WIDTH {
            let x_mult = self.screen_mult*x;
            let y_mult = self.screen_mult*y as u16;
            let id = self.get_pixel_color_index(linebuffer[x as usize], bgp, obp0, obp1);

            self.rects[id as usize].push(
                Rect::new(x_mult as i32, y_mult as i32, self.screen_mult as u32, self.screen_mult as u32)
            );
            tmp_linebuffer[x as usize] = id;
        }

        /*if self.has_line_changed(&tmp_linebuffer, y) {
            self.update_framebuffer(&tmp_linebuffer, y);
        }*/

        if y as u16 == SCREEN_HEIGHT-1 {
            self.draw_frame();
        }
    }

    pub fn draw_frame(&mut self) {
        self.canvas.set_draw_color(Color::WHITE);
        self.canvas.fill_rects(&self.rects[0b00]).unwrap();
        self.canvas.set_draw_color(Color::RGB(160, 160, 160));
        self.canvas.fill_rects(&self.rects[0b01]).unwrap();
        self.canvas.set_draw_color(Color::RGB(100, 100, 100));
        self.canvas.fill_rects(&self.rects[0b10]).unwrap();
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.fill_rects(&self.rects[0b11]).unwrap();

        self.canvas.present();

        self.rects = vec![vec![], vec![], vec![], vec![]]
    }

    pub fn set_title_fps(&mut self, fps :u16) {
        self.canvas.window_mut().set_title(&format!("fps {} | {}", fps, self.rom_path)).unwrap();
    }
}
