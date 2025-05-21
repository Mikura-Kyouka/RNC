use std::fs;
use std::env;
use std::io::{self, Read};
use std::path::Path;

mod ast;
mod lexer;
mod parser;
mod semantic;

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
    
    // 使用我们的解析器
    match parser::parse_source(&source_code) {
        Ok(program) => {
            println!("解析成功！程序结构：{:?}", program);
        },
        Err(error) => {
            eprintln!("错误: {}", error);
        }
    }
    
    Ok(())
}
