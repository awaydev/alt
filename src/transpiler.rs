use std::{collections::HashMap, fs::File, io::Read, process::exit};

use crate::{ir::{get_all_instructions, LogicOp, MathOp, Value}, lexer::lex, lexer_rules::get_lexer_rules};

pub fn transpile (instructions: Vec<Value>) -> String {
    let mut binds: HashMap<String, String> = HashMap::new();

    let code = format!(r#"
#![allow(warnings, unused)]

#[derive(PartialEq, Clone)]
enum Value {{
    String(String), Real(f64), Boolean(bool), Arr(Vec<Value>), Undefined
}}

fn get_float (v: Value) -> f64 {{
    match v {{
        Value::String(x) => {{ return x.len() as f64 }}
        // Value::Int(x) => {{ return x as f64 }}
        Value::Real(x) => {{ return x }}
        Value::Boolean(x) => {{ if x {{ return 1.0 }} else {{ return 0.0 }} }}
        Value::Arr(x) => {{ return x.len() as f64 }}
        _ => {{ return 0.0 }}
    }}
}}

fn fmgs (v: Value) -> String {{ match v {{ Value::String(x) => {{ return format!("\"{{x}}\"") }} _ => {{ return get_string(v) }} }} }}

fn get_string (v: Value) -> String {{
    match v {{
        Value::String(x) => {{ return x }}
        // Value::Int(x) => {{ return x.to_string() }}
        Value::Real(x) => {{ return x.to_string() }}
        Value::Boolean(x) => {{ return x.to_string() }}
        Value::Arr(x) => {{ format!("[{{}}]", x.iter().map(|v| fmgs(v.clone())).collect::<Vec<String>>().join(", ")) }}
        _ => {{ return "undefined".to_string() }}
    }}
}}

fn get_boolean (v: Value) -> bool {{
    match v {{
        Value::Boolean(x) => {{ return x }}
        Value::String(x) => {{ return true }}
        Value::Real(x) => {{ if x > 0.0 {{ return true }} else {{ return false }}  }}
        Value::Arr(x) => {{ return true }}
        _ => {{ return false }}
    }}
}}

fn get_array (v: Value) -> Vec<Value> {{
    match v {{
        Value::Arr(x) => {{ return x }}
        Value::String(x) => {{ return x.chars().map(|x| Value::String(x.to_string())).collect::<Vec<Value>>() }}
        Value::Real(x) => {{ return vec![Value::Real(0.0); x as usize] }}
        Value::Boolean(x) => {{ return vec![Value::Boolean(x)] }}
        _ => {{ return vec![] }}
    }}
}}

fn main () {{
    let mut stack: Vec<Value> = vec![];

// generated code
{}
}}    
    "#, instructions_to_code(instructions, &mut binds).join("\n"));

    return code
}

fn instructions_to_code (instructions: Vec<Value>, binds: &mut HashMap<String, String>) -> Vec<String> {
    let mut code_parts: Vec<String> = vec![];

    for ins in instructions {
        if let Some(x) = instruction_to_code(ins, binds) {
            code_parts.push(x);
        }
    }

    code_parts
}

fn instruction_to_code (instruction: Value, binds: &mut HashMap<String, String>) -> Option<String> {
    match instruction {
        Value::Real(y) => {return Some(format!("stack.push(Value::Real({y:?}));"))}
        Value::String(y) => {return Some(format!("stack.push(Value::String(\"{y}\".to_string()));"))}
        Value::Boolean(y) => {return Some(format!("stack.push(Value::Boolean({y}));"))}
        Value::MathOp(op) => {
            let mut a = "";
            match op {
                MathOp::Add => a = "b + a",
                MathOp::Div => a = "b / a",
                MathOp::Sub => a = "b - a",
                MathOp::Mul => a = "b * a",
                MathOp::Mod => a = "b % a",
                MathOp::BitShiftLeft => a = "((b as i32) << (a as i32)) as f64",
                MathOp::BitShiftRight => a = "((b as i32) >> (a as i32)) as f64",
                MathOp::Pow => a = "b.powf(a)"
            }

            return Some(format!("let a = get_float(stack.pop().unwrap()); let b = get_float(stack.pop().unwrap());
stack.push(Value::Real({a}));"))
        }
        Value::LogicOp(op) => {
            let mut a = "";
            match op {
                LogicOp::Eq => a = "b == a",
                LogicOp::NotEq => a = "b != a",
                LogicOp::Less => a = "get_float(b) < get_float(a)",
                LogicOp::LessEq => a = "get_float(b) <= get_float(a)",
                LogicOp::More => a = "get_float(b) > get_float(a)",
                LogicOp::MoreEq => a = "get_float(b) >= get_float(a)",
                LogicOp::Not => { return Some(format!("let a = stack.pop().unwrap(); stack.push(Value::Boolean(!get_boolean(a)));")) },
                LogicOp::And => a = "get_boolean(b) && get_boolean(a)",
                LogicOp::Or => a = "get_boolean(b) || get_boolean(a)"
            }

            return Some(format!("let a = stack.pop().unwrap(); let b = stack.pop().unwrap();
stack.push(Value::Boolean({a}));"))
        }
        Value::If(b) => {
            return Some(format!(r#"
let Value::Boolean(a) = stack.pop().unwrap() else {{ std::process::exit(1) }}; if a {{ {} }}
"#, instructions_to_code(b, binds).join("\n")))
        }
        Value::Else(b) => {
            return Some(format!(r#"else {{{}}}"#, instructions_to_code(b, binds).join("\n")))
        }
        Value::While(condition, body) => {
            let mut binds_local = binds.clone();
            let c = instructions_to_code(condition, &mut binds_local).join("\n");
            let b = instructions_to_code(body, &mut binds_local).join("\n");
            
            return Some(format!(r#"/* while */ loop {{
{c}let Value::Boolean(a) = stack.pop().unwrap() else {{ std::process::exit(1) }}; if !a {{ break; }};
{b}
}}"#))
        }
        Value::Loop(body) => {
            let mut binds_local = binds.clone();
            return Some(format!(r#"/* loop */ loop {{{}}}"#, instructions_to_code(body, &mut binds_local).join("\n")))
        }
        Value::Let(args, body) => {
            let mut binds = binds.clone();
            args.iter().for_each(|x| {conflict_binds_check(x, &mut binds); binds.insert(x.to_string(), format!("stack.push(__l_{x}.clone());"));});
            let parsed_args = args.iter().rev().map(|f| format!("let __l_{f} = stack.pop().unwrap();")).collect::<Vec<String>>().join("\n");
            let body = instructions_to_code(body, &mut binds).join("\n");
            return Some(format!(r#"/* let */ {{ {parsed_args} {body} }}"#))
        }
        Value::Var(name) => {
            if let Some(_) = binds.get(&name) {
                return Some(format!(r#"__l_{name} = stack.pop().unwrap();"#))
            }
            binds.insert(name.clone(), format!("stack.push(__l_{name}.clone());"));
            return Some(format!(r#"let mut __l_{name} = stack.pop().unwrap();"#))
        }
        Value::Macro(name, body) => {
            conflict_binds_check(&name, binds);
            binds.insert(name.clone(), format!("__m_{}!();", &name));
            let mut binds_local = binds.clone();
            let body = instructions_to_code(body, &mut binds_local).join("\n");
            return Some(format!(r#"macro_rules! __m_{name} {{
() => {{
{body}
}}
}}"#))
        }
        Value::Call(x) => {
            if let Some(y) = binds.get(&x.to_string()) {
                return Some(format!("{y}"))
            }
            else {
                eprintln!("ERROR: Undefined call of `{x}`");
                exit(1);
            }
        }
        Value::RustBind(name, body) => {
            conflict_binds_check(&name, binds);
            binds.insert(name, body.replace(r#"\""#, r#"""#));
        }
        Value::Use(path) => {
            let pathstr = path.clone().into_os_string().into_string().unwrap();
            if let None = binds.get(&pathstr) {
                binds.insert(pathstr.clone(), String::new());
                let mut nc = String::new();
                let _ = File::open(path).unwrap().read_to_string(&mut nc);
                let tks = lex(nc, get_lexer_rules());
                let ins = get_all_instructions(tks);
                return Some(format!("/* use {pathstr} */{}", instructions_to_code(ins, binds).join("\n")))
            }
        }
        Value::Array(x) => {
            let mut local_binds = binds.clone();
            return Some(format!(r#"/* [...] */ {{
let mut lstack = &mut stack;
let mut stack: Vec<Value> = vec![];
{}
lstack.push(Value::Arr(stack));
}}"#, instructions_to_code(x, &mut local_binds).join("\n")))
        }
        _ => {eprintln!("WARN: Unknown instruction: {instruction:?}");}
    }

    None
}

fn conflict_binds_check (name: &String, binds: &mut HashMap<String, String>) {
    if let Some(x) = binds.get(name) {
        if x.starts_with("__m") {
            eprintln!("ERROR: conflicting with the same defined macro: `{name}`. Macro is not allowed to redefine.");
            exit(1);
        }
    }
}