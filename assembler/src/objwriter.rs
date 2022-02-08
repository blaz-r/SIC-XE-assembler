use std::fs::File;
use std::io::Write;


pub fn write_lst_instr(output: &mut File, loc_counter: usize, mut machine_code: String, instr_line: &Vec<&str>) {
    // shorten long codes to 6 chars
    if machine_code.len() > 6 {
        machine_code = format!("{}..{}", &machine_code[0..2], &machine_code[machine_code.len()-2..machine_code.len()])
    }
    // first line in lst is loc. counter and machine code
    write!(output, "{:05X}  {:>6}    ", loc_counter, machine_code).expect("Can't write to lst file");
    // following lines are all instructions or comments
    let mut normal_spacing = false;
    for instr in instr_line.iter(){
        // if we encounter comment, char/hex init or register/indexed, we stop making long indents
        if instr.starts_with(".") || instr.starts_with("C'") || instr.starts_with("X'") || instr.ends_with(",") || *instr == "EQU" {
            normal_spacing = true;
        }
        if normal_spacing {
            write!(output, "{} ", instr).expect("Can't write to lst file");
        }
        else {
            write!(output, "{:<14}", instr).expect("Can't write to lst file");
        }
    }
    writeln!(output).expect("Can't write to lst file");
}


pub fn write_lst_comment(output: &mut File, comment: &String) {
    writeln!(output, "                 {}", comment).expect("Can't write to lst file");
}


pub fn write_obj_header(output: &mut File, name: String, start: usize, len: usize) {
    writeln!(output, "H{:<6}{:06X}{:06X}", name, start, len).expect("Can't write ob header");
}


pub fn write_obj_text(output: &mut File, start: usize, len: usize, code: String) {
    writeln!(output, "T{:06X}{:02X}{}", start, len, code).expect("Can't write obj text");
}


pub fn write_obj_mod(output: &mut File, mod_rec: String) {
    writeln!(output, "M{}", mod_rec).expect("Can't write obj modification record");
}


pub fn write_obj_end(output: &mut File, instr: usize) {
    writeln!(output, "E{:06X}", instr).expect("Can't write obj end");
}