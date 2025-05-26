use std::fs;
use std::env;
use std::io::{self, Read, Write};
use std::path::Path;
use crate::semantic_analyzer::SemanticAnalyzer;

mod ast;
mod lexer;
mod parser;
mod semantic_analyzer;
mod code_gen;

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
            println!("词法分析通过！");
            let mut token_file = fs::File::create("tokens.txt")?;
            for token in &tokens {
                writeln!(token_file, "{:?}", token)?;
            }
            // 继续语法分析
            match parser::parse_source(&source_code) {
                Ok(program) => {
                    println!("语法分析通过！");
                    // 输出AST到文件
                    let mut ast_file = fs::File::create("ast.txt")?;
                    writeln!(ast_file, "{:#?}", program)?;

                    // 语义分析
                    let mut semantic_analyzer = SemanticAnalyzer::new();
                    match semantic_analyzer.analyze(&program) {
                        Ok(_) => {
                            println!("语义分析通过！");
                            
                            let mut code_generator = code_gen::LoongArch32Reduce::new(semantic_analyzer.symbol_table.clone());
                            let assembly_code = code_generator.generate_code(&program);

                            let mut asm_file = fs::File::create("output.S")?;
                            writeln!(asm_file, "{}", assembly_code)?;
                            println!("汇编代码已输出到 output.S");
                        }
                        Err(errors) => {
                            println!("语义分析错误:");
                            for error in errors {
                                println!("  - {}", error);
                            }
                        }
                    }
                },
                Err(error) => {
                    eprintln!("语法分析错误: {}", error);
                }
            }
        }
        Err(e) => {
            eprintln!("词法分析错误: {}", e);
        }
    }
    Ok(())
}
