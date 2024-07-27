use std::collections::HashMap;
use crate::bytecode::value::{ Value, is_static_array };

pub fn transpile (instructions: Vec<Value>, binds: &mut HashMap<String, String>) -> String {
    let code = format!(r#"
#![allow(warnings, unused)]
mod alt;
use std::sync::{{Arc, Mutex}};

use alt::{{ value::*, display::*, stack::{{ pop, push }}, collections::{{ dict }}, ops::{{ set }}, r#ref::{{Ref, Covered}} }};

fn main () {{
    let mut stack: Vec<Value> = vec![];

// generated code
{}
    // dump stack
    println!("{{:?}}", Value::Arr(stack));
}}
"#, instructions_to_code(instructions, binds, 0).join("\n"));

    code
}

fn instructions_to_code (instructions: Vec<Value>, binds: &mut HashMap<String, String>, mode: i32) -> Vec<String> {
    let mut code_parts: Vec<String> = vec![];

    for instruction in instructions {
        if let Some(x) = instruction_to_code(instruction, binds, mode) {
            code_parts.push(x);
        }
    }

    return code_parts
}

fn instruction_to_code (instruction: Value, binds: &mut HashMap<String, String>, mode: i32) -> Option<String> {
    match instruction {
        Value::Array(_) | Value::Number(_) | Value::String(_) | Value::Get(_) | Value::NumOp(_, _, _) | Value::Not(_) | Value::LogOp(_, _, _) | Value::Ref(_)
        | Value::Call(_, _) | Value::Boolean(_) | Value::Dict(_, _) | Value::Pick(_, _) | Value::Type(_) | Value::RustReturnableBinding(_) => {
            return Some(match mode {
                3 => { format!("break 'block {};", unwrap_typed(instruction, binds)) }
                2 => { format!("result = {};", unwrap_typed(instruction, binds)) }
                1 => { format!("return {};", unwrap_typed(instruction, binds)) }
                0 => { format!("push(&mut stack, {});", unwrap_typed(instruction, binds)) }
                _ => { unwrap_typed(instruction, binds) }
            });
        }

        Value::Block(body) => return Some(format!("{{ {} }}", instructions_to_code(body, binds, mode).join("\n"))),

        Value::RefAssign(name, value) => {
            let v = unwrap_typed(*value, binds);
            if let Some(x) = binds.get(&name) {
                if x == "var" { return Some(format!("*_v_{name}.lock() = {v};")) }
                else { todo!() }
            }

            binds.insert(name.clone(), "var".to_string());

            return Some(format!("let mut _v_{name} = nvar!({v});"))
        }

        Value::Var(name, value) => {
            if name == "_" { return Some(format!("let _ = {};", unwrap_typed(*value, binds))) }

            if let Some(x) = binds.get(&name) {
                if x == "var" { return Some(format!("*_v_{name}.lock() = {};", unwrap_typed(*value, binds))) }
                else { todo!() }
            }

            binds.insert(name.clone(), "var".to_string());

            return Some(format!("let mut _v_{name} = {};", parse_value_as_ref(*value, binds)))
        }
        Value::Set(arr, index, value) => {
            return Some(format!("{{let index = {}; let value = {}; set({}, index, value); }}", unwrap_typed(*index, binds), unwrap_typed(*value, binds), parse_value_as_ref(*arr, binds)))
        },
        Value::Push(arr, value) => {
            return Some(format!("{{ let value = {}; let _ = {}.push(value); }}", unwrap_typed(*value, binds), unwrap_typed(*arr, binds)))
        },

        Value::If(condition, body) => {
            let condition = match *condition {
                Value::LogOp(_, _, _) | Value::Boolean(_) => unwrap_instruction(*condition, binds).unwrap(),
                _ => { format!("{}.cast_bool()", unwrap_typed(*condition, binds)) }
            };
            return Some(format!(r#"if {condition} {{ {} }}"#, instructions_to_code(body, &mut binds.clone(), mode).join("\n")))
        }
        Value::Else(body) => {
            return Some(format!(r#"else {{ {} }}"#, instructions_to_code(body, &mut binds.clone(), mode).join("\n")))
        }
        Value::ElseIf(condition, body) => {
            let condition = match *condition {
                Value::LogOp(_, _, _) | Value::Boolean(_) => unwrap_instruction(*condition, binds).unwrap(),
                _ => { format!("{}.cast_bool()", unwrap_typed(*condition, binds)) }
            };
            return Some(format!(r#"else if {condition} {{ {} }}"#, instructions_to_code(body, &mut binds.clone(), mode).join("\n")))
        }

        Value::Loop(body) => {
            return Some(format!(r#"loop {{{}}}"#, instructions_to_code(body, &mut binds.clone(), mode).join("\n")))
        }
        Value::Break => { return Some("break;".to_string()) }
        Value::Continue => { return Some("continue;".to_string()) }

        Value::Println(a) => { return Some(format!("println!(\"{{}}\", {});", unwrap_typed(*a, binds))) }

        Value::Fn(name, args, body) => {
            binds.insert(name.clone(), "function".to_string());
            let binds = &mut binds.clone();
            let r_args = || { let x = args.join(": Covered, mut _v_"); if args.len() > 0 { return format!("mut _v_{x}: Covered") } else { return x } };
            args.iter().for_each(|i| { binds.insert(i.clone(), "var".to_string()); });

            return Some(format!(r#"fn _f_{name} ({}) -> Value {{ {} Value::Empty }}"#, r_args(), instructions_to_code(body, binds, 1).join("\n")))
        }

        Value::Mov(into) => {
            if into == "_" { return Some(format!("let _ = pop(&mut stack);")) }
            binds.insert(into.clone(), "var".to_string());
            return Some(format!(r#"let _v_{into} = pop(&mut stack);"#))
        }
        Value::RustBinding(a) => {
            return Some(convert_rust_binding(a, binds));
        },
        _ => {}
    }
    None
}

fn unwrap_typed (instruction: Value, binds: &mut HashMap<String, String>) -> String {
    match instruction {
        Value::Number(a) => format!("Value::Number({a:?})"),
        Value::String(_) => format!("Value::String({})", unwrap_instruction(instruction, binds).unwrap()),
        Value::Boolean(_) => format!("Value::Boolean({})", unwrap_instruction(instruction, binds).unwrap()),
        Value::Get(name) => {
            if let Some(_) = binds.get("*NO_CLONE") { return format!("_v_{name}") }
            format!("_v_{name}.clone()")
        },
        Value::LogOp(_, _, _) | Value::Not(_) => format!("Value::Boolean({})", unwrap_instruction(instruction, binds).unwrap()),
        Value::NumOp(a, b, op) => {
            macro_rules! m { () => { return format!("({} {op} {})", unwrap_typed(*a, binds), unwrap_typed(*b, binds)); }; }
            match *a { Value::Get(_) | Value::NumOp(_, _, _) | Value::LogOp(_, _, _) => { m!(); } _ => {} }
            match *b { Value::Get(_) | Value::NumOp(_, _, _) | Value::LogOp(_, _, _) => { m!(); } _ => {} }
            return format!("Value::Number({})", unwrap_instruction(Value::NumOp(a, b, op), binds).unwrap())
        },
        Value::Array(body) => {
            println!("{body:?}");
            if is_static_array(&body) {
                return format!("Value::Arr(vec![{}])", instructions_to_code(body, binds, -1).join(", "));
            }
            format!("{{ let mut stack: Vec<Value> = vec![]; {} Value::Arr(stack) }}", instructions_to_code(body, &mut binds.clone(), 0).join("\n"))
        },
        Value::Call(_, _) => unwrap_instruction(instruction, binds).unwrap(),
        Value::Block(body) => format!("'block: {{ {} break 'block Value::Empty; }}", instructions_to_code(body, binds, 3).join("\n")),
        Value::Dict(k, v) => format!("Value::Dict(dict({}, {}))", unwrap_instruction(Value::Array(k), binds).unwrap(), unwrap_instruction(Value::Array(v), binds).unwrap()),
        Value::Pick(arr, index) => format!("{}[{}].clone()", unwrap_typed(*arr, binds), unwrap_typed(*index, binds)),
        Value::Type(_) => format!("Value::String({})", unwrap_instruction(instruction, binds).unwrap()),
        Value::RustReturnableBinding(a) => format!("{{ {} }}", convert_rust_binding(a, binds)),

        Value::Ref(x) => {
            if let Value::Get(x) = *x {
                format!("Value::Ref(_v_{x}.clone_ref())")
            }
            else {
                format!("Value::Ref(nvar!({}))", unwrap_typed(*x, binds))
            }
        },

        _ => "Value::Undefined".to_string()
    }
}

fn unwrap_instruction (instruction: Value, binds: &mut HashMap<String, String>) -> Option<String> {
    match instruction {
        Value::Number(a) => { return Some(format!("{a:?}")) }
        Value::String(a) => { return Some(format!("\"{a}\".to_string()")) }
        Value::Boolean(a) => { return Some(format!("{a:?}")) }
        
        Value::NumOp(a, b, op) => {
            let a = convert_number(*a, binds);
            let b = convert_number(*b, binds);
            if op == "<<" || op == ">>" { return Some(format!("((({a} as i64) {op} ({b} as i64)) as f64)")) }

            return Some(format!("({a} {op} {b})"))
        }

        Value::LogOp(a, b, op) => {
            let x;
            let y;
            match op.as_str() {
                "<" | ">" | "<=" | ">=" => {x = convert_number(*a, binds); y = convert_number(*b, binds);}
                _ => {x = unwrap_typed(*a, binds); y = unwrap_typed(*b, binds);}
            }
            if op == "=" { return Some(format!("({x} == {y})")) }

            return Some(format!("({x} {op} {y})"))
        }
        Value::Not(a) => { return Some(format!("!({})", unwrap_instruction(*a, binds).unwrap())) }

        Value::Get(name) => { return Some(format!("_v_{name}")) }
        Value::Ref(x) => {
            if let Value::Get(x) = *x {
                return Some(format!("_v_{x}"))
            }
            else {
                return Some(format!("nvar!({})", unwrap_typed(*x, binds)))
            }
        }

        Value::Call(name, args) => { return Some(format!("_f_{name}({})", args.iter().map(|i| parse_value_as_ref(i.clone(), binds)).collect::<Vec<String>>().join(", "))) }
        Value::Array(body) => {
            if is_static_array(&body) {
                return Some(format!("vec![{}]", instructions_to_code(body, binds, -1).join(", ")));
            }
            return Some(format!("{{ let mut stack: Vec<Value> = vec![]; {} stack }}", instructions_to_code(body, &mut binds.clone(), 0).join("\n")));
        }
        Value::Type(a) => { return Some(format!("{}.cast_type()", unwrap_typed(*a, binds))) },
        _ => {}
    }
    None
}

fn parse_value_as_ref (instruction: Value, binds: &mut HashMap<String, String>) -> String {
    if let Value::Ref(x) = instruction.clone() {
        if let Value::Get(x) = *x {
            return format!("_v_{x}.clone_ref()")
        }
    }
    return format!("nvar!({})", unwrap_typed(instruction, binds))
}

fn convert_number (instruction: Value, binds: &mut HashMap<String, String>) -> String {
    match instruction {
        Value::NumOp(_, _, _) | Value::Number(_) => unwrap_instruction(instruction, binds).unwrap(),
        Value::String(_) | Value::Array(_) => format!("{}.len() as f64", unwrap_instruction(instruction, binds).unwrap()),
        _ => format!("{}.cast_float()", unwrap_typed(instruction, binds))
    }
}

fn convert_rust_binding (binding: Vec<Value>, binds: &mut HashMap<String, String>) -> String {
    let mut res: Vec<String> = vec![];
    
    let mut i = 0;
    while i < binding.len() {
        match &binding[i] {
            Value::String(a) => { res.push(a.replace(r#"\""#, r#"""#)); }
            Value::Get(a) => { res.push(format!("_v_{a}")) }
            Value::Ref(a) => {
                if let Value::String(x) = *a.clone() {
                    res.push(format!("\"{x}\""));
                }
                else {
                    res.push(instruction_to_code(*a.clone(), binds, -1).unwrap());
                }
            }
            _ => { res.push(instruction_to_code(binding[i].clone(), binds, -1).unwrap()) }
        }
        i += 1;
    }

    return res.join("")
}