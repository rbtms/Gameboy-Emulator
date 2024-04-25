#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use gb::gbemulator::GBEmulator;

    pub fn _test_rom(file :&str) {
        let path = "roms/gb-test-roms-master";
        let path = format!("{}/{}", path, file);

        let mut gbemu = GBEmulator::new(&path, false);
        gbemu.init();

        let bus = gbemu.get_bus();
        let cpu = gbemu.get_cpu();

        let s_limit = 10; // 10s
        let start = SystemTime::now();

        // First fetch
        //for _ in 0..4 { cpu.tick(); }

        let mut output :String = String::new();
        while start.elapsed().unwrap().as_secs() < s_limit {
            cpu.tick();
            bus.borrow_mut().tick();

            if !cpu.is_wait() && bus.borrow().read(0xff02) == 0xFF {
                output.push(bus.borrow().read(0xFF01) as char);
                bus.borrow_mut().write(0xff02, 0);

                if output.contains("Passed") {
                    return;
                }
                else if output.contains("Failed") {
                    println!("{}", &output);
                    assert!(false, "Failed");
                }
            }
        }

        assert!(false, "Timeout");
    }

    pub fn load_md5(filename :&str) -> String {
        let file :String = std::fs::read_to_string("tests/blargg_expected_md5.txt").unwrap();
        let lines :Vec<&str> = file.lines().into_iter().collect();

        for line in lines {
            println!("{} filename {}", line, filename);
            if line.contains(&filename) {
                return line.split(":").into_iter().last().unwrap().to_string();
            }
        }

        panic!();
    }

    pub fn hash_matches(pixels :&Vec<u8>, hash :&String) -> bool {
        return format!("{:x}", md5::compute(&pixels)) == *hash;
    }

    pub fn test_rom(file :&str) {
        let path = "roms/gb-test-roms-master";
        let path = format!("{}/{}", path, file);

        let file = file.split("/").into_iter().last().unwrap();
        let hash = load_md5(&file);

        let mut gbemu = GBEmulator::new(&path, false);
        gbemu.init();

        let screen = gbemu.get_screen();
        let bus = gbemu.get_bus();
        let cpu = gbemu.get_cpu();

        let s_limit = 10; // 10s
        let start = SystemTime::now();
        let ticks_screenshot = 10000; // Get screen state every 10000 ticks;
        let mut ticks :u32 = 0;

        while start.elapsed().unwrap().as_secs() < s_limit {
            cpu.tick();
            bus.borrow_mut().tick();

            ticks += 1;
            if ticks == ticks_screenshot {
                ticks = 0;

                if hash_matches(&screen.borrow().get_pixels(), &hash) {
                    return;
                }
            }
        }

        assert!(false, "Timeout");
    }


    /*
     * Rom tests
     */
    #[test]
    pub fn test_rom_1_special()             { test_rom("cpu_instrs/individual/01-special.gb"); }
    #[test]
    pub fn test_rom_2_interrupts()          { test_rom("cpu_instrs/individual/02-interrupts.gb"); }
    #[test]
    pub fn test_rom_3_sp_hl()               { test_rom("cpu_instrs/individual/03-op sp,hl.gb"); }
    #[test]
    pub fn test_rom_4_r_imm()               { test_rom("cpu_instrs/individual/04-op r,imm.gb"); }
    #[test]
    pub fn test_rom_5_rp()                  { test_rom("cpu_instrs/individual/05-op rp.gb"); }
    #[test]
    pub fn test_rom_6_ld_r_r()              { test_rom("cpu_instrs/individual/06-ld r,r.gb"); }
    #[test]
    pub fn test_rom_7_jr_jp_call_ret_rst()  { test_rom("cpu_instrs/individual/07-jr,jp,call,ret,rst.gb"); }
    #[test]
    pub fn test_rom_8_misc_instrs()         { test_rom("cpu_instrs/individual/08-misc instrs.gb"); }
    #[test]
    pub fn test_rom_9_op_r_r()              { test_rom("cpu_instrs/individual/09-op r,r.gb"); }
    #[test]
    pub fn test_rom_10_bit_ops()            { test_rom("cpu_instrs/individual/10-bit ops.gb"); }
    #[test]
    pub fn test_rom_11_a_hl()               { test_rom("cpu_instrs/individual/11-op a,(hl).gb"); }

    #[test]
    pub fn test_rom_instr_timing()          { test_rom("instr_timing/instr_timing.gb"); }

    #[test]
    pub fn test_rom_mem_read_timing()       { test_rom("mem_timing/individual/01-read_timing.gb"); }
    #[test]
    pub fn test_rom_mem_write_timing()      { test_rom("mem_timing/individual/02-write_timing.gb"); }
    #[test]
    pub fn test_rom_mem_modify_timing()     { test_rom("mem_timing/individual/03-modify_timing.gb"); }
    #[test]
    pub fn test_rom_mem_2_read_timing()     { test_rom("mem_timing-2/rom_singles/02-01-read_timing.gb"); }
    #[test]
    pub fn test_rom_mem_2_write_timing()    { test_rom("mem_timing-2/rom_singles/02-02-write_timing.gb"); }
    #[test]
    pub fn test_rom_mem_2_modify_timing()   { test_rom("mem_timing-2/rom_singles/02-03-modify_timing.gb"); }

    // TODO: individuals
    //#[test]
    //pub fn test_rom_oam_bug()   { test_rom("oam_bug/oam_bug.gb"); }

    //#[test]
    //pub fn test_rom_halt_bug()   { test_rom("halt_bug.gb"); }
}

fn main() {
}

