use fancy_regex::Regex;

#[derive(Clone, Copy, Debug)]
pub enum TokenKind {
    String,
    Int, Real, Operator,
    Keyword, Comment, Assign, // Body
    Bracket, CurlyBrace, Logical // this kind of token is parsing in the compiler.rs file because of it can't be done with Regex
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenKind,
    pub value: String
}

#[derive(Clone)]
pub struct Rule {
    pub regex: Regex,
    pub typ: TokenKind
}

pub fn lex (code: String, ruleset: Vec<Rule>) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut col = 0;
    while col < code.len() {
        match get_token(&code[col..], &ruleset) {
            Some(x) => {tokens.push(x.0); col += x.1;}
            None => {col += 1;}
        }
    }

    tokens
}

fn get_token (code: &str, ruleset: &Vec<Rule>) -> Option<(Token, usize)> {
    for i in ruleset.iter() {
        if let Some(x) = i.regex.find(code).unwrap() {
            let value = x.as_str().to_string();
            let vlen = value.len();
            return Some((Token {
                typ: i.typ,
                value
            }, vlen))
        }
    }

    None
}