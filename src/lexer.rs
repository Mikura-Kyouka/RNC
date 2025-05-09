// src/lexer.rs
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[token("program", ignore(ascii_case))] Program,
    #[token("type", ignore(ascii_case))] Type,
    #[token("var", ignore(ascii_case))] Var,
    #[token("procedure", ignore(ascii_case))] Procedure,
    #[token("begin", ignore(ascii_case))] Begin,
    #[token("end", ignore(ascii_case))] End,
    #[token("integer", ignore(ascii_case))] Integer,
    #[token("char", ignore(ascii_case))] Char,
    #[token("array", ignore(ascii_case))] Array,
    #[token("of", ignore(ascii_case))] Of,
    #[token("record", ignore(ascii_case))] Record,
    #[token("if", ignore(ascii_case))] If,
    #[token("then", ignore(ascii_case))] Then,
    #[token("else", ignore(ascii_case))] Else,
    #[token("fi", ignore(ascii_case))] Fi,
    #[token("while", ignore(ascii_case))] While,
    #[token("do", ignore(ascii_case))] Do,
    #[token("endwh", ignore(ascii_case))] EndWh,
    #[token("read", ignore(ascii_case))] Read,
    #[token("write", ignore(ascii_case))] Write,
    #[token("return", ignore(ascii_case))] Return,

    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token(";")] Semicolon,
    #[token(":=")] Assign,
    #[token(".")] Dot,
    #[token(",")] Comma,
    #[token(":")] Colon,
    #[token("=")] Equal,
    #[token("<")] Less,
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Times,
    #[token("/")] Div,
    #[token(".." )] Range,

    #[regex("[0-9]+", |lex| lex.slice().parse().ok())] Int(i32),
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())] Ident(String),

    #[regex(r"[ \t\n\r]+", logos::skip)]
    Error,
}

pub fn lex(input: &str) -> Vec<(Token, std::ops::Range<usize>)> {
    Token::lexer(input)
        .spanned()
        .filter_map(|(token_result, range)| token_result.ok().map(|token| (token, range)))
        .collect()
}
