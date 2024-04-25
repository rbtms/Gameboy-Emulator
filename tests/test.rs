/*
 * Some tests are commented because they try to access illegal memory addresses
 */

#[cfg(test)]
mod tests {
    use gb::consts::*;
    use gb::gbemulator::GBEmulator;

    type CPU = gb::cpu::CPU;

    fn read_json(path :&str) -> serde_json::Value {
        let file :String = std::fs::read_to_string(path).unwrap();
        return serde_json::from_str(&file).unwrap();
    }

    // Initialize cpu
    fn load_json_test(json :&serde_json::Value, cpu :&mut CPU) {
        let initial :&serde_json::Value = &json["initial"];

        cpu.set_pc(initial["pc"].as_u64().unwrap() as u16);
        cpu.set_sp(initial["sp"].as_u64().unwrap() as u16);
        //cpu.set_ie(0, initial["ie"].as_u64().unwrap() as u8);
        //cpu.set_ime(initial["ime"].as_u64().unwrap() as u8);

        cpu.set_reg(REG_A, initial["a"].as_u64().unwrap() as u8);
        cpu.set_reg(REG_F, initial["f"].as_u64().unwrap() as u8);
        cpu.set_reg(REG_B, initial["b"].as_u64().unwrap() as u8);
        cpu.set_reg(REG_C, initial["c"].as_u64().unwrap() as u8);
        cpu.set_reg(REG_D, initial["d"].as_u64().unwrap() as u8);
        cpu.set_reg(REG_E, initial["e"].as_u64().unwrap() as u8);
        cpu.set_reg(REG_H, initial["h"].as_u64().unwrap() as u8);
        cpu.set_reg(REG_L, initial["l"].as_u64().unwrap() as u8);

        for addr_val in initial["ram"].as_array().unwrap() {
            let addr = addr_val[0].as_u64().unwrap() as u16;
            let val  = addr_val[1].as_u64().unwrap() as u8;

            cpu.write(addr, val);
        }
    }

    // Check final result
    fn check_json_test(json :&serde_json::Value, cpu :&CPU) {
        let _final :&serde_json::Value = &json["final"];

        let name :&str = json["name"].as_str().unwrap();

        let pc  = _final["pc"].as_u64().unwrap() as u16;
        let sp  =  _final["sp"].as_u64().unwrap() as u16;
        //let ime = _final["ime"].as_u64().unwrap() as u8;
        let (a, f, b, c, d, e, h, l) = (
            _final["a"].as_u64().unwrap() as u8,
            _final["f"].as_u64().unwrap() as u8,
            _final["b"].as_u64().unwrap() as u8,
            _final["c"].as_u64().unwrap() as u8,
            _final["d"].as_u64().unwrap() as u8,
            _final["e"].as_u64().unwrap() as u8,
            _final["h"].as_u64().unwrap() as u8,
            _final["l"].as_u64().unwrap() as u8
        );
        //let ei = _final["ei"].as_u64().unwrap_or_default() as u8;
        //let final_cycle_n = json["cycles"].as_array().unwrap().len() as u8 * 4;

        assert_eq!(cpu.get_pc(),   pc,  "{}: pc", name);
        assert_eq!(cpu.get_sp(),   sp,  "{}: sp", name);
        //assert_eq!(cpu.get_ime(),  ime, "{}: ime", name);

        assert_eq!(cpu.reg(REG_A), a,   "{}: a", name);
        assert_eq!(cpu.reg(REG_F), f,   "{}: f. flags: left {:#010b} right {:#010b}", name, cpu.reg(REG_F), f);
        assert_eq!(cpu.reg(REG_B), b,   "{}: b", name);
        assert_eq!(cpu.reg(REG_C), c,   "{}: c", name);
        assert_eq!(cpu.reg(REG_D), d,   "{}: d", name);
        assert_eq!(cpu.reg(REG_E), e,   "{}: e", name);
        assert_eq!(cpu.reg(REG_H), h,   "{}: h", name);
        assert_eq!(cpu.reg(REG_L), l,   "{}: l", name);
        //assert_eq!(cpu.wait_ime(), ei);

        //assert_eq!(cycle_n, final_cycle_n, "{}: cycle_n left {} right {}", name, cycle_n, final_cycle_n);

        for addr_val in _final["ram"].as_array().unwrap() {
            let addr = addr_val[0].as_u64().unwrap() as u16;
            let val  = addr_val[1].as_u64().unwrap() as u8;

            assert_eq!(cpu.read(addr), val, "{}: ram addr {}", name, addr);
        }
    }

