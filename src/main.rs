mod lexer;
mod lexer_rules;

use std::{collections::HashMap, fs::File, io::{Read, Write}, process::Command, path::absolute};

use lexer::lex;
use lexer_rules::get_lexer_rules;

mod bytecode;
mod transpiler;

use transpiler::transpile;
use bytecode::gen::get_all_instructions;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut input_file = &String::new();
    let mut output_file = &String::new();
    let mut compile = false;
    // let mut bytecode = false;

    if args.len() - 1 == 2 {
        input_file = &args[1];
        output_file = &args[2];
    }
    else if args.len() - 1 == 3 {
        input_file = &args[1];
        output_file = &args[2];
        if &args[3] == "compile" {compile = true;}
        // else if &args[3] == "dump_bytecode" {bytecode = true;}
    }

    let mut code = String::new();
    let _ = File::open(absolute(input_file).unwrap()).unwrap().read_to_string(&mut code);

    let tokens = lex(code, get_lexer_rules());
    println!("{tokens:?}");
    // dbg!(&tokens);

    let mut binds = HashMap::new();
    let ir = get_all_instructions(tokens, &mut vec![], &mut binds);
    dbg!(&ir);

    let mut binds = HashMap::new();

    let c = transpile(ir, &mut binds);
    let _ = (File::create("./cmp/src/main.rs").unwrap()).write_all(c.as_bytes());

    if compile {
        let _ = Command::new("rustc")
        .args(["./cmp/src/main.rs", "-C", "lto", "-C", "opt-level=3", "-o", absolute(output_file).unwrap().to_str().unwrap()]).status().unwrap();
    }
}
