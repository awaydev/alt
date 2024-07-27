use fancy_regex::Regex;
use crate::lexer::{Rule, TokenKind};

pub fn get_lexer_rules () -> Vec<Rule> {
    vec![
        Rule {
            typ: TokenKind::Real,
            regex: Regex::new(r#"^(\-?[0-9]+)\.([0-9]+)"#).unwrap()
        },
        Rule {
            typ: TokenKind::Int,
            regex: Regex::new(r"^\-?[0-9]+").unwrap()
        },
        Rule {
            typ: TokenKind::String,
            regex: Regex::new(r#"(?s)^(["'])(?:(?=(\\?))\2.)*?\1|^r\#"(.*?)"\#"#).unwrap()
        },
        Rule {
            typ: TokenKind::Comment,
            regex: Regex::new(r#"(?s)^;;((.*?);;)"#).unwrap()
        },
        Rule {
            typ: TokenKind::Assign,
            regex: Regex::new(r#"^(->|\=\:)"#).unwrap()
        },
        Rule {
            typ: TokenKind::Operator,
            regex: Regex::new(r#"^(\*\*|[-+/*%]|<<|>>)"#).unwrap()
        },
        Rule {
            typ: TokenKind::CurlyBracket,
            regex: Regex::new(r#"^(\{|\})"#).unwrap()
        },
        Rule {
            typ: TokenKind::Bracket,
            regex: Regex::new(r#"^(\[|\])"#).unwrap()
        },
        Rule {
            typ: TokenKind::SpecialSymbol,
            regex: Regex::new(r#"^(\#\!|\!\#)"#).unwrap()
        },
        Rule {
            typ: TokenKind::Logical,
            regex: Regex::new(r#"^(=|<=|>=|>|<|!=|\&\&|\|\||!|and|or|not)"#).unwrap()
        },
        Rule {
            typ: TokenKind::SpecialSymbol,
            regex: Regex::new(r#"^(\&)"#).unwrap()
        },
        Rule {
            typ: TokenKind::Keyword,
            regex: Regex::new(r#"^([A-Za-zА-Яа-я_:!]+\d*)"#).unwrap()
        }
    ]
}