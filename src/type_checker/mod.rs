use crate::ast::{Expr, Program, Stmt, BinaryOp, UnaryOp, Type, Parameter, FunctionType};
use std::collections::HashMap;

/// 类型检查错误
#[derive(Debug)]
pub enum TypeError {
    TypeMismatch {
        expected: Type,
        found: Type,
        location: String,
    },
    UndefinedVariable(String),
    UndefinedFunction(String),
    ArgumentCountMismatch {
        expected: usize,
        found: usize,
        function: String,
    },
    ArgumentTypeMismatch {
        expected: Type,
        found: Type,
        argument: usize,
        function: String,
    },
    ReturnTypeMismatch {
        expected: Type,
        found: Type,
        function: String,
    },
    CannotInferType(String),
    InvalidOperation {
        operator: String,
        left_type: Type,
        right_type: Type,
    },
}

type TypeResult<T> = Result<T, TypeError>;

/// 符号表条目
#[derive(Debug, Clone)]
struct Symbol {
    symbol_type: Type,
    is_mutable: bool,
}

/// 符号表（支持作用域）
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, symbol_type: Type, is_mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, Symbol { symbol_type, is_mutable });
        }
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }
}

/// 类型检查器
pub struct TypeChecker {
    symbol_table: SymbolTable,
    current_function_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbol_table: SymbolTable::new(),
            current_function_return_type: None,
        }
    }

    /// 检查程序
    pub fn check(&mut self, program: &Program) -> TypeResult<()> {
        for stmt in &program.statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    /// 检查语句
    fn check_statement(&mut self, stmt: &Stmt) -> TypeResult<()> {
        match stmt {
            Stmt::Expression(expr) => {
                self.infer_type(expr)?;
                Ok(())
            }

            Stmt::VarDeclaration {
                name,
                mutable,
                type_annotation,
                initializer,
            } => {
                let actual_type = if let Some(init) = initializer {
                    self.infer_type(init)?
                } else {
                    Type::Null
                };

                let var_type = if let Some(annotated_type) = type_annotation {
                    // 检查类型注解和初始化值是否匹配
                    if let Some(_init) = initializer {
                        if !annotated_type.is_compatible_with(&actual_type) && actual_type != Type::Unknown {
                            return Err(TypeError::TypeMismatch {
                                expected: annotated_type.clone(),
                                found: actual_type,
                                location: format!("variable declaration '{}'", name),
                            });
                        }
                    }
                    annotated_type.clone()
                } else {
                    // 类型推导 - 如果无法推导则使用Unknown
                    actual_type
                };

                self.symbol_table.define(name.clone(), var_type, *mutable);
                Ok(())
            }

            Stmt::FnDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => {
                // 构建函数类型
                let param_types: Vec<Type> = parameters
                    .iter()
                    .map(|p| p.type_annotation.clone().unwrap_or(Type::Unknown))
                    .collect();

                let ret_type = return_type.clone().unwrap_or(Type::Unknown);

                let function_type = Type::Function(FunctionType {
                    params: param_types.clone(),
                    return_type: Box::new(ret_type.clone()),
                });

                // 注册函数
                self.symbol_table.define(name.clone(), function_type, false);

                // 检查函数体
                self.symbol_table.push_scope();
                self.current_function_return_type = Some(ret_type);

                // 添加参数到作用域
                for param in parameters {
                    let param_type = param.type_annotation.clone().unwrap_or(Type::Unknown);
                    self.symbol_table.define(param.name.clone(), param_type, false);
                }

                // 检查函数体语句
                for stmt in body {
                    self.check_statement(stmt)?;
                }

                self.current_function_return_type = None;
                self.symbol_table.pop_scope();
                Ok(())
            }

            Stmt::Return { value } => {
                let return_type = if let Some(expr) = value {
                    self.infer_type(expr)?
                } else {
                    Type::Void
                };

                if let Some(expected_type) = &self.current_function_return_type {
                    if expected_type != &Type::Unknown
                        && return_type != Type::Unknown
                        && !expected_type.is_compatible_with(&return_type) {
                        return Err(TypeError::ReturnTypeMismatch {
                            expected: expected_type.clone(),
                            found: return_type,
                            function: "current function".to_string(),
                        });
                    }
                }

                Ok(())
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.infer_type(condition)?;
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    return Err(TypeError::TypeMismatch {
                        expected: Type::Bool,
                        found: cond_type,
                        location: "if condition".to_string(),
                    });
                }

                self.symbol_table.push_scope();
                for stmt in then_branch {
                    self.check_statement(stmt)?;
                }
                self.symbol_table.pop_scope();

                if let Some(else_stmts) = else_branch {
                    self.symbol_table.push_scope();
                    for stmt in else_stmts {
                        self.check_statement(stmt)?;
                    }
                    self.symbol_table.pop_scope();
                }

                Ok(())
            }

            Stmt::While { condition, body } => {
                let cond_type = self.infer_type(condition)?;
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    return Err(TypeError::TypeMismatch {
                        expected: Type::Bool,
                        found: cond_type,
                        location: "while condition".to_string(),
                    });
                }

                self.symbol_table.push_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.symbol_table.pop_scope();

                Ok(())
            }

            Stmt::For {
                variable,
                start,
                end,
                body,
            } => {
                let start_type = self.infer_type(start)?;
                let end_type = self.infer_type(end)?;

                if start_type != Type::Int && start_type != Type::Unknown {
                    return Err(TypeError::TypeMismatch {
                        expected: Type::Int,
                        found: start_type,
                        location: "for loop start".to_string(),
                    });
                }

                if end_type != Type::Int && end_type != Type::Unknown {
                    return Err(TypeError::TypeMismatch {
                        expected: Type::Int,
                        found: end_type,
                        location: "for loop end".to_string(),
                    });
                }

                self.symbol_table.push_scope();
                self.symbol_table.define(variable.clone(), Type::Int, true);

                for stmt in body {
                    self.check_statement(stmt)?;
                }

                self.symbol_table.pop_scope();
                Ok(())
            }

            Stmt::Print { value } => {
                self.infer_type(value)?;
                Ok(())
            }

            Stmt::Block { statements } => {
                self.symbol_table.push_scope();
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                self.symbol_table.pop_scope();
                Ok(())
            }
        }
    }

    /// 推断表达式类型
    fn infer_type(&mut self, expr: &Expr) -> TypeResult<Type> {
        match expr {
            Expr::Integer(_) => Ok(Type::Int),
            Expr::Float(_) => Ok(Type::Float),
            Expr::String(_) => Ok(Type::String),
            Expr::Boolean(_) => Ok(Type::Bool),

            Expr::Identifier(name) => {
                if let Some(symbol) = self.symbol_table.get(name) {
                    Ok(symbol.symbol_type.clone())
                } else {
                    Err(TypeError::UndefinedVariable(name.clone()))
                }
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_type = self.infer_type(left)?;
                let right_type = self.infer_type(right)?;

                match operator {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        // 允许Unknown类型参与运算
                        if left_type == Type::Unknown || right_type == Type::Unknown {
                            Ok(Type::Unknown)
                        } else if left_type.is_numeric() && right_type.is_numeric() {
                            // 如果有一个是float，结果是float
                            if left_type == Type::Float || right_type == Type::Float {
                                Ok(Type::Float)
                            } else {
                                Ok(Type::Int)
                            }
                        } else if operator == &BinaryOp::Add
                            && left_type == Type::String
                            && right_type == Type::String
                        {
                            Ok(Type::String)
                        } else {
                            Err(TypeError::InvalidOperation {
                                operator: format!("{:?}", operator),
                                left_type,
                                right_type,
                            })
                        }
                    }

                    BinaryOp::Modulo => {
                        if left_type == Type::Unknown || right_type == Type::Unknown {
                            Ok(Type::Unknown)
                        } else if left_type == Type::Int && right_type == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(TypeError::InvalidOperation {
                                operator: "modulo".to_string(),
                                left_type,
                                right_type,
                            })
                        }
                    }

                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual => Ok(Type::Bool),

                    BinaryOp::And | BinaryOp::Or => {
                        if left_type == Type::Unknown || right_type == Type::Unknown {
                            Ok(Type::Unknown)
                        } else if left_type == Type::Bool && right_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(TypeError::InvalidOperation {
                                operator: format!("{:?}", operator),
                                left_type,
                                right_type,
                            })
                        }
                    }
                }
            }

            Expr::Unary { operator, operand } => {
                let operand_type = self.infer_type(operand)?;

                match operator {
                    UnaryOp::Not => {
                        if operand_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(TypeError::TypeMismatch {
                                expected: Type::Bool,
                                found: operand_type,
                                location: "unary not operator".to_string(),
                            })
                        }
                    }
                    UnaryOp::Negate => {
                        if operand_type.is_numeric() {
                            Ok(operand_type)
                        } else {
                            Err(TypeError::TypeMismatch {
                                expected: Type::Int,
                                found: operand_type,
                                location: "unary negate operator".to_string(),
                            })
                        }
                    }
                }
            }

            Expr::Assign { name, value } => {
                let value_type = self.infer_type(value)?;

                if let Some(symbol) = self.symbol_table.get(name) {
                    // 只有当类型都不是Unknown时才检查类型兼容性
                    if symbol.symbol_type != Type::Unknown
                        && value_type != Type::Unknown
                        && !symbol.symbol_type.is_compatible_with(&value_type) {
                        return Err(TypeError::TypeMismatch {
                            expected: symbol.symbol_type.clone(),
                            found: value_type,
                            location: format!("assignment to variable '{}'", name),
                        });
                    }

                    Ok(value_type)
                } else {
                    Err(TypeError::UndefinedVariable(name.clone()))
                }
            }

            Expr::Call { callee, arguments } => {
                // 获取被调用函数的类型
                if let Expr::Identifier(func_name) = callee.as_ref() {
                    if let Some(symbol) = self.symbol_table.get(func_name) {
                        if let Type::Function(func_type) = &symbol.symbol_type {
                            // 检查参数数量
                            if func_type.params.len() != arguments.len() {
                                return Err(TypeError::ArgumentCountMismatch {
                                    expected: func_type.params.len(),
                                    found: arguments.len(),
                                    function: func_name.clone(),
                                });
                            }

                            // 克隆函数类型以避免借用冲突
                            let params = func_type.params.clone();
                            let return_type = *func_type.return_type.clone();

                            // 检查每个参数的类型
                            for (i, (param_type, arg)) in
                                params.iter().zip(arguments.iter()).enumerate()
                            {
                                let arg_type = self.infer_type(arg)?;
                                if !param_type.is_compatible_with(&arg_type) {
                                    return Err(TypeError::ArgumentTypeMismatch {
                                        expected: param_type.clone(),
                                        found: arg_type,
                                        argument: i + 1,
                                        function: func_name.clone(),
                                    });
                                }
                            }

                            // 返回函数的返回类型
                            Ok(return_type)
                        } else {
                            Err(TypeError::TypeMismatch {
                                expected: Type::Function(FunctionType {
                                    params: vec![],
                                    return_type: Box::new(Type::Unknown),
                                }),
                                found: symbol.symbol_type.clone(),
                                location: format!("function call '{}'", func_name),
                            })
                        }
                    } else {
                        Err(TypeError::UndefinedFunction(func_name.clone()))
                    }
                } else {
                    // 对于非标识符调用（如高阶函数），返回Unknown
                    Ok(Type::Unknown)
                }
            }

            Expr::Array { elements } => {
                if elements.is_empty() {
                    // 空数组需要类型注解，这里返回Unknown
                    Ok(Type::Unknown)
                } else {
                    // 推断数组元素类型（所有元素必须同类型）
                    let first_type = self.infer_type(&elements[0])?;
                    
                    for elem in elements.iter().skip(1) {
                        let elem_type = self.infer_type(elem)?;
                        // 数组要求严格的类型匹配，不允许类型自动转换
                        if first_type != elem_type && elem_type != Type::Unknown && first_type != Type::Unknown {
                            return Err(TypeError::TypeMismatch {
                                expected: first_type,
                                found: elem_type,
                                location: "array literal".to_string(),
                            });
                        }
                    }
                    
                    Ok(Type::Array(Box::new(first_type)))
                }
            }

            Expr::Index { object, index } => {
                let obj_type = self.infer_type(object)?;
                let idx_type = self.infer_type(index)?;
                
                // 索引必须是整数
                if idx_type != Type::Int && idx_type != Type::Unknown {
                    return Err(TypeError::TypeMismatch {
                        expected: Type::Int,
                        found: idx_type,
                        location: "array index".to_string(),
                    });
                }
                
                // 返回数组元素类型
                if let Some(element_type) = obj_type.get_element_type() {
                    Ok(element_type.clone())
                } else {
                    Ok(Type::Unknown)
                }
            }
            
            Expr::IndexAssign { object, index, value } => {
                let obj_type = self.infer_type(object)?;
                let idx_type = self.infer_type(index)?;
                let val_type = self.infer_type(value)?;
                
                // 索引必须是整数
                if idx_type != Type::Int && idx_type != Type::Unknown {
                    return Err(TypeError::TypeMismatch {
                        expected: Type::Int,
                        found: idx_type,
                        location: "array index".to_string(),
                    });
                }
                
                // 值类型必须与数组元素类型兼容
                if let Some(element_type) = obj_type.get_element_type() {
                    if !element_type.is_compatible_with(&val_type) && val_type != Type::Unknown {
                        return Err(TypeError::TypeMismatch {
                            expected: element_type.clone(),
                            found: val_type,
                            location: "array element assignment".to_string(),
                        });
                    }
                }
                
                Ok(val_type)
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_type_check_variable() {
        let input = "let x: int = 42;";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&program).is_ok());
    }

    #[test]
    fn test_type_check_type_mismatch() {
        let input = "let x: int = \"hello\";";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&program).is_err());
    }

    #[test]
    fn test_type_check_function() {
        let input = "fn add(a: int, b: int) -> int { return a + b; }";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&program).is_ok());
    }
}