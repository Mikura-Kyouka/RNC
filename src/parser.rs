use std::rc::Rc;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar); // assumes your LALRPOP file is grammar.lalrpop

use crate::lexer::{Token, lex};
use crate::ast::*; // assumes this file is your AST definition
use grammar::ProgramParser;

pub fn parse(source: &str) -> Result<Program, String> {
    let tokens = lex(source);
    let parser = ProgramParser::new();

    match parser.parse(source, &tokens) {
        Ok(program) => Ok(program),
        Err(err) => Err(format!("Parse error: {:?}", err)),
    }
}