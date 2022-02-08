use crate::commands::*;
use crate::parser::parse_expr;
use crate::objwriter::*;
use std::fs::File;
use std::collections::HashMap;

pub fn generate_obj(program: &Vec<String>, symtab: &HashMap<String, i32>, len: usize) -> Result<String, String> {

    // find start line by skipping leading comments and empty lines
    let mut st_index = 0;
    while program[st_index].starts_with(".") || program[st_index] == "" { st_index += 1;}

    let start: Vec<&str> = program[st_index].split(" ").collect();
    let name;
    if start[0].len() <= 6 {
        name = format!("{:<6}", start[0]);
    }
    else {
        return Err(format!("Name must be at most 6 characters wide, {} is {}", start[0], start[0].len()));
    }
    // output file used to create listing file
    let mut lst_file = File::create(format!("./{}.lst", name.trim())).unwrap();
    write_lst_instr(&mut lst_file, 0, "".to_owned(), &start);

    // modification records are added here by get_machine_code function
    let mut mod_records: Vec<String> = Vec::new();

    // current text record string
    let mut current_text_record = String::new();
    // txt_loc is used to store correct memory location in text record
    let mut text_loc = 0;
    
    let mut loc_counter: usize = start[2].parse::<usize>().unwrap();
    let mut prev_loc = loc_counter;
    let mut line_count = 1;

    // output file for object code
    let mut obj_file = File::create(format!("./{}.obj", name.trim())).unwrap();
    write_obj_header(&mut obj_file, name, loc_counter, len);

    // for end entry in obj, default is same as start
    let mut end_addr: i32 = loc_counter as i32;

    // -1 base means we don't have BASE in use
    let mut base: isize = -1;

    for line in program[st_index+1..].iter() {
        line_count += 1;
        if line == "" || line.starts_with(".") {
            // empty line or comment
            write_lst_comment(&mut lst_file, &line);
            continue;
        }

        let mut split: Vec<&str> = line.split(" ").collect();

        let mut machine_code: Result<String, String> = Ok("--42".to_owned());
        let mut org_flag = false;

        let mut instr_size: Result<usize, String> = Ok(0);
        if !is_instr(split[0]) && !is_directive(split[0]) {
            // 1st word is label

            if !symtab.contains_key(split[0]) {
                return Err(format!("Line {}, label {} doesn't appear as left label.", line_count, split[0]));
            }

            if is_instr(split[1]) {
                // label for instr
                instr_size = determine_command_size(split[1]);
                machine_code = get_machine_code(&split[1..].to_vec(), &symtab, &mut mod_records, loc_counter, base);
            }
            else if is_directive(split[1]) {
                // label for directive
                if split[1] == "EQU" {
                    write_lst_instr(&mut lst_file, loc_counter, "".to_owned(), &split);
                    continue;
                }
                instr_size = determine_res_size(split[1], &split[2..].to_vec());
                machine_code =  handle_res(split[1], &split[2..].to_vec());
            }
            else if split[1].starts_with(".") {
                // label + comment is still valid... kind of
                write_lst_comment(&mut lst_file, &line);
                continue;
            }
            else {
                return Err(format!("Line {}, not a valid format.", line_count));
            }
        }
        else if is_instr(split[0]) {
            // 1st word is an instruction
            instr_size = determine_command_size(split[0]);
            machine_code = get_machine_code(&split[0..].to_vec(), &symtab, &mut mod_records, loc_counter, base);

            // for lst file
            split.insert(0," ")
        }
        else if is_directive(split[0]){
            // 1st word is a directive
            match split[0]{
                "EQU" => return Err(format!("Line {}, can't use EQU without label", line_count)),
                "END" => if symtab.contains_key(&*split[1]) {
                            end_addr = symtab[&*split[1]];
                        },
                "ORG" => instr_size = match parse_expr(split[1]) {
                                        Ok(value) => if value >= 0 {
                                                        org_flag = true;
                                                        machine_code = Ok("".to_owned());

                                                        Ok(value as usize)
                                                     } 
                                                     else { 
                                                        return Err(format!("Line {}, ORG needs non-negative number", line_count))
                                                     },
                                        Err(msg) => Err(msg.to_owned())
                                      },
                "BASE"   => base = match parse_expr(split[1]) {
                                        Ok(value) => if value >= 0 && value < i32::pow(2, 24) {
                                                        value as isize
                                                     } 
                                                     else { 
                                                        return Err(format!("Line {}, BASE needs number in interval [0, 16777216]", line_count))
                                                     },
                                        Err(msg) => return Err(msg.to_owned())
                                    },
                "NOBASE" => base = -1,
                _     => {instr_size = determine_res_size(split[0], &split[1..].to_vec());
                          machine_code = handle_res(split[0], &split[1..].to_vec());}
            }
            // for lst file
            split.insert(0," ")
        }
        else {
            // invalid
            return Err(format!("Line {}, Not a valid instruction", line_count));
        }

        match instr_size {
            Ok(size) => loc_counter += size,
            Err(msg) => return Err(format!("{}, line {}", line_count, msg))
        }

        match machine_code {
            Ok(value) => if value != "--42" {
                            if value == "RESW" || value == "RESB" || org_flag {
                                write_lst_instr(&mut lst_file, prev_loc, "".to_owned(), &split);

                                // if there is anything in text record to be written we must write it before we make empty space for reservation
                                if current_text_record.len() > 0 {
                                    // size / 2 because txt record is in nibbles
                                    write_obj_text(&mut obj_file, text_loc, current_text_record.len() / 2, current_text_record.clone());
                                    // adjust text record memory location
                                    text_loc += current_text_record.len() / 2;

                                    current_text_record = String::new();
                                }
                                // if directive was for reservation we adjust memory location to make that "reservation space"
                                if value == "RESW" || value == "RESB" {
                                    text_loc += loc_counter - prev_loc;
                                }
                                // org, we need to set text loc to loc counter which contains new org address
                                else {
                                    text_loc = loc_counter;
                                }
                            }
                            else {
                                write_lst_instr(&mut lst_file, prev_loc, value.clone(), &split);

                                // machine code is added to text record
                                current_text_record.push_str(&value);
                                // while text record is larger than the limit, write it to obj file
                                // this is used to split long char arrays into multiple lines
                                while current_text_record.len() >= 60 {
                                    write_obj_text(&mut obj_file, text_loc, 30, (&current_text_record[..60]).to_owned());
                                    current_text_record = (&current_text_record[60..]).to_owned();
                                    text_loc += 30;
                                }
                            }
                         },
            Err(msg) => return Err(format!("{}, line {}", line_count, msg))
        }

        prev_loc = loc_counter;
    }
    if current_text_record.len() > 0 {
        // at the end we still need to write obj file, since it might have not been written if it's too short
        write_obj_text(&mut obj_file, text_loc, current_text_record.len() / 2, current_text_record);
    }

    // write modification record after all text records
    for mod_rec in mod_records {
        write_obj_mod(&mut obj_file, mod_rec)
    }
    write_obj_end(&mut obj_file, end_addr as usize);

    Ok("OK".to_owned())
}