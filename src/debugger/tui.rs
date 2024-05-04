use std::{collections::VecDeque, io};
use std::cell::Ref;

use tui::{
    backend::CrosstermBackend,
    widgets::{List, Block, Borders, ListItem, ListState, Paragraph},
    layout::Rect,
    style::{Style, Modifier},
    Terminal
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::cpu::CPU;
use crate::bus::Bus;
use crate::debugger::Instruction;
use crate::consts::*;

pub type TerminalCrossterm = Terminal<CrosstermBackend<io::Stdout>>;

/* Builds the instruction dissasembly widget */
fn build_instrs_list(instrs :&Vec<Instruction>) -> (List, ListState) {
    let list_items = instrs
        .iter()
        .map(|instr| ListItem::new(format!("{}", instr)))
        .collect::<Vec<ListItem>>();

    let list = List::new(list_items)
        .block( Block::default()
            .title("Dissasembly")
            .borders(Borders::ALL)
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">>");

    let mut state = ListState::default();
    state.select(Some(0));

    return (list, state);
}

/* Builds the last executed instructions widget */
fn build_lastinstrs_list(instrs :&VecDeque<Instruction>) -> List {
    let instrs = instrs.clone();

    let list_items = instrs
        .iter()
        .map(|instr| ListItem::new(format!("{}", instr)))
        .collect::<Vec<ListItem>>();

    let list = List::new(list_items)
        .block( Block::default()
            .title("Last instructions")
            .borders(Borders::ALL)
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">>");

    return list;
}


/* Builds the cpu state widget */
fn build_cpustate_text<'a>(cpu :&'a CPU, bus :&'a Ref<Bus>) -> Paragraph<'a> {
    let pc :u16 = if cpu.get_pc() == 0 {
        0
    } else {
        cpu.get_pc() - if cpu.get_opcode() == 0xcb {
            1
        } else {
            1
        }
    };

    let int = match pc {
        0x40 => "VBlank INT",
        0x48 => "STAT   INT",
        0x50 => "Timer  INT",
        0x58 => "Serial INT",
        0x60 => "Joypad INT",
        _ => ""
    };

    let timer_freq = [1024, 16, 64, 256][(bus.read(ADDR_TAC) & 3) as usize];
    let is_timer_enabled = (bus.read(ADDR_TAC) >> 2) & 1 == 1;

    let pc :u16 = if cpu.get_pc() == 0 {
        0
    } else {
        cpu.get_pc()- if cpu.get_opcode() == 0xcb {
            1
        } else {
            1
        }
    };

    let text = format!("
    PC {:04X}
    SP {:04X} ({:04X})

    A  {:02X}   F  {:02X}   AF {:04X}
    B  {:02X}   C  {:02X}   BC {:04X}
    D  {:02X}   E  {:02X}   DE {:04X}
    H  {:02X}   L  {:02X}   HL {:04X}

    z  {}    IME {}
    h  {}    IE  {:05b}
    n  {}    IF  {:05b}
    c  {}    {}

    timer {}/{}  {}
    div   {:04X}

    ", pc,                  cpu.get_sp(),           ((cpu.read(cpu.get_sp()+1) as u16)<<8) | cpu.read(cpu.get_sp()) as u16,
       cpu.reg(REG_A),      cpu.reg(REG_F),         cpu.reg16(REG_A, REG_F),
       cpu.reg(REG_B),      cpu.reg(REG_C),         cpu.reg16(REG_B, REG_C),
       cpu.reg(REG_D),      cpu.reg(REG_E),         cpu.reg16(REG_D, REG_E),
       cpu.reg(REG_H),      cpu.reg(REG_L),         cpu.reg16(REG_H, REG_L),
       cpu.flag_z(),        cpu.get_ime(),
       cpu.flag_h(),        bus.read(ADDR_IE)&0x1f,
       cpu.flag_n(),        bus.read(ADDR_IF)&0x1f,
       cpu.flag_c(),        int,
       bus.timer_counter(), timer_freq, if is_timer_enabled {"ON"} else {"OFF"},
       bus.div_counter(),
    );

    let text_state = Paragraph::new(text)
        .block( Block::default()
            .title("CPU State")
            .borders(Borders::ALL)
        );

    return text_state;
}

/* Builds the hardware register widget */
fn build_hwreg_text<'a>(bus :&'a Ref<'a, Bus>) -> Paragraph<'a> {
    let text = format!("
    Timer registers

    DIV  {:02X}   TIMA  {:02X}   TMA  {:02X}   TAC  {:02X}

    LCD registers

    LDCD {:02X}   STAT  {:02X}   SCY  {:02X}   SCX  {:02X}
    LY   {:02X}   LYC   {:02X}   WY   {:02X}   WX   {:02X}
    BGP  {:02X}   OBP0  {:02X}   OBP1 {:02X}

    Audio registers

    NR11 {:02X}   NR12  {:02X}   NR13 {:02X}   NR14 {:02X}
    NR21 {:02X}   NR22  {:02X}   NR23 {:02X}   NR24 {:02X}
    NR30 {:02X}   NR31  {:02X}   NR32 {:02X}   NR33 {:02X}
    NR34 {:02X}   NR41  {:02X}   NR42 {:02X}   NR43 {:02X}
    NR44 {:02X}   NR50  {:02X}   NR51 {:02X}   NR52 {:02X}

    Joypad register

    P1   {:02X}
    
    DMA register

    DMA  {:02X}
    ",  bus.read(ADDR_DIV),  bus.read(ADDR_TIMA), bus.read(ADDR_TMA),
        bus.read(ADDR_TAC),  bus.read(ADDR_LCDC), bus.read(ADDR_STAT),
        bus.read(ADDR_SCY),  bus.read(ADDR_SCX),  bus.read(ADDR_LY),
        bus.read(ADDR_LYC),  bus.read(ADDR_WY),   bus.read(ADDR_WX),
        bus.read(ADDR_BGP),  bus.read(ADDR_OBP0), bus.read(ADDR_OBP1),
        bus.read(ADDR_NR11), bus.read(ADDR_NR12), bus.read(ADDR_NR13),
        bus.read(ADDR_NR14), bus.read(ADDR_NR21), bus.read(ADDR_NR22),
        bus.read(ADDR_NR23), bus.read(ADDR_NR24), bus.read(ADDR_NR30),
        bus.read(ADDR_NR31), bus.read(ADDR_NR32), bus.read(ADDR_NR33),
        bus.read(ADDR_NR34), bus.read(ADDR_NR41), bus.read(ADDR_NR42),
        bus.read(ADDR_NR43), bus.read(ADDR_NR44), bus.read(ADDR_NR50),
        bus.read(ADDR_NR51), bus.read(ADDR_NR52),
        bus.read(ADDR_P1),   bus.read(ADDR_DMA)
    );

    let text_state = Paragraph::new(text)
        .block( Block::default()
            .title("Hardware Registers")
            .borders(Borders::ALL)
        );

    return text_state;
}

pub struct DebuggerTUI {
    terminal :TerminalCrossterm,
    is_done :bool
}

impl DebuggerTUI {
    pub fn new() -> DebuggerTUI {
        let stdout  = io::stdout();
        let backend = CrosstermBackend::new(stdout);

        return DebuggerTUI {
            terminal: Terminal::new(backend).unwrap(),
            is_done: false
        };
    }

    /* Returns whether the TUI has finished running */
    pub fn is_done(&self) -> bool { return self.is_done; }

    /* Render screen and read user input */
    pub fn update(&mut self,
        instrs      :&Vec<Instruction>,
        last_instrs :&VecDeque<Instruction>,
        cpu :&CPU,
        bus :&Ref<Bus>
    ) -> u16 {
        self.render(instrs, last_instrs, cpu, bus);
        return self.read_input(instrs, last_instrs, cpu, bus);
    }

    /* Builds and renders all screen widgets */
    pub fn render(&mut self,
        instrs      :&Vec<Instruction>,
        last_instrs :&VecDeque<Instruction>,
        cpu :&CPU,
        bus :&Ref<Bus>
    ) {
        self.terminal.draw( |f| {
            let size = f.size();

            let (list_instrs, mut state_instrs) = build_instrs_list(instrs);
            let list_lastinstrs = build_lastinstrs_list(last_instrs);
            let text_state = build_cpustate_text(cpu, bus);
            let text_reg   = build_hwreg_text(bus);
            
            f.render_stateful_widget(list_instrs,
                Rect::new(0, 0, size.width/3, size.height),
                &mut state_instrs
            );
            f.render_widget(text_state,
                Rect::new(size.width/3, 0, size.width/3, size.height/2)
            );
            f.render_widget(list_lastinstrs,
                Rect::new(size.width/3, size.height - size.height/2, size.width/3, size.height/2)
            );
            f.render_widget(text_reg,
                Rect::new(2*size.width/3, 0, size.width/3, size.height)
            );
        }).unwrap();
    }

    /*
     * Blocks the debugger until it receives input.
     * Returns number of cycles to skip.
     */
    pub fn read_input(&mut self,
        instrs      :&Vec<Instruction>,
        last_instrs :&VecDeque<Instruction>,
        cpu :&CPU,
        bus :&Ref<Bus>
    ) -> u16 {
        loop {
            match read().expect("Failed to read event") {
                // Key input
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            match c {
                                // Quit
                                'q' | 'Q' => {self.close(); return 0;},
                                // Skip n instrs (-target instr)
                                '1'       => return 10    - 1,
                                '2'       => return 100   - 1,
                                '3'       => return 1000  - 1,
                                '4'       => return 10000 - 1,
                                _         => {}
                            }
                        },
                        KeyCode::Enter => return 0,
                        _ => {}
                    }
                },
                // Re-render the screen on resize
                Event::Resize(_w, _h) => self.render(instrs, last_instrs, cpu, bus),
                _  => {}
            }
        }
    }

    /* Initialize the TUI */
    pub fn initialize(&mut self) {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        enable_raw_mode().unwrap();
    }

    /* Close the TUI */
    pub fn close(&mut self) {
        disable_raw_mode().unwrap();

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap();

        self.terminal.show_cursor().unwrap();
        self.is_done = true;
    }
}

#[allow(dead_code)]
fn main() {
}