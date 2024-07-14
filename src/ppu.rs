use std::rc::Rc;
use std::cell::RefCell;

extern crate sdl2;

use crate::screen::Screen;
use crate::interruptManager::InterruptManager;
use crate::consts::*;

const TILE_W    :u16 = 8;
const TILE_H    :u16 = 8;
const TILE_VRAM_SIZE :u16 = 16; // 16 byte long

const ADDR_VRAM_0     :u16 = 0x8000;
//const ADDR_VRAM_1     :u16 = 0x8800;
const ADDR_VRAM_2     :u16 = 0x9000;
const ADDR_TILEMAPS_0 :u16 = 0x9800;
const ADDR_TILEMAPS_1 :u16 = 0x9C00;
//const ADDR_OAM        :u16 = 0xFE00;

/*
 * TODO: Midframe clock pause
 */

#[derive(PartialEq, Debug)]
enum STATMode {
    HBlank    = 0,
    VBlank    = 1,
    OAMSearch = 2,
    Drawing   = 3
}

#[derive(Debug, Copy)]
pub enum Palette {
    BGP,
    OBP0,
    OBP1
}

impl Clone for Palette {
    fn clone(&self) -> Palette {
        return *self;
    }
}

struct ObjFlags {
    bg_over_obj: bool,
    y_flip: bool,
    x_flip: bool,
    palette_n: bool
}

struct Object {
    y: u8,
    x: u8,
    tile_index: u8,
    flags: ObjFlags
}

#[derive(Debug, Copy)]
struct ObjPixel {
    color_id    : u8,
    x_priority  : u8,
    bg_over_obj : bool,
    palette     : Palette
}

impl Clone for ObjPixel {
    fn clone(&self) -> ObjPixel {
        return ObjPixel {
            color_id     : self.color_id,
            x_priority   : self.x_priority,
            bg_over_obj  : self.bg_over_obj,
            palette      : self.palette
        }
    }
}

#[derive(Debug, Copy)]
pub struct Pixel {
    color_id: u8,
    palette: Palette
}
impl Clone for Pixel {
    fn clone(&self) -> Pixel {
        return Pixel {
            color_id : self.color_id,
            palette  : self.palette
        }
    }
}
impl Pixel {
    pub fn get_id(&self) -> u8 { return self.color_id; }
    pub fn get_palette(&self) -> Palette { return self.palette; }
}

pub struct PPU {
    screen      : Rc<RefCell<Screen>>,
    vram        : [u8;0x2000],  // 0x8000 - 0x9FFF
    oam         : [u8;0xA0],    // 0xFE00 - 0xFE9F
    current_dot : u16,
    linebuffer  : Vec<Pixel>,
    line_objs   : Vec<Object>,
    int         : Rc<RefCell<InterruptManager>>,
    ly   :u8, lyc  :u8, scx  :u8, scy  :u8,
    wx   :u8, wy   :u8, ldcd :u8, stat :u8,
    bgp  :u8, obp0 :u8, obp1 :u8,

    is_window_enable: bool,
    window_counter: u8,
    is_oam_dma: bool,

    // Skip first frame at the start or after the ppu has been disabled
    // (helps avoiding artifacts on tests)
    has_drawn_first_frame: bool,
}

impl PPU {
    pub fn new(screen :Rc<RefCell<Screen>>, int :Rc<RefCell<InterruptManager>>) -> PPU {
        return PPU {
            screen,
            vram        : [0;0x2000],
            oam         : [0;(OAM_END-OAM_START+1) as usize],
            current_dot : 0,
            linebuffer  : vec![],
            line_objs   : vec![],
            int,
            // Boot values
            ly : 0, lyc: 0, scx : 0,    scy : 0,
            wy : 0, wx : 0, ldcd: 0x91, stat: 0x85,
            bgp: 0xFC, obp0: 0xFC, obp1: 0xFC,

            is_window_enable : false,
            window_counter   : 0,

            is_oam_dma       : false,

            has_drawn_first_frame : false,
        };
    }

