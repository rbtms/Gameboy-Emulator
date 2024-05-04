use std::cell::RefCell;
use std::rc::Rc;

use crate::bus::Bus;
use crate::interruptManager::InterruptManager;
use crate::consts::*;

mod opcodes;

pub struct CPU {            
    regs                            : [REG;REG_N as usize],
    sp                              : SPSIZE,                   // Stack Pointer
    pc                              : PCSIZE,                   // Program Counter
    bus                             : Rc<RefCell<Bus>>,         // Refcell to keep it for the debugger
    opcode                          : u8,                       // Actual opcode
    opcode_cb                       : u8,                       // Actual opcode
    is_cb_opcode                    : bool,
    has_fetched_cb_opcode           : bool,

    is_instr_done                   : bool,

    instr_m_cycle                   : u8,      // Actual m-cycle executing the current instruction
    cache                           : [u8;4],  // Cache for storing temporary values between M-cycles
    cache16                         : [u16;4], // Cache for storing temporary values between M-cycles

    int                             : Rc<RefCell<InterruptManager>>,
    is_transfer_control_interrupt   : bool,
    transfer_control_addr           : u16,

    // Schedule next M-cycle for IME
    schedule_ime                    : bool,                    

    // HALT/STOP flags
    is_halt                         : bool,
    enable_halt_bug                 : bool,

    is_stop                         : bool,

    // Timer counters
    t_cycle                         : u64,
}

impl CPU {
    pub fn new(bus :Rc<RefCell<Bus>>, int :Rc<RefCell<InterruptManager>>) -> CPU {
        return CPU {
            regs      : [0;REG_N as usize],
            pc        : 0,
            sp        : 0,
            opcode    : 0x00,
            opcode_cb : 0x00,
            bus,

            is_cb_opcode          : false,
            has_fetched_cb_opcode : false,
            is_instr_done         : false,  // Flag to know when to reset instruction
                                            // parameters and update the debugger
            int,
            is_transfer_control_interrupt   : false,
            transfer_control_addr           : 0x0000,

            cache           : [0;4],
            cache16         : [0;4],

            schedule_ime    : false,

            is_halt         : false, // Wait for interruptions
            enable_halt_bug : false,

            is_stop         : false, // Stop CPU

            instr_m_cycle   : 0,
            t_cycle         : 0,
        };
    }

    pub fn init(&mut self) {
        self.pc = 0x0100;
        self.sp = 0xFFFE;

        // Header checksum
        let f = if self.read(0x014d) == 0x00 { 0x80 } else { 0xB0 };

        self.set_reg(REG_A, 0x01);
        self.set_reg(REG_F, f);
        self.set_reg(REG_B, 0x00);
        self.set_reg(REG_C, 0x13);
        self.set_reg(REG_D, 0x00);
        self.set_reg(REG_E, 0xD8);
        self.set_reg(REG_H, 0x01);
        self.set_reg(REG_L, 0x4D);

        self.prefetch_opcode();
    }

    // For debugger and tests
    pub fn is_new_instr(&self)  -> bool   { return self.is_instr_done; }
    pub fn instr_m_cycle(&self) -> u8     { return self.instr_m_cycle; }
    pub fn get_opcode(&self)    -> u8     { return self.opcode; }
    pub fn get_opcode_cb(&self) -> u8     { return self.opcode_cb; }
    pub fn get_ime(&self)       -> bool   { return self.int.borrow().get_ime(); }
    pub fn get_pc(&self)        -> PCSIZE { return self.pc; }
    pub fn is_wait(&self)       -> bool   { return false; }
    pub fn get_sp(&self)        -> SPSIZE { return self.sp; }
    pub fn set_ime(&mut self, val :bool)  { self.int.borrow_mut().set_ime(val); }
    pub fn set_pc(&mut self, val :PCSIZE) { self.pc = val; }
    pub fn set_sp(&mut self, val :SPSIZE) { self.sp = val; }

    /*
     * Utility functions
     */
    pub fn to_u16(&self, hi :u8, lo :u8) -> u16 {
        return ((hi as u16) << 8) | lo as u16;
    }

    pub fn upper(&self, n :u16) -> u8 {
        return (n>>8) as u8;
    }

    pub fn lower(&self, n :u16) -> u8 {
        return (n & 0xff) as u8;
    }

    /*
     * Registers
     */

    // Stack pointer
    pub fn push(&mut self, n :RAMVAL) {
        self.sp = self.sp.wrapping_sub(1);
        self.write(self.sp as RAMINDEX, n);
    }

    pub fn pop(&mut self) -> RAMVAL {
        let val = self.read(self.sp as RAMINDEX);
        self.sp = self.sp.wrapping_add(1);

        return val;
    }