    fn test_json(path :String) {
        let test_json :serde_json::Value = read_json(&path);

        for t in test_json.as_array().unwrap() {
            println!("n {}", t["name"].as_str().unwrap());
            
            let mut gbemu = GBEmulator::new(&path, false);
            gbemu.init();

            let mut cpu = gbemu.get_cpu();

            let final_cycle_n = t["cycles"].as_array().unwrap().len() as u8 * 4;
            load_json_test(&t, &mut cpu);

            // fetch the instruction
            cpu.tick();
            cpu.tick();
            cpu.tick();
            cpu.tick();
            //println!("cycles: {}", final_cycle_n);
            for _ in 0..(final_cycle_n) {
                cpu.tick();
            }

            check_json_test(&t, &cpu);
        }
    }

    #[test]
    fn test_ops() {
        let tests = [
            "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d", "0e", "0f",
            /*"10",*/ "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "1c", "1d", "1e", "1f",
            "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "2a", "2b", "2c", "2d", "2e", "2f",
            "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "3a", "3b", "3c", "3d", "3e", "3f",
            "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4a", "4b", "4c", "4d", "4e", "4f",
            "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "5a", "5b", "5c", "5d", "5e", "5f",
            "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6a", "6b", "6c", "6d", "6e", "6f",
            "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", "7a", "7b", "7c", "7d", "7e", "7f",
            "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "8a", "8b", "8c", "8d", "8e", "8f",
            "90", "91", "92", "93", "94", "95", "96", "97", "98", "99", "9a", "9b", "9c", "9d", "9e", "9f",
            "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aa", "ab", "ac", "ad", "ae", "af",
            "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9", "ba", "bb", "bc", "bd", "be", "bf",

            "c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9", "ca",       "cc", "cd", "ce", "cf",
            "d0", "d1", "d2",       "d4", "d5", "d6", "d7", "d8", "d9", "da",       "dc",       "de", "df",
            "e0", "e1", "e2",             "e5", "e6", "e7", "e8", "e9", "ea",                   "ee", "ef",
            "f0", "f1", "f2", "f3",       "f5", "f6", "f7", "f8", "f9", "fa", "fb",             "fe", "ff" 
        ];

        for file_n in tests {
            println!("Testing {}", file_n);
            test_json(format!("tests/v1/{}.json", file_n));
        }
    }

    #[test]
    fn test_ops_cb() {
        let tests = [
            "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d", "0e", "0f",
            "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "1c", "1d", "1e", "1f",
            "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "2a", "2b", "2c", "2d", "2e", "2f",
            "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "3a", "3b", "3c", "3d", "3e", "3f",
            "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4a", "4b", "4c", "4d", "4e", "4f",
            "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "5a", "5b", "5c", "5d", "5e", "5f",
            "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6a", "6b", "6c", "6d", "6e", "6f",
            "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", "7a", "7b", "7c", "7d", "7e", "7f",
            "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "8a", "8b", "8c", "8d", "8e", "8f",
            "90", "91", "92", "93", "94", "95", "96", "97", "98", "99", "9a", "9b", "9c", "9d", "9e", "9f",
            "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aa", "ab", "ac", "ad", "ae", "af",
            "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9", "ba", "bb", "bc", "bd", "be", "bf",
            "c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9", "ca", "cb", "cc", "cd", "ce", "cf",
            "d0", "d1", "d2", "d3", "d4", "d5", "d6", "d7", "d8", "d9", "da", "db", "dc", "dd", "de", "df",
            "e0", "e1", "e2", "e3", "e4", "e5", "e6", "e7", "e8", "e9", "ea", "eb", "ec", "ed", "ee", "ef",
            "f0", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "fa", "fb", "fc", "fd", "fe", "ff"
        ];

        for file_n in tests {
            println!("Testing cb {}", file_n);
            test_json(format!("tests/v1/cb {}.json", file_n));
        }
    }
}

fn main() {
}

