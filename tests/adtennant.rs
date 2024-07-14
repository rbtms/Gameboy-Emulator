/*
 * Some tests are commented because they try to access illegal memory addresses
 */

#[cfg(test)]
mod tests {
    use gb::consts::*;
    use gb::gbemulator::GBEmulator;
    type CPU = gb::cpu::CPU;

    const JSON_PATH :&str = "tests/adtennant/v2";
    const TEST_ROM_PATH :&str = "tests/roms/mbcTest.gb";

    /* Test parsing */

    struct TestState {
        a :u8, f:u8, b :u8, c :u8, d :u8, e :u8, h :u8, l :u8,
        pc :u16, sp :u16, ram :Vec<(u16, u8)>
    }

    struct Test {
        name      :String,
        s_initial :TestState,
        s_final   :TestState,
        s_cycles  :Vec<(u16, u8)> // State in every cycle
    }

    /*
     Parses a JSON value array in the format [[u16, u8]...] into a Vec<(u16, u8)>
     */
    fn parse_tuple_array(arr :&serde_json::Value) -> Vec<(u16, u8)> {
        let mut v :Vec<(u16, u8)> = vec![];

        for addr_val in arr.as_array().unwrap() {
            /* Some values are Null in the final cycles of the JSON files. Omitting this leads to
                an incorrect number of final cycles. */
            if addr_val.is_null() {
                v.push((0, 0));
            } else {
                let addr = addr_val[0].as_u64().unwrap() as u16;
                let val  = addr_val[1].as_u64().unwrap() as u8;

                v.push((addr, val));
            }
        }

        v
    }

    /*
     Parse a JSON test file
     */
    fn parse_tests(file_path :&str) -> Vec<Test> {
        let file :String = std::fs::read_to_string(file_path).unwrap();
        let json :serde_json::Value = serde_json::from_str(&file).unwrap();
        let mut tests = vec![];

        for t in json.as_array().unwrap() {
            let initial :&serde_json::Value = &t["initial"];
            let _final  :&serde_json::Value = &t["final"];

            let test = Test {
                name: t["name"].as_str().unwrap().to_owned(),
                s_initial: TestState {
                    a:   initial["a"].as_u64().unwrap() as u8,
                    f:   initial["f"].as_u64().unwrap() as u8,
                    b:   initial["b"].as_u64().unwrap() as u8,
                    c:   initial["c"].as_u64().unwrap() as u8,
                    d:   initial["d"].as_u64().unwrap() as u8,
                    e:   initial["e"].as_u64().unwrap() as u8,
                    h:   initial["h"].as_u64().unwrap() as u8,
                    l:   initial["l"].as_u64().unwrap() as u8,
                    pc:  initial["pc"].as_u64().unwrap() as u16,
                    sp:  initial["sp"].as_u64().unwrap() as u16,
                    ram: parse_tuple_array(&initial["ram"]),
                },

                s_final: TestState {
                    a:   _final["a"].as_u64().unwrap() as u8,
                    f:   _final["f"].as_u64().unwrap() as u8,
                    b:   _final["b"].as_u64().unwrap() as u8,
                    c:   _final["c"].as_u64().unwrap() as u8,
                    d:   _final["d"].as_u64().unwrap() as u8,
                    e:   _final["e"].as_u64().unwrap() as u8,
                    h:   _final["h"].as_u64().unwrap() as u8,
                    l:   _final["l"].as_u64().unwrap() as u8,
                    pc:  _final["pc"].as_u64().unwrap() as u16,
                    sp:  _final["sp"].as_u64().unwrap() as u16,
                    ram: parse_tuple_array(&_final["ram"]),
                },

                s_cycles: parse_tuple_array(&t["cycles"]),
            };

            tests.push(test);
        }

        tests
    }

    
    /* Actual tests */

    /*
     Initialize CPU registers, PC, SP and RAM values.
     */
    fn init_test(test :&Test, cpu :&mut CPU) {
        cpu.set_pc(test.s_initial.pc - 1);
        cpu.set_sp(test.s_initial.sp);

        cpu.set_reg(REG_A, test.s_initial.a);
        cpu.set_reg(REG_F, test.s_initial.f);
        cpu.set_reg(REG_B, test.s_initial.b);
        cpu.set_reg(REG_C, test.s_initial.c);
        cpu.set_reg(REG_D, test.s_initial.d);
        cpu.set_reg(REG_E, test.s_initial.e);
        cpu.set_reg(REG_H, test.s_initial.h);
        cpu.set_reg(REG_L, test.s_initial.l);

        // Initialize RAM
        for (addr, val) in &test.s_initial.ram {
            cpu.write(*addr, *val);
        }
    }