    pub fn init(&mut self) {}

    pub fn set_oam_dma(&mut self, val :bool) {
        self.is_oam_dma = val;
    }

    // Allow the bus to write to OAM when OAM DMA is active
    pub fn write_oam_dma(&mut self, addr :u16, val :u8) {
        self.oam[(addr-OAM_START) as usize] = val;
    }

    fn can_access_vram(&self) -> bool {
        return !(self.is_lcd_enabled() && self.mode() == STATMode::Drawing);
    }

    fn can_access_oam(&self) -> bool {
        return !(
            self.is_lcd_enabled()
        && (self.mode() == STATMode::OAMSearch || self.mode() == STATMode::Drawing)
        );
    }

    // For internal use
    fn vram(&self, addr :u16) -> u8 {
        return self.vram[(addr-VRAM_START) as usize];
    }
    // For internal use
    fn oam(&self, addr :u16) -> u8 {
        if self.is_oam_dma { return 0xFF; }
        else { return self.oam[(addr-OAM_START) as usize]; }
    }

    fn write_ldcd(&mut self, val :u8) {
        self.ldcd = val;

        if !self.is_lcd_enabled() {
            self.write_ly(0);
            self.set_mode(STATMode::VBlank);
        }
    }

    fn write_ly(&mut self, val :u8) {
        self.ly = val;
        self.current_dot = 0;

        self.check_eq_ly_lyc();

        // VBlank period interrupt
        if self.ly == 144 {
            self.window_counter = 0;
            self.set_mode(STATMode::VBlank);
            self.int.borrow_mut().request_interrupt(Interrupt::VBlank);
        }
        // Reset LY
        else if self.ly == 154 {
            self.ly = 0;
            self.has_drawn_first_frame = true;
        }
    }

    fn check_eq_ly_lyc(&mut self) {
        // STAT interrupt if LY == LYC
        if self.ly == self.lyc {
            self.stat |= 0b00000100; // bit 2 is set when LY == LYC
            self.stat |= 0b01000000; // set LY == LYC as the interrupt source
            self.int.borrow_mut().request_interrupt(Interrupt::STAT);
        }
        // LY != LYC
        else {
            // Disable flags
            self.stat &= 0b10111011;
        }
    }

    fn mode(&self) -> STATMode {
        return match self.stat & 3 {
            0 => STATMode::HBlank,
            1 => STATMode::VBlank,
            2 => STATMode::OAMSearch,
            3 => STATMode::Drawing,
            _ => panic!()
        }
    }

    fn set_mode(&mut self, mode :STATMode) {
        let mode_n = mode as u8;
        self.stat = (self.stat & 0xFC) | mode_n;

        // Set STAT interrupt source
        if mode_n <= 3 {
            self.stat = (self.stat & 0xFC) | mode_n;

            // Request an interrupt if the mode coincides with the selected interrupt mode
            // Bit 3: Mode 0, Bit 4: Mode 1, Bit 5: Mode 2, Bit 6: LCY=LY
            if self.stat & (1<<(mode_n+3)) == 1 {
                self.int.borrow_mut().request_interrupt(Interrupt::STAT);
            }
        } else {
            panic!("Invalid mode.");
        }
    }

    /* LDCD */
    fn is_set(&self, n :u8, i :u8)  -> bool { return (n >> i) & 1 == 1; }
    fn is_lcd_enabled(&self)        -> bool { return self.is_set(self.ldcd, 7); }
    fn window_tilemap_area(&self)   -> u16  { return if self.is_set(self.ldcd, 6) {ADDR_TILEMAPS_1} else {ADDR_TILEMAPS_0}; }
    fn is_window_enabled(&self)     -> bool { return self.is_set(self.ldcd, 5); }
    fn bg_win_tile_data_area(&self) -> bool { return self.is_set(self.ldcd, 4); }
    fn bg_tilemap_area(&self)       -> u16  { return if self.is_set(self.ldcd, 3) {ADDR_TILEMAPS_1} else {ADDR_TILEMAPS_0}; }
    fn obj_size(&self)              -> u8   { return if self.is_set(self.ldcd, 2) {16} else {8}; }
    fn are_objs_enabled(&self)      -> bool { return self.is_set(self.ldcd, 1); }
    fn is_bg_window_enabled(&self)  -> bool { return self.is_set(self.ldcd, 0); }

