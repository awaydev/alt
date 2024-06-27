mod lexer;
mod lexer_rules;

use std::{fs::File, io::{Read, Write}, process::Command};

use lexer::lex;
use lexer_rules::get_lexer_rules;

mod transpiler;
use transpiler::transpile;

mod ir;
use ir::get_all_instructions;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut input_file = &String::new();
    let mut output_file = &String::new();
    let mut compile = false;
    let mut bytecode = false;

    if args.len() - 1 == 2 {
        input_file = &args[1];
        output_file = &args[2];
    }
    else if args.len() - 1 == 3 {
        input_file = &args[1];
        output_file = &args[2];
        if &args[3] == "compile" {compile = true;}
        else if &args[3] == "dump_bytecode" {bytecode = true;}
    }

    let mut code = String::new();
    let _ = File::open(input_file).unwrap().read_to_string(&mut code);

    let tokens = lex(code, get_lexer_rules());
    
    let ir = get_all_instructions(tokens);
    if bytecode { println!("{ir:?}"); }

    let c = transpile(ir);
    let _ = (File::create(output_file).unwrap()).write_all(c.as_bytes());

    if compile {
        let _ = Command::new("rustc")
        .args([output_file, "-C", "lto", "-C", "opt-level=3"]).status().unwrap();
    }
}
