use std::fmt;
use std::collections::HashMap;

use crate::consts::*;
use crate::gbemulator::GBEmulator;
use crate::cpu::CPU;

mod instrs;
mod tui;

use self::tui::DebuggerTUI;

const IO_ADDR_TEXT :[(u16, &str);55] = [
    (0xFF00, "P1"),     (0xFF01, "SB"),    (0xFF02, "SC"),
    (0xFF04, "DIV"),    (0xFF05, "TIMA"),  (0xFF06, "TMA"),
    (0xFF07, "TAC"),    (0xFF0F, "IF"),    (0xFF10, "NR10"),
    (0xFF11, "NR11"),   (0xFF12, "NR12"),  (0xFF13, "NR13"),
    (0xFF14, "NR14"),   (0xFF16, "NR21"),  (0xFF17, "NR22"),
    (0xFF18, "NR23"),   (0xFF19, "NR24"),  (0xFF1A, "NR30"),
    (0xFF1B, "NR31"),   (0xFF1C, "NR32"),  (0xFF1D, "NR33"),
    (0xFF1E, "NR34"),   (0xFF20, "NR41"),  (0xFF21, "NR42"),
    (0xFF22, "NR43"),   (0xFF23, "NR44"),  (0xFF24, "NR50"),
    (0xFF25, "NR51"),   (0xFF26, "NR52"),  (0xFF40, "LCDC"),
    (0xFF41, "STAT"),   (0xFF42, "SCY"),   (0xFF43, "SCX"),
    (0xFF44, "LY"),     (0xFF45, "LYC"),   (0xFF46, "DMA"), 
    (0xFF47, "BGP"),    (0xFF48, "OBP0"),  (0xFF49, "OBP1"),
    (0xFF4A, "WY"),     (0xFF4B, "WX"),    (0xFF4D, "KEY1"),
    (0xFF4F, "VBK"),    (0xFF51, "HDMA1"), (0xFF52, "HDMA2"),
    (0xFF53, "HDMA3"),  (0xFF54, "HDMA4"), (0xFF55, "HDMA5"),
    (0xFF56, "RP"),     (0xFF68, "BCPS"),  (0xFF69, "BCPD"),
    (0xFF6A, "OCPS"),   (0xFF6B, "OCPD"),  (0xFF70, "SVBK"),
    (0xFFFF, "IE")
];

#[derive(Clone)]
pub struct Instruction {
    pos    :u16,
    opcode :u16, // 2B to encode cb-prefixed instructions
    text   :String
}

impl Instruction {
    pub fn new(pos :u16, opcode :u16, text :&str) -> Instruction {
        return Instruction { pos, opcode, text: text.to_string() };
    }

    pub fn get_pos(&self) -> u16 { return self.pos; }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f :&mut fmt::Formatter) -> fmt::Result {
        if self.opcode>>8 == 0xcb {
            write!(f, "  [{:04X}]  {:04X}    {}", self.pos, self.opcode, self.text) }
        else {
            write!(f, "  [{:04X}]    {:02X}    {}", self.pos, self.opcode, self.text) }
    }
}

pub struct Debugger {
    gbemu               :GBEmulator,
    tui                 :DebuggerTUI,       // TUI object
    instrs              :Vec<Instruction>,  // Instruction dissasembled in the last cycle
    last_instrs         :Vec<Instruction>,  // Last executed instructions
    wait_instr_n        :u16,               // Wait for n instructions to update the TUI
    has_breakpoint_addr :bool,              // Whether it has a breakpoint address to start
    has_breakpoint_op   :bool,              // Whether it has a breakpoint opcode to start
    breakpoint_addr     :u16,               // The breakpoint address to start
    breakpoint_op       :u8,                // The breakpoint opcode to start
    io_table            :HashMap<u16, &'static str>,
}

impl Debugger {
    pub fn new(gbemu :GBEmulator) -> Debugger {
        return Debugger {
            gbemu               : gbemu,
            tui                 : DebuggerTUI::new(),
            instrs              : vec![],
            last_instrs         : vec![],
            wait_instr_n        : 0,
            has_breakpoint_addr : false,
            has_breakpoint_op   : false,
            breakpoint_addr     : 0x0000,
            breakpoint_op       : 0x00,
            io_table :HashMap::from(IO_ADDR_TEXT),
        }
    }

    /* Initialize the TUI */
    pub fn init(&mut self) {
        self.gbemu.init();
        self.tui.initialize();
    }

    /* Close the TUI */
    pub fn close_ui(&mut self) { self.tui.close(); }


    pub fn run(&mut self) {
        while !self.tui.is_done() {
            self.gbemu.get_bus().borrow_mut().tick();
            self.gbemu.get_cpu_mut().tick();

            if self.gbemu.get_cpu().is_new_instr() {
                self.update();
            }
        }
    }

    /* Convert two u8 to u16. Utility function. */
    fn to_u16(&self, hi :u8, lo :u8) -> u16 {
        return ((hi as u16) << 8) | (lo as u16);
    }

    /* Set a breakpoint address */
    pub fn set_breakpoint_addr(&mut self, addr :u16) {
        self.has_breakpoint_addr = true;
        self.breakpoint_addr = addr;
    }

    /* Set a breakpoint opcode */
    pub fn set_breakpoint_opcode(&mut self, instr :u8) {
        self.has_breakpoint_op = true;
        self.breakpoint_op = instr;
    }

