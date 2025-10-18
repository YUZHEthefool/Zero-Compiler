mod lexer;
mod parser;
mod ast;
mod interpreter;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <source_file.zero>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    run(&source);
}

fn run(source: &str) {
    // 词法分析
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize();

    // 语法分析
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(prog) => prog,
        Err(err) => {
            eprintln!("Parse error: {:?}", err);
            process::exit(1);
        }
    };

    // 解释执行
    let mut interpreter = Interpreter::new();
    if let Err(err) = interpreter.interpret(program) {
        eprintln!("Runtime error: {:?}", err);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let source = r#"
            let x = 10;
            let y = 20;
            print(x + y);
        "#;
        run(source);
    }

    #[test]
    fn test_function() {
        let source = r#"
            fn add(a, b) {
                return a + b;
            }
            
            let result = add(5, 3);
            print(result);
        "#;
        run(source);
    }
}
