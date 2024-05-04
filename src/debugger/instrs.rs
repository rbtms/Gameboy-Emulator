use crate::debugger::Debugger;

impl Debugger {
    pub fn instr_text(&self, op :u8) -> String {
        match op {
            /* nop */                                                                                     
            0x00 => "NOP",

            /* ld */                                                                                      
            0x40 => "LD B B",                   // B <- B                                                 
            0x41 => "LD B C",                   // B <- C                                                 
            0x42 => "LD B D",                   // B <- D                                                 
            0x43 => "LD B E",                   // B <- E
            0x44 => "LD B H",                   // B <- H                                                 
            0x45 => "LD B L",                   // B <- L                                                 
            0x47 => "LD B A",                   // B <- A
            0x48 => "LD C B",                   // C <- B
            0x49 => "LD C C",                   // C <- C
            0x4a => "LD C D",                   // C <- D
            0x4b => "LD C E",                   // C <- E
            0x4c => "LD C H",                   // C <- H
            0x4d => "LD C L",                   // C <- L
            0x4f => "LD C A",                   // C <- A
            0x50 => "LD D B",                   // D <- B
            0x51 => "LD D C",                   // D <- C
            0x52 => "LD D D",                   // D <- D
            0x53 => "LD D E",                   // D <- E
            0x54 => "LD D H",                   // D <- H
            0x55 => "LD D L",                   // D <- L
            0x57 => "LD D A",                   // D <- A
            0x58 => "LD E B",                   // E <- B
            0x59 => "LD E C",                   // E <- C
            0x5a => "LD E D",                   // E <- D
            0x5b => "LD E E",                   // E <- E
            0x5c => "LD E H",                   // E <- H
            0x5d => "LD E L",                   // E <- L
            0x5f => "LD E A",                   // E <- A
            0x60 => "LD H B",                   // H <- B
            0x61 => "LD H C",                   // H <- C
            0x62 => "LD H D",                   // H <- D
            0x63 => "LD H E",                   // H <- E
            0x64 => "LD H H",                   // H <- H
            0x65 => "LD H L",                   // H <- L
            0x67 => "LD H A",                   // H <- A
            0x68 => "LD L B",                   // L <- B
            0x69 => "LD L C",                   // L <- C
            0x6a => "LD L D",                   // L <- D
            0x6b => "LD L E",                   // L <- E
            0x6c => "LD L H",                   // L <- H
            0x6d => "LD L L",                   // L <- L
            0x6f => "LD L A",                   // L <- A
            0x78 => "LD A B",                   // A <- B
            0x79 => "LD A C",                   // A <- C
            0x7a => "LD A D",                   // A <- D
            0x7b => "LD A E",                   // A <- E
            0x7c => "LD A H",                   // A <- H
            0x7d => "LD A L",                   // A <- L
            0x7f => "LD A A",                   // A <- A
            0x70 => "LD (HL) B",                // (HL) <- B
            0x71 => "LD (HL) C",                // (HL) <- C
            0x72 => "LD (HL) D",                // (HL) <- D
            0x73 => "LD (HL) E",                // (HL) <- E
            0x74 => "LD (HL) H",                // (HL) <- H
            0x75 => "LD (HL) L",                // (HL) <- L
            0x06 => "LD B {n}",                 // B <- n
            0x0e => "LD C {n}",                 // C <- n
            0x16 => "LD D {n}",                 // D <- n
            0x1e => "LD E {n}",                 // E <- n
            0x26 => "LD H {n}",                 // H <- n
            0x2e => "LD L {n}",                 // L <- n
            0x3e => "LD A {n}",                 // A <- n
            0x0a => "LD A (BC)",                // A <- (BC)
            0x1a => "LD A (DE)",                // A <- (DE)
            0x46 => "LD B (HL)",                // B <- (HL)
            0x4e => "LD C (HL)",                // C <- (HL)
            0x56 => "LD D (HL)",                // D <- (HL)
            0x5e => "LD E (HL)",                // E <- (HL)
            0x66 => "LD H (HL)",                // H <- (HL)
            0x6e => "LD L (HL)",                // L <- (HL)
            0x7e => "LD A (HL)",                // A <- (HL)
            0x02 => "LD (BC) A",                // (BC) <- A
            0x12 => "LD (DE) A",                // (DE) <- A
            0x77 => "LD (HL) A",                // (HL) <- A
            0x01 => "LD BC {nn}",               // BC <- nn
            0x11 => "LD DE {nn}",               // DE <- nn
            0x21 => "LD HL {nn}",               // HL <- nn
            0x36 => "LD (HL) {n}",              // (HL) <- n
            0xfa => "LD A ({nn})",              // A <- (nn)
            0xea => "LD ({nn}) A",              // (nn) <- A
            0x08 => "LD ({nn}) SP",             // nn <- sp
            0x31 => "LD SP {nn}",               // SP <- nn
            0xf9 => "LD SP HL",                 // SP <- HL
            0x22 => "LD (HL++) A",              // (HL++) <- A
            0x2a => "LD A (HL++)",              // A <- (HL++)
            0x32 => "LD (HL--) A",              // (HL) <- A, HL--
            0x3a => "LD A (HL--)",              // A <- (HL), HL--
            0xe0 => "LD ({io+n}) A",              // write A to io-port 0xFF00 + n
            0xf0 => "LD A ({io+n})",            // read from io-port 0xFF00 + n to A
            0xf2 => "LD A (0xFF00 + C)",        // read from io-port 0xFF00 + C to A
            0xe2 => "LD (0xFF00 + C) A",        // write A to io-port 0xFF00 + C
            0xf8 => "LD HL SP+{n}",             // HL = SP + n

            /* inc */
            0x04 => "INC B",                    // B++, set flags
            0x0c => "INC C",                    // C++, set flags
            0x14 => "INC D",                    // D++, set flags
            0x1c => "INC E",                    // E++, set flags
            0x24 => "INC H",                    // H++, set flags
            0x2c => "INC L",                    // L++, set flags
            0x3c => "INC A",                    // A++, set flags
            0x03 => "INC BC",                   // BC++
            0x13 => "INC DE",                   // DE++
            0x23 => "INC HL",                   // HL++
            0x33 => "INC SP",                   // SP++
            0x34 => "INC (HL)",                 // (HL)++

            /* dec */
            0x05 => "DEC B",                    // B--, set flags
            0x0d => "DEC C",                    // C--, set flags
            0x15 => "DEC D",                    // D--, set flags
            0x1d => "DEC E",                    // E--, set flags
            0x25 => "DEC H",                    // H--, set flags
            0x2d => "DEC L",                    // L--, set flags
            0x3d => "DEC A",                    // A--, set flags
            0x0b => "DEC BC",                   // BC--
            0x1b => "DEC DE",                   // DE--
            0x2b => "DEC HL",                   // HL--
            0x35 => "DEC (HL)",                 // (HL)--
            0x3b => "DEC SP",                   // SP--

            /* add */
            0x80 => "ADD A B",                  // A += B
            0x81 => "ADD A C",                  // A += C
            0x82 => "ADD A D",                  // A += D
            0x83 => "ADD A E",                  // A += E
            0x84 => "ADD A H",                  // A += H
            0x85 => "ADD A L",                  // A += L
            0x87 => "ADD A A",                  // A += A 
            0x86 => "ADD A (HL)",               // A += (HL)
            0xc6 => "ADD A {n}",                // A += n
            0x09 => "ADD HL BC",                // HL += BC
            0x19 => "ADD HL DE",                // HL += DE
            0x29 => "ADD HL HL",                // HL += HL
            0x39 => "ADD HL SP",                // HL += SP
            0xe8 => "ADD SPS {n_i8}",           // SP += n
            0x88 => "ADC A B",                  // A += B with carry, set flags
            0x89 => "ADC A C",                  // A += B with carry, set flags
            0x8a => "ADC A D",                  // A += B with carry, set flags
            0x8b => "ADC A E",                  // A += B with carry, set flags
            0x8c => "ADC A H",                  // A += B with carry, set flags
            0x8d => "ADC A L",                  // A += B with carry, set flags
            0x8f => "ADC A A",                  // A += A with carry, set flags
            0x8e => "ADC A (HL)",               // A += (HL) with carry, set flags
            0xce => "ADC A {n}",                // A += n with carry, set flags

            /* sub */
            0x90 => "SUB A B",                  // A -= B, set flags
            0x91 => "SUB A C",                  // A -= C, set flags
            0x92 => "SUB A D",                  // A -= D, set flags
            0x93 => "SUB A E",                  // A -= E, set flags
            0x94 => "SUB A H",                  // A -= H, set flags
            0x95 => "SUB A L",                  // A -= L, set flags
            0x97 => "SUB A A",                  // A -= A, set flags
            0x96 => "SUB A (HL)",               // A -= (HL), set flags
            0xd6 => "SUB A {n}",                // A -= n
            0x98 => "SBC A B",                  // A -= B - c, set flags
            0x99 => "SBC A C",                  // A -= C - c, set flags
            0x9a => "SBC A D",                  // A -= D - c, set flags
            0x9b => "SBC A E",                  // A -= E - c, set flags
            0x9c => "SBC A H",                  // A -= H - c, set flags
            0x9d => "SBC A L",                  // A -= L - c, set flags
            0x9f => "SBC A A",                  // A -= A - c, set flags
            0x9e => "SBC A (HL)",               // A -= (HL) - c, set flags
            0xde => "SBC A {n}",                // A <- n

            /* rot left */
            0x07 => "RLCA",                     // rot A left, set flags
            0x17 => "RLA",                      // rot A left with carry, set flags

            /* rot right */
            0x0f => "RRCA",                     // rot A right, set flags
            0x1f => "RRA",                      // rot A right with carry, set flags

            /* stop */
            0x10 => "STOP",                     // STOP: Halt CPU and LCD display until button pressed

            /* daa */
            0x27 => "DAA",                      // adjust A to BCD

            /* cpl */
            0x2f => "CPL A",                    // complement of A

            /* scf */
            0x37 => "SCF",                      // set carry flag

            /* ccf */
            0x3f => "CCF",                      // carry flag complement

            /* halt */
            0x76 => "HALT",                     // HALT: Power down the CPU until an interrupt occurs

            /* and */
            0xa0 => "AND A B",                  // A &= B, set flags
            0xa1 => "AND A C",                  // A &= C, set flags
            0xa2 => "AND A D",                  // A &= D, set flags
            0xa3 => "AND A E",                  // A &= E, set flags
            0xa4 => "AND A H",                  // A &= H, set flags
            0xa5 => "AND A L",                  // A &= L, set flags
            0xa7 => "AND A A",                  // A &= A, set flags
            0xa6 => "AND A (HL)",               // A &= HL, set flags
            0xe6 => "AND A {n}",                // A &= n

            /* xor */
            0xa8 => "XOR A B",                  // A ^= B, set flags
            0xa9 => "XOR A C",                  // A ^= B, set flags
            0xaa => "XOR A D",                  // A ^= B, set flags
            0xab => "XOR A E",                  // A ^= B, set flags
            0xac => "XOR A H",                  // A ^= B, set flags
            0xad => "XOR A L",                  // A ^= B, set flags
            0xaf => "XOR A A",                  // A ^= A, set flags
            0xae => "XOR A (HL)",               // A ^= (hl), set flags
            0xee => "XOR A {n}",                // A ^= n

            /* or */
            0xb0 => "OR A B",                   // A |= B, set flags
            0xb1 => "OR A C",                   // A |= C, set flags
            0xb2 => "OR A D",                   // A |= D, set flags
            0xb3 => "OR A E",                   // A |= E, set flags
            0xb4 => "OR A H",                   // A |= H, set flags
            0xb5 => "OR A L",                   // A |= L, set flags
            0xb7 => "OR A A",                   // A |= A, set flags
            0xb6 => "OR A (HL)",                // A |= (HL), set flags
            0xf6 => "OR A {n}",                 // A |= n

            /* cp */
            0xb8 => "COMP A B",                 // comp A B, set flags
            0xb9 => "COMP A C",                 // comp A C, set flags
            0xba => "COMP A D",                 // comp A D, set flags
            0xbb => "COMP A E",                 // comp A E, set flags
            0xbc => "COMP A H",                 // comp A H, set flags
            0xbd => "COMP A L",                 // comp A L, set flags
            0xbf => "COMP A A",                 // comp A A, set flags
            0xbe => "COMP A (HL)",              // comp A (HL), set flags
            0xfe => "COMP A {n}",               // comp A n

            /* push */
            0xc5 => "PUSH BC",                  // push BC
            0xd5 => "PUSH DE",                  // push DE
            0xe5 => "PUSH HL",                  // push HL
            0xf5 => "PUSH AF",                  // push AF

            /* pop */
            0xc1 => "POP BC",                   // BC <- pop()
            0xd1 => "POP DE",                   // DE <- pop()
            0xe1 => "POP HL",                   // HL <- pop()
            0xf1 => "POP AF",                   // AF <- pop()

            /* jp */
            0xc2 => "JMP {nn} if z == 0",       // jmp nn if z == 0
            0xca => "JMP {nn} if z == 1",       // jmp nn if z == 1
            0xd2 => "JMP {nn} if c == 0",       // jmp nn if c == 0
            0xda => "JMP {nn} if c == 1",       // jmp nn if c == 1
            0xc3 => "JMP {nn}",                 // jmp nn 
            0xe9 => "JMP HL",                   // jmp HL

            /* jr */
            0x18 => "JR {pc+n_i8}",                   // PC += n
            0x20 => "JR {pc+n_i8} if z == 0",         // PC += n if z == 0
            0x28 => "JR {pc+n_i8} if z == 1",         // pc += n if z == 1
            0x30 => "JR {pc+n_i8} if c == 0",         // PC += n if c == 0
            0x38 => "JR {pc+n_i8} if c == 1",         // PC += n if c == 1

            /* call */
            0xc4 => "CALL {nn} if z == 0",      // call nn if z == 0
            0xcc => "CALL {nn} if z == 1",      // call nn if z == 1
            0xd4 => "CALL {nn} if c == 0",      // call nn if c == 0
            0xdc => "CALL {nn} if c == 1",      // call nn if c == 1
            0xcd => "CALL {nn}",                // call nn

            /* ret */
            0xc9 => "RET",                      // return to addr in top of stack
            0xd9 => "RETI",                     // ret, enable interrupts

            0xc0 => "RET if z == 0",            // ret if z == 1
            0xc8 => "RET if z == 1",            // ret if z == 0
            0xd0 => "RET if c == 0",            // ret if c == 1
            0xd8 => "RET if c == 1",            // ret if c == 0

            /* CB-prefixed opcodes */
            0xcb => "cb: unimplemented",
            0xf3 => "DI",                       // disable interrupts
            0xfb => "EI",                       // enable interrupts

            /* rst */
            0xc7 => "RST 0000h",                 // PC = 0x00
            0xcf => "RST 0008h",                 // PC = 0x08
            0xd7 => "RST 0010h",                 // PC = 0x10
            0xdf => "RST 0018h",                 // PC = 0x18
            0xe7 => "RST 0020h",                 // PC = 0x20
            0xef => "RST 0028h",                 // PC = 0x28
            0xf7 => "RST 0030h",                 // PC = 0x30
            0xff => "RST 0038h",                 // PC = 0x38

            /* undefined opcodes */
            0xd3 => "undefined",
            0xdb => "undefined",
            0xdd => "undefined",
            0xe3 => "undefined",
            0xe4 => "undefined",
            0xeb => "undefined",
            0xec => "undefined",
            0xed => "undefined",
            0xf4 => "undefined",
            0xfc => "undefined",
            0xfd => "undefined",
        }.to_string()
    }

