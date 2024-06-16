use std::fmt;
use std::collections::{HashMap, VecDeque};

use crate::consts::*;
use crate::gbemulator::GBEmulator;
use crate::cpu::CPU;

mod instrs;
mod tui;

use self::tui::DebuggerTUI;

// TODO: After a jump it should not sustract from the PC

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
    last_instrs         :VecDeque<Instruction>,  // Last executed instructions
    wait_instr_n        :u16,               // Wait for n instructions to update the TUI
    has_breakpoint_addr :bool,              // Whether it has a breakpoint address to start
    breakpoint_addr     :u16,               // The breakpoint address to start
    io_table            :HashMap<u16, &'static str>,
}

impl Debugger {
    pub fn new(gbemu :GBEmulator, has_breakpoint :bool, breakpoint_addr :u16) -> Debugger {
        return Debugger {
            gbemu,
            tui                 : DebuggerTUI::new(),
            instrs              : vec![],
            last_instrs         : VecDeque::with_capacity(100),
            wait_instr_n        : 0,
            has_breakpoint_addr : has_breakpoint,
            breakpoint_addr     : breakpoint_addr,
            io_table :HashMap::from(IO_ADDR_TEXT),
        }
    }

    /* Initialize the TUI */
    pub fn init(&mut self) {
        self.gbemu.init();
        self.tui.init();
    }

    pub fn run(&mut self) {
        while !self.tui.is_done() {
            // TODO: Remove. For tests.
            if self.gbemu.get_cpu().get_pc() > 0xFFF0 { self.tui.close(); println!("end"); return; }

            if self.gbemu.get_cpu().is_new_instr() {
                self.update();
            }

            self.gbemu.get_bus().borrow_mut().tick();
            self.gbemu.get_cpu_mut().tick();
        }
    }

    /* Convert two u8 to u16. Utility function. */
    fn to_u16(&self, hi :u8, lo :u8) -> u16 {
        return ((hi as u16) << 8) | (lo as u16);
    }

    /* Returns whether it has reached a breakpoint */
    fn has_reached_breakpoint(&mut self) -> bool {
        // Check for a breakpoint address
        if self.has_breakpoint_addr && self.gbemu.get_cpu().get_pc()-1 == self.breakpoint_addr {
            self.has_breakpoint_addr = false;
        }

        return !self.has_breakpoint_addr;
    }

    /* Update the debugger state and render the TUI */
    fn update(&mut self) {
        // If it has a breakpoint, check if it has reached it
        if !self.has_reached_breakpoint() {
            return;
        }

        // Dont update the UI for n M-Cycles depending on user input
        if self.wait_instr_n == 0 {
            self.dissasemble_instrs();

            let bus = self.gbemu.get_bus();
            let bus = bus.borrow();
            let cpu :&CPU = self.gbemu.get_cpu();

            self.wait_instr_n = self.tui.update(
                &self.instrs, &self.last_instrs,
                cpu,  &bus
            );
        } else {
            self.wait_instr_n -= 1;
        }
    }

    /* Replace instruction text variables like {n} or {nn} */
    fn replace_variables(&self, s :String, pc :u16) -> String {
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
    fn dissasemble_instrs(&mut self) {
        let bus = self.gbemu.get_bus();
        let bus = bus.borrow();
        self.instrs = vec![];

        // TODO: 1/1? Is this a bug?
        // TODO: Why is this needed?
        let mut pc :u16 = if self.gbemu.get_cpu().get_pc() == 0 {
            0
        } else {
            self.gbemu.get_cpu().get_pc() - if self.gbemu.get_cpu().get_opcode() == 0xcb {
                1
            } else {
                1
            }
        };
        
        let actual_pc = pc;

        // Parse the next 200 bytes, which is enough even considering that
        // the next 100 instructions are 0xCB prefixed
        while pc < 0xffff && pc-actual_pc < 200 {
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

            // Add only the actual instruction
            if instr.get_pos() == actual_pc {
                self.last_instrs.push_front(instr.clone());
            
                if self.last_instrs.len() > 100 {
                    self.last_instrs.pop_back();
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
}