    fn get_bg_tilemap_addr(&self, y :u16, x :u16) -> u16 {
        return self.vram(self.bg_tilemap_area() + y*32 + x) as u16;
    }
    fn get_window_tilemap_addr(&self, y :u16, x :u16) -> u16 {
        return self.vram(self.window_tilemap_area() + y*32 + x) as u16;
    }

    fn get_tile_vram_addr(&self, tile_i :u16) -> u16 {
        let tile_data_area = self.bg_win_tile_data_area();

        // VRAM address of the tile
        return if tile_data_area {
            ADDR_VRAM_0 + TILE_VRAM_SIZE*tile_i
        } else if tile_i < 128 {
            ADDR_VRAM_2 + tile_i*TILE_VRAM_SIZE
        } else {
            ADDR_VRAM_2 + (tile_i-128)*TILE_VRAM_SIZE - 128*TILE_VRAM_SIZE
        };
    }

    fn get_bit_id(&self, byte_hi :u8, byte_lo :u8, bit_i :u8) -> u8 {
        let b_lo = (byte_lo >> bit_i) & 1;
        let b_hi = (byte_hi >> bit_i) & 1;
        return (b_hi << 1) | b_lo;
    }

    fn buf_bg_scanline(&mut self) {
        let y_tile = ((self.scy as u16 + self.ly as u16)/TILE_H) % 32;
        let tile_line_y = (self.scy as u16 + self.ly as u16) % 8;

        // If scx%8 != 0 wait (in this case approximate since its done in one tick) until it
        // discards enough pixels from the leftmost tile TODO: It shows artifacts when
        // approximating it to the right
        let mut x = self.scx;

        // 0..160/8
        while self.linebuffer.len() < SCREEN_WIDTH as usize {
            let x_tile    = x as u16/TILE_W;
            let tile_i    = self.get_bg_tilemap_addr(y_tile, x_tile);
            let addr_vram = self.get_tile_vram_addr(tile_i);

            let byte_lo = self.vram(addr_vram + 2*tile_line_y);
            let byte_hi = self.vram(addr_vram + 2*tile_line_y + 1);

            let limit = if x == self.scx {8-(self.scx%8)} else {8};
            for b_i in (0..limit).rev() {
                self.linebuffer.push( Pixel {
                    color_id: self.get_bit_id(byte_hi, byte_lo, b_i),
                    palette: Palette::BGP
                });
                
                x = x.wrapping_add(1);
            }
        }
    }

    fn buf_window_scanline(&mut self) {
        let y_tilemap = (self.window_counter/8) as u16;
        let tile_line_y = (self.window_counter) % 8;
        
        // 0..160/8
        for x_off in 0..20 {
            let wx = self.wx-7;
            let tile_i    = self.get_window_tilemap_addr(y_tilemap, x_off);
            let addr_vram = self.get_tile_vram_addr(tile_i);

            let byte_lo = self.vram(addr_vram + 2*tile_line_y as u16);
            let byte_hi = self.vram(addr_vram + 2*tile_line_y as u16 + 1);

            for b_i in 0..8 {
                let color_id = self.get_bit_id(byte_hi, byte_lo, b_i);
                let index = wx as u16 + x_off*TILE_W + (7-b_i) as u16;

                if index < SCREEN_WIDTH {
                    self.linebuffer[index as usize] = Pixel {color_id, palette: Palette::BGP};
                }
            }
        }
    }

