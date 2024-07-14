use std::env;
use gb::consts::Config;
use gb::debugger::Debugger;


fn parse_args() -> Config {
    let mut args :Vec<String> = env::args().collect();

    // --debug
    let is_debug = args.contains(&"--debug".to_string());
    if is_debug {
        let index = args.iter().position(|s| *s == "--debug").unwrap();
        args.remove(index);
    }

    // --mult, screen size multiplier. Used by screen, not here
    let is_mult = args.contains(&"--mult".to_string());
    let mut screen_mult = 2;
    if is_mult {
        let index = args.iter().position(|s| (*s).contains("--mult")).unwrap();
        args.remove(index);
        screen_mult = args[index].parse().unwrap();
        args.remove(index);
    }

    // --debug
    let has_breakpoint = args.contains(&"--breakpoint".to_string());
    let mut breakpoint_addr :u16 = 0x0000;
    if has_breakpoint {
        let index = args.iter().position(|s| *s == "--breakpoint").unwrap();
        breakpoint_addr = u16::from_str_radix(&args[index+1].clone(), 16).unwrap();
     
        args.remove(index);
        args.remove(index);
    }


    // rom path
    let rom_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "tests/roms/blargg/cpu_instrs/cpu_instrs.gb"
            .to_string()
    };

    return Config {
        rom_path,
        is_debug,
        has_breakpoint,
        breakpoint_addr,
        screen_mult
    };
}

fn main() {
    let config = parse_args();

    let mut gbemu = gb::gbemulator::GBEmulator::new(
        &config.rom_path,
        config.screen_mult
    );

    if config.is_debug {
        let mut debugger = Debugger::new(gbemu, config.has_breakpoint, config.breakpoint_addr);
        debugger.init();
        debugger.run();
    } else {
        gbemu.init();
        gbemu.run();   
    }
}
