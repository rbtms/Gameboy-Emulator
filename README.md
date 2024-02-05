# Gameboy Emulator

Gameboy Emulator (DMG) written in Rust.This emulator aims to emulate in a cycle-accurate way the behaviour of the first gameboy model.
It uses a central bus architecture and RAM splitting to remove the components' need to know details about other components.

## Gifs

Pokemon|Tetris
--|--
| ![pokemon](https://github.com/rbtms/Gameboy-Emulator/assets/14959143/cdd506ee-380b-4bad-a37a-8f42ea428382) |  ![tetris](https://github.com/rbtms/Gameboy-Emulator/assets/14959143/6a5a3487-45ba-4ef9-b03d-b3038401b72d) |

## How to run

1. Clone the repository
2. Execute "cargo run \<ROM path\>"

## Supported features in the current version
| Component | CPU | PPU | Joypad | Screen | Timers | Interrupts | DMA | MBC1 | MBC2 | MBC3 | MBC5 | Serial  | APU |
|-----------|-----|-----|--------|--------|--------|------------|-----|------|------|------|------|-----|-----|
| Supported | ✅   | ✅   | ✅      | ✅      | ✅      | ✅          | ✅   | ✅    | ✅    | ✅    | ❌    | ❌    | ❌   |

## Passed tests in the current version

Please note that I have only run the tests relevant to the DMG and not other gameboy models.

<table>
<tr><th>Blargg</th>
<tr><td>

| Test          | Passed |
|---------------|--------|
| cpu_instrs/   | ✅     |
| dmg_sound/    | ❌     |
| instr_timing/ | ✅     |
| mem_timing/   | ✅     |
| mem_timing-2/ | ✅     |
| oam_bug.gb    | ❌     |
| halt_bug.gb   | ❌     |

</td></tr> </table>

<table>
<th>Mooneye</th></tr>
</td><td>

| Test          | Passed |
|---------------|--------|
| bits/mem_oam.gb                     | ✅ |
| bits/reg_f.gb                       | ✅ |
| bits/unused_hwio-GS.gb              | ✅ |
| instr/daa.gb                        | ✅ |
| interrupts/ie_push.gb               | ❌ |
| oam_dma/basic.gb                    | ✅ |
| oam_dma/reg_read.gb                 | ✅ |
| oam_dma/sources-GS.gb               | ❌ |
| ppu/hblank_ly_scx_timing-GS.gb      | ❌ |
| ppu/intr_1_2_timing-GS.gb           | ❌ |
| ppu/intr_2_0_timing.gb              | ❌ |
| ppu/intr_2_mode0_timing.gb          | ❌ |
| ppu/intr_2_mode0_timing_sprites.gb  | ❌ |
| ppu/intr_2_mode3_timing.gb          | ❌ |
| ppu/intr_2_oam_ok_timing.gb         | ❌ |
| ppu/lcdon_timing-GS.gb              | ❌ |
| ppu/lcdon_write_timing-GS.gb        | ❌ |
| ppu/stat_irq_blocking.gb            | ❌ |
| ppu/stat_lyc_onoff.gb               | ❌ |
| ppu/vblank_stat_intr-GS.gb          | ❌ |
| serial/boot_sclk_align-dmgABCmgb.gb | ❌ |
| timer/div_write.gb                  | ✅ |
| timer/rapid_toggle.gb               | ✅ |
| timer/tim00.gb                      | ✅ |
| timer/tim00_div_trigger.gb          | ✅ |
| timer/tim01.gb                      | ✅ |
| timer/tim01_div_trigger.gb          | ✅ |
| timer/tim10.gb                      | ✅ |
| timer/tim10_div_trigger.gb          | ✅ |
| timer/tim11.gb                      | ✅ |
| timer/tim11_div_trigger.gb          | ✅ |
| timer/tima_reload.gb                | ✅ |
| timer/tima_write_reloading.gb       | ❌ |
| timer/tma_write_reloading.gb        | ❌ |
| add_sp_e_timing.gb                  | ❌ |
| boot_div-dmgABCmgb.gb               | ✅ |
| boot_hwio-dmgABCmgb.gb              | ❌ |
| boot_regs-dmgABC.gb                 | ✅ |
| call_cc_timing.gb                   | ❌ |
| call_cc_timing2.gb                  | ✅ |
| call_timing.gb                      | ❌ |
| call_timing2.gb                     | ✅ |
| di_timing-GS.gb                     | ❌ |
| div_timing.gb                       | ✅ |
| ei_sequence.gb                      | ✅ |
| ei_timing.gb                        | ✅ |
| halt_ime0_ei.gb                     | ✅ |
| halt_ime0_nointr_timing.gb          | ❌ |
| halt_ime1_timing.gb                 | ✅ |
| halt_ime1_timing2-GS.gb             | ❌ |
| if_ie_registers.gb                  | ✅ |
| intr_timing.gb                      | ✅ |
| jp_cc_timing.gb                     | ❌ |
| jp_timing.gb                        | ❌ |
| ld_hl_sp_e_timing.gb                | ❌ |
| oam_dma_restart.gb                  | ❌ |
| oam_dma_start.gb                    | ❌ |
| oam_dma_timing.gb                   | ❌ |
| pop_timing.gb                       | ✅ |
| push_timing.gb                      | ✅ |
| rapid_di_ei.gb                      | ✅ |
| ret_cc_timing.gb                    | ❌ |
| ret_timing.gb                       | ❌ |
| reti_intr_timing.gb                 | ✅ |
| reti_timing.gb                      | ❌ |
| rst_timing.gb                       | ✅ |

</td></tr> </table>
