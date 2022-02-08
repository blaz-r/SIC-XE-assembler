use crate::parser::*;
use std::collections::HashMap;


const OPCODES: [(&str, u8); 59] = [("ADD", 0x18),
                                    ("ADDF", 0x58),
                                    ("ADDR", 0x90),
                                    ("AND", 0x40),
                                    ("CLEAR", 0xB4),
                                    ("COMP", 0x28),
                                    ("COMPF", 0x88),
                                    ("COMPR", 0xA0),
                                    ("DIV", 0x24),
                                    ("DIVF", 0x64),
                                    ("DIVR", 0x9C),
                                    ("FIX", 0xC4),
                                    ("FLOAT", 0xC0),
                                    ("HIO", 0xF4),
                                    ("J", 0x3C),
                                    ("JEQ", 0x30),
                                    ("JGT", 0x34),
                                    ("JLT", 0x38),
                                    ("JSUB", 0x48),
                                    ("LDA", 0x00),
                                    ("LDB", 0x68),
                                    ("LDCH", 0x50),
                                    ("LDF", 0x70),
                                    ("LDL", 0x08),
                                    ("LDS", 0x6C),
                                    ("LDT", 0x74),
                                    ("LDX", 0x04),
                                    ("LPS", 0xD0),
                                    ("MUL", 0x20),
                                    ("MULF", 0x60),
                                    ("MULR", 0x98),
                                    ("NORM", 0xC8),
                                    ("OR", 0x44),
                                    ("RD", 0xD8),
                                    ("RMO", 0xAC),
                                    ("RSUB", 0x4C),
                                    ("SHIFTL", 0xA4),
                                    ("SHIFTR", 0xA8),
                                    ("SIO", 0xF0),
                                    ("SSK", 0xEC),
                                    ("STA", 0x0C),
                                    ("STB", 0x78),
                                    ("STCH", 0x54),
                                    ("STF", 0x80),
                                    ("STI", 0xD4),
                                    ("STL", 0x14),
                                    ("STS", 0x7C),
                                    ("STSW", 0xE8),
                                    ("STT", 0x84),
                                    ("STX", 0x10),
                                    ("SUB", 0x1C),
                                    ("SUBF", 0x5C),
                                    ("SUBR", 0x94),
                                    ("SVC", 0xB0),
                                    ("TD", 0xE0),
                                    ("TIO", 0xF8),
                                    ("TIX", 0x2C),
                                    ("TIXR", 0xB8),
                                    ("WD", 0xDC)];

const DIRECTIVES: [&str; 11] = ["START", "END", "BYTE", "WORD", "RESB", "RESW", "EXPORTS", "BASE", "NOBASE", "ORG", "EQU"];

const FORMAT1: [&str; 6] = ["FIX", "FLOAT", "HIO", "NORM", "SIO", "TIO"];
const FORMAT2: [&str; 11] = ["ADDR", "CLEAR", "COMPR", "DIVR", "MULR", "RMO", "SHIFTL", "SHIFTR", "SUBR", "TIXR", "SVC"];

const TWO_OP: [&str; 8] = ["ADDR", "COMPR", "DIVR", "MULR", "RMO", "SHIFTL", "SHIFTR", "SUBR"];


// check if given word is a mnemonic for instruction
pub fn is_instr(mnem: &str) -> bool {
    let mut clean_mnem = mnem;
    if mnem.starts_with("+") {
        clean_mnem = &clean_mnem[1..];
    }
    OPCODES.iter().any(|(m, _)| *m == clean_mnem)
}


// check if given word is a directive
pub fn is_directive(dir: &str) -> bool {
    DIRECTIVES.contains(&dir)
}


pub fn determine_format(mnem: &str) -> Result<usize, String> {
    if FORMAT1.contains(&mnem) {
        return Ok(1);
    }
    else if FORMAT2.contains(&mnem) {
        return Ok(2);
    }
    else if mnem.starts_with("+") {
        return Ok(4);
    }
    else if is_instr(&mnem) {
        return Ok(3);
    }
    Err("Not a valid format".to_owned())
}


pub fn determine_command_size(mnem: &str) -> Result<usize, String> {
    // instruction size is equal to format
    determine_format(&mnem)
}


// determine size of reservation
pub fn determine_res_size(dir: &str, value: &Vec<&str>) -> Result<usize, String> {
    /*
        BYTE and WORD can be either 1 number or HEX/CHAR array
        BYTE is occupies 1byte and WORD occupies 3 bytes
        when using hex/char arrays, length is max{<type_size>, <len_of_init>}
    */
    // join value in case char array contains spaces
    let init_str = value.join(" ");
    match dir {
        "BYTE" => parse_init_size(&init_str.clone(), 1),
        "RESB" => parse_res_size(&value[0], 1),
        "WORD" => parse_init_size(&init_str.clone(), 3),
        "RESW" => parse_res_size(&value[0], 3),
        _ => Err("Invalid reservation".to_owned())
    }
}


