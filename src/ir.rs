use crate::lexer::{Token, TokenKind};
use std::path::{self, PathBuf};

#[derive(Debug)]
pub enum MathOp { Add, Sub, Div, Mul, Mod, BitShiftLeft, BitShiftRight, Pow }

#[derive(Debug)]
pub enum LogicOp { More, Less, MoreEq, LessEq, NotEq, Eq, Or, And, Not }

#[derive(Debug)]
pub enum Value {
    Boolean(bool), Real(f64),
    String(String),
    Let(Vec<String>, Vec<Value>) /* <args> <body> */, Var(String) /* <name> */,
    MathOp(MathOp),
    LogicOp(LogicOp),
    If(Vec<Value>), Else(Vec<Value>), While(Vec<Value>, Vec<Value>) /* <condition> <body> */, Loop(Vec<Value>) /* <body> */,
    Body(Vec<Token>), Array(Vec<Value>),
    Call(String), Macro(String, Vec<Value>) /* <name> <body> */,
    RustBind(String, String) /* <name> <rustcode> */, // TODO: Namespace(String),
    Use(PathBuf)
}

pub fn get_all_instructions (tokens: Vec<Token>) -> Vec<Value> {
    let mut instructions: Vec<Value> = vec![];

    let mut token = 0;
    while token < tokens.len() {
        if let Some(x) = get_instruction(&tokens[token..].to_vec()) {
            instructions.push(x.0);
            token += x.1;
        }
        else { token += 1; }
    }
    
    instructions
}