    fn buf_obj_scanline(&mut self) {
        let mut buf = [
            ObjPixel { color_id: 0, x_priority: 0, bg_over_obj: false, palette: Palette::OBP0 };
            SCREEN_WIDTH as usize
        ];

        for obj in self.line_objs.iter() {
            // The object isn't hidden
            if obj.x > 0 && obj.x < 168 {
                // Normalize in the range 0-144 and 0-160
                //let y = y;
                //let x = *x;

                // Line to draw within the 8x(8/16) tile
                // y <= LY
                let line_i = if obj.flags.y_flip { 7-((self.ly+16)-obj.y) } else {(self.ly+16)-obj.y};
                let byte_lo = self.vram(ADDR_VRAM_0 + obj.tile_index as u16 * 16 + line_i as u16 * 2);
                let byte_hi = self.vram(ADDR_VRAM_0 + obj.tile_index as u16 * 16 + line_i as u16 * 2 + 1);

                let limit = if obj.x < 8 {obj.x} else {8};
                for b_i in (0..limit).rev() {
                    let color_id = self.get_bit_id(byte_hi, byte_lo, b_i);
                    let mut index = if obj.flags.x_flip { obj.x + b_i } else { obj.x + (7-b_i) };
                    if obj.x < 8 { index += 8-obj.x; }
                    let index = (index-8) as usize;

                    // The pixel is visible
                    if (index as u16) < SCREEN_WIDTH
                    // The actual pixel is transparent
                    && (buf[index].color_id == 0 || obj.x < buf[index].x_priority) { 
                        buf[index] = ObjPixel {
                            color_id,
                            x_priority: obj.x,
                            bg_over_obj: obj.flags.bg_over_obj,
                            palette: if obj.flags.palette_n { Palette::OBP1 } else { Palette::OBP0 }
                        }
                    }
                }
            }
        }

        // Draw over the linebuffer
        for i in 0..SCREEN_WIDTH as usize {
            // Dont draw transparent pixels
            if buf[i].color_id != 0 && (self.linebuffer[i].color_id == 0x00 || !buf[i].bg_over_obj) {
                self.linebuffer[i] = Pixel { color_id: buf[i].color_id, palette: buf[i].palette };
            }
        }
    }

    fn oam_search(&mut self) {
        self.line_objs = vec![];

        for oam_addr in (OAM_START..OAM_END).step_by(4) {
            let mut y           = self.oam(oam_addr);
            let x               = self.oam(oam_addr+1);
            let mut tile_index  = self.oam(oam_addr+2);
            let attrs           = self.oam(oam_addr+3);

            let flags = ObjFlags {
                bg_over_obj: self.is_set(attrs, 7),
                y_flip: self.is_set(attrs, 6),
                x_flip: self.is_set(attrs, 5),
                palette_n: self.is_set(attrs, 4)
            };

            if self.obj_size() == 8 {
                if y <= self.ly+16 && (self.ly+16)-y < 8 && self.line_objs.len() < 10 {
                    self.line_objs.push(Object { y, x, tile_index, flags });
                }
            // obj size == 16
            } else if y <= self.ly+16 && (self.ly+16)-y < 16 && self.line_objs.len() < 10 {
                let y_flip = self.is_set(attrs, 6);
                tile_index &= 0xFE; // Ignore LSB

                // 1st object
                if (self.ly+16) - y < 8 {
                    if y_flip { tile_index += 1; }  // 1st object and y_flip => 2nd object with y_flip
                // 2nd object
                } else {
                    y += 8;
                    if !y_flip { tile_index += 1; } // 2nd object and y_flip => 1st object with y_flip
                }
                
                // Ignore last bit
                self.line_objs.push(Object { y, x, tile_index, flags });
            }
        }
    }

    fn fill_linebuffer(&mut self) {
        for _ in 0..SCREEN_WIDTH {
            self.linebuffer.push(
                Pixel { color_id: 0, palette: Palette::BGP }
            );
        }
    }

