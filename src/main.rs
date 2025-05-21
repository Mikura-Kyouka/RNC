use std::fs;
use std::env;
use std::io::{self, Read, Write};
use std::path::Path;
use crate::semantic_analyzer::SemanticAnalyzer;

mod ast;
mod lexer;
mod parser;
mod semantic;
mod semantic_analyzer;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    let source_code = if args.len() > 1 {
        // 从文件读取
        let path = Path::new(&args[1]);
        fs::read_to_string(path)?
    } else {
        // 从标准输入读取
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    // 词法分析并输出token序列
    match lexer::lex(&source_code) {
        Ok(tokens) => {
            // 输出token序列到文件
            let mut token_file = fs::File::create("tokens.txt")?;
            for token in &tokens {
                writeln!(token_file, "{:?}", token)?;
            }
            // 继续语法分析
            match parser::parse_source(&source_code) {
                Ok(program) => {
                    println!("解析成功！程序结构：{:?}", program);
                    // 输出AST到文件
                    let mut ast_file = fs::File::create("ast.txt")?;
                    writeln!(ast_file, "{:#?}", program)?;

                    // 语义分析
                    let mut semantic_analyzer = SemanticAnalyzer::new();
                    match semantic_analyzer.analyze(&program) {
                        Ok(_) => println!("语义分析通过！"),
                        Err(errors) => {
                            println!("语义分析发现错误:");
                            for error in errors {
                                println!("  - {}", error);
                            }
                        }
                    }
                },
                Err(error) => {
                    eprintln!("错误: {}", error);
                }
            }
        }
        Err(e) => {
            eprintln!("词法分析错误: {}", e);
        }
    }
    Ok(())
}
