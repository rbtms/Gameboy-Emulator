#[cfg(test)]
mod tests {
    use gb::gbemulator::GBEmulator;
    use gb::screen::Screen;
    use md5;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::env;

    const ROMS_FOLDER   :&str = "tests/roms";

    pub fn take_screenshot_on_quit(file :&str) {
        let path = format!("{}/{}", ROMS_FOLDER, file);
        println!("path: {}", path);

        let mut gbemu = GBEmulator::new(&path, 2);
        gbemu.init();

        let screen = gbemu.get_screen();
        
        loop {
            gbemu.run_frame();

            if gbemu.is_quit() {
                print_screen_hash(file, screen);
                return;
            }
        }
    }

    pub fn print_screen_hash(path :&str, screen :Rc<RefCell<Screen>>) {
        let rom_file :&str = path.split("/").into_iter().last().unwrap();
        let pixels = screen.borrow().get_pixels(); 
        let digest = md5::compute(&pixels);

        println!("{}:{:x}", rom_file, digest);
    }

    #[test]
    pub fn print_screen_md5() {
        let rom_path: String = env::vars()
            .find(|(key, _)| key == "GB_ROM_PATH")
            .unwrap().1;
        
        take_screenshot_on_quit(&rom_path);
    }
}

fn main() {
}