// handle instructions with format 2
fn handle_format_2(operands: &Vec<&str>, mnem: &str, opcode: u8) -> Result<String, String> {
    // instruction takes 2 operands
    if TWO_OP.contains(&mnem) {
        // since there can be space after comma: either r1,r2 or r1, r2, we join them to r1,r2
        let val_join;
        match operands[0].ends_with(",") {
            false => val_join = operands[0].to_owned(),
            true => val_join = format!("{}{}", operands[0], operands[1]),
        }
        let val_split: Vec<&str> = val_join.split(",").map(|val| val.trim()).collect();

        let mut code_build = format!("{:02X}", opcode);
        // first operand is always register
        match parse_reg(val_split[0]) {
            Ok(value) => code_build.push_str(&format!("{:01X}", value)),
            Err(msg)  => return Err(msg.to_owned())
        }
        // second operand in shift is number
        if mnem == "SHIFTL" || mnem == "SHIFTR" {
            match parse_num(val_split[1]) {
                // interval of shift is in [0, 255]
                Ok(value) => if value > 0 && value < 16 {
                                code_build.push_str(&format!("{:01X}", value))
                             }
                             else {
                                 return Err("Shift value needs to be in interval [0, 16]".to_owned())
                             },
                Err(msg)  => return Err(msg.to_owned())
            }
        } 
        // if it's not shift operation, second operand is reg
        else {
            match parse_reg(val_split[1]) {
                Ok(value) => code_build.push_str(&format!("{:01X}", value)),
                Err(msg)  => return Err(msg.to_owned())
            }
        }

        return Ok(code_build);
    }
    // instruction takes only one operand
    else {
        if mnem == "TIXR" || mnem == "CLEAR" {
            // both take register operand
            let reg = match parse_reg(operands[0]) {
                        Ok(value) => value << 4,
                        Err(msg) => return Err(msg.to_owned())
                      };
            return Ok(format!("{:02X}{:02X}", opcode, reg));
        }
        // SVC
        else {
            // takes 8bit number
            let reg = match parse_num(operands[0]) {
                        Ok(value) => value,
                        Err(msg) => return Err(msg.to_owned())
                      };
            return Ok(format!("{:02X}{:02X}", opcode, reg));
        }
    }
}


fn handle_pc_relative(opcode: u8, pc: usize, address: i32, mut xbpe_offset: i32) -> Result<String, String> {
    let offset = address - (pc + 3) as i32;     // pc + 3 since PC actually points to next instr
    // we are limited to 12 bits for offset
    if (offset >= - i32::pow(2, 11)) && (offset <= i32::pow(2, 11) - 1) {
        // we clear upper 20 bits of 32 bit offset to get 12bit offset
        xbpe_offset |= offset & !(0xFFFFF << 12);
        // set P bit
        xbpe_offset |= 1 << 13;

        return Ok(format!("{:02X}{:04X}", opcode, xbpe_offset));
    }
    
    Err("Can't use PC relative".to_owned())
}


fn handle_base_relative(opcode: u8, address: i32, mut xbpe_offset: i32, base: isize) -> Result<String, String> {
    if base != -1 {
        let offset = address - base as i32;
        // we are limited to 12 bits for offset, but it needs to be positive
        if (offset >= 0) && (offset <= i32::pow(2, 12) - 1) {
            // we clear upper 20 bits of 32 bit offset to get 12bit offset
            xbpe_offset |= offset & !(0xFFFFF << 12);
            // set B bit
            xbpe_offset |= 1 << 14;

            return Ok(format!("{:02X}{:04X}", opcode, xbpe_offset));
        }
    }
    Err("Can't use Base relative".to_owned())
}


fn handle_direct(opcode: u8, pc: usize, address: i32, mut xbpe_offset: i32, mod_record: &mut Vec<String>) -> Result<String, String> {
    if (address >= 0) && (address <= i32::pow(2, 12) - 1) {
        xbpe_offset |= address;
        // b and p stay 0

        // needs mod record
        mod_record.push(format!("{:06X}{:02X}", pc + 1, 3));

        return Ok(format!("{:02X}{:04X}", opcode, xbpe_offset));
    }
    Err("Can't use Direct".to_owned())
}


fn handle_old_sic(opcode: u8, address: i32) -> Result<String, String> {
    if (address >= 0) && (address <= i32::pow(2, 15) - 1) {
        // odl sic is in interval [0, 32767]

        return Ok(format!("{:02X}{:04X}", opcode, address));
    }
    Err("Can't use SIC".to_owned())
}


