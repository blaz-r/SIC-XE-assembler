use std::collections::HashMap;
use crate::parser::parse_expr;

#[derive(Debug)]
pub struct EquExpression {
    dep_count: usize,
    expression: String,
    value: i32,
    valid_value: bool
}


// used when inserting direct value, ie: EQU *
pub fn insert_equ_number(equtab: &mut HashMap<String, EquExpression>, name: &str, value: i32) {
    let entry = EquExpression{dep_count: 0, expression: String::new(), value: value, valid_value: true};
    equtab.insert(name.to_owned(), entry);
}


// used to insert arbitrary expression, examples: EQU 42, EQU LEN+BEN
pub fn insert_equ_str(equtab: &mut HashMap<String, EquExpression>, name: &str, equexpr: &Vec<&str>) -> Result<u8, String> {
    // join entire expression and remove the comment
    let mut equ_str = String::new();
    for w in equexpr {
        if w.starts_with(".") {
            break;
        }
        equ_str.push_str(w);
    }
    // split by operators
    let mut split_vars: Vec<String> = equ_str.split(&['*', '/', '+', '-'][..]).map(|s| s.to_string()).collect();
    // filter to keep only variables
    split_vars = split_vars.into_iter().filter(|var| var.parse::<i32>().is_err()).collect();

    let mut expr_val = 0;

    // used to recognize if expression was evaluated or not
    let mut valid = false;

    if split_vars.len() == 0 {
        // expression is pure numbers
        valid = true;
        match parse_expr(&equ_str) {
            Ok(val) => expr_val = val,
            Err(msg) => return Err(msg.to_owned())
        }
    }

    // insert expression entry to table
    let entry = EquExpression{dep_count: split_vars.len(), expression: equ_str, value: expr_val, valid_value: valid};
    equtab.insert(name.to_string(), entry);

    Ok(1)
}


// before resolving equs insert already known variables
fn insert_known_vars(symtab: &HashMap<String, i32>, equtab: &mut HashMap<String, EquExpression>) -> Result<u8, String> {
    for (_, expr) in equtab.iter_mut() {
        // if it's already pure number skip
        if expr.dep_count == 0 {
            continue;
        }
        let mut equ_str = expr.expression.clone();

        // split by operators
        let mut split_vars: Vec<String> = equ_str.split(&['*', '/', '+', '-'][..]).map(|s| s.to_string()).collect();
        // filter to keep only variables
        split_vars = split_vars.into_iter().filter(|var| var.parse::<i32>().is_err()).collect();

        // insert already known variables
        let mut dep_count = split_vars.len();
        for var in split_vars.iter() {
            if symtab.contains_key(var) {
                let filled_equ_str = (&equ_str).replace(&*var, &(symtab[var].to_string()));
                equ_str = filled_equ_str;
                dep_count -= 1;
            }
        }

        expr.expression = equ_str;
        expr.dep_count = dep_count;
    }
    Ok(1)
}


// resolve all equs
pub fn resolve_equs(symtab: &mut HashMap<String, i32>, equtab: &mut HashMap<String, EquExpression>) -> Result<u8, String> {
    // first insert known variables from symtab
    match insert_known_vars(&symtab, equtab) {
        Ok(_) => (),
        Err(msg) => return Err(msg)
    }

    // repeat until all equs are resolved, if equs are more than 420000 deep or we are stuck, break
    let mut limiter = 420000;
    while equtab.len() > 0 && limiter > 0 {
        let mut current_resolved: String = String::new();
        let mut resolved_val: i32 = 0;

        // find equ that has no dependencies, ie. dep count is 0
        for (name, equ_expr) in equtab.iter() {
            if equ_expr.dep_count == 0 {
                current_resolved = name.to_owned();
                resolved_val = equ_expr.value;

                // if value isn't already pure, evaluate it
                if !equ_expr.valid_value {
                    match parse_expr(&equ_expr.expression) {
                        Ok(val) => resolved_val = val,
                        Err(msg) => return Err(msg.to_owned())
                    }
                }
            }
        }
        
        // update all expressions where current resolved one appears
        for (_, equ_expr) in equtab.iter_mut() {
            if (&equ_expr.expression).contains(&current_resolved) {
                let filled_dependent = (&equ_expr.expression).replace(&current_resolved, &(resolved_val.to_string()));
                equ_expr.expression = filled_dependent;
                equ_expr.dep_count -= 1;
            }
        }

        equtab.remove(&current_resolved);
        symtab.insert(current_resolved, resolved_val);

        limiter -= 1;
    }
    if limiter > 0 {
        Ok(1)
    }
    else {
        Err("Endless cycle in resolving EQUs".to_owned())
    }
}