    // CPU registers
    pub fn reg(&self, r :REGINDEX) -> REG {
        if r == REG_F {
            // Lower 4 bits of F always return 0
            return self.regs[r as usize] & 0xF0;
        }
        else {
            return self.regs[r as usize];
        }
    }

    pub fn set_reg(&mut self, r :REGINDEX, val :REG) {
        if r == REG_F {
            self.regs[r as usize] = val & 0xf0; // F only can have its first 4 bits set
        }
        else {
            self.regs[r as usize] = val;
        }
    }

    /* Get 2-byte register */
    pub fn reg16(&self, r_hi :REGINDEX, r_lo :REGINDEX) -> u16 {
        return ((self.regs[r_hi as usize] as u16) << 8)
            | (self.regs[r_lo as usize] as u16);
    }

    /* Set 2-byte register */
    pub fn set_reg16(&mut self, r_hi :REGINDEX, r_lo :REGINDEX, val :u16) {
        self.set_reg(r_hi, (val >> 8) as REG);
        self.set_reg(r_lo, (val & 0xff) as REG);
    }

    /* Get Z */
    pub fn flag_z(&self) -> u8 {
        return (self.regs[REG_F as usize] >> 7) & 1;
    }

    /* Get N */
    pub fn flag_n(&self) -> u8 {
        return (self.regs[REG_F as usize] >> 6) & 1;
    }

    /* Get H */
    pub fn flag_h(&self) -> u8 {
        return (self.regs[REG_F as usize] >> 5) & 1;
    }

    /* Get C */
    pub fn flag_c(&self) -> u8 {
        return (self.regs[REG_F as usize] >> 4) & 1;
    }

    /* Set all the flags at once */
    pub fn set_flags(&mut self, z :u8, n :u8, h :u8, c :u8) {
        let flags = (z << 7) | (n << 6) | (h << 5) | (c << 4);
        self.set_reg(REG_F, flags);
    }

    /*
     * RAM utility functions
     */

    pub fn read(&self, addr :RAMINDEX) -> RAMVAL {
        return self.bus.borrow().read(addr);
    }

    pub fn write(&self, addr :RAMINDEX, val :RAMVAL) {
        self.bus.borrow_mut().write(addr, val);
    }

    /* Read RAM and increase PC */
    pub fn fetch(&mut self) -> RAMVAL {
        let val :RAMVAL = self.read(self.pc);

        // If the HALT bug is in place, dont increment PC
        // this time
        //if !self.enable_halt_bug {
            self.pc = self.pc.wrapping_add(1);
        /*} else {
            self.enable_halt_bug = false;
        }*/

        return val;
    }

    /* Prefetch next opcode and mark instruction as done */
    pub fn prefetch_opcode(&mut self) {
        self.opcode = self.fetch();
        self.is_instr_done = true;
    }

    /*
     * Instruction/cycle loop
     */

    /* Panic if the opcode is invalid */
    pub fn op_undefined(&self, op :u8) {
        panic!("[Error] run_instr(): 0x{:02X} is not a valid instruction", op);
    }