fn handle_format_3(operands: &Vec<&str>, symtab: &HashMap<String, i32>, mod_record: &mut Vec<String>, opcode: u8, pc: usize, base: isize) -> Result<String, String> {
    let mut xbpe_offset = 0;

    let operand: String;
    let bits_ni;
    let mut immediate_value = false;    // when immediate operand is actual number
    
    if operands[0].starts_with("#") {
        // ***** IMMEDIATE *****

        // indexed not allowed with immediate
        match parse_indexed(&operands) {
            Some(_) => return Err("Indexed not allowed here".to_owned()),
            None => ()
        };

        operand = operands[0][1..].to_owned();
        bits_ni = 1;

        // operand is immediate number
        if !symtab.contains_key(&*operand) {
            immediate_value = true;
        }
    }
    else if operands[0].starts_with("@") {
        // ***** INDIRECT *****

        // indexed not allowed with indirect
        match parse_indexed(&operands) {
            Some(_) => return Err("Indexed not allowed here".to_owned()),
            None => ()
        };

        operand = operands[0][1..].to_owned();
        bits_ni = 2;
    }
    else {
        // simple or sic
        
        // label is either alone or also with index -> label, X
        let mut label = operands[0].to_string();
        match parse_indexed(&operands) {
            Some(x_label) => {label = x_label;
                              xbpe_offset |= 1 << 15},   // turn on X bit
            None => ()
        };

        operand = label;
        bits_ni = 3;
    }
    
    let operand_val;
    if immediate_value {
        match parse_num(&operand) {
            Ok(value) => operand_val = value,
            Err(msg) => return Err(msg.to_owned())
        }

        // we are limited to 20 bits
        if (operand_val >= - i32::pow(2, 11)) && (operand_val <= i32::pow(2, 11) - 1) {
            // if negative clear upper 20 bits
            return Ok(format!("{:02X}{:04X}", opcode | bits_ni, xbpe_offset | operand_val & !(0xFFFFF << 12)));
        }
        else {
            return Err("Immediate value must be on interval [-2048, 2047]".to_owned());
        }
    }
    else {
        if symtab.contains_key(&*operand) {
            operand_val = symtab[&*operand];
        
        }
        // label must exist on left side
        else {
            return Err(format!("Symbol {} does not appear as a left label", operand))
        }

        // first try PC-relative
        match handle_pc_relative(opcode | bits_ni, pc, operand_val, xbpe_offset) {
            Ok(code) => return Ok(code),
            _ => ()
        }
        // then try base, base is -1 if it's not set
        match handle_base_relative(opcode | bits_ni, operand_val, xbpe_offset, base) {
            Ok(code) => return Ok(code),
            _ => ()
        }
        // then try direct
        match handle_direct(opcode | bits_ni, pc, operand_val, xbpe_offset, mod_record) {
            Ok(code) => return Ok(code),
            _ => ()
        }
        if bits_ni == 3 {
            // finally try old SIC
            match handle_old_sic(opcode, operand_val | xbpe_offset) {
                Ok(code) => return Ok(code),
                _ => return Err("Offset is too great for any instruction".to_owned())
            }
        }
        else {
            return Err("Offset is too great for any instruction".to_owned())
        }
    }
}


fn handle_format_4(operands: &Vec<&str>, symtab: &HashMap<String, i32>, mod_record: &mut Vec<String>, opcode: u8,  pc: usize) -> Result<String, String> {
    // set e bit
    let mut xbpe_offset = 1 << 20;

    let operand: String;
    let bits_ni;
    let mut immediate_value = false;    // only when immediate operand is actual number

    if operands[0].starts_with("#") {
        // ***** IMMEDIATE *****

        // indexed not allowed with immediate
        match parse_indexed(&operands) {
            Some(_) => return Err("Indexed not allowed here".to_owned()),
            None => ()
        };

        operand = operands[0][1..].to_owned();
        bits_ni = 1;

        // operand is immediate number
        if !symtab.contains_key(&*operand) {
            immediate_value = true;
        }
    }
    else if operands[0].starts_with("@") {
        // ***** INDIRECT *****

        // indexed not allowed with indirect
        match parse_indexed(&operands) {
            Some(_) => return Err("Indexed not allowed here".to_owned()),
            None => ()
        };

        operand = operands[0][1..].to_owned();
        bits_ni = 2;
    }
    else {
        // ***** SIMPLE *****
        
        // label is either alone or also with index -> label, X
        let mut label = operands[0].to_string();
        match parse_indexed(&operands) {
            Some(x_label) => {label = x_label;
                              xbpe_offset |= 1 << 23},   // turn on X bit
            None => ()
        };
        
        operand = label;
        bits_ni = 0;
    }

    let operand_val;
    if immediate_value {
        // operand is immediate number
        match parse_num(&operand) {
            Ok(value) => operand_val = value,
            Err(msg) => return Err(msg.to_owned())
        }

        // we are limited to 20 bits
        if (operand_val >= - i32::pow(2, 19)) && (operand_val <= i32::pow(2, 19) - 1) {
            // if negative clear upper 12 bits
            return Ok(format!("{:02X}{:06X}", opcode | bits_ni, xbpe_offset | operand_val & !(0xFFF << 20)));
        }
        else {
            return Err("Immediate value must be on interval [-524288, 524287]".to_owned());
        }
    }
    else {
        if symtab.contains_key(&*operand) {
            operand_val = symtab[&*operand];
        
        }
        // label must exist on left side
        else {
            return Err(format!("Symbol {} does not appear as a left label", operand))
        }

        if (operand_val >= 0) && (operand_val <= i32::pow(2, 20) - 1) {
            xbpe_offset |= operand_val;
            // b and p stay 0

            // needs mod record
            mod_record.push(format!("M{:06X}{:02X}", pc + 1, 5));

            return Ok(format!("{:02X}{:06X}", opcode | bits_ni, xbpe_offset));
        }
        else {
            return Err("Invalid extended, must be in range [0, 1048576]".to_owned())
        }
    }
}


