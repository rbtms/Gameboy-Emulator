use std::env;
use std::collections::HashMap;

fn parse_args() -> HashMap<&'static str, String> {
    let mut args :Vec<String> = env::args().collect();

    // --debug
    let is_debug = args.contains(&"--debug".to_string());
    if is_debug {
        let index = args.iter().position(|s| *s == "--debug").unwrap();
        args.remove(index);
    }

    // --mult, screen size multiplier. Used by screen, not here
    let is_mult = args.contains(&"--mult".to_string());
    if is_mult {
        let index = args.iter().position(|s| (*s).contains("--mult")).unwrap();
        args.remove(index);
    }


    // rom path
    let rom_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "tests/roms/blargg/cpu_instrs/cpu_instrs.gb"
            .to_string()
    };

    return HashMap::from([
        ("rom_path", rom_path),
        ("is_debug", is_debug.to_string())
    ]);
}

fn main() {
    let args = parse_args();   

    let mut gbemu = gb::gbemulator::GBEmulator::new(
        &args["rom_path"],
        args.get("is_debug").unwrap() == "true"
    );
    
    gbemu.init();
    gbemu.run();
}