    /* Run the current instruction */
    pub fn run_instr(&mut self) {
        match self.opcode {
            /* nop */
            0x00 => self.nop(),

            /* ld */
            0x40 => self.ld_r_r(REG_B, REG_B),          // B <- B
            0x41 => self.ld_r_r(REG_B, REG_C),          // B <- C
            0x42 => self.ld_r_r(REG_B, REG_D),          // B <- D
            0x43 => self.ld_r_r(REG_B, REG_E),          // B <- E
            0x44 => self.ld_r_r(REG_B, REG_H),          // B <- H
            0x45 => self.ld_r_r(REG_B, REG_L),          // B <- L
            0x47 => self.ld_r_r(REG_B, REG_A),          // B <- A
            0x48 => self.ld_r_r(REG_C, REG_B),          // C <- B
            0x49 => self.ld_r_r(REG_C, REG_C),          // C <- C
            0x4a => self.ld_r_r(REG_C, REG_D),          // C <- D
            0x4b => self.ld_r_r(REG_C, REG_E),          // C <- E
            0x4c => self.ld_r_r(REG_C, REG_H),          // C <- H
            0x4d => self.ld_r_r(REG_C, REG_L),          // C <- L
            0x4f => self.ld_r_r(REG_C, REG_A),          // C <- A
            0x50 => self.ld_r_r(REG_D, REG_B),          // D <- B
            0x51 => self.ld_r_r(REG_D, REG_C),          // D <- C
            0x52 => self.ld_r_r(REG_D, REG_D),          // D <- D
            0x53 => self.ld_r_r(REG_D, REG_E),          // D <- E
            0x54 => self.ld_r_r(REG_D, REG_H),          // D <- H
            0x55 => self.ld_r_r(REG_D, REG_L),          // D <- L
            0x57 => self.ld_r_r(REG_D, REG_A),          // D <- A
            0x58 => self.ld_r_r(REG_E, REG_B),          // E <- B
            0x59 => self.ld_r_r(REG_E, REG_C),          // E <- C
            0x5a => self.ld_r_r(REG_E, REG_D),          // E <- D
            0x5b => self.ld_r_r(REG_E, REG_E),          // E <- E
            0x5c => self.ld_r_r(REG_E, REG_H),          // E <- H
            0x5d => self.ld_r_r(REG_E, REG_L),          // E <- L
            0x5f => self.ld_r_r(REG_E, REG_A),          // E <- A
            0x60 => self.ld_r_r(REG_H, REG_B),          // H <- B
            0x61 => self.ld_r_r(REG_H, REG_C),          // H <- C
            0x62 => self.ld_r_r(REG_H, REG_D),          // H <- D
            0x63 => self.ld_r_r(REG_H, REG_E),          // H <- E
            0x64 => self.ld_r_r(REG_H, REG_H),          // H <- H
            0x65 => self.ld_r_r(REG_H, REG_L),          // H <- L
            0x67 => self.ld_r_r(REG_H, REG_A),          // H <- A
            0x68 => self.ld_r_r(REG_L, REG_B),          // L <- B
            0x69 => self.ld_r_r(REG_L, REG_C),          // L <- C
            0x6a => self.ld_r_r(REG_L, REG_D),          // L <- D
            0x6b => self.ld_r_r(REG_L, REG_E),          // L <- E
            0x6c => self.ld_r_r(REG_L, REG_H),          // L <- H
            0x6d => self.ld_r_r(REG_L, REG_L),          // L <- L
            0x6f => self.ld_r_r(REG_L, REG_A),          // L <- A
            0x78 => self.ld_r_r(REG_A, REG_B),          // A <- B
            0x79 => self.ld_r_r(REG_A, REG_C),          // A <- C
            0x7a => self.ld_r_r(REG_A, REG_D),          // A <- D
            0x7b => self.ld_r_r(REG_A, REG_E),          // A <- E
            0x7c => self.ld_r_r(REG_A, REG_H),          // A <- H
            0x7d => self.ld_r_r(REG_A, REG_L),          // A <- L
            0x7f => self.ld_r_r(REG_A, REG_A),          // A <- A

            0x70 => self.ld_hl_r(REG_B),                             // (HL) <- B
            0x71 => self.ld_hl_r(REG_C),                             // (HL) <- C
            0x72 => self.ld_hl_r(REG_D),                             // (HL) <- D
            0x73 => self.ld_hl_r(REG_E),                             // (HL) <- E
            0x74 => self.ld_hl_r(REG_H),                             // (HL) <- H
            0x75 => self.ld_hl_r(REG_L),                             // (HL) <- L

            0x06 => self.ld_r_n(REG_B),                              // B <- n
            0x0e => self.ld_r_n(REG_C),                              // C <- n
            0x16 => self.ld_r_n(REG_D),                              // D <- n
            0x1e => self.ld_r_n(REG_E),                              // E <- n
            0x26 => self.ld_r_n(REG_H),                              // H <- n
            0x2e => self.ld_r_n(REG_L),                              // L <- n
            0x3e => self.ld_r_n(REG_A),                              // A <- n

            0x0a => self.ld_r_rr(REG_A, REG_B, REG_C),  // A <- (BC)
            0x1a => self.ld_r_rr(REG_A, REG_D, REG_E),  // A <- (DE)
            0x46 => self.ld_r_rr(REG_B, REG_H, REG_L),  // B <- (HL)
            0x4e => self.ld_r_rr(REG_C, REG_H, REG_L),  // C <- (HL)
            0x56 => self.ld_r_rr(REG_D, REG_H, REG_L),  // D <- (HL)
            0x5e => self.ld_r_rr(REG_E, REG_H, REG_L),  // E <- (HL)
            0x66 => self.ld_r_rr(REG_H, REG_H, REG_L),  // H <- (HL)
            0x6e => self.ld_r_rr(REG_L, REG_H, REG_L),  // L <- (HL)
            0x7e => self.ld_r_rr(REG_A, REG_H, REG_L),  // A <- (HL)

            0x02 => self.ld_rr_a(REG_B, REG_C),         // (BC) <- A
            0x12 => self.ld_rr_a(REG_D, REG_E),         // (DE) <- A
            0x77 => self.ld_rr_a(REG_H, REG_L),         // (HL) <- A

            0x01 => self.ld_rr_nn(REG_B, REG_C),        // BC <- nn
            0x11 => self.ld_rr_nn(REG_D, REG_E),        // DE <- nn
            0x21 => self.ld_rr_nn(REG_H, REG_L),        // HL <- nn

            0x36 => self.ld_hl_n(),                                // (HL) <- n
                
            0xfa => self.ld_a_nn(),                                // A <- (nn)
            0xea => self.ld_nn_a(),                                // (nn) <- A
                
            0x08 => self.ld_nn_sp(),                               // nn <- sp
            0x31 => self.ld_sp_nn(),                               // SP <- nn
                
            0xf9 => self.ld_sp_hl(),                               // SP <- HL
                
            0x22 => self.ldi_hl_a(),                               // (HL++) <- A
            0x2a => self.ldi_a_hl(),                               // A <- (HL++)
                
            0x32 => self.ldd_hl_a(),                               // (HL) <- A, HL--
            0x3a => self.ldd_a_hl(),                               // A <- (HL), HL--
                
            0xe0 => self.ldh_n_a(),                                // write A to io-port 0xFF00 + n
            0xf0 => self.ldh_a_n(),                                // read from io-port 0xFF00 + n to A
            0xf2 => self.ldh_a_c(),                                // read from io-port 0xFF00 + C to A
            0xe2 => self.ldh_c_a(),                                // write A to io-port 0xFF00 + C
                
            0xf8 => self.ldhl_sp_n(),                              // HL = SP + n

            /* inc */      
            0x04 => self.inc_r(REG_B),                             // B++, set flags
            0x0c => self.inc_r(REG_C),                             // C++, set flags
            0x14 => self.inc_r(REG_D),                             // D++, set flags
            0x1c => self.inc_r(REG_E),                             // E++, set flags
            0x24 => self.inc_r(REG_H),                             // H++, set flags
            0x2c => self.inc_r(REG_L),                             // L++, set flags
            0x3c => self.inc_r(REG_A),                             // A++, set flags
            
            0x03 => self.inc_rr(REG_B, REG_C),          // BC++
            0x13 => self.inc_rr(REG_D, REG_E),          // DE++
            0x23 => self.inc_rr(REG_H, REG_L),          // HL++
            
            0x33 => self.inc_sp(),                                 // SP++
            0x34 => self.inc_hl(),                                 // (HL)++

            /* dec */
            0x05 => self.dec_r(REG_B),                             // B--, set flags
            0x0d => self.dec_r(REG_C),                             // C--, set flags
            0x15 => self.dec_r(REG_D),                             // D--, set flags
            0x1d => self.dec_r(REG_E),                             // E--, set flags
            0x25 => self.dec_r(REG_H),                             // H--, set flags
            0x2d => self.dec_r(REG_L),                             // L--, set flags
            0x3d => self.dec_r(REG_A),                             // A--, set flags

            0x0b => self.dec_rr(REG_B, REG_C),          // BC--
            0x1b => self.dec_rr(REG_D, REG_E),          // DE--
            0x2b => self.dec_rr(REG_H, REG_L),          // HL--

            0x35 => self.dec_hl(),                                 // (HL)--
                
            0x3b => self.dec_sp(),                                 // SP--
                
            /* add */             
            0x80 => self.add_a_r(REG_B),                           // A += B
            0x81 => self.add_a_r(REG_C),                           // A += C
            0x82 => self.add_a_r(REG_D),                           // A += D
            0x83 => self.add_a_r(REG_E),                           // A += E
            0x84 => self.add_a_r(REG_H),                           // A += H
            0x85 => self.add_a_r(REG_L),                           // A += L
            0x87 => self.add_a_r(REG_A),                           // A += A 
            0x86 => self.add_a_hl(),                               // A += (HL)
            0xc6 => self.add_a_n(),                                // A += n
            0x09 => self.add_hl_rr(REG_B, REG_C),       // HL += BC
            0x19 => self.add_hl_rr(REG_D, REG_E),       // HL += DE
            0x29 => self.add_hl_rr(REG_H, REG_L),       // HL += HL
            0x39 => self.add_hl_sp(),                              // HL += SP
            0xe8 => self.add_sp_n(),                               // SP += n

            0x88 => self.adc_a_r(REG_B),                           // A += B with carry, set flags
            0x89 => self.adc_a_r(REG_C),                           // A += C with carry, set flags
            0x8a => self.adc_a_r(REG_D),                           // A += D with carry, set flags
            0x8b => self.adc_a_r(REG_E),                           // A += E with carry, set flags
            0x8c => self.adc_a_r(REG_H),                           // A += H with carry, set flags
            0x8d => self.adc_a_r(REG_L),                           // A += L with carry, set flags
            0x8f => self.adc_a_r(REG_A),                           // A += A with carry, set flags
            0x8e => self.adc_a_hl(),                               // A += (HL) with carry, set flags
            0xce => self.adc_a_n(),                                // A += n with carry, set flags
                
            /* sub */          
            0x90 => self.sub_a_r(REG_B),                           // A -= B, set flags
            0x91 => self.sub_a_r(REG_C),                           // A -= C, set flags
            0x92 => self.sub_a_r(REG_D),                           // A -= D, set flags
            0x93 => self.sub_a_r(REG_E),                           // A -= E, set flags
            0x94 => self.sub_a_r(REG_H),                           // A -= H, set flags
            0x95 => self.sub_a_r(REG_L),                           // A -= L, set flags
            0x97 => self.sub_a_r(REG_A),                           // A -= A, set flags
            0x96 => self.sub_a_hl(),                               // A -= (HL), set flags
            0xd6 => self.sub_a_n(),                                // A -= n
                
            0x98 => self.sbc_a_r(REG_B),                           // A -= B - c, set flags
            0x99 => self.sbc_a_r(REG_C),                           // A -= C - c, set flags
            0x9a => self.sbc_a_r(REG_D),                           // A -= D - c, set flags
            0x9b => self.sbc_a_r(REG_E),                           // A -= E - c, set flags
            0x9c => self.sbc_a_r(REG_H),                           // A -= H - c, set flags
            0x9d => self.sbc_a_r(REG_L),                           // A -= L - c, set flags
            0x9f => self.sbc_a_r(REG_A),                           // A -= A - c, set flags
            0x9e => self.sbc_a_hl(),                               // A -= (HL) - c, set flags
            0xde => self.sbc_a_n(),                                // A <- (n)
                
            /* rot left */         
            0x07 => self.rlca(),                                   // rot A left, set flags
            0x17 => self.rla(),                                    // rot A left with carry, set flags
                
            /* rot right */        
            0x0f => self.rrca(),                                   // rot A right, set flags
            0x1f => self.rra(),                                    // rot A right with carry, set flags
                
            /* stop */         
            0x10 => self.stop(),                                   // STOP: Halt CPU and LCD display until button pressed
                
            /* daa */
            0x27 => self.daa(),                                    // adjust A to BCD
                
            /* cpl */          
            0x2f => self.cpl(),                                    // complement of A
                
            /* scf */          
            0x37 => self.scf(),                                    // set carry flag
                
            /* ccf */          
            0x3f => self.ccf(),                                    // carry flag complement
                
            /* halt */         
            0x76 => self.halt(),                                   // HALT: Power down the CPU until an interrupt occurs
                
            /* and */          
            0xa0 => self.and_a_r(REG_B),                           // A &= B, set flags
            0xa1 => self.and_a_r(REG_C),                           // A &= C, set flags
            0xa2 => self.and_a_r(REG_D),                           // A &= D, set flags
            0xa3 => self.and_a_r(REG_E),                           // A &= E, set flags
            0xa4 => self.and_a_r(REG_H),                           // A &= H, set flags
            0xa5 => self.and_a_r(REG_L),                           // A &= L, set flags
            0xa7 => self.and_a_r(REG_A),                           // A &= A, set flags
            0xa6 => self.and_a_hl(),                               // A &= (HL), set flags
            0xe6 => self.and_a_n(),                                // A &= n
                
            /* xor */          
            0xa8 => self.xor_a_r(REG_B),                           // A ^= B, set flags
            0xa9 => self.xor_a_r(REG_C),                           // A ^= C, set flags
            0xaa => self.xor_a_r(REG_D),                           // A ^= D, set flags
            0xab => self.xor_a_r(REG_E),                           // A ^= E, set flags
            0xac => self.xor_a_r(REG_H),                           // A ^= H, set flags
            0xad => self.xor_a_r(REG_L),                           // A ^= L, set flags
            0xaf => self.xor_a_r(REG_A),                           // A ^= A, set flags
            0xae => self.xor_a_hl(),                               // A ^= (HL), set flags
            0xee => self.xor_a_n(),                                // A ^= n

            /* or */
            0xb0 => self.or_a_r(REG_B),                            // A |= B, set flags
            0xb1 => self.or_a_r(REG_C),                            // A |= C, set flags
            0xb2 => self.or_a_r(REG_D),                            // A |= D, set flags
            0xb3 => self.or_a_r(REG_E),                            // A |= E, set flags
            0xb4 => self.or_a_r(REG_H),                            // A |= H, set flags
            0xb5 => self.or_a_r(REG_L),                            // A |= L, set flags
            0xb7 => self.or_a_r(REG_A),                            // A |= A, set flags
            0xb6 => self.or_a_hl(),                                // A |= (HL), set flags
            0xf6 => self.or_a_n(),                                 // A |= n
                
            /* cp */           
            0xb8 => self.cp_a_r(REG_B),                            // comp A B, set flags
            0xb9 => self.cp_a_r(REG_C),                            // comp A C, set flags
            0xba => self.cp_a_r(REG_D),                            // comp A D, set flags
            0xbb => self.cp_a_r(REG_E),                            // comp A E, set flags
            0xbc => self.cp_a_r(REG_H),                            // comp A H, set flags
            0xbd => self.cp_a_r(REG_L),                            // comp A L, set flags
            0xbf => self.cp_a_r(REG_A),                            // comp A A, set flags
            0xbe => self.cp_a_hl(),                                // comp A (HL), set flags
            0xfe => self.cp_a_n(),                                 // comp A n
                
            /* push */
            0xc5 => self.push_rr(REG_B, REG_C),         // push BC
            0xd5 => self.push_rr(REG_D, REG_E),         // push DE
            0xe5 => self.push_rr(REG_H, REG_L),         // push HL
            0xf5 => self.push_rr(REG_A, REG_F),         // push AF

            /* pop */
            0xc1 => self.pop_rr(REG_B, REG_C),          // BC <- pop()
            0xd1 => self.pop_rr(REG_D, REG_E),          // DE <- pop()
            0xe1 => self.pop_rr(REG_H, REG_L),          // HL <- pop()
            0xf1 => self.pop_rr(REG_A, REG_F),          // AF <- pop()

            /* jp */
            0xc2 => self.jp_cc_nn(JmpCond::NZ),              // jmp nn if z == 1
            0xca => self.jp_cc_nn(JmpCond::Z),               // jmp nn if z == 0
            0xd2 => self.jp_cc_nn(JmpCond::NC),              // jmp nn if c == 1
            0xda => self.jp_cc_nn(JmpCond::C),               // jmp nn if c == 0
            
            0xc3 => self.jp_nn(),                                 // jmp nn 
            0xe9 => self.jp_hl(),                                 // jmp HL

            /* jr */
            0x18 => self.jr(),                                     // PC += n
            0x20 => self.jr_cc_n(JmpCond::NZ),               // PC += n if z == 1
            0x28 => self.jr_cc_n(JmpCond::Z),                // pc += n if z == 0
            0x30 => self.jr_cc_n(JmpCond::NC),               // PC += n if c == 1
            0x38 => self.jr_cc_n(JmpCond::C),                // PC += n if c == 0

            /* call */      
            0xc4 => self.call_cc_nn(JmpCond::NZ),            // call nn if z == 1
            0xcc => self.call_cc_nn(JmpCond::Z),             // call nn if z == 0
            0xd4 => self.call_cc_nn(JmpCond::NC),            // call nn if c == 1
            0xdc => self.call_cc_nn(JmpCond::C),             // jmp  nn if c == 0
            0xcd => self.call_nn(),                                // call nn

            /* ret */
            0xc9 => self.ret(),                                    // return to addr in top of stack
            0xd9 => self.reti(),                                   // ret, enable interrupts

            0xc0 => self.ret_cc(JmpCond::NZ),               // ret if z == 1
            0xc8 => self.ret_cc(JmpCond::Z),                // ret if z == 0
            0xd0 => self.ret_cc(JmpCond::NC),               // ret if c == 1
            0xd8 => self.ret_cc(JmpCond::C),                // ret if c == 0

            /* CB-prefixed opcodes */
            0xcb => {
                // If its the first cycle on the cb opcode,
                // spend another 4 t-cycles fetching it
                if !self.is_cb_opcode {
                    self.is_cb_opcode = true;
                    self.opcode_cb = self.fetch();
                    self.instr_m_cycle = 0;

                    return;
                }

                // Flag to prevent interrupts between CB prefixed fetches
                if !self.has_fetched_cb_opcode {
                    self.has_fetched_cb_opcode = true;
                }

                match self.opcode_cb {
                    /* left rotate */
                    0x00 => self.rlc_r(REG_B),          // left rotate B
                    0x01 => self.rlc_r(REG_C),          // left rotate C
                    0x02 => self.rlc_r(REG_D),          // left rotate D
                    0x03 => self.rlc_r(REG_E),          // left rotate E
                    0x04 => self.rlc_r(REG_H),          // left rotate H
                    0x05 => self.rlc_r(REG_L),          // left rotate L
                    0x07 => self.rlc_r(REG_A),          // left rotate A
                    0x06 => self.rlc_hl(),              // left rotate HL

                    /* right rotate */
                    0x08 => self.rrc_r(REG_B),          // right rotate B
                    0x09 => self.rrc_r(REG_C),          // right rotate C
                    0x0a => self.rrc_r(REG_D),          // right rotate D
                    0x0b => self.rrc_r(REG_E),          // right rotate E
                    0x0c => self.rrc_r(REG_H),          // right rotate H
                    0x0d => self.rrc_r(REG_L),          // right rotate L
                    0x0f => self.rrc_r(REG_A),          // right rotate A
                    0x0e => self.rrc_hl(),              // right rotate (HL)

                    /* left rotate with carry */
                    0x10 => self.rl_r(REG_B),           // left rotate B with carry
                    0x11 => self.rl_r(REG_C),           // left rotate C with carry
                    0x12 => self.rl_r(REG_D),           // left rotate D with carry
                    0x13 => self.rl_r(REG_E),           // left rotate E with carry
                    0x14 => self.rl_r(REG_H),           // left rotate H with carry
                    0x15 => self.rl_r(REG_L),           // left rotate L with carry
                    0x17 => self.rl_r(REG_A),           // left rotate A with carry
                    0x16 => self.rl_hl(),               // left rotate (HL) with carry

                    /* right rotate with carry */
                    0x18 => self.rr_r(REG_B),           // right rotate B with carry
                    0x19 => self.rr_r(REG_C),           // right rotate C with carry
                    0x1a => self.rr_r(REG_D),           // right rotate D with carry
                    0x1b => self.rr_r(REG_E),           // right rotate E with carry
                    0x1c => self.rr_r(REG_H),           // right rotate H with carry
                    0x1d => self.rr_r(REG_L),           // right rotate L with carry
                    0x1f => self.rr_r(REG_A),           // right rotate A with carry
                    0x1e => self.rr_hl(),               // right rotate (HL) with carry

                    /* shift left into carry */
                    0x20 => self.sla_r(REG_B),          // shift B left into c. LSB of r set to 0
                    0x21 => self.sla_r(REG_C),          // shift C left into c. LSB of r set to 0
                    0x22 => self.sla_r(REG_D),          // shift D left into c. LSB of r set to 0
                    0x23 => self.sla_r(REG_E),          // shift E left into c. LSB of r set to 0
                    0x24 => self.sla_r(REG_H),          // shift H left into c. LSB of r set to 0
                    0x25 => self.sla_r(REG_L),          // shift L left into c. LSB of r set to 0
                    0x27 => self.sla_r(REG_A),          // shift A left into c. LSB of r set to 0
                    0x26 => self.sla_hl(),              // shift (HL) left into c. LSB of (HL) set to 0

                    /* shift right into carry */
                    0x28 => self.sra_r(REG_B),          // shift B right into c. MSB of r doesnt change
                    0x29 => self.sra_r(REG_C),          // shift C right into c. MSB of r doesnt change
                    0x2a => self.sra_r(REG_D),          // shift D right into c. MSB of r doesnt change
                    0x2b => self.sra_r(REG_E),          // shift E right into c. MSB of r doesnt change
                    0x2c => self.sra_r(REG_H),          // shift H right into c. MSB of r doesnt change
                    0x2d => self.sra_r(REG_L),          // shift L right into c. MSB of r doesnt change
                    0x2f => self.sra_r(REG_A),          // shift A right into c. MSB of r doesnt change
                    0x2e => self.sra_hl(),              // shift (HL) right into c. MSB of (HL) doesnt change

                    0x38 => self.srl_r(REG_B),          // shift B right into c. MSB set to 0
                    0x39 => self.srl_r(REG_C),          // shift C right into c. MSB set to 0
                    0x3a => self.srl_r(REG_D),          // shift D right into c. MSB set to 0
                    0x3b => self.srl_r(REG_E),          // shift E right into c. MSB set to 0
                    0x3c => self.srl_r(REG_H),          // shift H right into c. MSB set to 0
                    0x3d => self.srl_r(REG_L),          // shift L right into c. MSB set to 0
                    0x3f => self.srl_r(REG_A),          // shift A right into c. MSB set to 0
                    0x3e => self.srl_hl(),              // shift (HL) right into c. MSB set to 0

                    /* swap nibbles */
                    0x30 => self.swap_r(REG_B),         // swap lower and upper nibbles of B
                    0x31 => self.swap_r(REG_C),         // swap lower and upper nibbles of C
                    0x32 => self.swap_r(REG_D),         // swap lower and upper nibbles of D
                    0x33 => self.swap_r(REG_E),         // swap lower and upper nibbles of E
                    0x34 => self.swap_r(REG_H),         // swap lower and upper nibbles of H
                    0x35 => self.swap_r(REG_L),         // swap lower and upper nibbles of L
                    0x37 => self.swap_r(REG_A),         // swap lower and upper nibbles of A
                    0x36 => self.swap_hl(),             // swap lower and upper nibbles of (HL)

                    /* bit / set / reset */
                    _ => {
                        let (hi, lo) = (self.opcode_cb >> 4, self.opcode_cb & 0x0f);

                        let b = match hi {
                            0x04 | 0x08 | 0x0c => 1 - (lo < 0x08) as u8,
                            0x05 | 0x09 | 0x0d => 3 - (lo < 0x08) as u8,
                            0x06 | 0x0a | 0x0e => 5 - (lo < 0x08) as u8,
                            0x07 | 0x0b | 0x0f => 7 - (lo < 0x08) as u8,
                            _ => panic!("0xcb bit/set/res: Invalid bit")
                        };

                        if lo == 0x06 || lo == 0x0e { // HL
                            match self.opcode_cb {
                                0x40..=0x7f => self.bit_hl(b), // test bit b of (HL)
                                0x80..=0xbf => self.res_hl(b), // reset bit b of (HL)
                                0xc0..=0xff => self.set_hl(b), // set bit b of (HL)
                                _ => panic!("0xcb bit/set/res HL: Invalid range: 0x{:x}", self.opcode_cb)
                            }
                        }
                        else { // r
                            let r = match lo {
                                0x00 | 0x08 => REG_B,
                                0x01 | 0x09 => REG_C,
                                0x02 | 0x0a => REG_D,
                                0x03 | 0x0b => REG_E,
                                0x04 | 0x0c => REG_H,
                                0x05 | 0x0d => REG_L,
                                0x07 | 0x0f => REG_A,
                                _ => panic!("0xcb bit/set/res: Invalid register")
                            };

                            match self.opcode_cb {
                                0x40..=0x7f => self.bit_r(r, b), // test bit b of r
                                0x80..=0xbf => self.res_r(r, b), // reset bit b of r
                                0xc0..=0xff => self.set_r(r, b), // set bit b of r
                                _ => panic!("0xcb bit/set/res: Invalid range: 0x{:x}", self.opcode_cb)
                            }
                        }
                    }
                }
            },
            0xf3 => self.di(),                          // disable interrupts
            0xfb => self.ei(),                          // enable interrupts

            /* rst */
            0xc7 => self.rst(0x00),                // PC = 0x00
            0xcf => self.rst(0x08),                // PC = 0x08
            0xd7 => self.rst(0x10),                // PC = 0x10
            0xdf => self.rst(0x18),                // PC = 0x18
            0xe7 => self.rst(0x20),                // PC = 0x20
            0xef => self.rst(0x28),                // PC = 0x28
            0xf7 => self.rst(0x30),                // PC = 0x30
            0xff => self.rst(0x38),                // PC = 0x38

            /* undefined opcodes */
            0xd3 => self.op_undefined(0xd3),
            0xdb => self.op_undefined(0xdb),
            0xdd => self.op_undefined(0xdd),
            0xe3 => self.op_undefined(0xe3),
            0xe4 => self.op_undefined(0xe4),
            0xeb => self.op_undefined(0xeb),
            0xec => self.op_undefined(0xec),
            0xed => self.op_undefined(0xed),
            0xf4 => self.op_undefined(0xf4),
            0xfc => self.op_undefined(0xfc),
            0xfd => self.op_undefined(0xfd),
        };
    }

