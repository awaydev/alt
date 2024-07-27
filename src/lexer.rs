use fancy_regex::Regex;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenKind {
    String,
    Int, Real, Operator,
    Keyword, Comment, Assign,
    Bracket, CurlyBracket, Logical,
    SpecialSymbol
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub typ: TokenKind,
    pub value: String,
    pub line: usize, pub col: usize, pub loc: String
}

#[derive(Clone)]
pub struct Rule {
    pub regex: Regex,
    pub typ: TokenKind
}

pub fn lex (code: String, ruleset: Vec<Rule>) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let code_chars = code.chars().collect::<Vec<char>>();

    let mut col = 0;
    let mut loc_col = 0;
    let mut line = 1;
    while col < code_chars.len() {
        if code_chars[col] == '\n' { line += 1; loc_col = 0; }
        match get_token(&code_chars[col..], &ruleset, line, loc_col) {
            Some(x) => {tokens.push(x.0); col += x.1; loc_col += x.1;}
            None => {col += 1; loc_col += 1;}
        }
    }

    tokens
}

fn get_token (code: &[char], ruleset: &Vec<Rule>, line: usize, col: usize) -> Option<(Token, usize)> {
    for i in ruleset.iter() {
        if let Some(x) = i.regex.find(code.iter().collect::<String>().as_str()).unwrap() {
            let value = x.as_str().to_string();
            let vlen = value.chars().count();
            return Some((Token {
                typ: i.typ,
                value,
                line, col, loc: String::new()
            }, vlen))
        }
    }

    None
}