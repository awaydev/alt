use std::{collections::{HashMap, HashSet}, fs::File, io::Read, path::absolute};

use crate::lexer::{lex, Token, TokenKind};
use crate::lexer_rules::get_lexer_rules;

use super::parse::*;

use crate::bytecode::value::Value;

macro_rules! bc_error {
    ($token:expr, $msg:expr) => {
        panic!("{}:{} {}", $token.line, $token.col, $msg);
    };
}

macro_rules! get_mode {
    ($binds:expr, $y:tt, $($x:expr),+) => {
        if let Some(Value::ParseModes(v)) = $binds.get("=") {
            $(
                if let Some(_) = v.get($x) { $y }
            )+
        }
    };
}
macro_rules! add_mode {
    ($binds:expr, $($x:expr),+) => {
        if let Some(Value::ParseModes(v)) = $binds.get("=") {
            let mut v = v.clone(); $(v.insert($x.to_string()))+;
            $binds.insert("=".to_string(), Value::ParseModes(v));
        }
        else { $binds.insert("=".to_string(), Value::ParseModes(HashSet::from([$($x.to_string(),)+]))); }
    };
}

pub fn get_all_instructions (tokens: Vec<Token>, instructions: &mut Vec<Value>, binds: &mut HashMap<String, Value>) -> Vec<Value> {
    let mut token = 0;
    while token < tokens.len() {
        token += get_instruction(&tokens[token..].to_vec(), instructions, binds);
    }
    
    instructions.to_vec()
}

