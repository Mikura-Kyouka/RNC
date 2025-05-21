// src/lexer.rs
use logos::Logos;
use std::ops::Range;

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
    
    #[regex(r"\{[^}]*\}", logos::skip)]
    
    #[regex(r"[ \t\n\r]+", logos::skip)]
    Error,
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub span: Range<usize>,
    pub line: usize,
    pub column: usize,
}

pub fn lex(input: &str) -> Result<Vec<TokenInfo>, String> {
    let mut tokens = Vec::new();
    let mut line_starts = vec![0];
    for (i, c) in input.char_indices() {
        if c == '\n' {
            line_starts.push(i + 1);
        }
    }

    for (token_result, range) in Token::lexer(input).spanned() {
        match token_result {
            Ok(token) => {
                // 计算行号和列号
                let (line, column) = {
                    let mut line = 1;
                    for (idx, &start) in line_starts.iter().enumerate() {
                        if start > range.start {
                            break;
                        }
                        line = idx + 1;
                    }
                    let line_start = line_starts.get(line - 1).copied().unwrap_or(0);
                    (line, range.start - line_start + 1)
                };
                tokens.push(TokenInfo {
                    token,
                    span: range,
                    line,
                    column,
                });
            }
            Err(_) => {
                // 计算错误位置
                let (line, column) = {
                    let mut line = 1;
                    for (idx, &start) in line_starts.iter().enumerate() {
                        if start > range.start {
                            break;
                        }
                        line = idx + 1;
                    }
                    let line_start = line_starts.get(line - 1).copied().unwrap_or(0);
                    (line, range.start - line_start + 1)
                };
                return Err(format!(
                    "词法分析错误: 第{}行第{}列，无法解析范围 {:?} 的内容",
                    line, column, range
                ));
            }
        }
    }
    Ok(tokens)
}
