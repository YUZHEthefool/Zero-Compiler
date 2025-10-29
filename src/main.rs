mod lexer;
mod parser;
mod ast;
mod bytecode;
mod compiler;
mod vm;
mod type_checker;

// 保留旧的解释器用于对比
mod interpreter;

use lexer::Lexer;
use parser::Parser;
use compiler::Compiler;
use vm::VM;
use type_checker::TypeChecker;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <source_file.zero>", args[0]);
        eprintln!("       {} --old <source_file.zero>  (use old interpreter)", args[0]);
        process::exit(1);
    }

    let use_old = args.len() > 2 && args[1] == "--old";
    let filename = if use_old { &args[2] } else { &args[1] };

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    if use_old {
        println!("Using old tree-walking interpreter...");
        run_old(&source);
    } else {
        println!("Using bytecode compiler + VM...");
        run(&source);
    }
}

/// 新的字节码编译器 + VM执行
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

    // 类型检查
    let mut type_checker = TypeChecker::new();
    if let Err(err) = type_checker.check(&program) {
        eprintln!("Type error: {:?}", err);
        process::exit(1);
    }

    // 编译为字节码
    let mut compiler = Compiler::new();
    let chunk = match compiler.compile(program) {
        Ok(chunk) => chunk,
        Err(err) => {
            eprintln!("Compile error: {:?}", err);
            process::exit(1);
        }
    };

    // 调试：打印反汇编代码
    if env::var("ZERO_DEBUG").is_ok() {
        chunk.disassemble("main");
    }

    // VM执行
    let mut vm = VM::new();
    if let Err(err) = vm.execute(chunk) {
        eprintln!("Runtime error: {:?}", err);
        process::exit(1);
    }
}

/// 旧的树遍历解释器（用于对比）
fn run_old(source: &str) {
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
    let mut interpreter = interpreter::Interpreter::new();
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

    #[test]
    fn test_bytecode_vs_interpreter() {
        let source = r#"
            let x = 100;
            let y = 200;
            print(x + y);
        "#;
        
        println!("\n=== Bytecode VM ===");
        run(source);
        
        println!("\n=== Old Interpreter ===");
        run_old(source);
    }

    #[test]
    fn test_control_flow() {
        let source = r#"
            let x = 15;
            if x > 10 {
                print(x);
            }
            
            let i = 0;
            while i < 3 {
                print(i);
                i = i + 1;
            }
        "#;
        run(source);
    }

    #[test]
    fn test_functions() {
        let source = r#"
            fn multiply(a, b) {
                return a * b;
            }
            
            fn factorial(n) {
                if n <= 1 {
                    return 1;
                }
                return n * factorial(n - 1);
            }
            
            print(multiply(6, 7));
            print(factorial(5));
        "#;
        run(source);
    }

    #[test]
    fn test_type_annotations() {
        let source = r#"
            let x: int = 42;
            let y: float = 3.14;
            let s: string = "hello";
            let b: bool = true;
            print(x);
            print(y);
            print(s);
            print(b);
        "#;
        run(source);
    }

    #[test]
    fn test_typed_function() {
        let source = r#"
            fn add(a: int, b: int) {
                return a + b;
            }
            
            let result = add(10, 20);
            print(result);
        "#;
        run(source);
    }

    #[test]
    fn test_mixed_type_annotations() {
        let source = r#"
            fn multiply(a, b: int) {
                return a * b;
            }
            
            let x = 5;
            let result = multiply(x, 10);
            print(result);
        "#;
        run(source);
    }

}
