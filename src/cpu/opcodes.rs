use crate::consts::*;
use crate::cpu::CPU;

impl CPU {
    // r1 <- r2
    pub fn ld_r_r(&mut self, r_to :REGINDEX, r_from :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                self.set_reg(r_to, self.reg(r_from));
                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // r <- n
    pub fn ld_r_n(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.fetch();
                self.set_reg(r, val);
            }
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A <- (nn)
    pub fn ld_a_nn(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(), // lo
            2 => {
                let hi = self.fetch();
                self.cache16[0] = self.to_u16(hi, self.cache[0]);
            },
            3 => self.set_reg(REG_A, self.read(self.cache16[0])),
            4 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // (rr) <- a
    pub fn ld_rr_a(&mut self, r_hi :REGINDEX, r_lo :REGINDEX) {
        match self.instr_m_cycle {
            1 => self.write(self.reg16(r_hi, r_lo), self.reg(REG_A)),
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // (nn) <- A
    pub fn ld_nn_a(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(), // lo
            2 => {
                let hi = self.fetch();
                self.cache16[0] = self.to_u16(hi, self.cache[0]);
            },
            3 => self.write(self.cache16[0], self.reg(REG_A)),
            4 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A < (rr)
    pub fn ld_r_rr(&mut self, r_to :REGINDEX, r_from_hi :REGINDEX, r_from_lo :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg16(r_from_hi, r_from_lo);
                self.set_reg(r_to, self.read(val));
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // Copy next 2 bytes to a 16b register
    pub fn ld_rr_nn(&mut self, r_hi :REGINDEX, r_lo :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let lo = self.fetch();
                self.set_reg(r_lo, lo);
            },
            2 => {
                let hi = self.fetch();
                self.set_reg(r_hi, hi);
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // (nn, nn+1) <- sp
    pub fn ld_nn_sp(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(), // lo
            2 => self.cache[1] = self.fetch(), // hi
            3 => { // set lo
                self.cache16[0] = self.to_u16(self.cache[1], self.cache[0]);
                self.write(self.cache16[0], self.lower(self.sp));
            },
            // set hi
            4 => self.write(self.cache16[0]+1, self.upper(self.sp)),
            5 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // sp <- HL
    pub fn ld_sp_hl(&mut self) {
        match self.instr_m_cycle {
            1 => self.sp = self.reg16(REG_H, REG_L),
            // Can't prefetch the same cycle it fetches
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // sp <- nn
    pub fn ld_sp_nn(&mut self) {
        match self.instr_m_cycle {
            1 => self.sp = (self.sp & 0xff00) |   self.fetch() as u16,          // set lo
            2 => self.sp = (self.sp & 0x00ff) | ((self.fetch() as u16) << 8),   // set hi
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // (HL) <- n
    pub fn ld_hl_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => self.write(self.reg16(REG_H, REG_L), self.reg(r)),
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }


    // (HL) <- n
    pub fn ld_hl_n(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(),
            2 => self.write(self.reg16(REG_H, REG_L), self.cache[0]),
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // HL <- sp + i8
    // TODO: Check
    pub fn ldhl_sp_n(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let n = self.fetch() as i8;
                let sp_lo = self.lower(self.sp);

                let sp = self.sp.wrapping_add(n as u16);
                let h = (sp_lo << 4).overflowing_add((n as u8) << 4).1;
                let c = sp_lo.overflowing_add(n as u8).1;

                self.set_reg16(REG_H, REG_L, sp);
                self.set_flags(
                    0,          // z: reset
                    0,          // n: reset
                    h as u8,    // h: TODO: set or reset according to operation
                    c as u8     // c: TODO: set or reset according go operation
                );
            },
            // TODO: Can prefetches happen on cycles set as internal?
            2 => { /* Internal */ },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // Read from io-port C
    pub fn ldh_a_c(&mut self) {
        match self.instr_m_cycle {
            1 => self.set_reg(REG_A, self.read(0xFF00 + self.reg(REG_C) as RAMINDEX)),
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // Read from io-port n
    pub fn ldh_a_n(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(),
            2 => self.set_reg(REG_A, self.read(0xFF00 + self.cache[0] as RAMINDEX)),
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // Write to io-port n
    pub fn ldh_c_a(&mut self) {
        match self.instr_m_cycle {
            1 => self.write(0xFF00 + self.reg(REG_C) as RAMINDEX, self.reg(REG_A)),
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // Write to io-port n
    pub fn ldh_n_a(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(),
            2 => self.write(0xFF00 + self.cache[0] as RAMINDEX, self.reg(REG_A)),
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // (HL) = A, HL++
    pub fn ldi_hl_a(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let hl = self.reg16(REG_H, REG_L);

                self.write(hl, self.reg(REG_A));
                self.set_reg16(REG_H, REG_L, hl.wrapping_add(1));
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A = (HL), HL++
    pub fn ldi_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let hl = self.reg16(REG_H, REG_L);

                self.set_reg(REG_A, self.read(hl));
                self.set_reg16(REG_H, REG_L, hl.wrapping_add(1));
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // (HL) <- A, HL--
    pub fn ldd_hl_a(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let hl = self.reg16(REG_H, REG_L);

                self.write(hl as RAMINDEX, self.reg(REG_A));
                self.set_reg16(REG_H, REG_L, hl.wrapping_sub(1));
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A <- (HL), HL--
    pub fn ldd_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let hl = self.reg16(REG_H, REG_L);
                let val = self.read(hl);

                self.set_reg(REG_A, val);
                self.set_reg16(REG_H, REG_L, hl.wrapping_sub(1));
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // push rr
    // TODO: Check
    pub fn push_rr(&mut self, r_hi :REGINDEX, r_lo :REGINDEX) {
        match self.instr_m_cycle {
            1 => { /* Internal */ },
            2 => self.push(self.reg(r_hi)),
            3 => self.push(self.reg(r_lo)),
            4 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // pop rr
    pub fn pop_rr(&mut self, r_hi :REGINDEX, r_lo :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let lo = self.pop();
                self.set_reg(r_lo, lo);
            },
            2 => {
                let hi = self.pop();
                self.set_reg(r_hi, hi);
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    /*
     * Arithmetic instructions
     */

    // r++, set flags
    pub fn inc_r(&mut self, r :REG) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(r);
                let new_val = val.wrapping_add(1);

                self.set_reg(r, new_val);
                self.set_flags(
                    if new_val == 0x00 {1} else {0},        // z: 1 if result is 0
                    0,                                      // n: reset
                    if (val & 0x0f) == 0x0f {1} else {0},   // h: set if there is a carry from bit 3
                    //    (lower 4 bits are 0xffff before inc)
                    self.flag_c(),                          // c: Not affected
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // rr++
    pub fn inc_rr(&mut self, r_hi :REG, r_lo :REG) {
        match self.instr_m_cycle {
            1 => { // fetch
                let (lo, c) = self.reg(r_lo).overflowing_add(1);
                self.set_reg(r_lo, lo);
                self.cache[0] = c as u8;
            },
            2 => { // internal
                let hi = self.reg(r_hi);
                self.set_reg(r_hi, hi.wrapping_add(self.cache[0])); // add carry

                self.prefetch_opcode();

            },
            _ => panic!()
        }
    }

    // sp++
    pub fn inc_sp(&mut self) {
        match self.instr_m_cycle {
            1 => { // set lo
                let (sp_lo, c) = self.lower(self.sp).overflowing_add(1);
                self.sp = (self.sp & 0xff00) | (sp_lo as u16);
                self.cache[0] = c as u8;
            },
            2 => { // set hi, internal
                let sp_hi = ((self.sp>>8) as u8).wrapping_add(self.cache[0]) as u16; // add carry
                self.sp = (self.sp & 0x00ff) | (sp_hi << 8);
                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // (HL)++
    pub fn inc_hl(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.read(self.reg16(REG_H, REG_L)),
            2 => {
                let hl = self.cache[0].wrapping_add(1);

                self.write(self.reg16(REG_H, REG_L), hl);
                self.set_flags(
                    (hl == 0) as u8,                // z: set if result is 0
                    0,                              // n: reset
                    ((hl & 0x0f) == 0x00) as u8,    // h: set if there is a carry from bit 3
                    self.flag_c()                   // c: not affected
                );
            },
            // TODO: Can opcodes be prefetched on internal cycles?
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // SP--
    pub fn dec_sp(&mut self) {
        match self.instr_m_cycle {
            1 => { // set lo
                let (sp_lo, c) = self.lower(self.sp).overflowing_sub(1);
                self.sp = (self.sp & 0xff00) | (sp_lo as u16);
                self.cache[0] = c as u8;
            },
            2 => { // set hi
                let sp_hi = ((self.sp>>8) as u8).wrapping_sub(self.cache[0]) as u16; // add carry
                self.sp = (self.sp & 0x00ff) | (sp_hi << 8);

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // (HL)--, set flags
    pub fn dec_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0] = self.read(self.cache16[0]);
            },
            2 => {
                let hl = self.cache16[0];
                let val = self.cache[0];
                let new_val = val.wrapping_sub(1);

                self.write(hl, new_val);
                self.set_flags(
                    (new_val == 0) as u8,                   // z: set if result is 0
                    1,                                      // n: reset
                    if (val & 0x0f) == 0x00 {1} else {0},   // h: set if there wasnt a borrow from bit 4
                    self.flag_c()                           // c: not affected
                );
            },
            // TODO: Can opcodes be prefetched on internal cycles?
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A += r
    pub fn add_a_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let r_val = self.reg(r);
                let (new_a, c) = a.overflowing_add(r_val);

                self.set_reg(REG_A, new_a);
                self.set_flags(
                    (new_a == 0) as u8,                         // z: set if result is 0
                    0,                                          // n: reset
                    (((a & 0x0F) + (r_val & 0x0F)) >> 4) & 1,   // h: set if carry from bit 3
                    c as u8                                     // c: set if there is a carry from bit 7
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // A += n
    pub fn add_a_n(&mut self) {
        match self.instr_m_cycle {
            1 => { // read
                let a = self.reg(REG_A);
                let val = self.fetch();
                let (new_a, c) = a.overflowing_add(val);

                self.set_reg(REG_A, new_a);
                self.set_flags(
                    (new_a == 0) as u8,                     // z: set if result is 0
                    0,                                      // n: reset
                    (((a & 0x0F) + (val & 0x0F)) >> 4) & 1, // h: set if carry from bit 3
                    c as u8                                 // c: set if there is a carry from bit 7
                );
            }
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A += r with carry
    pub fn adc_a_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => { // fetch
                let a = self.reg(REG_A);
                let r_val = self.reg(r);

                let (a1, c1) = a.overflowing_add(r_val);
                let (a2, c2) = a1.overflowing_add(self.flag_c());

                let h = (((a & 0x0f) + (r_val & 0x0f) + self.flag_c()) >> 4) & 1;

                self.set_reg(REG_A, a2);
                self.set_flags(
                    (a2 == 0) as u8,    // z: set if result is 0
                    0,                  // n: reset
                    h,                  // h: set if carry from bit 3
                    (c1 || c2) as u8    // c: set if carry from bit 7
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // A += (HL) with carry
    pub fn adc_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let rr = self.reg16(REG_H, REG_L);
                let val = self.read(rr as RAMINDEX);

                let (a1, c1) = a.overflowing_add(val);
                let (a2, c2) = a1.overflowing_add(self.flag_c());

                let h = (((a & 0x0f) + (val & 0x0f) + self.flag_c()) >> 4) & 1;

                self.set_reg(REG_A, a2);
                self.set_flags(
                    (a2 == 0) as u8,    // z: set if result is 0
                    0,                  // n: reset
                    h,                  // h: set if carry from bit 3
                    (c1 || c2) as u8    // c: set if carry from bit 7
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A += n with carry
    pub fn adc_a_n(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let val = self.fetch();

                let (a1, c1) = a.overflowing_add(val);
                let (a2, c2) = a1.overflowing_add(self.flag_c());

                let h = (((a & 0x0f) + (val & 0x0f) + self.flag_c()) >> 4) & 1;

                self.set_reg(REG_A, a2);
                self.set_flags(
                    (a2 == 0) as u8,    // z: set if result is 0
                    0,                  // n: reset
                    h,                  // h: set if carry from bit 3
                    (c1 || c2) as u8    // c: set if carry from bit 7
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A += (HL)
    pub fn add_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let val = self.read(self.reg16(REG_H, REG_L));

                let (new_a, c) = a.overflowing_add(val);

                self.set_reg(REG_A, new_a);
                self.set_flags(
                    (new_a == 0) as u8,                     // z: set if result is 0
                    0,                                      // n: reset
                    (((a & 0x0f) + (val & 0x0f)) >> 4) & 1, // h: set if carry from bit 3
                    c as u8                                 // c: set if there is a carry from bit 7
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }

    }

    // HL += rr, set flags
    // TODO: debug
    pub fn add_hl_rr(&mut self, r_hi :REG, r_lo :REG) {
        match self.instr_m_cycle {
            1 => { // fetch, set L
                let (l, c)  = self.reg(r_lo).overflowing_add(self.reg(REG_L));

                self.set_reg(REG_L, l);
                self.cache[0] = c as u8;
            },
            2 => { // internal, set H
                let hi = self.reg(r_hi);

                let (h1, c1) = self.reg(REG_H).overflowing_add(hi);
                let (h2, c2) = h1.overflowing_add(self.cache[0]);

                let half = (((self.reg(REG_H) & 0x0F)
                        .wrapping_add(hi & 0x0F)
                        .wrapping_add(self.cache[0] & 0x0f)) >> 4) & 1;

                self.set_reg(REG_H, h2);
                self.set_flags(
                    self.flag_z(),   // z: not affected
                    0,               // n: reset
                    half,            // h: set if carry from bit 11
                    (c1 || c2) as u8          // c: set if carry from bit 15
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // SP += n. n is a 8b signed integer
    // TODO: debug
    pub fn add_sp_n(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(),
            2 => { // internal, write sp lo
                let n = self.cache[0] as i8;
                let sp_lo = self.lower(self.sp);

                let sp = self.sp.wrapping_add(n as u16);
                let h = (sp_lo << 4).overflowing_add((n as u8) << 4).1;
                let c = sp_lo.overflowing_add(n as u8).1;

                self.sp = sp & 0x00FF; // write lower
                self.cache[1] = (sp >> 8) as u8;
                self.set_flags(
                    0,          // z: reset
                    0,          // n: reset
                    h as u8,    // h: TODO: set or reset according to operation
                    c as u8     // c: TODO: set or reset according go operation
                );
            },
            // write sp hi
            3 => {
                self.sp = (self.sp & 0x00FF) | ((self.cache[1] as u16) << 8);
            },
            4 => {
                self.prefetch_opcode();
            }
            _ => panic!()
        }
    }

    // HL += SP, set flags
    // TODO: debug
    pub fn add_hl_sp(&mut self) {
        match self.instr_m_cycle {
            1 => { // write L
                let (l, c) = self.lower(self.sp).overflowing_add(self.reg(REG_L));

                self.set_reg(REG_L, l);
                self.cache[0] = c as u8;

            },
            2 => { // write H
                let sp_hi = self.upper(self.sp);

                let (h1, c1) = self.reg(REG_H).overflowing_add(sp_hi);
                let (h2, c2) = h1.overflowing_add(self.cache[0]);

                let half = (((self.reg(REG_H) & 0x0F)
                        .wrapping_add(sp_hi & 0x0F)
                        .wrapping_add(self.cache[0] & 0x0f)) >> 4) & 1;

                self.set_reg(REG_H, h2);
                self.set_flags(
                    self.flag_z(),   // z: not affected
                    0,               // n: reset
                    half,            // h: set if carry from bit 11
                    (c1 || c2) as u8          // c: set if carry from bit 15
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // r--, set flags
    pub fn dec_r(&mut self, r :REG) {
        match self.instr_m_cycle {
            // fetch
            1 => {
                let val = self.reg(r);
                let new_val = val.wrapping_sub(1);

                self.set_reg(r, new_val);
                self.set_flags(
                    (new_val == 0) as u8,        // z: 1 if result is 0
                    1,                                      // n: set
                    if (val & 0x0f) == 0x00 {1} else {0},   // h: set if there wasnt a borrow from bit 4
                    self.flag_c()                           // c: Not affected
                );

                self.prefetch_opcode();
            }
            _ => panic!()
        }
    }

    // rr--
    // TODO: debug
    pub fn dec_rr(&mut self, r_hi :REG, r_lo :REG) {
        match self.instr_m_cycle {
            1 => { // fetch
                let (lo, c) = self.reg(r_lo).overflowing_sub(1);
                self.set_reg(r_lo, lo);
                self.cache[0] = c as u8;
            },
            2 => { // internal
                let hi = self.reg(r_hi);
                self.set_reg(r_hi, hi.wrapping_sub(self.cache[0])); // add carry
                
                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // A -= r
    pub fn sub_a_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => { // fetch
                let a = self.reg(REG_A);
                let r_val = self.reg(r);
                let (new_a, c) = a.overflowing_sub(r_val);

                let h = ((a & 0x0f).wrapping_sub(r_val & 0x0f) >> 4) & 1;

                self.set_reg(REG_A, new_a);
                self.set_flags(
                    (new_a == 0) as u8,     // z: set if result is 0
                    1,                      // n: reset
                    h,                      // h: set if there wasnt a borrow from bit 4
                    c as u8                 // c: set if there is a carry from bit 7
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // A -= n
    pub fn sub_a_n(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let val = self.fetch();
                let (new_a, c) = a.overflowing_sub(val);

                let h = ((a & 0x0f).wrapping_sub(val & 0x0f) >> 4) & 1;

                self.set_reg(REG_A, new_a);
                self.set_flags(
                    (new_a == 0) as u8,     // z: set if result is 0
                    1,                      // n: reset
                    h,                      // h: set if there wasnt a borrow from bit 4
                    c as u8                 // c: set if there is a carry from bit 7
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }


    // A -= r
    pub fn sub_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let val = self.read(self.reg16(REG_H, REG_L));
                let a = self.reg(REG_A);
                let (new_a, c) = a.overflowing_sub(val);

                let h = ((a & 0x0f).wrapping_sub(val & 0x0f) >> 4) & 1;

                self.set_reg(REG_A, new_a);
                self.set_flags(
                    (new_a == 0) as u8,     // z: set if result is 0
                    1,                      // n: reset
                    h,                      // h: set if there wasnt a borrow from bit 4
                    c as u8                 // c: set if there is a carry from bit 7
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }


    // A -= r + c
    pub fn sbc_a_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => { // fetch
                let a = self.reg(REG_A);
                let r_val = self.reg(r);

                let (a1, c1) = a.overflowing_sub(r_val);
                let (a2, c2) = a1.overflowing_sub(self.flag_c());

                let h = ((a & 0x0f).wrapping_sub(r_val & 0x0f)
                    .wrapping_sub(self.flag_c() & 0x0f)
                    >> 4) & 1;

                self.set_reg(REG_A, a2);
                self.set_flags(
                    (a2 == 0) as u8,    // z: set if result is 0
                    1,                  // n: reset
                    h,                  // h: set if carry from bit 3
                    (c1 || c2) as u8    // c: set if carry from bit 7
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }

    }

    // A -= (HL) + c
    pub fn sbc_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let val = self.read(self.reg16(REG_H, REG_L));
                let a = self.reg(REG_A);

                let (a1, c1) = a.overflowing_sub(val);
                let (a2, c2) = a1.overflowing_sub(self.flag_c());

                let h = ((a & 0x0f).wrapping_sub(val & 0x0f)
                    .wrapping_sub(self.flag_c() & 0x0f)
                    >> 4) & 1;

                self.set_reg(REG_A, a2);
                self.set_flags(
                    (a2 == 0) as u8,    // z: set if result is 0
                    1,                  // n: reset
                    h,                  // h: set if carry from bit 3
                    (c1 || c2) as u8    // c: set if carry from bit 7
                );
            },
            2 => self.prefetch_opcode(), 
            _ => panic!()
        }
    }

    // A -= n + c
    pub fn sbc_a_n(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let val = self.fetch();
                let a = self.reg(REG_A);

                let (a1, c1) = a.overflowing_sub(val);
                let (a2, c2) = a1.overflowing_sub(self.flag_c());

                let h = ((a & 0x0f).wrapping_sub(val & 0x0f)
                    .wrapping_sub(self.flag_c() & 0x0f)
                    >> 4) & 1;

                self.set_reg(REG_A, a2);
                self.set_flags(
                    (a2 == 0) as u8,    // z: set if result is 0
                    1,                  // n: reset
                    h,                  // h: set if carry from bit 3
                    (c1 || c2) as u8    // c: set if carry from bit 7
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }


    // adjust A to BCD, set flags
    pub fn daa(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let mut a = self.reg(REG_A);
                let f = self.reg(REG_F);
                let (n, h, mut c) = ((f >> 6)&1, (f >> 5)&1, (f >> 4)&1);

                // Last operation was an addition
                if n == 0 {
                    // There was a carry or BCD(A) > $99 -> hi += 6 to $0
                    if c == 1 || a > 0x99          { a = a.wrapping_add(0x60); c = 1; }
                    // There was a half carry or BCD(lo A) > 9 -> lo += 6 to $0
                    if h == 1 || (a & 0x0f > 0x09) { a = a.wrapping_add(0x06); }
                }
                // Last operation was a substraction
                else {
                    // There was a carry -> hi -= 6 to 9
                    if c == 1 { a = a.wrapping_sub(0x60); }
                    // There was a carry -> lo -= 6 to 9
                    if h == 1 { a = a.wrapping_sub(0x06); }
                }

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if A is 0
                    n,              // n: not affected
                    0,              // h: reset
                    c               // c: set according to the operation
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }


    /*
     * Logic instructions
     */

    // A &= r
    pub fn and_a_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A) & self.reg(r);

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if result is 0
                    0,              // reset
                    1,              // set
                    0               // reset
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // A &= n
    pub fn and_a_n(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A) & self.fetch();

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if result is 0
                    0,              // reset
                    1,              // set
                    0               // reset
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A &= (HL)
    pub fn and_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A) & self.read(self.reg16(REG_H, REG_L) as RAMINDEX);

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if result is 0
                    0,              // reset
                    1,              // set
                    0               // reset
                );
            },
            2 => self.prefetch_opcode(), 
            _ => panic!()
        }
    }

    // A ^= r
    pub fn xor_a_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A) ^ self.reg(r);

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if result is 0
                    0,              // reset
                    0,              // reset
                    0               // reset
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // A &= (HL)
    pub fn xor_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A) ^ self.read(self.reg16(REG_H, REG_L) as RAMINDEX);

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if result is 0
                    0,              // reset
                    0,              // reset
                    0               // reset
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }


    // A ^= n
    pub fn xor_a_n(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A) ^ self.fetch();

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if result is 0
                    0,              // reset
                    0,              // reset
                    0               // reset
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // A |= r
    pub fn or_a_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A) | self.reg(r);

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if result is 0
                    0,              // reset
                    0,              // reset
                    0               // reset
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // A |= (HL)
    pub fn or_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A) | self.read(self.reg16(REG_H, REG_L) as RAMINDEX);

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if result is 0
                    0,              // reset
                    0,              // reset
                    0               // reset
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }

    }

    // A |= n
    pub fn or_a_n(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A) | self.fetch();

                self.set_reg(REG_A, a);
                self.set_flags(
                    (a == 0) as u8, // z: set if result is 0
                    0,              // reset
                    0,              // reset
                    0               // reset
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }

    }

    // a < r
    pub fn cp_a_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let val = self.reg(r);

                self.set_flags(
                    (a == val) as u8,                     // z: a == r
                    1,                                    // n: set
                    ((a & 0x0f) < (val & 0x0f)) as u8,    // h: no borrow from bit 4 (a < r)
                    (a < val) as u8                       // z: no borrow from bit 7 (a < r)
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // a < (HL)
    pub fn cp_a_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let addr = self.reg16(REG_H, REG_L);
                let val = self.read(addr);

                self.set_flags(
                    (a == val) as u8,                     // z: a == r
                    1,                                    // n: set
                    ((a & 0x0f) < (val & 0x0f)) as u8,    // h: no borrow from bit 4 (a < r)
                    (a < val) as u8                       // z: no borrow from bit 7 (a < r)
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }

    }

    // a < n
    pub fn cp_a_n(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let n = self.fetch();

                self.set_flags(
                    (a == n) as u8,                     // z: a == r
                    1,                                  // n: set
                    ((a & 0x0f) < (n & 0x0f)) as u8,    // h: no borrow from bit 4 (a < r)
                    (a < n) as u8                       // z: no borrow from bit 7 (a < r)
                );

            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }

    }

    // rotate A left with carry, set flags
    pub fn rla(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let new_a = (a << 1) | self.flag_c(); // carry

                self.set_reg(REG_A, new_a);
                self.set_flags(
                    0,              // z = reset
                    0,              // n = reset
                    0,              // h = reset
                    (a >> 7) & 1    // c = bit 7 of A before rotating
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // rotate r left with carry, set flags
    pub fn rl_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(r);
                let new_val = (val << 1) | self.flag_c(); // carry

                self.set_reg(r, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = 0 (TODO: Its different depending on the source)
                    0,                      // n = reset
                    0,                      // h = reset
                    (val >> 7) & 1          // c = bit 7 of A before rotating
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }

    }

    // rotate (HL) left with carry, set flags
    pub fn rl_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0] = self.read(self.cache16[0]);
            },
            2 => {
                let hl = self.cache16[0];
                let val = self.cache[0];
                let new_val = (val << 1) | self.flag_c(); // carry

                self.write(hl, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = 0 (TODO: Its different depending on the source)
                    0,                      // n = reset
                    0,                      // h = reset
                    (val >> 7) & 1          // c = bit 7 of A before rotating
                );
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }

    }

    // rotate A left, set flags
    pub fn rlca(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(REG_A);
                let new_val :REG = (val << 1) | ((val >> 7) & 1);

                self.set_reg(REG_A, new_val);
                self.set_flags(
                    0,                      // z = reset
                    0,                      // n = reset
                    0,                      // h = reset
                    (val >> 7) & 1          // c = bit 7 of A before rotating
                );
                
                self.prefetch_opcode();
            },
            _ => panic!()
        }

    }

    // rotate r left, set flags. 
    pub fn rlc_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(r);
                let new_val :REG = (val << 1) | ((val >> 7) & 1);

                self.set_reg(r, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = 1 if result is 0
                    0,                      // n = reset
                    0,                      // h = reset
                    (val >> 7) & 1          // c = bit 7 of A before rotating
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }

    }

    // rotate HL left, set flags. 
    pub fn rlc_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0]   = self.read(self.cache16[0]);
            },
            2 => {
                let addr = self.cache16[0];
                let val  = self.cache[0];
                let new_hl = ((val & 0x7f) << 1) | ((val >> 7) & 1);

                self.write(addr, new_hl);
                self.set_flags(
                    (new_hl == 0) as u8,    // z = 1 if result is 0
                    0,                      // n = reset
                    0,                      // h = reset
                    (val >> 7) & 1          // c = bit 7 of A before rotating
                );
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // rotate a right with carry, set flags
    pub fn rra(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let new_a = (a >> 1) | (self.flag_c() << 7);

                self.set_reg(REG_A, new_a);
                self.set_flags(
                    0,              // z = 0 (TODO: Its different depending on the source)
                    0,              // n = reset
                    0,              // h = reset
                    a & 1           // c = bit 0 of A before rotating
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // rotate r right with carry, set flags
    pub fn rr_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(r);
                let new_val = (val >> 1) | (self.flag_c() << 7);

                self.set_reg(r, new_val);
                self.set_flags(
                    (new_val == 0) as u8, // z = 0 (TODO: Its different depending on the source)
                    0,                    // n = reset
                    0,                    // h = reset
                    val & 1               // c = bit 0 of A before rotating
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }

    }

    // rotate (HL) right with carry, set flags
    pub fn rr_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0]   = self.read(self.cache16[0]);
            },
            2 => {
                let hl  = self.cache16[0];
                let val = self.cache[0];
                let new_val = (val >> 1) | (self.flag_c() << 7);

                self.write(hl, new_val);
                self.set_flags(
                    (new_val == 0) as u8, // z = 0 (TODO: Its different depending on the source)
                    0,                  // n = reset
                    0,                  // h = reset
                    val & 1               // c = bit 0 of A before rotating
                );
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }

    }


    // rotate A right, set flags
    pub fn rrca(&mut self) {
        match self.instr_m_cycle {
            1 => {
                let a = self.reg(REG_A);
                let new_a = (a >> 1) | ((a & 1) << 7);

                self.set_reg(REG_A, new_a);
                self.set_flags(
                    0,              // (TODO: Its different depending on the source)
                    0,              // n = reset
                    0,              // h = reset
                    a & 1           // c = bit 0 of A before rotating
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // rotate r right set flags
    pub fn rrc_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(r);
                let new_val = (val >> 1) | ((val&1) << 7);

                self.set_reg(r, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = 0
                    0,                      // n = reset
                    0,                      // h = reset
                    val & 1                 // c = bit 0 of r before rotating
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // rotate HL right, set flags. 
    pub fn rrc_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0]   = self.read(self.cache16[0]);
            },
            2 => {
                let addr = self.cache16[0];
                let val  = self.cache[0];
                let new_hl = (val >> 1) | ((val&1) << 7);

                self.write(addr, new_hl);
                self.set_flags(
                    (new_hl == 0) as u8,    // z = result == 0
                    0,                      // n = reset
                    0,                      // h = reset
                    val & 1                 // c = bit 0 of r before rotating
                );
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // shift left r into carry. LSB of r is set to 0
    pub fn sla_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(r);
                let new_val = (val & 0x7f) << 1;

                self.set_reg(r, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = result == 0
                    0,                      // n = reset
                    0,                      // h = reset
                    (val >> 7) & 1          // c = MSB of r
                );
                
                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // shift left (HL) into carry. LSB of (HL) is set to 0
    pub fn sla_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0]   = self.read(self.cache16[0]);
            },
            2 => {
                let hl = self.cache16[0];
                let val = self.cache[0];
                let new_val = (val & 0x7f) << 1;

                self.write(hl, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = result == 0
                    0,                      // n = reset
                    0,                      // h = reset
                    (val >> 7) & 1          // c = MSB of (HL)
                )
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // shift right r into carry. MSB of r doesn't change
    pub fn sra_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(r);
                let new_val = (val >> 1) | (val & 0x80);

                self.set_reg(r, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = result == 0
                    0,                      // n = reset
                    0,                      // h = reset
                    val & 1                 // c = MSB of r
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // shift right (HL) into carry. MSB of (HL) doesn't change
    pub fn sra_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0]   = self.read(self.cache16[0]);
            },
            2 => {
                let hl = self.cache16[0];
                let val = self.cache[0];
                let new_val = (val >> 1) | (val & 0x80);

                self.write(hl, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = result == 0
                    0,                      // n = reset
                    0,                      // h = reset
                    val & 1                 // c = MSB of r
                )
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // shift right r into carry. MSB of r set to 0
    pub fn srl_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(r);
                let new_val = val >> 1;

                self.set_reg(r, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = result == 0
                    0,                      // n = reset
                    0,                      // h = reset
                    val & 1                 // c = MSB of r
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // shift right (HL) into carry. MSB of (HL) set to 0
    pub fn srl_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0]   = self.read(self.cache16[0]);
            },
            2 => {
                let hl = self.cache16[0];
                let val = self.cache[0];
                let new_val = val >> 1;

                self.write(hl, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = result == 0
                    0,                      // n = reset
                    0,                      // h = reset
                    val & 1                 // c = MSB of r
                )
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // swap lower and upper nibbles of r
    pub fn swap_r(&mut self, r :REGINDEX) {
        match self.instr_m_cycle {
            1 => {
                let val = self.reg(r);
                let new_val = ((val & 0x0f) << 4) | (val >> 4);

                self.set_reg(r, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = result == 0
                    0,                      // n = reset
                    0,                      // h = reset
                    0,                      // c = reset
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }

    }

    // swap upper and lower nibbles of (HL)
    pub fn swap_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0]   = self.read(self.cache16[0]);
            },
            2 => {
                let hl = self.cache16[0];
                let val = self.cache[0];
                let new_val = ((val & 0x0f) << 4) | (val >> 4);

                self.write(hl, new_val);
                self.set_flags(
                    (new_val == 0) as u8,   // z = result == 0
                    0,                      // n = reset
                    0,                      // h = reset
                    0,                      // c = reset
                );
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }

    }

    // complement of A
    pub fn cpl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.set_reg(REG_A, self.reg(REG_A) ^ 0xff);
                self.set_flags(
                    self.flag_z(),      // z = not affected
                    1,                  // n = set
                    1,                  // h = set
                    self.flag_c()       // c = not affected
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    /*
     * Single-bit operation instructions
     */

    // test bit b in r
    pub fn bit_r(&mut self, r: REGINDEX, b :u8) {
        match self.instr_m_cycle {
            1 => {
                self.set_flags(
                    (self.reg(r) >> b) ^ 1,     // z: set if bit b is 0
                    0,                          // n: reset
                    1,                          // h: set
                    self.flag_c()               // c: not affected
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // test bit b in (HL)
    pub fn bit_hl(&mut self,b :u8) {
        match self.instr_m_cycle {
            1 => {
                let hl = self.read(self.reg16(REG_H, REG_L));

                self.set_flags(
                    (hl >> b) ^ 1,   // z: set if bit b is 0
                    0,                          // n: reset
                    1,                          // h: set
                    self.flag_c()               // c: not affected
                );
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // set bit b on r
    pub fn set_r(&mut self, r :REGINDEX, b :u8) {
        match self.instr_m_cycle {
            1 => {
                self.set_reg(r, self.reg(r) | (2 as u8).pow(b as u32));
                self.prefetch_opcode();
            }
            _ => panic!()
        }

    }

    // set bit b on (HL)
    pub fn set_hl(&mut self, b :u8) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0]   = self.read(self.cache16[0]);
            },
            2 => {
                let hl = self.cache16[0];
                let val = self.cache[0];

                self.write(hl, val | (2 as u8).pow(b as u32));
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }

    }

    // set bit b on r
    pub fn res_r(&mut self, r :REGINDEX, b :u8) {
        match self.instr_m_cycle {
            1 => {
                self.set_reg(r, self.reg(r) & (0xff ^ (2 as u8).pow(b as u32)));
                self.prefetch_opcode();
            }
            _ => panic!()
        }
    }

    // set bit b on (HL)
    pub fn res_hl(&mut self, b :u8) {
        match self.instr_m_cycle {
            1 => {
                self.cache16[0] = self.reg16(REG_H, REG_L);
                self.cache[0]   = self.read(self.cache16[0]);
            },
            2 => {
                let hl = self.cache16[0];
                let val = self.cache[0];

                self.write(hl, val & (0xff ^ (2 as u8).pow(b as u32)));
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }


    /*
     * CPU control instructions
     */

    // nop
    pub fn nop(&mut self) {
        match self.instr_m_cycle {
            1 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // TODO: Halt CPU and LCD display until button pressed
    // TODO: Check
    pub fn stop(&mut self) {
        match self.instr_m_cycle {
            1 => {
                println!("stop(): unimplemented");

                // TODO: start increasing it again afterwards
                self.is_stop = true;
                self.write(ADDR_DIV, 0x00); // Reset DIV when calling stop and
            },
            2 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // Power down the CPU until an interrupt occurs
    // TODO: Skip next instruction after a HALT
    pub fn halt(&mut self) {
        //println!("{} halt {}", self.pc, self.instr_m_cycle);

        match self.instr_m_cycle {
            1 => {
                self.is_halt = true;

                // There is an interrupt pending and IME=0
                if !self.int.borrow().get_ime() {
                    self.enable_halt_bug = true;
                }

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // enable interrupts
    pub fn ei(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.schedule_ime = true;
                self.prefetch_opcode();
            }
            _ => panic!()
        }
    }

    // disable interrupts TODO: Some sources say the interrupts are disabled
    // after the next instruction is executed
    pub fn di(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.int.borrow_mut().set_ime(false);
                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // complement carry flag
    pub fn ccf(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.set_flags(
                    self.flag_z(),          // z: not affected
                    0,                      // n: reset
                    0,                      // h: reset
                    (self.flag_c()+1) % 2   // c: !c
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }

    }

    // set carry flag
    pub fn scf(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.set_flags(
                    self.flag_z(),  // z: not affected
                    0,                  // n: reset
                    0,                  // h: reset
                    1                   // c: set
                );

                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    /*
     * Jump instructions
     */

    pub fn _check_jmp_condition(&self, mode :JmpCond) -> bool {
        return match mode {
            JmpCond::NZ => self.flag_z() == 0, // z == 0
            JmpCond::Z  => self.flag_z() == 1, // z == 1
            JmpCond::NC => self.flag_c() == 0, // c == 0
            JmpCond::C  => self.flag_c() == 1  // c == 1
        };
    }

    // PC = nn
    pub fn jp_nn(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(), // lo
            2 => self.cache[1] = self.fetch(), // hi
            3 => self.pc = self.to_u16(self.cache[1], self.cache[0]),
            4 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // PC = n if condition
    pub fn jp_cc_nn(&mut self, cond :JmpCond) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(),
            2 => self.cache[1] = self.fetch(),
            /*
             * If the condition == false, it prefetches the next opcode
             * and the instruction ends. If condition = true, it waits
             * another M-cycle to prefetch the opcode.
             */
            3 => {
                if self._check_jmp_condition(cond) {
                    self.pc = self.to_u16(self.cache[1], self.cache[0]);  
                }
                else {
                    self.prefetch_opcode();
                }
            },
            4 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // PC = HL
    pub fn jp_hl(&mut self) {
        match self.instr_m_cycle {
            1 => {
                self.pc = self.reg16(REG_H, REG_L);
                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }

    // PC += n
    pub fn jr(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(),
            2 => {
                let val = self.cache[0] as i8;
                self.pc = self.pc.wrapping_add(val as u16);
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // PC += n if condition
    pub fn jr_cc_n(&mut self, cond :JmpCond) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(),
            /*
             * If the condition == false, it prefetches the next opcode
             * and the instruction ends. If condition = true, it waits
             * another M-cycle to prefetch the opcode.
             */
            2 => {
                if self._check_jmp_condition(cond) {
                    let val = self.cache[0] as i8;
                    self.pc = self.pc.wrapping_add(val as u16);
                }
                else {
                    self.prefetch_opcode();
                }
            },
            3 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // PC = n
    pub fn call_nn(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(), // lo
            2 => self.cache[1] = self.fetch(), // hi
            3 => { /* Internal */ },
            4 => self.push(self.upper(self.pc)),
            5 => {
                self.push(self.lower(self.pc));
                self.pc = self.to_u16(self.cache[1], self.cache[0]);
            }
            6 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // call if condition
    pub fn call_cc_nn(&mut self, cond :JmpCond) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.fetch(), // lo
            2 => self.cache[1] = self.fetch(), // hi
            /*
             * If the condition == false, it prefetches the next opcode
             * and the instruction ends. If condition = true, it waits
             * another M-cycle to prefetch the opcode.
             */
            3 => {
                if !self._check_jmp_condition(cond) {
                    self.prefetch_opcode();
                }
            },
            4 => self.push(self.upper(self.pc)),
            5 => {
                self.push(self.lower(self.pc));
                self.pc = self.to_u16(self.cache[1], self.cache[0]);
            }
            6 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // return to addr in top of stack
    pub fn ret(&mut self) {
        match self.instr_m_cycle {
            1 => self.cache[0] = self.pop(), // lo
            2 => self.cache[1] = self.pop(), // hi
            3 => self.pc = self.to_u16(self.cache[1], self.cache[0]),
            4 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // return if condition
    pub fn ret_cc(&mut self, cond :JmpCond) {
        match self.instr_m_cycle {
            1 => { /* Internal */ }, // TODO: Check timing
            2 => {
                if !self._check_jmp_condition(cond) {
                    self.prefetch_opcode()
                }
                else {
                    self.cache[0] = self.pop(); // lo
                }
            },
            3 => self.cache[1] = self.pop(), // hi
            4 => self.pc = self.to_u16(self.cache[1], self.cache[0]),
            5 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // ret, enable interrupts
    pub fn reti(&mut self) {
        self.ret();
        
        // Prefetch in the next M-cycle after setting pc
        // TODO: Check timing
        if self.instr_m_cycle == 4 {
            self.int.borrow_mut().set_ime(true);
        }
    }

    // reset PC
    pub fn rst(&mut self, addr :u16) {
        match self.instr_m_cycle {
            1 => { /* Internal */ },
            2 => self.push(self.upper(self.pc)),
            3 => {
                self.push(self.lower(self.pc));
                self.pc = addr;
            }
            4 => self.prefetch_opcode(),
            _ => panic!()
        }
    }

    // Not an opcode
    // Transfer control to an interruption address
    pub fn transfer_control_int(&mut self, addr :u16) {
        match self.instr_m_cycle {
            /* Wait 2 M-cycles */
            1 => { /* Nop */ },
            2 => { /* Nop */ },
            /* Push PC */
            3 => self.push(self.upper(self.pc)),
            4 => self.push(self.lower(self.pc)),
            5 => {
                self.pc = addr;
                self.prefetch_opcode();
            },
            _ => panic!()
        }
    }
}
