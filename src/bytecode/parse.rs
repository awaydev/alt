use crate::lexer::{TokenKind, Token};

pub fn parse_pair_symbols (tokens: &Vec<Token>, pair: (&str, &str)) -> Option<(Vec<Token>, usize)> {
    let mut pair_joined = 0;
    let mut children: Vec<Token> = vec![];

    if tokens[0].value == pair.0.to_string() {
        pair_joined += 1;
    }
    else { return None }

    let mut token = 1;
    while token < tokens.len() {
        if tokens[token].value == pair.0.to_string() {
            pair_joined += 1;
        }

        if tokens[token].value == pair.1.to_string() && pair_joined > 0 {
            if pair_joined == 1 {
                return Some((children, token+1))
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

pub fn parse_args (tokens: &Vec<Token>, start: usize) -> (Vec<String>, usize) {
    let mut args: Vec<String> = vec![];
    let mut last = start;
    for i in &tokens[start..] {
        if let TokenKind::Keyword = i.typ { last += 1; args.push(i.value.clone()); }
        else { break; }
    }
    
    (args, last)
}

pub fn parse_string (value: String) -> String {
    let mut result;

    if value.starts_with("r") {
        result = value[3..value.len()-2].to_string();
        result = result.replace(r#"""#, r#"\""#);
    }
    else {result = value[1..value.len()-1].to_string();}

    result
}