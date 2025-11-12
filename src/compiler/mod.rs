use crate::ast::{Expr, Program, Stmt, BinaryOp, UnaryOp, Parameter};
use crate::bytecode::{Chunk, OpCode, Value, Function};
use std::collections::HashMap;

/// 编译错误
#[derive(Debug)]
pub enum CompileError {
    UndefinedVariable(String),
    TooManyConstants,
    TooManyLocals,
    InvalidBreakContinue,
    UndefinedStruct(String),
    UndefinedField(String, String), // (struct_name, field_name)
}

type CompileResult<T> = Result<T, CompileError>;

/// 局部变量信息
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
    is_mutable: bool,
}

/// 作用域深度
#[derive(Debug)]
struct Scope {
    depth: usize,
}

/// 结构体定义信息
#[derive(Debug, Clone)]
struct StructDef {
    fields: Vec<String>,  // 字段名列表（按顺序）
}

/// 字节码编译器
pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    loop_starts: Vec<usize>,      // 循环开始位置栈
    loop_breaks: Vec<Vec<usize>>,  // 循环break跳转位置栈
    structs: HashMap<String, StructDef>, // 结构体定义
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loop_starts: Vec::new(),
            loop_breaks: Vec::new(),
            structs: HashMap::new(),
        }
    }

    /// 编译程序
    pub fn compile(&mut self, program: Program) -> CompileResult<Chunk> {
        for stmt in program.statements {
            self.compile_statement(stmt)?;
        }
        
        // 添加Halt指令
        self.emit(OpCode::Halt, 0);
        
        Ok(self.chunk.clone())
    }

    /// 编译语句
    fn compile_statement(&mut self, stmt: Stmt) -> CompileResult<()> {
        match stmt {
            Stmt::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(OpCode::Pop, 0);
            }

            Stmt::StructDeclaration { name, fields } => {
                // 注册结构体定义
                let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                self.structs.insert(name, StructDef { fields: field_names });
                // 结构体声明在运行时不需要操作
            }

            Stmt::TypeAlias { name: _, target_type: _ } => {
                // 类型别名在编译时处理，运行时不需要操作
            }

            Stmt::VarDeclaration { name, mutable, type_annotation: _, initializer } => {
                if let Some(init) = initializer {
                    self.compile_expression(init)?;
                } else {
                    self.emit(OpCode::LoadNull, 0);
                }

                if self.scope_depth == 0 {
                    // 全局变量
                    let idx = self.identifier_constant(&name)?;
                    self.emit(OpCode::StoreGlobal(idx), 0);
                    self.emit(OpCode::Pop, 0);
                } else {
                    // 局部变量
                    self.add_local(name, mutable)?;
                }
            }

            Stmt::FnDeclaration { name, parameters, return_type: _, body } => {
                let function = self.compile_function(name.clone(), &parameters, body)?;
                let idx = self.chunk.add_constant(Value::Function(function));
                self.emit(OpCode::LoadConst(idx), 0);
                
                if self.scope_depth == 0 {
                    let name_idx = self.identifier_constant(&name)?;
                    self.emit(OpCode::StoreGlobal(name_idx), 0);
                    self.emit(OpCode::Pop, 0);
                } else {
                    self.add_local(name, false)?;
                }
            }

            Stmt::Return { value } => {
                if let Some(expr) = value {
                    self.compile_expression(expr)?;
                } else {
                    self.emit(OpCode::LoadNull, 0);
                }
                self.emit(OpCode::Return, 0);
            }

            Stmt::If { condition, then_branch, else_branch } => {
                self.compile_expression(condition)?;
                
                let then_jump = self.emit_jump(OpCode::JumpIfFalse(0));
                self.emit(OpCode::Pop, 0);
                
                self.begin_scope();
                for stmt in then_branch {
                    self.compile_statement(stmt)?;
                }
                self.end_scope();
                
                let else_jump = self.emit_jump(OpCode::Jump(0));
                self.patch_jump(then_jump);
                self.emit(OpCode::Pop, 0);
                
                if let Some(else_stmts) = else_branch {
                    self.begin_scope();
                    for stmt in else_stmts {
                        self.compile_statement(stmt)?;
                    }
                    self.end_scope();
                }
                
                self.patch_jump(else_jump);
            }

            Stmt::While { condition, body } => {
                let loop_start = self.chunk.len();
                self.loop_starts.push(loop_start);
                self.loop_breaks.push(Vec::new());
                
                self.compile_expression(condition)?;
                let exit_jump = self.emit_jump(OpCode::JumpIfFalse(0));
                self.emit(OpCode::Pop, 0);
                
                self.begin_scope();
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                self.end_scope();
                
                self.emit(OpCode::Loop(loop_start), 0);
                self.patch_jump(exit_jump);
                self.emit(OpCode::Pop, 0);
                
                // 修补所有break跳转
                if let Some(breaks) = self.loop_breaks.pop() {
                    for break_jump in breaks {
                        self.patch_jump(break_jump);
                    }
                }
                self.loop_starts.pop();
            }

            Stmt::For { variable, start, end, body } => {
                self.begin_scope();
                
                // 初始化循环变量
                self.compile_expression(start)?;
                self.add_local(variable.clone(), true)?;
                
                // 计算结束值
                self.compile_expression(end)?;
                let end_local = self.locals.len();
                self.add_local("__end__".to_string(), false)?;
                
                let loop_start = self.chunk.len();
                self.loop_starts.push(loop_start);
                self.loop_breaks.push(Vec::new());
                
                // 条件检查: i < end
                let var_slot = self.resolve_local(&variable)?;
                self.emit(OpCode::LoadLocal(var_slot), 0);
                self.emit(OpCode::LoadLocal(end_local), 0);
                self.emit(OpCode::Less, 0);
                
                let exit_jump = self.emit_jump(OpCode::JumpIfFalse(0));
                self.emit(OpCode::Pop, 0);
                
                // 循环体
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                
                // 递增: i = i + 1
                self.emit(OpCode::LoadLocal(var_slot), 0);
                let one_idx = self.chunk.add_constant(Value::Integer(1));
                self.emit(OpCode::LoadConst(one_idx), 0);
                self.emit(OpCode::Add, 0);
                self.emit(OpCode::StoreLocal(var_slot), 0);
                self.emit(OpCode::Pop, 0);
                
                self.emit(OpCode::Loop(loop_start), 0);
                self.patch_jump(exit_jump);
                self.emit(OpCode::Pop, 0);
                
                // 修补break跳转
                if let Some(breaks) = self.loop_breaks.pop() {
                    for break_jump in breaks {
                        self.patch_jump(break_jump);
                    }
                }
                self.loop_starts.pop();
                
                self.end_scope();
            }

            Stmt::Print { value } => {
                self.compile_expression(value)?;
                self.emit(OpCode::Print, 0);
            }

            Stmt::Block { statements } => {
                self.begin_scope();
                for stmt in statements {
                    self.compile_statement(stmt)?;
                }
                self.end_scope();
            }
        }

        Ok(())
    }

    /// 编译表达式
    fn compile_expression(&mut self, expr: Expr) -> CompileResult<()> {
        match expr {
            Expr::StructLiteral { struct_name, fields } => {
                // 编译结构体字面量
                // 注意：字段按照定义顺序压入栈
                // 由于我们没有类型信息，这里假设字段已经按正确顺序提供
                // 实际应该从类型检查器获取结构体定义

                // 先保存字段数量
                let field_count = fields.len();

                // 按照字段声明顺序编译字段值
                for (_field_name, field_value) in fields {
                    self.compile_expression(field_value)?;
                }

                // 推送结构体名称到栈
                let name_idx = self.chunk.add_constant(Value::String(struct_name));
                self.emit(OpCode::LoadConst(name_idx), 0);

                // 创建结构体（字段数量作为参数）
                self.emit(OpCode::NewStruct(field_count), 0);
            }

            Expr::FieldAccess { object, field } => {
                // 编译字段访问
                // 注意：这需要知道结构体类型才能确定字段索引
                // 简化实现：假设字段按字母顺序或声明顺序索引
                // 这里我们简单地使用0作为占位符
                // 完整实现需要从类型检查器传递类型信息

                self.compile_expression(*object)?;

                // 使用0作为占位符索引（需要类型信息来正确实现）
                let _ = field; // 忽略字段名
                self.emit(OpCode::FieldGet(0), 0);
            }

            Expr::FieldAssign { object, field, value } => {
                // 编译字段赋值
                // 类似于数组索引赋值，需要确保结构体被正确更新

                let var_name = if let Expr::Identifier(name) = object.as_ref() {
                    Some(name.clone())
                } else {
                    None
                };

                self.compile_expression(*object)?;
                self.compile_expression(*value)?;

                // 使用0作为占位符索引（需要类型信息来正确实现）
                let _ = field; // 忽略字段名
                self.emit(OpCode::FieldSet(0), 0);

                // 如果object是标识符，将修改后的结构体存回
                if let Some(name) = var_name {
                    if let Ok(slot) = self.resolve_local(&name) {
                        self.emit(OpCode::StoreLocal(slot), 0);
                    } else {
                        let idx = self.identifier_constant(&name)?;
                        self.emit(OpCode::StoreGlobal(idx), 0);
                    }
                }
            }

            Expr::Integer(n) => {
                let idx = self.chunk.add_constant(Value::Integer(n));
                self.emit(OpCode::LoadConst(idx), 0);
            }

            Expr::Float(f) => {
                let idx = self.chunk.add_constant(Value::Float(f));
                self.emit(OpCode::LoadConst(idx), 0);
            }

            Expr::String(s) => {
                let idx = self.chunk.add_constant(Value::String(s));
                self.emit(OpCode::LoadConst(idx), 0);
            }

            Expr::Boolean(b) => {
                let idx = self.chunk.add_constant(Value::Boolean(b));
                self.emit(OpCode::LoadConst(idx), 0);
            }

            Expr::Identifier(name) => {
                if let Ok(slot) = self.resolve_local(&name) {
                    self.emit(OpCode::LoadLocal(slot), 0);
                } else {
                    let idx = self.identifier_constant(&name)?;
                    self.emit(OpCode::LoadGlobal(idx), 0);
                }
            }

            Expr::Binary { left, operator, right } => {
                // 短路求值优化
                match operator {
                    BinaryOp::And => {
                        self.compile_expression(*left)?;
                        let jump = self.emit_jump(OpCode::JumpIfFalse(0));
                        self.emit(OpCode::Pop, 0);
                        self.compile_expression(*right)?;
                        self.patch_jump(jump);
                        return Ok(());
                    }
                    BinaryOp::Or => {
                        self.compile_expression(*left)?;
                        let jump = self.emit_jump(OpCode::JumpIfTrue(0));
                        self.emit(OpCode::Pop, 0);
                        self.compile_expression(*right)?;
                        self.patch_jump(jump);
                        return Ok(());
                    }
                    _ => {}
                }

                self.compile_expression(*left)?;
                self.compile_expression(*right)?;

                match operator {
                    BinaryOp::Add => self.emit(OpCode::Add, 0),
                    BinaryOp::Subtract => self.emit(OpCode::Subtract, 0),
                    BinaryOp::Multiply => self.emit(OpCode::Multiply, 0),
                    BinaryOp::Divide => self.emit(OpCode::Divide, 0),
                    BinaryOp::Modulo => self.emit(OpCode::Modulo, 0),
                    BinaryOp::Equal => self.emit(OpCode::Equal, 0),
                    BinaryOp::NotEqual => self.emit(OpCode::NotEqual, 0),
                    BinaryOp::Greater => self.emit(OpCode::Greater, 0),
                    BinaryOp::GreaterEqual => self.emit(OpCode::GreaterEqual, 0),
                    BinaryOp::Less => self.emit(OpCode::Less, 0),
                    BinaryOp::LessEqual => self.emit(OpCode::LessEqual, 0),
                    BinaryOp::And | BinaryOp::Or => unreachable!(), // 已处理
                };
            }

            Expr::Unary { operator, operand } => {
                self.compile_expression(*operand)?;
                match operator {
                    UnaryOp::Negate => self.emit(OpCode::Negate, 0),
                    UnaryOp::Not => self.emit(OpCode::Not, 0),
                };
            }

            Expr::Assign { name, value } => {
                self.compile_expression(*value)?;
                
                if let Ok(slot) = self.resolve_local(&name) {
                    self.emit(OpCode::StoreLocal(slot), 0);
                } else {
                    let idx = self.identifier_constant(&name)?;
                    self.emit(OpCode::StoreGlobal(idx), 0);
                }
            }

            Expr::Call { callee, arguments } => {
                self.compile_expression(*callee)?;
                
                for arg in arguments.iter() {
                    self.compile_expression(arg.clone())?;
                }
                
                self.emit(OpCode::Call(arguments.len()), 0);
            }

            Expr::Array { elements } => {
                // 编译每个数组元素
                let len = elements.len();
                for element in elements {
                    self.compile_expression(element)?;
                }
                // 创建数组（栈上的元素会被收集到数组中）
                self.emit(OpCode::NewArray(len), 0);
            }

            Expr::Index { object, index } => {
                // 编译数组和索引表达式
                self.compile_expression(*object)?;
                self.compile_expression(*index)?;
                // 执行数组索引访问
                self.emit(OpCode::ArrayGet, 0);
            }
            
            Expr::IndexAssign { object, index, value } => {
                // 对于数组元素赋值，我们需要特殊处理来确保原数组被更新
                // 如果object是标识符，我们需要：
                // 1. 加载数组
                // 2. 加载索引
                // 3. 加载值
                // 4. 执行ArraySet（修改数组并将新数组留在栈上）
                // 5. 将新数组存回变量

                // 先检查是否是标识符，保存名称
                let var_name = if let Expr::Identifier(name) = object.as_ref() {
                    Some(name.clone())
                } else {
                    None
                };

                // 编译表达式
                self.compile_expression(*object)?;
                self.compile_expression(*index)?;
                self.compile_expression(*value)?;
                // ArraySet返回修改后的数组
                self.emit(OpCode::ArraySet, 0);

                // 如果object是标识符，将修改后的数组存回
                if let Some(name) = var_name {
                    if let Ok(slot) = self.resolve_local(&name) {
                        self.emit(OpCode::StoreLocal(slot), 0);
                    } else {
                        let idx = self.identifier_constant(&name)?;
                        self.emit(OpCode::StoreGlobal(idx), 0);
                    }
                }
                // 否则留在栈上作为表达式结果
            }
        }

        Ok(())
    }

    /// 编译函数
    fn compile_function(
        &mut self,
        name: String,
        parameters: &[Parameter],
        body: Vec<Stmt>,
    ) -> CompileResult<Function> {
        let mut function_compiler = Compiler::new();
        function_compiler.begin_scope();
        
        // 添加参数为局部变量
        for param in parameters {
            function_compiler.add_local(param.name.clone(), false)?;
        }
        
        // 编译函数体
        for stmt in body {
            function_compiler.compile_statement(stmt)?;
        }
        
        // 如果没有显式return，添加返回null
        function_compiler.emit(OpCode::LoadNull, 0);
        function_compiler.emit(OpCode::Return, 0);
        
        Ok(Function {
            name,
            arity: parameters.len(),
            chunk: function_compiler.chunk,
            locals_count: function_compiler.locals.len(),
        })
    }

    // 辅助方法
    fn emit(&mut self, op: OpCode, line: usize) {
        self.chunk.write(op, line);
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit(op, 0);
        self.chunk.len() - 1
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.len();
        self.chunk.code[offset] = match self.chunk.code[offset] {
            OpCode::Jump(_) => OpCode::Jump(jump),
            OpCode::JumpIfFalse(_) => OpCode::JumpIfFalse(jump),
            OpCode::JumpIfTrue(_) => OpCode::JumpIfTrue(jump),
            _ => panic!("Can only patch jump instructions"),
        };
    }

    fn identifier_constant(&mut self, name: &str) -> CompileResult<usize> {
        let value = Value::String(name.to_string());
        Ok(self.chunk.add_constant(value))
    }

    fn add_local(&mut self, name: String, is_mutable: bool) -> CompileResult<()> {
        if self.locals.len() >= 256 {
            return Err(CompileError::TooManyLocals);
        }
        
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
            is_mutable,
        });
        
        Ok(())
    }

    fn resolve_local(&self, name: &str) -> CompileResult<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Ok(i);
            }
        }
        Err(CompileError::UndefinedVariable(name.to_string()))
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        
        // 清理当前作用域的局部变量
        while !self.locals.is_empty() 
            && self.locals.last().unwrap().depth > self.scope_depth 
        {
            self.emit(OpCode::Pop, 0);
            self.locals.pop();
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}