fn get_instruction (tokens: &Vec<Token>) -> Option<(Value, usize)> {
    let token = &tokens[0];
    match token.typ {
        TokenKind::Int => {return Some((Value::Real(format!("{}.0", token.value).parse::<f64>().unwrap()), 1))}
        TokenKind::Real => {return Some((Value::Real(token.value.parse::<f64>().unwrap()), 1))}
        TokenKind::String => {
            return Some((Value::String(parse_string(token.value.clone())), 1))
        }
        TokenKind::Operator => {
            match token.value.as_str() {
                "+" => {return Some((Value::MathOp(MathOp::Add), 1))}
                "-" => {return Some((Value::MathOp(MathOp::Sub), 1))}
                "/" => {return Some((Value::MathOp(MathOp::Div), 1))}
                "*" => {return Some((Value::MathOp(MathOp::Mul), 1))}
                "%" => {return Some((Value::MathOp(MathOp::Mod), 1))}
                ">>" => {return Some((Value::MathOp(MathOp::BitShiftRight), 1))}
                "<<" => {return Some((Value::MathOp(MathOp::BitShiftLeft), 1))}
                "**" => {return Some((Value::MathOp(MathOp::Pow), 1))}
                _ => {}
            }
        }
        TokenKind::Logical => {
            match token.value.as_str() {
                ">" => {return Some((Value::LogicOp(LogicOp::More), 1))}
                ">=" => {return Some((Value::LogicOp(LogicOp::MoreEq), 1))}
                "<" => {return Some((Value::LogicOp(LogicOp::Less), 1))}
                "<=" => {return Some((Value::LogicOp(LogicOp::LessEq), 1))}
                "!=" => {return Some((Value::LogicOp(LogicOp::NotEq), 1))}
                "=" => {return Some((Value::LogicOp(LogicOp::Eq), 1))}
                "&&" | "and" => {return Some((Value::LogicOp(LogicOp::And), 1))}
                "||" | "or" => {return Some((Value::LogicOp(LogicOp::Or), 1))}
                "!" | "not" => {return Some((Value::LogicOp(LogicOp::Not), 1))}
                _ => {}
            }
        }
        TokenKind::Assign => {
            if let TokenKind::Keyword = tokens[1].typ {
                return Some((Value::Var(tokens[1].value.to_string()), 2))
            }
        }
        TokenKind::Keyword => {
            match token.value.as_str() {
                "if" => {
                    match parse_pair_symbols(&tokens[1..].to_vec(), ("{", "}")) {
                        Some(x) => {
                            if let Value::Body(b) = x.0 {
                                return Some((Value::If(get_all_instructions(b)), x.1))
                            }
                        }
                        None => {}
                    }
                }
                "else" => {
                    match parse_pair_symbols(&tokens[1..].to_vec(), ("{", "}")) {
                        Some(x) => {
                            if let Value::Body(b) = x.0 {
                                return Some((Value::Else(get_all_instructions(b)), x.1))
                            }
                        }
                        None => {}
                    }
                }
                "while" => {
                    let mut condition: Vec<Value> = vec![];
                    let mut last = 0;
                    for i in &tokens[1..] {
                        last += 1;
                        if i.value == "{" { condition = get_all_instructions(tokens[1..last].to_vec()); break; }
                    }
                    match parse_pair_symbols(&tokens[last..].to_vec(), ("{", "}")) {
                        Some(body) => {
                            if let Value::Body(b) = body.0 {
                                return Some((Value::While(condition, get_all_instructions(b)), body.1 + last))
                            }
                        }
                        None => {}
                    }
                }
                "loop" => {
                    match parse_pair_symbols(&tokens[1..].to_vec(), ("{", "}")) {
                        Some(body) => {
                            if let Value::Body(b) = body.0 {
                                return Some((Value::Loop(get_all_instructions(b)), body.1))
                            }
                        }
                        None => {}
                    }
                }
                "let" => {
                    let mut args: Vec<String> = vec![];
                    let mut last = 0;
                    for i in &tokens[1..] {
                        last += 1;
                        if let TokenKind::Keyword = i.typ {args.push(i.value.clone());}
                        else {break;}
                    }
                    match parse_pair_symbols(&tokens[last..].to_vec(), ("{", "}")) {
                        Some(body) => {
                            if let Value::Body(b) = body.0 {
                                let l = args.len();
                                return Some((Value::Let(args, get_all_instructions(b)), body.1 + l))
                            }
                        }
                        None => {}
                    }
                }
                "macro" => {
                    if let TokenKind::Keyword = tokens[1].typ {
                        match parse_pair_symbols(tokens, ("{", "}")) {
                            Some(x) => {
                                if let Value::Body(b) = x.0 {
                                    return Some((Value::Macro(tokens[1].value.to_string(), get_all_instructions(b)), x.1))
                                }
                            }
                            None => {}
                        }
                    }
                }
                "true" | "false" => { return Some((Value::Boolean(token.value.parse::<bool>().unwrap()), 1)) }
                "bind" => {
                    if let TokenKind::Keyword = tokens[1].typ {
                        if let TokenKind::String = tokens[2].typ {
                            return Some((Value::RustBind(tokens[1].value.clone(), parse_string(tokens[2].value.clone())), 3))
                        }
                    }
                }
                "use" => {
                    let mut path = String::new();
                    if let TokenKind::Keyword = tokens[1].typ { path = format!("./alt/{}.alt", tokens[1].value) }
                    if let TokenKind::String = tokens[1].typ { path = parse_string(tokens[1].value.clone()) }
                    return Some((Value::Use(path::absolute(path).unwrap()), 2))
                }
                // "namespace" => {
                //     if let TokenKind::Keyword = tokens[1].typ { return Some((Value::Namespace(tokens[1].value.to_string()), 2)) }
                // }
                _ => {
                    return Some((Value::Call(token.value.to_string()), 1))
                }
            }
        }
        TokenKind::Bracket => {
            match parse_pair_symbols(tokens, ("[", "]")) {
                Some(b) => {
                    if let Value::Body(a) = b.0 {
                        let body = get_all_instructions(a);
                        return Some((Value::Array(body), b.1))
                    }
                }
                None => {}
            }
        }
        _ => {}
    }

    None
}

fn parse_pair_symbols (tokens: &Vec<Token>, pair: (&str, &str)) -> Option<(Value, usize)> {
    let mut pair_joined = 0;
    let mut children: Vec<Token> = vec![];

    let mut token = 0;
    while token < tokens.len() {
        if tokens[token].value == pair.0.to_string() {
            pair_joined += 1;
        }

        if tokens[token].value == pair.1.to_string() && pair_joined > 0 {
            if pair_joined == 1 {
                return Some((Value::Body(children), token+1))
            }
            else {
                pair_joined -= 1;
            }
        }

        if pair_joined > 0 {
            children.push(tokens[token].clone());
        }
    
        token += 1;
    }
    
    None
}

fn parse_string (value: String) -> String {
    let mut result = String::new();

    if value.starts_with("r") {
        result = value[3..value.len()-2].to_string();
        result = result.replace(r#"""#, r#"\""#);
    }
    else {result = value[1..value.len()-1].to_string();}

    result
}