    /* Returns whether it has reached a breakpoint */
    pub fn has_reached_breakpoint(&mut self) -> bool {
        if self.has_breakpoint_addr {
            if self.gbemu.get_cpu().get_pc()-1 == self.breakpoint_addr {
                self.has_breakpoint_addr = false;
            } 
        }
        if self.has_breakpoint_op {
            if self.gbemu.get_cpu().get_opcode() == self.breakpoint_op {
                self.has_breakpoint_op = false;
            }
        }

        return !self.has_breakpoint_addr && !self.has_breakpoint_op;
    }

    /* Update the debugger state and render the TUI */
    pub fn update(&mut self) {
        // If it has a breakpoint, check if it has reached it
        if !self.has_reached_breakpoint() {
            return;
        }

        // Dont update the UI for n instructions
        if self.wait_instr_n == 0 {
            self.dissasemble();

            let bus = self.gbemu.get_bus();
            let bus = bus.borrow();
            let cpu :&CPU = self.gbemu.get_cpu();

            self.wait_instr_n = self.tui.update(
                &self.instrs, &self.last_instrs,
                &cpu,  &bus
            );
        } else {
            self.wait_instr_n -= 1;
        }
    }

    /* Replace instruction text variables like {n} or {nn} */
    pub fn replace_variables(&self, s :String, pc :u16) -> String {
        let bus = self.gbemu.get_bus();
        let bus = bus.borrow();

        // Replace n
        let mut s = s.replace("{n}", &format!("{:02X}h",
            bus.read(pc+1)
        // Replace pc + signed integer
        )).replace("{pc+n_i8}", &format!("{:04X}h",
            (pc+2).wrapping_add((bus.read(pc+1) as i8) as u16)
        // Replace nn
        )).replace("{nn}", &format!("{:04X}h",
            self.to_u16(bus.read(pc+2), bus.read(pc+1))
        ));

        // Replace 0xff00 + unsigned integer
        if s.contains("{io+n}") {
            let io_port = 0xFF00 + bus.read(pc+1) as u16;
            let tmp;

            s = s.replace("{io+n}", if self.io_table.contains_key(&io_port) {
                self.io_table.get(&io_port).unwrap()
            } else {
                tmp = format!("{:04X}h", io_port);
                &tmp
            });
        }

        return s;
    }
   
    /* Dissasemble ROM instructions */
    pub fn dissasemble(&mut self) {
        let bus = self.gbemu.get_bus();
        let bus = bus.borrow();

        // 1/1? Is this a bug?
        let mut pc :u16 = self.gbemu.get_cpu().get_pc() - if self.gbemu.get_cpu().get_opcode() == 0xcb {1} else {1};
        self.instrs = vec![];

        while pc < 0xffff-2 {
            let opcode = bus.read(pc) as u16;
            
            // Fetch opcode
            let mut text = if opcode == 0xcb {
                self.instr_cb_text(bus.read(pc+1))
            } else {
                self.instr_text(opcode as u8)
            };

            // cb-prefixed opcodes dont have replacements
            text = self.replace_variables(text, pc);

            // Build the instruction
            let instr = Instruction::new(
                pc,
                if opcode == 0xcb { self.to_u16(opcode as u8, bus.read(pc+1)) } else {opcode},
                &text
            );

            // Push the new instruction if the vector is empty or its not the same
            // as the last one
            if instr.get_pos() == pc-1 {
                if self.last_instrs.len() == 0
                || self.last_instrs.last().unwrap().get_pos() != instr.get_pos() {
                    self.last_instrs.push(instr.clone());
                }
            }

            // Add instruction
            self.instrs.push(instr);

            // Increment pc
            if text == "undefined" { pc += 1; }
            else if opcode == 0xcb { pc += 2; }
            else { pc += OP_BYTE_LEN[opcode as usize] as u16; }
        }
    }

    /* Print some CPU state. Its old but sometimes useful */
    pub fn print_state(&self) {
        let cpu :&CPU = self.gbemu.get_cpu();

        println!("
+-----------+
| CPU State |
+-----------+

  PC : {:04X}                                                       
  SP : {:04X}                                                       
  IE : {:08b}                                                       
  IF : {:08b} (IME {})                                              
                                                                   
  is_wait : {}                                                      
                                                                   
  CPU Registers:                                                    
       A : 0x{:02X}    B : 0x{:02X}    D : 0x{:02X}    H : 0x{:02X}  
       F : 0x{:02X}    C : 0x{:02X}    E : 0x{:02X}    L : 0x{:02X}  
                                                                   
  Hardware Registers                                                
      DIV : 0x{:02X}  TIMA : 0X{:02X}  TMA : 0x{:02X}  TAC : 0x{:02x}

",
    cpu.get_pc(), cpu.get_sp(), cpu.read(ADDR_IE), cpu.read(ADDR_IF), cpu.get_ime(),
    cpu.is_wait(),
    cpu.reg(REG_A), cpu.reg(REG_B), cpu.reg(REG_D), cpu.reg(REG_H),
    cpu.reg(REG_F), cpu.reg(REG_C), cpu.reg(REG_E), cpu.reg(REG_L),
    cpu.read(ADDR_DIV), cpu.read(ADDR_TIMA), cpu.read(ADDR_TMA), cpu.read(ADDR_TAC));
    }
}