    pub fn tick(&mut self) {
        self.t_cycle += 1; // Update the actual cycle n;
        self.is_instr_done = false;

        // Why % and not == 4 ?
        if self.t_cycle%4 == 0 {
            self.t_cycle = 0;

            // 1 M-cycle
            // TODO: Check off-by-one errors
            if self.is_halt {
                // Wait for ie and if
                if ((self.read(ADDR_IE)&0x1F) & (self.read(ADDR_IF)&0x1F)) != 0 {
                    self.is_halt = false;
                    self.handle_interrupts();

                    //if self.enable_halt_bug {
                    //    self.opcode = self.read(self.pc);
                    //}
                }
            } else {
                self.instr_m_cycle += 1;

                // Enable IME after one M-cycle
                if self.schedule_ime {
                    self.schedule_ime = false;
                    self.int.borrow_mut().set_ime(true);
                }

                if self.is_transfer_control_interrupt {
                    self.transfer_control_int(self.transfer_control_addr);
                } else {
                    self.run_instr();
                }

                if self.is_instr_done {
                    self.instr_m_cycle = 0;
                    self.is_transfer_control_interrupt = false;
                    
                    self.is_cb_opcode          = false;
                    self.has_fetched_cb_opcode = false;

                    self.handle_interrupts();
                }
            }
        }
    }

    /*
     Check if there are any interrupts and if so, disable the first available and jump
     to the corresponding interrupt address
     */
    pub fn handle_interrupts(&mut self) {
        if self.int.borrow().has_interrupts() {
            let interrupt_opt = self.int.borrow().get_interrupt();

            // If it's not None
            if let Some(interrupt) = interrupt_opt {
                let addr = self.int.borrow().get_jmp_address(&interrupt);
                //println!("[interrupt] {:?} {:04x} pc {:04x} sp {:04x}", &interrupt, addr, self.pc, self.sp);
                self.int.borrow_mut().disable_interrupt_request(&interrupt);

                self.is_transfer_control_interrupt = true;
                self.transfer_control_addr = addr;

                self.pc -= 1; // TODO: Why?
                // TODO: Why?
                self.is_cb_opcode          = false;
                self.has_fetched_cb_opcode = false;
            }
        }
    }
}
