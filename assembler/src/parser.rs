use std::fs;
use regex::Regex;
use std::cmp;


/* 
    read asm file to cleaned format
    all empty lines, with arbitrary number of white space, are replaced with ""
    all spaces is reduced to one space " "
*/
pub fn read_asm_file(filename: &str) -> Vec<String> {
    let data = fs::read_to_string(filename).expect("Can't read given file. Make sure you specified the right path.");
    let re_all_white = Regex::new(r"\s+").unwrap();
    // clean input so all whitespace is reduced to 1 space
    let cleaned: Vec<String> = data.lines().map(|line| re_all_white.replace_all(&line, " ").trim().to_string()).collect();
    // don't remove empty lines for accurate line error messaging
    // cleaned = cleaned.iter().filter(|&line| line != "").map(|line| line.trim().to_string()).collect();
    
    cleaned
}


// parse number in binary, octal, hex and decimal format
pub fn parse_num(num_str: &str) -> Result<i32, &str> {
    let num_upper = num_str.to_uppercase();
    let parsable; 
    let radix;
    
    if num_upper.starts_with("0X") {
        parsable = &num_str[2..];
        radix = 16;
    }
    else if num_upper.starts_with("0B") {
        parsable = &num_str[2..];
        radix = 2;
    }
    else if num_upper.starts_with("0O") {
        parsable = &num_str[2..];
        radix = 8;
    }
    else {
        parsable = num_str;
        radix = 10;
    }

    match i32::from_str_radix(parsable, radix) {
        Ok(value) => Ok(value),
        Err(_) => Err("Failed parsing number")
    }
}


// parse hex from in init X'<hex val>'
pub fn parse_hex(val_str: &str) -> Result<ResType, &str> {
    let mut data: Vec<u8> = Vec::new();

    // read value in between apostrophes
    let mut val_chars: Vec<char> = match val_str.split("'").nth(1) {
                                        Some(value) if value != "" => value.chars().collect(),
                                        _ => return Err("Invalid hex format. Use: X'<hex val>'. Example: X'42'")
                                   };
    // pad value with leading 0 if its size is not divisible by 2
    if val_chars.len() % 2 != 0 {
        val_chars.insert(0, '0');
    }
    let mut i = 0;
    // read bytes to array, 2 nibbles form one byte
    while i < val_chars.len() {
        let curr_byte = &val_chars[i..i+2].iter().collect::<String>();
        let byte_val = match u8::from_str_radix(curr_byte, 16) {
                            Ok(value) => value,
                            Err(_) => return Err("Can't parse hex expression")
                       };
        data.push(byte_val);
        i += 2;
    }

    Ok(ResType::Hex(data))
}


// parse char from init C'<char val>'
pub fn parse_char(val_str: &str) -> Result<ResType, &str> {
    // read value in between apostrophes and transform it to chars
    let val_chars: Vec<u8> = match val_str.split("'").nth(1) {
                                        Some(value) if value != "" => value.chars().map(|c| c as u8).collect(),
                                        _ => return Err("Invalid char format. Use: C'<char val>'. Example: C'SIC'")
                                   };

    Ok(ResType::Char(val_chars))
}


pub enum ResType {
    Num(usize),
    Hex(Vec<u8>),
    Char(Vec<u8>)
}


// parse init value when using BYTE and WORD
pub fn parse_init(num_str: &str) -> Result<ResType, &str> {
    match num_str.chars().nth(0).unwrap() {
        'X' => parse_hex(num_str),
        'C' => parse_char(num_str),
        // if it is num, we split by space and take only number (1st el), this avoids edge case of comments after res
        _ => match parse_num(num_str.split(" ").next().unwrap()) {
                Ok(value) => Ok(ResType::Num(value as usize)),
                Err(msg) => Err(msg)
        }
    }
    
}


// parse size of init value
pub fn parse_init_size(num_str: &str, type_size: usize) -> Result<usize, String> {
    match parse_init(num_str) {
        Ok(result) => match result {
                        // when dealing with just a number size is equal to type size
                        ResType::Num(_) => Ok(type_size),
                        // when dealing with hex arrays and char arrays, size needs to be at least size of type, but can be greater
                        ResType::Hex(vec) => Ok(cmp::max(type_size, vec.len())),
                        ResType::Char(vec) => Ok(cmp::max(type_size, vec.len()))
                      },
        Err(msg) => Err(msg.to_owned())
    }
}


// parse size of reservation
pub fn parse_res_size(num_str: &str, type_size: usize) -> Result<usize, String> {
    match parse_num(num_str) {
        Ok(value) => if value < 1 {
                        return Err("Reservations need to be greater than 0".to_owned())
                    }
                    else {
                        Ok(type_size * value as usize)
                    },
        Err(msg) => Err(msg.to_owned())
    }
}


