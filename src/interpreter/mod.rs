use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp, Parameter};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Function {
        parameters: Vec<Parameter>,
        body: Vec<Stmt>,
    },
    Null,
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Function { .. } => "<function>".to_string(),
            Value::Null => "null".to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeMismatch(String),
    DivisionByZero,
    InvalidOperation(String),
    ReturnValue(Value),
}

type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn get(&self, name: &str) -> RuntimeResult<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }

    pub fn set(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, program: Program) -> RuntimeResult<()> {
        for stmt in program.statements {
            self.execute_statement(&stmt)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: &Stmt) -> RuntimeResult<Value> {
        match stmt {
            Stmt::Expression(expr) => self.evaluate_expression(expr),

            Stmt::VarDeclaration {
                name,
                mutable: _,
                type_annotation: _,
                initializer,
            } => {
                let value = if let Some(init) = initializer {
                    self.evaluate_expression(init)?
                } else {
                    Value::Null
                };
                self.environment.define(name.clone(), value);
                Ok(Value::Null)
            }

            Stmt::FnDeclaration {
                name,
                parameters,
                return_type: _,
                body,
            } => {
                let func = Value::Function {
                    parameters: parameters.clone(),
                    body: body.clone(),
                };
                self.environment.define(name.clone(), func);
                Ok(Value::Null)
            }

            Stmt::Return { value } => {
                let return_value = if let Some(expr) = value {
                    self.evaluate_expression(expr)?
                } else {
                    Value::Null
                };
                Err(RuntimeError::ReturnValue(return_value))
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = self.evaluate_expression(condition)?;

                if condition_value.is_truthy() {
                    for stmt in then_branch {
                        self.execute_statement(stmt)?;
                    }
                } else if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(Value::Null)
            }

            Stmt::While { condition, body } => {
                while self.evaluate_expression(condition)?.is_truthy() {
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(Value::Null)
            }

            Stmt::For {
                variable,
                start,
                end,
                body,
            } => {
                let start_val = self.evaluate_expression(start)?;
                let end_val = self.evaluate_expression(end)?;

                if let (Value::Integer(start_i), Value::Integer(end_i)) = (start_val, end_val) {
                    self.environment.push_scope();

                    for i in start_i..end_i {
                        self.environment
                            .define(variable.clone(), Value::Integer(i));

                        for stmt in body {
                            self.execute_statement(stmt)?;
                        }
                    }

                    self.environment.pop_scope();
                    Ok(Value::Null)
                } else {
                    Err(RuntimeError::TypeMismatch(
                        "For loop requires integer range".to_string(),
                    ))
                }
            }

            Stmt::Print { value } => {
                let result = self.evaluate_expression(value)?;
                println!("{}", result.to_string());
                Ok(Value::Null)
            }

            Stmt::Block { statements } => {
                self.environment.push_scope();

                for stmt in statements {
                    self.execute_statement(stmt)?;
                }

                self.environment.pop_scope();
                Ok(Value::Null)
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        match expr {
            Expr::Integer(i) => Ok(Value::Integer(*i)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Identifier(name) => self.environment.get(name),

            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left, operator, right),

            Expr::Unary { operator, operand } => self.evaluate_unary(operator, operand),

            Expr::Call { callee, arguments } => self.evaluate_call(callee, arguments),

            Expr::Assign { name, value } => {
                let val = self.evaluate_expression(value)?;
                self.environment.set(name, val.clone())?;
                Ok(val)
            }

            Expr::Array { elements } => {
                // 数组字面量 - 暂时返回占位值
                // TODO: 实现完整的数组支持
                Ok(Value::String(format!("Array[{}]", elements.len())))
            }

            Expr::Index { object, index } => {
                // 数组索引 - 暂时返回占位值
                // TODO: 实现完整的数组索引支持
                Err(RuntimeError::InvalidOperation(
                    "Array indexing not yet implemented".to_string(),
                ))
            }
            
            Expr::IndexAssign { object, index, value } => {
                // 数组索引赋值 - 暂时返回占位值
                // TODO: 实现完整的数组索引赋值支持
                let val = self.evaluate_expression(value)?;
                Ok(val)
            }
        }
    }

    fn evaluate_binary(
        &mut self,
        left: &Expr,
        operator: &BinaryOp,
        right: &Expr,
    ) -> RuntimeResult<Value> {
        let left_val = self.evaluate_expression(left)?;
        let right_val = self.evaluate_expression(right)?;

        match operator {
            BinaryOp::Add => match (left_val, right_val) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
                (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 + r)),
                (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l + r as f64)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                _ => Err(RuntimeError::TypeMismatch("Invalid addition".to_string())),
            },

            BinaryOp::Subtract => match (left_val, right_val) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
                (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 - r)),
                (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l - r as f64)),
                _ => Err(RuntimeError::TypeMismatch(
                    "Invalid subtraction".to_string(),
                )),
            },

            BinaryOp::Multiply => match (left_val, right_val) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
                (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 * r)),
                (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l * r as f64)),
                _ => Err(RuntimeError::TypeMismatch(
                    "Invalid multiplication".to_string(),
                )),
            },

            BinaryOp::Divide => match (left_val, right_val) {
                (Value::Integer(l), Value::Integer(r)) => {
                    if r == 0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Integer(l / r))
                    }
                }
                (Value::Float(l), Value::Float(r)) => {
                    if r == 0.0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Float(l / r))
                    }
                }
                (Value::Integer(l), Value::Float(r)) => {
                    if r == 0.0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Float(l as f64 / r))
                    }
                }
                (Value::Float(l), Value::Integer(r)) => {
                    if r == 0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Float(l / r as f64))
                    }
                }
                _ => Err(RuntimeError::TypeMismatch("Invalid division".to_string())),
            },

            BinaryOp::Modulo => match (left_val, right_val) {
                (Value::Integer(l), Value::Integer(r)) => {
                    if r == 0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Integer(l % r))
                    }
                }
                _ => Err(RuntimeError::TypeMismatch("Invalid modulo".to_string())),
            },

            BinaryOp::Equal => Ok(Value::Boolean(self.values_equal(&left_val, &right_val))),
            BinaryOp::NotEqual => Ok(Value::Boolean(!self.values_equal(&left_val, &right_val))),

            BinaryOp::Less => match (left_val, right_val) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l < r)),
                _ => Err(RuntimeError::TypeMismatch("Invalid comparison".to_string())),
            },

            BinaryOp::LessEqual => match (left_val, right_val) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l <= r)),
                _ => Err(RuntimeError::TypeMismatch("Invalid comparison".to_string())),
            },

            BinaryOp::Greater => match (left_val, right_val) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l > r)),
                _ => Err(RuntimeError::TypeMismatch("Invalid comparison".to_string())),
            },

            BinaryOp::GreaterEqual => match (left_val, right_val) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l >= r)),
                _ => Err(RuntimeError::TypeMismatch("Invalid comparison".to_string())),
            },

            BinaryOp::And => Ok(Value::Boolean(left_val.is_truthy() && right_val.is_truthy())),
            BinaryOp::Or => Ok(Value::Boolean(left_val.is_truthy() || right_val.is_truthy())),
        }
    }

    fn evaluate_unary(&mut self, operator: &UnaryOp, operand: &Expr) -> RuntimeResult<Value> {
        let value = self.evaluate_expression(operand)?;

        match operator {
            UnaryOp::Not => Ok(Value::Boolean(!value.is_truthy())),
            UnaryOp::Negate => match value {
                Value::Integer(i) => Ok(Value::Integer(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(RuntimeError::TypeMismatch("Invalid negation".to_string())),
            },
        }
    }

    fn evaluate_call(&mut self, callee: &Expr, arguments: &[Expr]) -> RuntimeResult<Value> {
        let func = self.evaluate_expression(callee)?;

        if let Value::Function { parameters, body } = func {
            if parameters.len() != arguments.len() {
                return Err(RuntimeError::TypeMismatch(format!(
                    "Expected {} arguments, got {}",
                    parameters.len(),
                    arguments.len()
                )));
            }

            self.environment.push_scope();

            for (param, arg) in parameters.iter().zip(arguments.iter()) {
                let arg_value = self.evaluate_expression(arg)?;
                self.environment.define(param.name.clone(), arg_value);
            }

            let result = match self.execute_function_body(&body) {
                Ok(_) => Ok(Value::Null),
                Err(RuntimeError::ReturnValue(val)) => Ok(val),
                Err(e) => Err(e),
            };

            self.environment.pop_scope();
            result
        } else {
            Err(RuntimeError::TypeMismatch(
                "Not a callable function".to_string(),
            ))
        }
    }

    fn execute_function_body(&mut self, body: &[Stmt]) -> RuntimeResult<Value> {
        for stmt in body {
            self.execute_statement(stmt)?;
        }
        Ok(Value::Null)
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l == r,
            (Value::Float(l), Value::Float(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}