// get machine code from assembly code
pub fn get_machine_code(instr: &Vec<&str>, symtab: &HashMap<String, i32>, mod_record: &mut Vec<String>, pc: usize, base: isize) -> Result<String, String> {
    let mnem = instr[0];
    let mut clean_mnem = mnem;
    if mnem.starts_with("+") {
        clean_mnem = &mnem[1..];
    }

    let opcode = OPCODES.iter().filter(|(m, _)| *m == clean_mnem).next().unwrap().1;
    let format = determine_format(mnem).unwrap();

    if format == 1 {
        return Ok(format!("{:02X}", opcode));
    }

    else if format == 2 {
        return handle_format_2(&instr[1..].to_vec(), &mnem, opcode);
    }
    else if format == 3 {
        // rsub takes no arguments
        if clean_mnem == "RSUB" {
            return Ok(format!("{:0<6X}", opcode | 3));
        }

        return handle_format_3(&instr[1..].to_vec(), &symtab, mod_record, opcode, pc, base);
    }
    else if format == 4 {
        return handle_format_4(&instr[1..].to_vec(), &symtab, mod_record, opcode, pc);
    }
    else {
        return Err("Not a valid format".to_owned());
    }
}


pub fn handle_res(dir: &str, value: &Vec<&str>) -> Result<String, String> {
    /*
        BYTE and WORD can be either 1 number or HEX/CHAR array
    */
    // join value in case char array contains spaces
    let init_str = value.join(" ");
    if dir == "BYTE" {
        let init_value = parse_init(&init_str);
        match init_value {
            Ok(value) => {  match value {
                                // save number value as hex byte iee 2 nibbles
                                ResType::Num(val) => return Ok(format!("{:02X}", val)),
                                // concat all bytes together where each occupies 1byte ie. 2 nibbles
                                ResType::Char(vec) => return Ok(vec.iter().map(|hex| format!("{:02X}", hex)).collect::<Vec<String>>().join("")),
                                ResType::Hex(vec) => return Ok(vec.iter().map(|hex| format!("{:02X}", hex)).collect::<Vec<String>>().join(""))
                            }
                         },
            Err(msg) => return Err(msg.to_owned())
        }
    }
    else if dir == "WORD" {
        let init_value = parse_init(&init_str);
        match init_value {
            Ok(value) => {  match value {
                                // save number value as hex word iee 6 nibbles
                                ResType::Num(val) => return Ok(format!("{:06X}", val)),
                                // concat all bytes together where each occupies 1byte ie. 2 nibbles, but pad to get minimal 3 bytes
                                ResType::Char(vec) => { let mut word_vec = vec.iter().map(|hex| format!("{:02X}", hex)).collect::<Vec<String>>();
                                                        // pad with zeros to get at least width of 3
                                                        while word_vec.len() < 3 { word_vec.insert(0, "00".to_owned()) };
                                                        return Ok(word_vec.join(""))},
                                ResType::Hex(vec) => { let mut word_vec = vec.iter().map(|hex| format!("{:02X}", hex)).collect::<Vec<String>>();
                                                       while word_vec.len() < 3 { word_vec.insert(0, "00".to_owned()) };
                                                       return Ok(word_vec.join(""))},
                            }
                         },
            Err(msg) => return Err(msg.to_owned())
        }
    }
    else if dir == "RESW" {
        return Ok("RESW".to_owned());
    }
    else if dir == "RESB" {
        return Ok("RESB".to_owned());
    }
    else {
        return Err("Invalid reservation".to_owned());
    }
}