use crate::ast::{Expr, Program, Stmt, BinaryOp, UnaryOp, Type, Parameter, FunctionType, MethodDeclaration};
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
    ImmutableAssignment {
        variable: String,
    },
    BreakOutsideLoop,
    ContinueOutsideLoop,
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

/// 方法签名信息
#[derive(Debug, Clone)]
struct MethodSignature {
    params: Vec<Type>,
    return_type: Type,
}

/// 类型检查器
pub struct TypeChecker {
    symbol_table: SymbolTable,
    current_function_return_type: Option<Type>,
    loop_depth: usize,  // 追踪循环嵌套深度
    methods: HashMap<String, HashMap<String, MethodSignature>>,  // type_name -> (method_name -> signature)
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbol_table: SymbolTable::new(),
            current_function_return_type: None,
            loop_depth: 0,
            methods: HashMap::new(),
        }
    }

    /// 解析类型（将Named类型解析为实际类型）
    fn resolve_type(&self, t: &Type) -> Type {
        match t {
            Type::Named(name) => {
                // 查找符号表中的类型别名或结构体定义
                if let Some(symbol) = self.symbol_table.get(name) {
                    // 递归解析，防止链式别名
                    self.resolve_type(&symbol.symbol_type)
                } else {
                    // 如果找不到定义，保持原样（后续会报错）
                    t.clone()
                }
            }
            Type::Array(element_type) => {
                // 递归解析数组元素类型
                Type::Array(Box::new(self.resolve_type(element_type)))
            }
            Type::Function(func_type) => {
                // 递归解析函数参数和返回类型
                let params = func_type.params.iter()
                    .map(|p| self.resolve_type(p))
                    .collect();
                let return_type = Box::new(self.resolve_type(&func_type.return_type));
                Type::Function(FunctionType { params, return_type })
            }
            Type::Struct(struct_type) => {
                // 递归解析结构体字段类型
                let fields = struct_type.fields.iter()
                    .map(|f| crate::ast::StructField {
                        name: f.name.clone(),
                        field_type: self.resolve_type(&f.field_type),
                    })
                    .collect();
                Type::Struct(crate::ast::StructType {
                    name: struct_type.name.clone(),
                    fields,
                })
            }
            // 其他类型直接返回
            _ => t.clone(),
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
            Stmt::StructDeclaration { name, fields } => {
                // 注册结构体类型
                let struct_type = Type::Struct(crate::ast::StructType {
                    name: name.clone(),
                    fields: fields.clone(),
                });
                self.symbol_table.define(name.clone(), struct_type, false);
                Ok(())
            }

            Stmt::TypeAlias { name, target_type } => {
                // 注册类型别名
                self.symbol_table.define(name.clone(), target_type.clone(), false);
                Ok(())
            }

            Stmt::ImplBlock { type_name, methods } => {
                // 验证类型存在
                if self.symbol_table.get(type_name).is_none() {
                    return Err(TypeError::UndefinedVariable(format!("Type {} not found", type_name)));
                }

                // 注册所有方法
                let mut method_map = HashMap::new();

                for method in methods {
                    // 构建方法签名（不包含 self 参数）
                    let param_types: Vec<Type> = method.parameters
                        .iter()
                        .map(|p| p.type_annotation.clone().unwrap_or(Type::Unknown))
                        .collect();

                    let ret_type = method.return_type.clone().unwrap_or(Type::Void);

                    method_map.insert(
                        method.name.clone(),
                        MethodSignature {
                            params: param_types.clone(),
                            return_type: ret_type.clone(),
                        },
                    );

                    // 检查方法体
                    self.symbol_table.push_scope();
                    self.current_function_return_type = Some(ret_type);

                    // 添加 self 参数到作用域
                    if let Some(symbol) = self.symbol_table.get(type_name) {
                        self.symbol_table.define("self".to_string(), symbol.symbol_type.clone(), false);
                    }

                    // 添加其他参数到作用域
                    for param in &method.parameters {
                        let param_type = param.type_annotation.clone().unwrap_or(Type::Unknown);
                        self.symbol_table.define(param.name.clone(), param_type, false);
                    }

                    // 检查方法体
                    for stmt in &method.body {
                        self.check_statement(stmt)?;
                    }

                    self.symbol_table.pop_scope();
                    self.current_function_return_type = None;
                }

                // 注册方法到方法表
                self.methods.insert(type_name.clone(), method_map);

                Ok(())
            }

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
                    // 解析类型注解（处理类型别名）
                    let resolved_annotated = self.resolve_type(annotated_type);
                    let resolved_actual = self.resolve_type(&actual_type);

                    // 检查类型注解和初始化值是否匹配
                    if let Some(_init) = initializer {
                        if !resolved_annotated.is_compatible_with(&resolved_actual) && resolved_actual != Type::Unknown {
                            return Err(TypeError::TypeMismatch {
                                expected: resolved_annotated.clone(),
                                found: resolved_actual,
                                location: format!("variable declaration '{}'", name),
                            });
                        }
                    }
                    resolved_annotated
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
                    let resolved_expected = self.resolve_type(expected_type);
                    let resolved_return = self.resolve_type(&return_type);

                    if resolved_expected != Type::Unknown
                        && resolved_return != Type::Unknown
                        && !resolved_expected.is_compatible_with(&resolved_return) {
                        return Err(TypeError::ReturnTypeMismatch {
                            expected: resolved_expected,
                            found: resolved_return,
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

                self.loop_depth += 1;
                self.symbol_table.push_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.symbol_table.pop_scope();
                self.loop_depth -= 1;

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

                self.loop_depth += 1;
                self.symbol_table.push_scope();
                self.symbol_table.define(variable.clone(), Type::Int, true);

                for stmt in body {
                    self.check_statement(stmt)?;
                }

                self.symbol_table.pop_scope();
                self.loop_depth -= 1;
                Ok(())
            }

            Stmt::Break => {
                if self.loop_depth == 0 {
                    return Err(TypeError::BreakOutsideLoop);
                }
                Ok(())
            }

            Stmt::Continue => {
                if self.loop_depth == 0 {
                    return Err(TypeError::ContinueOutsideLoop);
                }
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
            Expr::StructLiteral { struct_name, fields } => {
                // 查找结构体类型
                if let Some(symbol) = self.symbol_table.get(struct_name) {
                    let struct_type = self.resolve_type(&symbol.symbol_type);

                    // 验证字段
                    if let Type::Struct(ref struct_def) = struct_type {
                        // 检查字段数量
                        if fields.len() != struct_def.fields.len() {
                            return Err(TypeError::TypeMismatch {
                                expected: struct_type.clone(),
                                found: Type::Unknown,
                                location: format!("struct {} requires {} fields, but {} provided",
                                    struct_name, struct_def.fields.len(), fields.len()),
                            });
                        }

                        // 检查每个字段的类型
                        for (field_name, field_expr) in fields {
                            let field_type = self.infer_type(field_expr)?;

                            // 查找字段定义
                            let field_def = struct_def.fields.iter().find(|f| &f.name == field_name);
                            if let Some(def) = field_def {
                                let expected_type = self.resolve_type(&def.field_type);
                                if !field_type.is_compatible_with(&expected_type) {
                                    return Err(TypeError::TypeMismatch {
                                        expected: expected_type,
                                        found: field_type,
                                        location: format!("field {} in struct {}", field_name, struct_name),
                                    });
                                }
                            } else {
                                return Err(TypeError::UndefinedVariable(
                                    format!("field {} not found in struct {}", field_name, struct_name)
                                ));
                            }
                        }
                    }

                    Ok(struct_type)
                } else {
                    Err(TypeError::UndefinedVariable(struct_name.clone()))
                }
            }

            Expr::FieldAccess { object, field } => {
                let obj_type = self.infer_type(object)?;
                match obj_type {
                    Type::Struct(struct_type) => {
                        for f in &struct_type.fields {
                            if &f.name == field {
                                return Ok(f.field_type.clone());
                            }
                        }
                        Err(TypeError::UndefinedVariable(format!("Field {} not found", field)))
                    }
                    _ => Err(TypeError::InvalidOperation {
                        operator: "field access".to_string(),
                        left_type: obj_type,
                        right_type: Type::Unknown,
                    }),
                }
            }

            Expr::FieldAssign { object, field, value } => {
                let obj_type = self.infer_type(object)?;
                let val_type = self.infer_type(value)?;
                match obj_type {
                    Type::Struct(struct_type) => {
                        for f in &struct_type.fields {
                            if &f.name == field {
                                let resolved_field = self.resolve_type(&f.field_type);
                                let resolved_val = self.resolve_type(&val_type);

                                if !resolved_field.is_compatible_with(&resolved_val) && resolved_val != Type::Unknown {
                                    return Err(TypeError::TypeMismatch {
                                        expected: resolved_field,
                                        found: resolved_val,
                                        location: format!("field assignment to {}", field),
                                    });
                                }
                                return Ok(val_type);
                            }
                        }
                        Err(TypeError::UndefinedVariable(format!("Field {} not found", field)))
                    }
                    _ => Err(TypeError::InvalidOperation {
                        operator: "field assignment".to_string(),
                        left_type: obj_type,
                        right_type: val_type,
                    }),
                }
            }

            Expr::Integer(_) => Ok(Type::Int),
            Expr::Float(_) => Ok(Type::Float),
            Expr::String(_) => Ok(Type::String),
            Expr::Boolean(_) => Ok(Type::Bool),
            Expr::Char(_) => Ok(Type::Char),

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
                    // 检查可变性
                    if !symbol.is_mutable {
                        return Err(TypeError::ImmutableAssignment {
                            variable: name.clone(),
                        });
                    }

                    let resolved_symbol = self.resolve_type(&symbol.symbol_type);
                    let resolved_value = self.resolve_type(&value_type);

                    // 只有当类型都不是Unknown时才检查类型兼容性
                    if resolved_symbol != Type::Unknown
                        && resolved_value != Type::Unknown
                        && !resolved_symbol.is_compatible_with(&resolved_value) {
                        return Err(TypeError::TypeMismatch {
                            expected: resolved_symbol,
                            found: resolved_value,
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
                                let resolved_param = self.resolve_type(param_type);
                                let resolved_arg = self.resolve_type(&arg_type);

                                if !resolved_param.is_compatible_with(&resolved_arg) {
                                    return Err(TypeError::ArgumentTypeMismatch {
                                        expected: resolved_param,
                                        found: resolved_arg,
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

            Expr::MethodCall { object, method, arguments } => {
                // 获取对象的类型
                let obj_type = self.infer_type(object)?;
                let obj_type = self.resolve_type(&obj_type);

                // 根据对象类型查找方法
                let type_name = match &obj_type {
                    Type::Struct(struct_type) => struct_type.name.clone(),
                    Type::Named(name) => name.clone(),
                    _ => {
                        return Err(TypeError::InvalidOperation {
                            operator: "method call".to_string(),
                            left_type: obj_type,
                            right_type: Type::Unknown,
                        });
                    }
                };

                // 查找方法签名并克隆以避免借用冲突
                let method_sig = self.methods
                    .get(&type_name)
                    .and_then(|type_methods| type_methods.get(method))
                    .cloned()
                    .ok_or_else(|| TypeError::UndefinedFunction(format!("Method {} not found on type {}", method, type_name)))?;

                // 检查参数数量
                if method_sig.params.len() != arguments.len() {
                    return Err(TypeError::ArgumentCountMismatch {
                        expected: method_sig.params.len(),
                        found: arguments.len(),
                        function: format!("{}.{}", type_name, method),
                    });
                }

                // 检查每个参数的类型
                for (i, (param_type, arg)) in method_sig.params.iter().zip(arguments.iter()).enumerate() {
                    let arg_type = self.infer_type(arg)?;
                    let resolved_param = self.resolve_type(param_type);
                    let resolved_arg = self.resolve_type(&arg_type);

                    if !resolved_param.is_compatible_with(&resolved_arg) && resolved_arg != Type::Unknown {
                        return Err(TypeError::ArgumentTypeMismatch {
                            expected: resolved_param,
                            found: resolved_arg,
                            argument: i + 1,
                            function: format!("{}.{}", type_name, method),
                        });
                    }
                }

                // 返回方法的返回类型
                Ok(method_sig.return_type.clone())
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
                    let resolved_element = self.resolve_type(element_type);
                    let resolved_val = self.resolve_type(&val_type);

                    if !resolved_element.is_compatible_with(&resolved_val) && resolved_val != Type::Unknown {
                        return Err(TypeError::TypeMismatch {
                            expected: resolved_element,
                            found: resolved_val,
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
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&program).is_ok());
    }

    #[test]
    fn test_type_check_type_mismatch() {
        let input = "let x: int = \"hello\";";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&program).is_err());
    }

    #[test]
    fn test_type_check_function() {
        let input = "fn add(a: int, b: int) -> int { return a + b; }";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&program).is_ok());
    }
}