    fn render_scanline(&mut self) {
        if self.is_lcd_enabled() {
            self.linebuffer.clear();

            if self.is_bg_window_enabled() {
                self.buf_bg_scanline();

                if self.is_window_enabled() && self.is_window_enable {
                    self.buf_window_scanline();
                    self.is_window_enable = false;
                    self.window_counter += 1;
                }
            } else {
                self.fill_linebuffer(); // To not send an empty vector to the screen
            }

            if self.are_objs_enabled() {
                self.buf_obj_scanline();
            }

            if self.has_drawn_first_frame {
                self.screen.borrow_mut().draw_linebuffer(
                    &self.linebuffer, self.ly, self.bgp, self.obp0, self.obp1
                );
            }
        } else {
            self.has_drawn_first_frame = false;
        }
    }

    pub fn tick(&mut self) {
        if !self.is_lcd_enabled() { return; }
        // Dot numbers are +1 to prevent off-by-one errors
        self.current_dot += 1;

        // Vblank: Increase LY every 456 dots
        if self.ly >= 144 {
            if self.current_dot == 457 {
                self.write_ly(self.ly+1);
            }
        }
        else {
            match self.current_dot {
                // OAM search: 80 dots
                1 => {
                    self.set_mode(STATMode::OAMSearch);
                    self.oam_search();
                }
                // Render OAM and VRAM: 168...291 dots
                // 168 + 10 per tile in a given line
                81 => {
                    self.set_mode(STATMode::Drawing);

                    // Enable drawing the window this line
                    // TODO: Why is wx < 166 and not < 159, given the screen size?
                    if self.wy < 143 && self.wx < 166 && self.ly >= self.wy && self.is_window_enabled() {
                        self.is_window_enable = true;
                    }

                    self.render_scanline();
                },
                // HBlank: 85...208 dots
                372 => self.set_mode(STATMode::HBlank),
                // 456: End of scanline
                457 => self.write_ly(self.ly+1),
                _ => {}
            }
        }
    }
}

impl ComponentWithMemory for PPU {
    fn read(&self, addr :u16) -> u8 {
        return match addr {
            VRAM_START..=VRAM_END => if self.can_access_vram() {
                self.vram[(addr-VRAM_START) as usize]
            } else {
                0xff
            },
            OAM_START..=OAM_END   => if self.can_access_oam()  {
                self.oam[(addr-OAM_START) as usize]
            } else {
                0xff
            },
            ADDR_LY   => self.ly,
            ADDR_LYC  => self.lyc,
            ADDR_WY   => self.wy,
            ADDR_WX   => self.wx,
            ADDR_SCY  => self.scy,
            ADDR_SCX  => self.scx,
            ADDR_BGP  => self.bgp,
            ADDR_OBP0 => self.obp0,
            ADDR_OBP1 => self.obp1,
            ADDR_LCDC => self.ldcd,
            ADDR_STAT => self.stat | 0b10000000, // bit 7 is always 1
            _ => panic!()
        }
    }

    fn write(&mut self, addr :u16, val :u8) {
        return match addr {
            VRAM_START..=VRAM_END => if self.can_access_vram() {
                self.vram[(addr-VRAM_START) as usize] = val;
            },
            OAM_START..=OAM_END => if self.can_access_oam() {
                self.oam[(addr-OAM_START) as usize] = val;
            },
            ADDR_LY   => { self.write_ly(0); }, // Read only. It resets when its written to
            ADDR_LYC  => { self.lyc = val; self.check_eq_ly_lyc(); },
            ADDR_WY   => self.wy  = val,
            ADDR_WX   => self.wx  = val,
            ADDR_SCY  => self.scy = val,
            ADDR_SCX  => self.scx = val,
            ADDR_BGP  => self.bgp = val,
            ADDR_OBP0 => self.obp0 = val,
            ADDR_OBP1 => self.obp1 = val,
            ADDR_LCDC => self.write_ldcd(val),
            ADDR_STAT => self.stat = val,
            _ => panic!()
        }
    }
}