fn get_instruction (tokens: &Vec<Token>, instructions: &mut Vec<Value>, binds: &mut HashMap<String, Value>) -> usize {
    let token = &tokens[0];

    match token.typ {
        TokenKind::String => {instructions.push(Value::String(parse_string(token.value.clone()))); return 1}
        TokenKind::Int => {instructions.push(Value::Number(format!("{}.0", token.value).parse::<f64>().unwrap())); return 1}
        TokenKind::Real => {instructions.push(Value::Number(token.value.parse::<f64>().unwrap())); return 1}
        TokenKind::Operator | TokenKind::Logical => {
            let v = token.value.as_str();
            let b = Box::new(popv(instructions).expect(&format!("{}:{} соси хуй 1", token.line, token.col)));
            if v == "not" || v == "!" { instructions.push(Value::Not(b).process()); return 1 }
            let a = Box::new(popv(instructions).expect(&format!("{}:{} соси хуй 2", token.line, token.col)));
            match v {
                "+" | "-" | "/" | "*" | "%" | "<<" | ">>" => instructions.push(Value::NumOp(a, b, token.value.clone()).process()),
                "**" => instructions.push(Value::Pow(a, b)),
                "=" | "!=" | "&&" | "||" | "<" | ">" | "<=" | ">=" => instructions.push(Value::LogOp(a, b, token.value.clone()).process()),
                "and" => instructions.push(Value::LogOp(a, b, "&&".to_string())),
                "or" => instructions.push(Value::LogOp(a, b, "||".to_string())),
                _ => {}
            }
            return 1
        }
        TokenKind::Assign => {
            if let TokenKind::Keyword = tokens[1].typ {
                let a = popv(instructions).expect(&format!("{}:{} cannot get value to assign to variable.", token.line, token.col));
                if let Some(Value::Ref(_)) = binds.get(tokens[1].value.as_str()) {
                    instructions.push(Value::RefAssign(tokens[1].value.clone(), Box::new(a)));
                    return 2
                }
                binds.insert(tokens[1].value.clone(), Value::Get(tokens[1].value.clone()));
                instructions.push(Value::Var(tokens[1].value.clone(), Box::new(a)));
            }
            else { bc_error!(token, "Variable name must be a keyword"); }

            return 2
        }
        TokenKind::SpecialSymbol => {
            match token.value.as_str() {
                "&" => {
                    let a = popv(instructions).unwrap();
                    instructions.push(Value::Ref(Box::new(a)));
                }
                _ => {}
            }
            return 1
        }
        TokenKind::Keyword => {
            match token.value.as_str() {
                "true" | "false" => { instructions.push(Value::Boolean(token.value == "true")); return 1 }
                "if" => {
                    match parse_pair_symbols(&tokens[1..].to_vec(), ("{", "}")) {
                        Some(x) => {
                            let condition = instructions.pop().expect(&format!("{}:{} cannot parse condition for if-statement", token.line, token.col));
                            if let Value::Boolean(false) = condition { instructions.push(Value::FailedIf); return x.1 }
                            let body = get_all_instructions(x.0, &mut vec![], &mut binds.clone());
                            if let Value::Boolean(true) = condition { instructions.push(Value::Block(body)); instructions.push(Value::PassedIf); return x.1 }
                            instructions.push(Value::If(Box::new(condition), body));

                            return x.1
                        }
                        None => {}
                    }
                }
                "else" => {
                    match parse_pair_symbols(&tokens[1..].to_vec(), ("{", "}")) {
                        Some(x) => {
                            let previous_ins = instructions.last().unwrap();
                            if let Value::PassedIf = previous_ins { instructions.pop(); return x.1 }
                            let body = get_all_instructions(x.0, &mut vec![], &mut binds.clone());
                            if let Value::FailedIf = previous_ins { instructions.pop(); instructions.push(Value::Block(body)); return x.1 }
                            instructions.push(Value::Else(body));
                            return x.1
                        }
                        None => {
                            if tokens[1].value == "if".to_string() {
                                let c = instructions.pop().unwrap();
                                let previous_ins = instructions.last().unwrap();
                                let b = parse_pair_symbols(&tokens[2..].to_vec(), ("{", "}")).unwrap();
                                if let Value::PassedIf = previous_ins { return b.1 + 1 }
                                let body = get_all_instructions(b.0, &mut vec![], &mut binds.clone());
                                if let Value::FailedIf = previous_ins {
                                    if let Value::Boolean(true) = c { instructions.pop(); instructions.push(Value::Block(body)); instructions.push(Value::PassedIf); return b.1 + 1 }
                                    else if let Value::Boolean(false) = c { return b.1 + 1 }
                                }
                                instructions.push(Value::ElseIf(Box::new(c), body));
                                return b.1 + 1
                            }
                        }
                    }
                }
                "loop" => {
                    match parse_pair_symbols(&tokens[1..].to_vec(), ("{", "}")) {
                        Some(b) => {
                            let binds = &mut binds.clone();
                            add_mode!(binds, "loop");
                            instructions.push(Value::Loop(get_all_instructions(b.0, &mut vec![], binds)));
                            return b.1
                        }
                        None => {}
                    }
                }
                "break" => { get_mode!(binds, { instructions.push(Value::Break); return 1; }, "loop"); bc_error!(token, "`break` can be used only in loops"); }
                "continue" => { get_mode!(binds, { instructions.push(Value::Continue); return 1; }, "loop"); bc_error!(token, "`continue` can be used only in loops"); }
                "let" => {
                    let (mut args, last) = parse_args(tokens, 1);
                    match parse_pair_symbols(&tokens[last..].to_vec(), ("{", "}")) {
                        Some(body) => {
                            let mut binds = binds.clone();
                            let l = args.len();
                            args.reverse();
                            args.iter().for_each(|i| {
                                binds.insert(i.clone(), {
                                    if i.clone().starts_with(":") { instructions.pop().expect("Cannot get instruction in `let`") } else { popv(instructions).expect("Cannot pop value in `let`") }
                                    // TODO: better errors
                                });
                            });
                            get_all_instructions(body.0, instructions, &mut binds);
                            return body.1 + l
                        }
                        None => {}
                    }
                }
                "times" => {
                    let times = popv(instructions).unwrap();
                    if let Value::Number(a) = times {
                        let body = parse_pair_symbols(&tokens[1..].to_vec(), ("{", "}")).unwrap();
                        let mut i = 0;
                        while i < a as i64 {
                            get_all_instructions(body.0.clone(), instructions, binds);
                            i += 1;
                        }

                        return body.1
                    }
                }
                "do" => {
                    let body = parse_pair_symbols(&tokens[1..].to_vec(), ("{", "}")).unwrap();
                    instructions.push(Value::Raw(Box::new(Value::Do(body.0, 0))));
                    return body.1
                }
                "unwrap" => {
                    if let Value::Raw(a) = instructions.pop().unwrap() {
                        if let Value::Do(a, _) = *a {
                            get_all_instructions(a, instructions, binds);
                        }
                        else { instructions.push(*a); }
                    }
                }
                "do:add_tokens" => {
                    if let Value::String(a) = popv(instructions).unwrap() {
                        if let Value::Raw(b) = instructions.pop().unwrap() {
                            if let Value::Do(mut b, _) = *b {
                                b.append(&mut lex(a, get_lexer_rules()));
                                instructions.push(Value::Do(b, 0));
                                return 1
                            }
                        }
                    }
                    bc_error!(token, "cannot parse raw value. Make sure you are using it like `do {...} \"...\" do:add_tokens`");
                }
                "macro" | "macro:b:" => {
                    if let TokenKind::Keyword = tokens[1].typ {
                        let name = tokens[1].value.clone();
                        let body = parse_pair_symbols(&tokens[2..].to_vec(), ("#!", "!#")).expect("Cannot parse body of macro. Make sure that body is covered in #!..#!");
                        // if let Some(x) = body.0.iter().find(|x| x.value == name) {
                        //     bc_error!(x, "unavoidable infinite self-expansion");
                        // }
                        binds.insert(name, Value::Do(body.0, (token.value == "macro:b:") as i32));
                        return body.1 + 1
                    }
                }
                "fn" => {
                    if let TokenKind::Keyword = tokens[1].typ {
                        let (args, last) = parse_args(tokens, 2);
                        let body = parse_pair_symbols(&tokens[last..].to_vec(), ("{", "}")).unwrap();
                        binds.insert(tokens[1].value.clone(), Value::TCall(tokens[1].value.clone(), args.len()));
                        let mut binds = binds.clone();
                        for (name, value) in binds.clone().iter() { if let Value::Get(_) = value { binds.remove(name); } } // remove variables that out of scope of variable
                        binds.insert("=".to_string(), Value::ParseModes(HashSet::from(["fn".to_string()])));
                        // args.iter().for_each(|i| { binds.insert(i.clone(), Value::Get(i.clone())); });
                        args.iter().for_each(|i| { binds.insert(i.clone(), Value::Ref(Box::new(Value::Get(i.clone())))); });
                        instructions.push(Value::Fn(tokens[1].value.clone(), args, get_all_instructions(body.0, &mut vec![], &mut binds)));

                        return body.1 + last
                    }
                }
                "type" => { let a = popv(instructions).unwrap(); instructions.push(Value::Type(Box::new(a))) }
                "pick" => {
                    let index = popv(instructions).unwrap();
                    let arr = popv(instructions).unwrap();
                    if arr.is_static() && index.is_static() { instructions.push(arr[index].clone()); return 1 }
                    instructions.push(Value::Pick(Box::new(arr), Box::new(index)));
                }
                "set" => { // can be implemented with rust_exec instruction
                    let value = Box::new(popv(instructions).unwrap());
                    let index = Box::new(popv(instructions).unwrap());
                    let arr = Box::new(popv(instructions).unwrap());
                    instructions.push(Value::Set(arr, index, value));
                }
                "mov" => {
                    if let TokenKind::Keyword = tokens[1].typ {
                        get_mode!(binds, {
                            binds.insert(tokens[1].value.clone(), Value::Get(tokens[1].value.clone()));
                            instructions.push(Value::Mov(tokens[1].value.clone()));
                            return 2
                        }, "array");
                    }
                    bc_error!(token, "`mov` is not available out of the arrays");
                }
                "push" => { // can be implemented with rust_exec instruction
                    let value = Box::new(popv(instructions).unwrap());
                    let parent = Box::new(popv(instructions).unwrap());
                    instructions.push(Value::Push(parent, value));
                }
                "dict" => {
                    let Value::Array(v) = popv(instructions).unwrap() else { todo!() }; let Value::Array(k) = popv(instructions).unwrap() else { todo!() };
                    instructions.push(Value::Dict(k, v));
                    return 1
                }
                "println" => {
                    let x = popv(instructions).expect("println cannot pop the thing");
                    instructions.push(Value::Println(Box::new(x)));
                    return 1
                }
                ":rust!" | ":rust!:" => {
                    if let Some(Value::Array(a)) = popv(instructions) {
                        match token.value.as_str() {
                            ":rust!" => instructions.push(Value::RustBinding(a)),
                            ":rust!:" => instructions.push(Value::RustReturnableBinding(a)),
                            _ => {}
                        };
                    }
                    // error handling
                }
                "use" => {
                    let path = popv(instructions).expect("cannot get path");
                    if let Value::String(path) = path {
                        if let Some(_) = binds.get(&path) { return 1; }
                        binds.insert(path.clone(), Value::Undefined);
                        let path = absolute(path).unwrap();
                        
                        let mut code = String::new();
                        let _ = File::open(path).unwrap().read_to_string(&mut code);
                        let tokens = lex(code, get_lexer_rules());
                        get_all_instructions(tokens, instructions, binds);
                    }
                }
                ":current_code_place!:" => { instructions.push(Value::String(format!("{}:{}", token.line, token.col))) }
                _ => {
                    if let Some(x) = binds.get(&token.value) {
                        if let Value::Do(x, offset) = x {
                            let x = if offset == &1 {
                                let (mut body, len) = parse_pair_symbols(&tokens[1..].to_vec(), ("{", "}")).unwrap();
                                body.insert(0, Token { value: "do".to_string(), typ: TokenKind::Keyword, line: token.line, col: token.col, loc: token.loc.clone() });
                                body.insert(1, Token { value: "{".to_string(), typ: TokenKind::CurlyBracket, line: token.line, col: token.col, loc: token.loc.clone() });
                                body.push(Token { value: "}".to_string(), typ: TokenKind::CurlyBracket, line: token.line, col: token.col, loc: token.loc.clone() });
                                body.extend(x.iter().cloned());
                                body.extend(tokens[len..].iter().cloned());
                                body
                            } else {
                                let mut x: Vec<Token> = x.iter().map(|x| Token { typ: x.typ, value: x.value.clone(), line: token.line, col: token.col, loc: token.loc.clone() }).clone().collect();
                                x.extend(tokens[1..].iter().cloned());
                                dbg!(&x);
                                x
                            };
                            let len = x.len();
                            get_all_instructions(x, instructions, binds);
                            return len;
                        }
                        else if let Value::TCall(name, n) = x {
                            let mut a = vec![];
                            for _ in 0 .. *n { a.push(popv(instructions).unwrap()); }
                            a.reverse();
                            instructions.push(Value::Call(name.clone(), a));
                        }
                        else { instructions.push(x.clone()); }
                    }
                    else { bc_error!(token, format!("unknown keyword: `{}`", token.value)); }
                }
            }
            
            return 1
        }
        TokenKind::CurlyBracket => {
            if token.value == "{" {
                let b = parse_pair_symbols(tokens, ("{", "}")).expect(&format!("{}:{}", token.line, token.col));
                let binds = &mut binds.clone();
                let l = instructions.len();
                get_all_instructions(b.0, instructions, binds);
                let body = if instructions.len() <= l { instructions.drain(instructions.len()-1..l-1).collect() } else { instructions.drain(l..instructions.len()).collect() };
                instructions.push(Value::Block(body));
                return b.1
            }
            return 1
        }
        TokenKind::Bracket => {
            match parse_pair_symbols(tokens, ("[", "]")) {
                Some(b) => {
                    let binds = &mut binds.clone();
                    binds.insert("=".to_string(), Value::ParseModes(HashSet::from(["array".to_string()])));
                    let body = get_all_instructions(b.0, &mut vec![], binds);
                    instructions.push(Value::Array(body));
                    return b.1
                }
                None => {}
            }
            return 1
        }
        _ => { return 1 }
    }
}

// pop valuе. use it for non-raw execution of instruction (e.g. sum numbers, println...)
pub fn popv (instructions: &mut Vec<Value>) -> Option<Value> {
    let x = instructions.pop()?;

    match x {
        Value::Array(_) | Value::Number(_) | Value::String(_) | Value::Boolean(_) | Value::NumOp(_, _, _) | Value::LogOp(_, _, _) | Value::Get(_) | Value::Call(_, _) | Value::Dict(_, _)
        | Value::Pick(_, _) | Value::Block(_) | Value::Type(_) | Value::RustReturnableBinding(_)
        | Value::Ref(_) | Value::Undefined => Some(x),
        Value::Else(_) => {
            let mut block: Vec<Value> = vec![x];
            let mut v = instructions.pop()?;
            loop {
                block.push(v.clone());
                if let Value::If(_, _) = v { break; }
                v = instructions.pop()?;
            }
            block.reverse();
            
            Some(Value::Block(block))
        }
        _ => {
            let y = popv(instructions);
            instructions.push(x);
            return y
        }
    }
}