use crate::commands::*;
use crate::parser::parse_num;
use crate::equresolution::*;
use std::collections::HashMap;

pub fn get_symbol_table(program: &Vec<String>) -> Result<(usize, HashMap<String, i32>), String> { 
    let mut symtab: HashMap<String, i32> = HashMap::new();

    let mut equtab: HashMap<String, EquExpression> = HashMap::new();

    let mut loc_counter: usize;
    // find start index, skip leading empty lines or comments
    let mut st_index = 0;
    while program[st_index].starts_with(".") || program[st_index] == "" { st_index += 1;}

    let start: Vec<&str> = program[st_index].split(" ").collect();
    if start[0] == "START" {
        return Err("Program needs to start with <name of program> START <address>".to_owned());
    }
    else if start[1] == "START"{
        loc_counter = start[2].parse::<usize>().unwrap();
    }
    else {
        return Err("Program needs to start with <name of program> START <address>".to_owned());
    }

    let mut line_count = 1;

    for line in program[st_index+1..].iter() {
        line_count += 1;
        if line == "" || line.starts_with(".") {
            // empty or comment line
            continue;
        }

        let split: Vec<&str> = line.split(" ").collect();

        let instr_size: Result<usize, String>;
        if !is_instr(split[0]) && !is_directive(split[0]) {
            // 1st word is label

            if symtab.contains_key(split[0]) {
                return Err(format!("Line {}, duplicate label: {}", line_count, split[0]));
            }

            if is_instr(split[1]) {
                // label for instr
                instr_size = determine_command_size(split[1]);
            }
            else if is_directive(split[1]) {
                // label for directive
                if split[1] == "EQU" {
                    // equ contains current addr
                    if split[2] == "*" {
                        insert_equ_number(&mut equtab, split[0], loc_counter as i32);
                    }
                    // equ contains value
                    else {
                        

                        match insert_equ_str(&mut equtab, split[0], &split[2..].to_vec()) {
                            Ok(_) => (),
                            Err(msg) => return Err(format!("Line {}, failed inserting EQU {}", line_count, msg))
                        }
                    }
                    continue;
                }
                instr_size = determine_res_size(split[1], &split[2..].to_vec());
            }
            else if split[1].starts_with(".") {
                // label + comment is still valid... kind of
                instr_size = Ok(0);
            }
            else {
                return Err(format!("Line {}, not a valid format.", line_count));
            }

            // 1st word is label
            symtab.insert(split[0].to_owned(), loc_counter as i32);
        }
        else if is_instr(split[0]) {
            // 1st word is an instruction
            instr_size = determine_command_size(split[0]);
        }
        else if is_directive(split[0]){
            // 1st word is a directive
            match split[0]{
                "EQU" => return Err(format!("Line {}, can't use EQU without label", line_count)),
                "END" => if split.len() == 1 || symtab.contains_key(&*split[1]) {
                            break;
                         }
                         else {
                            return Err(format!("Line {}, END directive invalid here. (END can't be used as label)", line_count));   
                         },
                "ORG" => instr_size = match parse_num(split[1]) {
                                        Ok(value) => if value > 0 {
                                                        Ok(value as usize)
                                                     } 
                                                     else { 
                                                        return Err(format!("Line {}, ORG needs positive number", line_count))
                                                     },
                                        Err(msg) => Err(msg.to_owned())
                                      },
                "BASE" | "NOBASE" => continue,
                _     => instr_size = determine_res_size(split[0], &split[1..].to_vec())
            }
        }
        else {
            // invalid
            return Err(format!("Line {}, Not a valid instruction", line_count));
        }

        match instr_size {
            Ok(size) => loc_counter += size,
            Err(msg) => return Err(format!("{}, line {}", msg, line_count))
        }
    }

    match resolve_equs(&mut symtab, &mut equtab) {
        Ok(_) => Ok((loc_counter, symtab)),
        Err(msg) => Err(format!("Failed resolving EQUs, {}", msg))
    }
}