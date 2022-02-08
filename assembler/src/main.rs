mod parser;
mod symbols;
mod commands;
mod assemble;
mod objwriter;
mod equresolution;

use std::env;

fn main() {
    let asm_file;
    match env::args().nth(1) {
        Some(file) => asm_file = file,
        None => {println!("No asm file provided."); return;}
    }

    let asm_in = parser::read_asm_file(&asm_file);

    // first pass
    let symtab;
    let len;
    match symbols::get_symbol_table(&asm_in) {
        Ok(value) => {symtab = value.1; len = value.0},
        Err(msg) => { println!("Error generating symtab: {}", msg); return; }
    }

    match assemble::generate_obj(&asm_in, &symtab, len) {
        Ok(_) => (),
        Err(msg) => { println!("Error assembling: {}", msg); return; }
    }
}
