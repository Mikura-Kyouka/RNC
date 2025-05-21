use crate::ast::*;
use crate::lexer::lex;
use crate::lexer::Token;
use lalrpop_util::ParseError;
use lalrpop_util::lalrpop_mod;

// 使用LALRPOP生成的解析器
lalrpop_mod!(pub grammar);

pub fn parse_source(source: &str) -> Result<Program, String> {
    // 词法分析
    let tokens = match lex(source) {
        Ok(tokens) => tokens,
        Err(e) => return Err(format!("Lexical error: {}", e)),
    };
    
    // 将TokenInfo转换为LALRPOP期望的(usize, Token, usize)格式
    let token_triples: Vec<(usize, Token, usize)> = tokens.into_iter()
        .map(|token_info| (token_info.span.start, token_info.token, token_info.span.end))
        .collect();

    // 语法分析
    match grammar::ProgramParser::new().parse(source, token_triples) {
        Ok(ast) => Ok(ast),
        Err(e) => {
            match e {
                ParseError::InvalidToken { location } => {
                    let (line, column) = get_line_column(source, location);
                    Err(format!("Invalid token at line {}, column {}", line, column))
                }
                ParseError::UnrecognizedEof { location, expected } => {
                    let (line, column) = get_line_column(source, location);
                    Err(format!(
                        "Unexpected end of file at line {}, column {}, expected: {}",
                        line, column, format_expected(&expected)
                    ))
                }
                ParseError::UnrecognizedToken { token, expected } => {
                    let (line, column) = get_line_column(source, token.0);
                    Err(format!(
                        "Unexpected token '{:?}' at line {}, column {}, expected: {}",
                        token.1, line, column, format_expected(&expected)
                    ))
                }
                ParseError::ExtraToken { token } => {
                    let (line, column) = get_line_column(source, token.0);
                    Err(format!("Extra token '{:?}' at line {}, column {}", token.1, line, column))
                }
                ParseError::User { error } => Err(format!("Error: {}", error)),
            }
        }
    }
}

// 获取给定位置的行号和列号
fn get_line_column(source: &str, position: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    
    for (i, c) in source.chars().enumerate() {
        if i >= position {
            break;
        }
        
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    
    (line, column)
}

// 格式化预期的标记列表，使其更易读
fn format_expected(expected: &[String]) -> String {
    if expected.is_empty() {
        return "no tokens".to_string();
    }
    
    // 简化长列表并美化输出
    let formatted: Vec<String> = expected.iter()
        .map(|s| s.trim_matches('"').to_string())
        .collect();
    
    if formatted.len() > 5 {
        format!("{} or ... (and {} more)", 
                formatted[..5].join(", "), 
                formatted.len() - 5)
    } else {
        formatted.join(", ")
    }
}

pub fn parse(source: &str, tokens: Vec<(usize, Token, usize)>) -> Result<Program, String> {
    match grammar::ProgramParser::new().parse(source, tokens) {
        Ok(ast) => Ok(ast),
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}