    /*
     Check the final result of a test. This includes the CPU registers, PC, SP,
     number of transcurred cycles and RAM values.
     */
    fn check_test_result(test :&Test, cpu :&CPU, cycle_n :u32) {
        let final_cycle_n = (test.s_cycles.len() as u32) * 4;

        assert_eq!(cpu.reg(REG_A), test.s_final.a,   "{}: a", test.name);
        assert_eq!(cpu.reg(REG_F), test.s_final.f,   "{}: f. flags: left {:#010b} right {:#010b}", test.name, cpu.reg(REG_F), test.s_final.f);
        assert_eq!(cpu.reg(REG_B), test.s_final.b,   "{}: b", test.name);
        assert_eq!(cpu.reg(REG_C), test.s_final.c,   "{}: c", test.name);
        assert_eq!(cpu.reg(REG_D), test.s_final.d,   "{}: d", test.name);
        assert_eq!(cpu.reg(REG_E), test.s_final.e,   "{}: e", test.name);
        assert_eq!(cpu.reg(REG_H), test.s_final.h,   "{}: h", test.name);
        assert_eq!(cpu.reg(REG_L), test.s_final.l,   "{}: l", test.name);

        assert_eq!(cpu.get_pc(),  test.s_final.pc,  "{}: pc", test.name);
        assert_eq!(cpu.get_sp(),  test.s_final.sp,  "{}: sp", test.name);

        assert_eq!(cycle_n, final_cycle_n, "{}: cycle_n left {} right {}", test.name, cycle_n, final_cycle_n);

        for (addr, val) in &test.s_final.ram {
              assert_eq!(cpu.read(*addr), *val, "{}: ram addr {}", test.name, *addr);
        }
    }

    fn run_test(path :String, gbemu :&mut GBEmulator) {
        let tests = parse_tests(&path);

        for test in tests {
            print!("> Test: n {} ...", test.name);
            
            let final_cycle_n = (test.s_cycles.len() as u8) * 4;
            let mut cycle_n = 0;
            
            gbemu.init();
            let bus = gbemu.get_bus();
            let mut cpu = gbemu.get_cpu_mut();
            cpu.init();

            init_test(&test, &mut cpu);

            // fetch the first instruction
            for _ in 0..4 {
                cpu.tick();
                bus.borrow_mut().tick();
            }

            for _ in 0..final_cycle_n {
                cpu.tick();
                bus.borrow_mut().tick();
                cycle_n += 1;
            }

            check_test_result(&test, &cpu, cycle_n);
            println!(" ok");
        }
    }

    #[test]
    fn test_ops() {
        // IMPORTANT: To pass this test, there is need to have unbounded access to RAM,
        // that is, to use a mock RAM in the Bus instead of the usual methods.
        let mut gbemu = GBEmulator::new(TEST_ROM_PATH, 2);

        // Omitting 0x10: STOP, 0x76: HALT, 0xF3: DI, 0xFB: EI
        let tests = [
            "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d", "0e", "0f",
                  "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "1c", "1d", "1e", "1f",
            "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "2a", "2b", "2c", "2d", "2e", "2f",
            "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "3a", "3b", "3c", "3d", "3e", "3f",
            "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4a", "4b", "4c", "4d", "4e", "4f",
            "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "5a", "5b", "5c", "5d", "5e", "5f",
            "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6a", "6b", "6c", "6d", "6e", "6f",
            "70", "71", "72", "73", "74", "75",       "77", "78", "79", "7a", "7b", "7c", "7d", "7e", "7f",
            "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "8a", "8b", "8c", "8d", "8e", "8f",
            "90", "91", "92", "93", "94", "95", "96", "97", "98", "99", "9a", "9b", "9c", "9d", "9e", "9f",
            "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aa", "ab", "ac", "ad", "ae", "af",
            "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9", "ba", "bb", "bc", "bd", "be", "bf",
            "c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9", "ca",       "cc", "cd", "ce", "cf",
            "d0", "d1", "d2",       "d4", "d5", "d6", "d7", "d8", "d9", "da",       "dc",       "de", "df",
            "e0", "e1", "e2",             "e5", "e6", "e7", "e8", "e9", "ea",                   "ee", "ef",
            "f0", "f1", "f2",             "f5", "f6", "f7", "f8", "f9", "fa",                   "fe", "ff" 
        ];

        for file_n in tests {
            println!("Testing {} ... ", file_n);
            run_test(format!("{}/{}.json", JSON_PATH, file_n), &mut gbemu);
        }
    }

    #[test]
    fn test_ops_cb() {
        let mut gbemu = GBEmulator::new(TEST_ROM_PATH, 1);
        
        println!("Testing cb");
        run_test(format!("{}/cb.json", JSON_PATH), &mut gbemu);
    }
}

fn main() {
}