// parse arithmetic expression
pub fn parse_expr(expr: &str) -> Result<i32, String> {
    let mut stack: Vec<String> = Vec::new();

    let exp_chars: Vec<char> = expr.chars().collect();
    // parse
    let mut last_val: String;
    let mut last_op: String;
    let mut current = String::new();

    for i in 0..expr.len() {
        let ch = exp_chars[i];
        
        // skip all white spaces in expression
        if ch.is_whitespace() {
            continue;
        }

        // while we are on digit push chars to current number string
        if ch.is_digit(10) {
            current.push(ch)
        }
        else if ch == '+' || ch == '-' || ch == '*' || ch == '/' {
            // when we encounter operator, we push current number to stack
            stack.push(current);
            current = String::new();

            // first operator is just pushed to stack since we need at least 2 numbers
            if stack.len() == 1 {
                stack.push(ch.to_string());
            }
            else {
                // get last value and operator
                last_val = match stack.pop() {
                            Some(val) => val,
                            None => return Err("Invalid expression".to_owned())
                        };
                last_op = match stack.pop() {
                    Some(val) => val,
                    None => return Err("Invalid expression".to_owned())
                };

                // if current operator is * or / and previous was + or - we push them back due to precedence
                if (ch == '*' || ch == '/') && (last_op == "+" || last_op == "-") {
                    stack.push(last_op);
                    stack.push(last_val);
                }
                else {
                    // parse both numbers from stack
                    let last_num = match last_val.parse::<i32>() {
                                      Ok(val) => val,
                                      Err(msg) => return Err(format!("Can't parse {}, {}", last_val, msg))
                                   };
                    let stack_num = match stack.pop().unwrap().parse::<i32>() {
                                    Ok(val) => val,
                                    Err(msg) => return Err(format!("Can't parse, {}", msg))
                                 };
                    // perform operation and push back to stack
                    match last_op.as_ref() {
                        "+" => stack.push(format!("{}", stack_num + last_num)),
                        "-" => stack.push(format!("{}", stack_num - last_num)),
                        "*" => stack.push(format!("{}", stack_num * last_num)),
                        "/" => stack.push(format!("{}", stack_num / last_num)),
                        _ => return Err(format!("Wrong operator {}", last_op))
                    }
                }
                // also push operator on stack
                stack.push(ch.to_string());
            }
        }
        else {
            return Err(format!("Invalid character {}", ch));
        }
    }
    // at the end push te current number, ie. the rightmost to stack
    stack.push(current);

    // evaluate
    let mut last_val: String;
    while stack.len() > 1 {
        last_val = stack.pop().unwrap();
        
        // get both numbers and parse them
        let op = stack.pop().unwrap();
        let stack_num = match stack.pop().unwrap().parse::<i32>() {
                            Ok(val) => val,
                            Err(msg) => return Err(format!("Can't parse, {}", msg))
                        };
        let last_num = match last_val.parse::<i32>() {
                            Ok(val) => val,
                            Err(msg) => return Err(format!("Can't parse {}, {}", last_val, msg))
                        };
        
        // apply operation
        match op.as_ref() {
            "+" => stack.push(format!("{}", stack_num + last_num)),
            "-" => stack.push(format!("{}", stack_num - last_num)),
            "*" => stack.push(format!("{}", stack_num * last_num)),
            "/" => stack.push(format!("{}", stack_num / last_num)),
            _ => return Err(format!("Wrong operator {}", op))
        }
    }

    // this also covers just 1 number without operations
    Ok(stack.pop().unwrap().parse::<i32>().unwrap())
}


pub fn parse_reg(reg_str: &str) -> Result<u8, &str> {
    // AXLBSTF → 0,1,2,3,4,5,6
    match reg_str {
        "A" => Ok(0),
        "X" => Ok(1),
        "L" => Ok(2),
        "B" => Ok(3),
        "S" => Ok(4),
        "T" => Ok(5),
        "F" => Ok(6),
        _   => Err("Not a valid register")
    }
}


pub fn parse_indexed(operands: &Vec<&str>) -> Option<String> {
    // since there can be space after comma: either var,X or var, X. We join them to var,X
    let val_join;
    match operands[0].ends_with(",") {
        // if it ends with , it means other reg is separated by space so we join
        true if operands.len() > 1 => val_join = format!("{}{}", operands[0], operands[1]),
        _ => val_join = operands[0].to_owned(),
    }
    let val_split: Vec<&str> = val_join.split(",").map(|val| val.trim()).collect();
    if val_split.len() < 2 || val_split[1] != "X" {
        return None;
    }
    else {
        return Some(val_split[0].to_owned());
    }
}