    pub fn instr_cb_text(&self, opcode_cb :u8) -> String {
        match opcode_cb {
            /* left rotate */
            0x00 => "RLC B",                // left rotate B
            0x01 => "RLC C",                // left rotate C
            0x02 => "RLC D",                // left rotate D
            0x03 => "RLC E",                // left rotate E
            0x04 => "RLC H",                // left rotate H
            0x05 => "RLC L",                // left rotate L
            0x07 => "RLC A",                // left rotate A
            0x06 => "RLC (HL)",             // left rotate HL

            /* right rotate */
            0x08 => "RRC B",                // right rotate B
            0x09 => "RRC C",                // right rotate C
            0x0a => "RRC D",                // right rotate D
            0x0b => "RRC E",                // right rotate E
            0x0c => "RRC H",                // right rotate H
            0x0d => "RRC L",                // right rotate L
            0x0f => "RRC A",                // right rotate A
            0x0e => "RRC (HL)",             // right rotate (HL)

            /* left rotate with carry */
            0x10 => "RL B",                 // left rotate B with carry
            0x11 => "RL C",                 // left rotate C with carry
            0x12 => "RL D",                 // left rotate D with carry
            0x13 => "RL E",                 // left rotate E with carry
            0x14 => "RL H",                 // left rotate H with carry
            0x15 => "RL L",                 // left rotate L with carry
            0x17 => "RL A",                 // left rotate A with carry
            0x16 => "RL (HL)",              // left rotate (HL) with carry

            /* right rotate with carry */
            0x18 => "RR B",                 // right rotate B with carry
            0x19 => "RR C",                 // right rotate C with carry
            0x1a => "RR D",                 // right rotate D with carry
            0x1b => "RR E",                 // right rotate E with carry
            0x1c => "RR H",                 // right rotate H with carry
            0x1d => "RR L",                 // right rotate L with carry
            0x1f => "RR A",                 // right rotate A with carry
            0x1e => "RR (HL)",              // right rotate (HL) with carry

            /* shift left into carry */
            0x20 => "SLA B",                // shift B left into c. LSB of r set to 0
            0x21 => "SLA C",                // shift C left into c. LSB of r set to 0
            0x22 => "SLA D",                // shift D left into c. LSB of r set to 0
            0x23 => "SLA E",                // shift E left into c. LSB of r set to 0
            0x24 => "SLA H",                // shift H left into c. LSB of r set to 0
            0x25 => "SLA L",                // shift L left into c. LSB of r set to 0
            0x27 => "SLA A",                // shift A left into c. LSB of r set to 0
            0x26 => "SLA (HL)",             // shift (HL) left into c. LSB of (HL) set to 0

            /* shift right into carry */
            0x28 => "SRA B",                // shift B right into c. MSB of r doesnt change
            0x29 => "SRA C",                // shift C right into c. MSB of r doesnt change
            0x2a => "SRA D",                // shift D right into c. MSB of r doesnt change
            0x2b => "SRA E",                // shift E right into c. MSB of r doesnt change
            0x2c => "SRA H",                // shift H right into c. MSB of r doesnt change
            0x2d => "SRA L",                // shift L right into c. MSB of r doesnt change
            0x2f => "SRA A",                // shift A right into c. MSB of r doesnt change
            0x2e => "SRA (HL)",             // shift (HL) right into c. MSB of (HL) doesnt change

            0x38 => "SRL B",                // shift B right into c. MSB set to 0
            0x39 => "SRL C",                // shift C right into c. MSB set to 0
            0x3a => "SRL D",                // shift D right into c. MSB set to 0
            0x3b => "SRL E",                // shift E right into c. MSB set to 0
            0x3c => "SRL H",                // shift H right into c. MSB set to 0
            0x3d => "SRL L",                // shift L right into c. MSB set to 0
            0x3f => "SRL A",                // shift A right into c. MSB set to 0
            0x3e => "SRL (HL)",             // shift (HL) right into c. MSB set to 0

            /* swap nibbles */
            0x30 => "SWAP B",               // swap lower and upper nibbles of B
            0x31 => "SWAP C",               // swap lower and upper nibbles of C
            0x32 => "SWAP D",               // swap lower and upper nibbles of D
            0x33 => "SWAP E",               // swap lower and upper nibbles of E
            0x34 => "SWAP H",               // swap lower and upper nibbles of H
            0x35 => "SWAP L",               // swap lower and upper nibbles of L
            0x37 => "SWAP A",               // swap lower and upper nibbles of A
            0x36 => "SWAP (HL)",            // swap lower and upper nibbles of (HL)

            /* bit / set / reset */
            _ => {
                let (hi, lo) = (opcode_cb >> 4, opcode_cb & 0x0f);

                let b = match hi {
                    0x04 | 0x08 | 0x0c => 1 - (lo < 0x08) as u8,
                    0x05 | 0x09 | 0x0d => 3 - (lo < 0x08) as u8,
                    0x06 | 0x0a | 0x0e => 5 - (lo < 0x08) as u8,
                    0x07 | 0x0b | 0x0f => 7 - (lo < 0x08) as u8,
                    _ => panic!("0xcb bit/set/res: Invalid bit")
                };

                if lo == 0x06 || lo == 0x0e { // HL
                    match opcode_cb {
                        0x40..=0x7f => return format!("TEST (HL) {}", b), // test bit b of (HL)
                        0x80..=0xbf => return format!("RES  (HL) {}", b), // reset bit b of (HL)
                        0xc0..=0xff => return format!("SET  (HL) {}", b), // set bit b of (HL)
                        _ => panic!("0xcb bit/set/res HL: Invalid range: 0x{:x}", opcode_cb)
                    }
                }
                else { // r
                    let r = match lo {
                        0x00 | 0x08 => "B",
                        0x01 | 0x09 => "C",
                        0x02 | 0x0a => "D",
                        0x03 | 0x0b => "E",
                        0x04 | 0x0c => "H",
                        0x05 | 0x0d => "L",
                        0x07 | 0x0f => "A",
                        _ => panic!("0xcb bit/set/res: Invalid register")
                    };

                    match opcode_cb {
                        0x40..=0x7f => return format!("TEST {} {}", r, b), // test bit b of r
                        0x80..=0xbf => return format!("RES  {} {}", r, b), // reset bit b of r
                        0xc0..=0xff => return format!("SET  {} {}", r, b), // set bit b of r
                        _ => panic!("0xcb bit/set/res: Invalid range: 0x{:x}", opcode_cb)
                    }
                }
            }
        }.to_string()
    }
}
