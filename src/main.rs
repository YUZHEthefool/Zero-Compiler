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
use bytecode::serializer::{BytecodeSerializer, BytecodeDeserializer};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <source_file.zero>", args[0]);
        eprintln!("       {} --old <source_file.zero>  (use old interpreter)", args[0]);
        eprintln!("       {} --compile <source_file.zero> <output.zbc>  (compile to bytecode)", args[0]);
        eprintln!("       {} --run <bytecode_file.zbc>  (run bytecode file)", args[0]);
        process::exit(1);
    }

    match args[1].as_str() {
        "--old" => {
            if args.len() < 3 {
                eprintln!("Usage: {} --old <source_file.zero>", args[0]);
                process::exit(1);
            }
            let source = read_source_file(&args[2]);
            println!("Using old tree-walking interpreter...");
            run_old(&source);
        }
        "--compile" => {
            if args.len() < 4 {
                eprintln!("Usage: {} --compile <source_file.zero> <output.zbc>", args[0]);
                process::exit(1);
            }
            let source = read_source_file(&args[2]);
            compile_to_bytecode(&source, &args[3]);
        }
        "--run" => {
            if args.len() < 3 {
                eprintln!("Usage: {} --run <bytecode_file.zbc>", args[0]);
                process::exit(1);
            }
            run_bytecode_file(&args[2]);
        }
        _ => {
            let source = read_source_file(&args[1]);
            println!("Using bytecode compiler + VM...");
            run(&source);
        }
    }
}

fn read_source_file(filename: &str) -> String {
    match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    }
}

/// 编译源代码到字节码文件
fn compile_to_bytecode(source: &str, output_file: &str) {
    println!("Compiling {} to {}...", "source", output_file);
    
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

    // 序列化并保存
    let file = match File::create(output_file) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error creating output file: {}", err);
            process::exit(1);
        }
    };

    let mut writer = BufWriter::new(file);
    if let Err(err) = BytecodeSerializer::serialize(&chunk, &mut writer) {
        eprintln!("Error serializing bytecode: {}", err);
        process::exit(1);
    }

    println!("Successfully compiled to {}", output_file);
}

/// 从字节码文件运行
fn run_bytecode_file(filename: &str) {
    println!("Loading bytecode from {}...", filename);
    
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error opening bytecode file: {}", err);
            process::exit(1);
        }
    };

    let mut reader = BufReader::new(file);
    let chunk = match BytecodeDeserializer::deserialize(&mut reader) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error deserializing bytecode: {}", err);
            process::exit(1);
        }
    };

    println!("Running bytecode...");
    
    // 调试：打印反汇编代码
    if env::var("ZERO_DEBUG").is_ok() {
        chunk.disassemble("loaded");
    }

    // VM执行
    let mut vm = VM::new();
    if let Err(err) = vm.execute(chunk) {
        eprintln!("Runtime error: {:?}", err);
        process::exit(1);
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
