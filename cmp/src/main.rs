
#![allow(warnings, unused)]
mod alt;
use std::sync::{Arc, Mutex};

use alt::{ value::*, display::*, stack::{ pop, push }, collections::{ dict }, ops::{ set }, r#ref::{Ref, Covered} };

fn main () {
    let mut stack: Vec<Value> = vec![];

// generated code
println!("{}", Value::String("                                      **".to_string()));
println!("{}", Value::String("                                     ***".to_string()));
println!("{}", Value::String("                                    ** *".to_string()));
println!("{}", Value::String("                                   *****".to_string()));
println!("{}", Value::String("                                  **   *".to_string()));
println!("{}", Value::String("                                 **** **".to_string()));
println!("{}", Value::String("                                **  ****".to_string()));
println!("{}", Value::String("                               ******  *".to_string()));
println!("{}", Value::String("                              **    ****".to_string()));
println!("{}", Value::String("                             ****  **  *".to_string()));
println!("{}", Value::String("                            **  ********".to_string()));
println!("{}", Value::String("                           ******      *".to_string()));
println!("{}", Value::String("                          **    **    **".to_string()));
println!("{}", Value::String("                         ****  ****  ***".to_string()));
println!("{}", Value::String("                        **  ****  **** *".to_string()));
println!("{}", Value::String("                       ******  ****  ***".to_string()));
println!("{}", Value::String("                      **    ****  **** *".to_string()));
println!("{}", Value::String("                     ****  **  ****  ***".to_string()));
println!("{}", Value::String("                    **  ********  **** *".to_string()));
println!("{}", Value::String("                   ******      ****  ***".to_string()));
println!("{}", Value::String("                  **    **    **  **** *".to_string()));
println!("{}", Value::String("                 ****  ****  ******  ***".to_string()));
println!("{}", Value::String("                **  ****  ****    **** *".to_string()));
println!("{}", Value::String("               ******  ****  **  **  ***".to_string()));
println!("{}", Value::String("              **    ****  ************ *".to_string()));
println!("{}", Value::String("             ****  **  ****          ***".to_string()));
println!("{}", Value::String("            **  ********  **        ** *".to_string()));
println!("{}", Value::String("           ******      ******      *****".to_string()));
println!("{}", Value::String("          **    **    **    **    **   *".to_string()));
println!("{}", Value::String("         ****  ****  ****  ****  **** **".to_string()));
println!("{}", Value::String("        **  ****  ****  ****  ****  ****".to_string()));
println!("{}", Value::String("       ******  ****  ****  ****  ****  *".to_string()));
println!("{}", Value::String("      **    ****  ****  ****  ****  ****".to_string()));
println!("{}", Value::String("     ****  **  ****  ****  ****  ****  *".to_string()));
println!("{}", Value::String("    **  ********  ****  ****  ****  ****".to_string()));
println!("{}", Value::String("   ******      ****  ****  ****  ****  *".to_string()));
println!("{}", Value::String("  **    **    **  ****  ****  ****  ****".to_string()));
println!("{}", Value::String(" ****  ****  ******  ****  ****  ****  *".to_string()));
println!("{}", Value::String(" *  ****  ****    ****  ****  ****  ****".to_string()));
    // dump stack
    println!("{:?}", Value::Arr(stack));
}
