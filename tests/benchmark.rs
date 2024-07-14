#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::SystemTime;
    use gb::gbemulator::GBEmulator;

    const ROMS_PATH :&str = "tests/roms/benchmark";

    #[derive(Debug)]
    struct BenchmarkResult {
        time_ms :u128,
        cycles  :u64
    }

    fn run_benchmark(path :&str) -> BenchmarkResult {            
        let mut cycle_n = 0;

        let mut gbemu = GBEmulator::new(&path, 2);

        gbemu.init();
        let bus = gbemu.get_bus();
        let cpu = gbemu.get_cpu_mut();
        cpu.init();

        let t_start = SystemTime::now();
        let mut instr_times = vec![];

        // fetch the first instruction
        for _ in 0..4 {
            cpu.tick();
            bus.borrow_mut().tick();
        }

        // While it has not reached the end of the program
        // 0x10 is the stopping signal
        // The 0xff00 is to assure it doesn't find it anywhere in the program
        while !(cpu.get_pc() >= 0xFF00 && cpu.get_opcode() == 0x10) {
            cpu.tick();
            //bus.borrow_mut().tick();

            // Add instruction time
            if cpu.is_new_instr() {
                instr_times.push(t_start.elapsed().unwrap().as_nanos());
            }

            cycle_n += 1;
        }

        let total_time_ms = t_start.elapsed().unwrap().as_millis();

        return BenchmarkResult {
            time_ms : total_time_ms,
            cycles  : cycle_n
        }
    }

    #[test]
    fn test_performance() {
        // IMPORTANT: To pass this test, there is need to have unbounded access to RAM,
        // that is, to use a mock RAM in the Bus instead of the usual methods.
        let dir = fs::read_dir(ROMS_PATH).unwrap();
        let mut benchmarks = vec![];

        for rom_path in dir {
            let path = rom_path.unwrap().path();
            let path = path.to_str().unwrap();

            let benchmark = run_benchmark(&path);

            benchmarks.push(benchmark);
        }

        //let cycles_per_s: f32 = 
        let mut total_cycles :u128 = 0;
        let mut total_time_ms = 0;

        for benchmark in benchmarks {
            total_cycles += benchmark.cycles as u128;
            total_time_ms += benchmark.time_ms;
        }

        let avg_cycles_per_s = (total_cycles as f32)/(total_time_ms as f32*0.001);
        println!();
        println!("Total cycles: {}", total_cycles);
        println!("Total time  : {}ms", total_time_ms);
        println!();
        println!("Average cycles per second: {}", avg_cycles_per_s);
        println!("Compared with DMG's clock: X {} times", avg_cycles_per_s/4194304.0);
    }
}

fn main() {
}
