#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use gb::gbemulator::GBEmulator;
    use gb::consts::*;

    type CPU = gb::cpu::CPU;
    type Bus = gb::bus::Bus;

    const ROM_FOLDER :&str = "tests/roms/mts/acceptance";
    const TIMEOUT_S  :u64  = 5; // 1 test might timeout too soon
    
    fn debug_mooneye_passed(cpu :&CPU) -> bool {
        return (cpu.reg(REG_B), cpu.reg(REG_C), cpu.reg(REG_D),
                cpu.reg(REG_E), cpu.reg(REG_H), cpu.reg(REG_L))
            == (3, 5, 8, 13, 21, 34);
    }

    fn debug_mooneye_failed(cpu :&CPU) -> bool {
        return (cpu.reg(REG_B), cpu.reg(REG_C), cpu.reg(REG_D),
                cpu.reg(REG_E), cpu.reg(REG_H), cpu.reg(REG_L))
            == (0x42, 0x42, 0x42, 0x42, 0x42, 0x42);
    }

    fn read_link_port(bus :&mut Bus) {
        let sb = 0xff01;

        if bus.read(sb) != 0x00 {
            bus.write(sb, 0x00);
        }
    }

    fn detect_ld_b_b(cpu :&CPU, bus :&Bus) -> bool {
        return bus.read(cpu.get_pc()) == 0x40; // LD B, B
    }

    pub fn test_rom(file :&str) {
        let path = format!("{}/{}", ROM_FOLDER, file);

        let mut gbemu = GBEmulator::new(&path, 2);
        gbemu.init();

        let bus = gbemu.get_bus();
        let cpu = gbemu.get_cpu_mut();

        let start = SystemTime::now();

        while start.elapsed().unwrap().as_secs() < TIMEOUT_S {
            if !cpu.is_wait() && detect_ld_b_b(&cpu, &bus.borrow_mut()) {
                if debug_mooneye_passed(&cpu) {
                    return;
                }
                else if debug_mooneye_failed(&cpu) {
                    assert!(false, "Failed");
                }
            }

            bus.borrow_mut().tick();
            cpu.tick();

            if !cpu.is_wait() {
                read_link_port(&mut bus.borrow_mut());

            }
        }

        assert!(false, "Timeout");
    }

    /*
     * Rom tests
     * TODO: ppu
     */
    #[test]
    pub fn test_bits_mem_oam() { test_rom("bits/mem_oam.gb"); }
    #[test]
    pub fn test_bits_reg_f() { test_rom("bits/reg_f.gb"); }
    /*
     Fails because:
     - FF00/P1  : It returns a different value.
     - FF26/NR52: Channel 1 is disabled at the start (should it be?).
     - FF41/STAT: Is incremented before it's obtained.
     */
    //#[test]
    //pub fn test_bits_unused_hwio() { test_rom("bits/unused_hwio-GS.gb"); }

    #[test]
    pub fn test_instr_daa() { test_rom("instr/daa.gb"); }

    #[test]
    pub fn test_int_ie_push() { test_rom("interrupts/ie_push.gb"); }

    #[test]
    pub fn test_oam_dma_basic() { test_rom("oam_dma/basic.gb"); }
    #[test]
    pub fn test_oam_dma_reg_read() { test_rom("oam_dma/reg_read.gb"); }
    // Fails because MBC5_RAM_BAT isn't supported
    //#[test]
    //pub fn test_oam_dma_sources() { test_rom("oam_dma/sources-GS.gb"); }

    #[test]
    pub fn test_timer_div_write() { test_rom("timer/div_write.gb"); }
    #[test]
    pub fn test_timer_rapid_toggle() { test_rom("timer/rapid_toggle.gb"); }
    #[test]
    pub fn test_timer_tim00() { test_rom("timer/tim00.gb"); }
    #[test]
    pub fn test_timer_tim00_div_trigger() { test_rom("timer/tim00_div_trigger.gb"); }
    #[test]
    pub fn test_timer_tim01() { test_rom("timer/tim01.gb"); }
    #[test]
    pub fn test_timer_tim01_div_trigger() { test_rom("timer/tim01_div_trigger.gb"); }
    #[test]
    pub fn test_timer_tim10() { test_rom("timer/tim10.gb"); }
    #[test]
    pub fn test_timer_tim10_div_trigger() { test_rom("timer/tim10_div_trigger.gb"); }
    #[test]
    pub fn test_timer_tim11() { test_rom("timer/tim11.gb"); }
    #[test]
    pub fn test_timer_tim11_div_trigger() { test_rom("timer/tim11_div_trigger.gb"); }
    #[test]
    pub fn test_timer_tima_reload() { test_rom("timer/tima_reload.gb"); }
    #[test]
    pub fn test_timer_tima_write_reloading() { test_rom("timer/tima_write_reloading.gb"); }
    #[test]
    pub fn test_timer_tma_write_reloading() { test_rom("timer/tma_write_reloading.gb"); }
    
    #[test]
    pub fn test_serial() { test_rom("serial/boot_sclk_align-dmgABCmgb.gb") }

    #[test]
    pub fn oam_dma_restart() { test_rom("oam_dma_restart.gb"); }
    #[test]
    pub fn oam_dma_start() { test_rom("oam_dma_start.gb"); }
    #[test]
    pub fn oam_dma_timing() { test_rom("oam_dma_timing.gb"); }

    #[test]
    pub fn ppu_hblank() { test_rom("ppu/hblank_ly_scx_timing-GS.gb"); }
    #[test]
    pub fn ppu_intr_1_timing() { test_rom("ppu/intr_1_2_timing-GS.gb"); }
    #[test]
    pub fn ppu_intr_2_timing() { test_rom("ppu/intr_2_0_timing.gb"); }
    #[test]
    pub fn ppu_intr_2_mode0_timing() { test_rom("ppu/intr_2_mode0_timing.gb"); }
    #[test]
    pub fn ppu_intr_2_mode0_timing_sprites() { test_rom("ppu/intr_2_mode0_timing_sprites.gb"); }
    #[test]
    pub fn ppu_intr_2_mode3_timing() { test_rom("ppu/intr_2_mode3_timing.gb"); }
    #[test]
    pub fn ppu_intr2_oam_ok_timing() { test_rom("ppu/intr_2_oam_ok_timing.gb"); }
    #[test]
    pub fn ppu_lcdon_timing() { test_rom("ppu/lcdon_timing-GS.gb"); }
    #[test]
    pub fn ppu_lcdon_write_timing() { test_rom("ppu/lcdon_write_timing-GS.gb"); }
    #[test]
    pub fn ppu_irq_blocking() { test_rom("ppu/stat_irq_blocking.gb"); }
    #[test]
    pub fn ppu_lyc_onoff() { test_rom("ppu/stat_lyc_onoff.gb"); }
    #[test]
    pub fn ppu_vblank_stat_intr() { test_rom("ppu/vblank_stat_intr-GS.gb"); }

    #[test]
    pub fn test_add_sp_e_timing() { test_rom("add_sp_e_timing.gb"); }
    #[test]
    pub fn test_boot_div() { test_rom("boot_div-dmgABCmgb.gb"); }
    #[test]
    pub fn test_boot_hwio() { test_rom("boot_hwio-dmgABCmgb.gb"); }
    #[test]
    pub fn test_boot_regs() { test_rom("boot_regs-dmgABC.gb"); }
    #[test]
    pub fn test_call_cc_timing() { test_rom("call_cc_timing.gb"); }
    #[test]
    pub fn test_call_cc_timing2() { test_rom("call_cc_timing2.gb"); }
    #[test]
    pub fn test_call_timing(){ test_rom("call_timing.gb"); }
    #[test]
    pub fn test_call_timing_2() { test_rom("call_timing2.gb"); }
    #[test]
    pub fn test_di_timing() { test_rom("di_timing-GS.gb"); }
    #[test]
    pub fn test_div_timing() { test_rom("div_timing.gb"); }
    #[test]
    pub fn test_ei_sequence() { test_rom("ei_sequence.gb"); }
    #[test]
    pub fn test_ei_timing() { test_rom("ei_timing.gb"); }
    #[test]
    pub fn test_halt_ime0_ei() { test_rom("halt_ime0_ei.gb"); }
    #[test]
    pub fn test_halt_ime0_nointr_timing() { test_rom("halt_ime0_nointr_timing.gb"); }
    #[test]
    pub fn test_halt_ime1_timing() { test_rom("halt_ime1_timing.gb"); }
    #[test]
    pub fn test_halt_ime1_timing2() { test_rom("halt_ime1_timing2-GS.gb"); }
    #[test]
    pub fn test_if_ie_registers() { test_rom("if_ie_registers.gb"); }
    #[test]
    pub fn test_intr_timing() { test_rom("intr_timing.gb"); }
    #[test]
    pub fn test_jp_cc_timing() { test_rom("jp_cc_timing.gb"); }
    #[test]
    pub fn test_jp_timing() { test_rom("jp_timing.gb"); }
    #[test]
    pub fn test_ld_hl_sp_e_timing() { test_rom("ld_hl_sp_e_timing.gb"); }
    #[test]
    pub fn test_pop_timing() { test_rom("pop_timing.gb"); }
    #[test]
    pub fn test_push_timing() { test_rom("push_timing.gb"); }
    #[test]
    pub fn test_rapid_di_ei() { test_rom("rapid_di_ei.gb"); }
    #[test]
    pub fn test_ret_cc_timing() { test_rom("ret_cc_timing.gb"); }
    #[test]
    pub fn test_ret_timing() { test_rom("ret_timing.gb"); }
    #[test]
    pub fn test_reti_intr_timing() { test_rom("reti_intr_timing.gb"); }
    #[test]
    pub fn test_reti_timing() { test_rom("reti_timing.gb"); }
    #[test]
    pub fn test_rst_timing() { test_rom("rst_timing.gb"); }

    #[test]
    pub fn mbc1_bits_bank1() { test_rom("../emulator-only/mbc1/bits_bank1.gb"); }
    #[test]
    pub fn mbc1_bits_bank2() { test_rom("../emulator-only/mbc1/bits_bank2.gb"); }
    #[test]
    pub fn mbc1_bits_mode() { test_rom("../emulator-only/mbc1/bits_mode.gb"); }
    #[test]
    pub fn mbc1_bits_ramg() { test_rom("../emulator-only/mbc1/bits_ramg.gb"); }
    #[test]
    pub fn mbc1_bits_16mb() { test_rom("../emulator-only/mbc1/rom_16Mb.gb"); }
    #[test]
    pub fn mbc1_bits_1mb() { test_rom("../emulator-only/mbc1/rom_1Mb.gb"); }
    #[test]
    pub fn mbc1_bits_2mb() { test_rom("../emulator-only/mbc1/rom_2Mb.gb"); }
    #[test]
    pub fn mbc1_bits_4mb() { test_rom("../emulator-only/mbc1/rom_4Mb.gb"); }
    #[test]
    pub fn mbc1_bits_512kb() { test_rom("../emulator-only/mbc1/rom_512kb.gb"); }

    #[test]
    pub fn mbc2_bits_ramg() { test_rom("../emulator-only/mbc2/bits_ramg.gb"); }
    #[test]
    pub fn mbc2_bits_romb() { test_rom("../emulator-only/mbc2/bits_romb.gb"); }
    #[test]
    pub fn mbc2_bits_unused() { test_rom("../emulator-only/mbc2/bits_unused.gb"); }
    #[test]
    pub fn mbc2_ram() { test_rom("../emulator-only/mbc2/ram.gb"); }
    #[test]
    pub fn mbc2_rom_1mb() { test_rom("../emulator-only/mbc2/rom_1Mb.gb"); }
    #[test]
    pub fn mbc2_rom_2mb() { test_rom("../emulator-only/mbc2/rom_2Mb.gb"); }
    #[test]
    pub fn mbc2_rom_512kb() { test_rom("../emulator-only/mbc2/rom_512kb.gb"); }

    // Currently returning "Not Supported"
    #[test]
    pub fn mbc5_rom_16mb() { test_rom("../emulator-only/mbc5/rom_16Mb.gb"); }
    #[test]
    pub fn mbc5_rom_1mb() { test_rom("../emulator-only/mbc5/rom_1Mb.gb"); }
    #[test]
    pub fn mbc5_rom_2mb() { test_rom("../emulator-only/mbc5/rom_2Mb.gb"); }
    #[test]
    pub fn mbc5_rom_32mb() { test_rom("../emulator-only/mbc5/rom_32Mb.gb"); }
    #[test]
    pub fn mbc5_rom_4mb() { test_rom("../emulator-only/mbc5/rom_4Mb.gb"); }
    #[test]
    pub fn mbc5_rom_512kb() { test_rom("../emulator-only/mbc5/rom_512kb.gb"); }
    #[test]
    pub fn mbc5_rom_64mb() { test_rom("../emulator-only/mbc5/rom_64Mb.gb"); }
    #[test]
    pub fn mbc5_rom_8mb() { test_rom("../emulator-only/mbc5/rom_8Mb.gb"); }

}

fn main() {
}
