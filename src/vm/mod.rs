use crate::bytecode::{Chunk, OpCode, Value, Function};
use std::collections::HashMap;

/// 虚拟机运行时错误
#[derive(Debug)]
pub enum VMError {
    StackUnderflow,
    StackOverflow,
    TypeError(String),
    UndefinedVariable(String),
    DivisionByZero,
    InvalidOperation(String),
}

type VMResult<T> = Result<T, VMError>;

/// 调用帧（用于函数调用）
#[derive(Debug, Clone)]
struct CallFrame {
    function: Function,
    ip: usize,              // 指令指针
    stack_offset: usize,    // 栈帧起始位置
}

/// Zero语言虚拟机
pub struct VM {
    stack: Vec<Value>,              // 值栈
    globals: HashMap<String, Value>, // 全局变量
    frames: Vec<CallFrame>,          // 调用栈
    current_frame: usize,            // 当前帧索引
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            frames: Vec::new(),
            current_frame: 0,
        }
    }

    /// 执行字节码
    pub fn execute(&mut self, chunk: Chunk) -> VMResult<()> {
        // 创建主函数帧
        let main_function = Function {
            name: "<script>".to_string(),
            arity: 0,
            chunk,
            locals_count: 0,
        };

        self.frames.push(CallFrame {
            function: main_function,
            ip: 0,
            stack_offset: 0,
        });

        self.run()
    }

    /// 主执行循环
    fn run(&mut self) -> VMResult<()> {
        loop {
            let frame = &self.frames[self.current_frame];
            
            // 调试输出（可选）
            #[cfg(debug_assertions)]
            {
                print!("Stack: [");
                for value in &self.stack {
                    print!("{:?}, ", value);
                }
                println!("]");
                frame.function.chunk.disassemble_instruction(
                    frame.ip,
                    &frame.function.chunk.code[frame.ip]
                );
            }

            let instruction = frame.function.chunk.code[frame.ip].clone();
            self.frames[self.current_frame].ip += 1;

            match instruction {
                OpCode::LoadConst(idx) => {
                    let value = self.frames[self.current_frame]
                        .function
                        .chunk
                        .constants[idx]
                        .clone();
                    self.push(value)?;
                }

                OpCode::LoadNull => {
                    self.push(Value::Null)?;
                }

                OpCode::LoadLocal(slot) => {
                    let offset = self.frames[self.current_frame].stack_offset;
                    let value = self.stack[offset + slot].clone();
                    self.push(value)?;
                }

                OpCode::StoreLocal(slot) => {
                    let value = self.peek(0)?.clone();
                    let offset = self.frames[self.current_frame].stack_offset;
                    self.stack[offset + slot] = value;
                }

                OpCode::LoadGlobal(idx) => {
                    let name = match &self.frames[self.current_frame]
                        .function
                        .chunk
                        .constants[idx]
                    {
                        Value::String(s) => s.clone(),
                        _ => return Err(VMError::TypeError("Expected string for variable name".to_string())),
                    };

                    let value = self.globals
                        .get(&name)
                        .ok_or_else(|| VMError::UndefinedVariable(name.clone()))?
                        .clone();
                    
                    self.push(value)?;
                }

                OpCode::StoreGlobal(idx) => {
                    let name = match &self.frames[self.current_frame]
                        .function
                        .chunk
                        .constants[idx]
                    {
                        Value::String(s) => s.clone(),
                        _ => return Err(VMError::TypeError("Expected string for variable name".to_string())),
                    };

                    let value = self.peek(0)?.clone();
                    self.globals.insert(name, value);
                }

                // 算术运算
                OpCode::Add => self.binary_op(|a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x + y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
                    (Value::Integer(x), Value::Float(y)) => Ok(Value::Float(x as f64 + y)),
                    (Value::Float(x), Value::Integer(y)) => Ok(Value::Float(x + y as f64)),
                    (Value::String(x), Value::String(y)) => Ok(Value::String(format!("{}{}", x, y))),
                    _ => Err(VMError::TypeError("Invalid operands for addition".to_string())),
                })?,

                OpCode::Subtract => self.binary_op(|a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x - y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
                    (Value::Integer(x), Value::Float(y)) => Ok(Value::Float(x as f64 - y)),
                    (Value::Float(x), Value::Integer(y)) => Ok(Value::Float(x - y as f64)),
                    _ => Err(VMError::TypeError("Invalid operands for subtraction".to_string())),
                })?,

                OpCode::Multiply => self.binary_op(|a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x * y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
                    (Value::Integer(x), Value::Float(y)) => Ok(Value::Float(x as f64 * y)),
                    (Value::Float(x), Value::Integer(y)) => Ok(Value::Float(x * y as f64)),
                    _ => Err(VMError::TypeError("Invalid operands for multiplication".to_string())),
                })?,

                OpCode::Divide => self.binary_op(|a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        if y == 0 {
                            return Err(VMError::DivisionByZero);
                        }
                        Ok(Value::Integer(x / y))
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if y == 0.0 {
                            return Err(VMError::DivisionByZero);
                        }
                        Ok(Value::Float(x / y))
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        if y == 0.0 {
                            return Err(VMError::DivisionByZero);
                        }
                        Ok(Value::Float(x as f64 / y))
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        if y == 0 {
                            return Err(VMError::DivisionByZero);
                        }
                        Ok(Value::Float(x / y as f64))
                    }
                    _ => Err(VMError::TypeError("Invalid operands for division".to_string())),
                })?,

                OpCode::Modulo => self.binary_op(|a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        if y == 0 {
                            return Err(VMError::DivisionByZero);
                        }
                        Ok(Value::Integer(x % y))
                    }
                    _ => Err(VMError::TypeError("Invalid operands for modulo".to_string())),
                })?,

                OpCode::Negate => {
                    let value = self.pop()?;
                    let result = match value {
                        Value::Integer(i) => Value::Integer(-i),
                        Value::Float(f) => Value::Float(-f),
                        _ => return Err(VMError::TypeError("Cannot negate non-numeric value".to_string())),
                    };
                    self.push(result)?;
                }

                // 比较运算
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(a == b))?;
                }

                OpCode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(a != b))?;
                }

                OpCode::Greater => self.comparison_op(|a, b| a > b)?,
                OpCode::GreaterEqual => self.comparison_op(|a, b| a >= b)?,
                OpCode::Less => self.comparison_op(|a, b| a < b)?,
                OpCode::LessEqual => self.comparison_op(|a, b| a <= b)?,

                // 逻辑运算
                OpCode::Not => {
                    let value = self.pop()?;
                    self.push(Value::Boolean(!value.is_truthy()))?;
                }

                OpCode::And => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(a.is_truthy() && b.is_truthy()))?;
                }

                OpCode::Or => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(a.is_truthy() || b.is_truthy()))?;
                }

                // 控制流
                OpCode::Jump(offset) => {
                    self.frames[self.current_frame].ip = offset;
                }

                OpCode::JumpIfFalse(offset) => {
                    let condition = self.peek(0)?;
                    if !condition.is_truthy() {
                        self.frames[self.current_frame].ip = offset;
                    }
                }

                OpCode::JumpIfTrue(offset) => {
                    let condition = self.peek(0)?;
                    if condition.is_truthy() {
                        self.frames[self.current_frame].ip = offset;
                    }
                }

                OpCode::Loop(offset) => {
                    self.frames[self.current_frame].ip = offset;
                }

                // 函数调用
                OpCode::Call(arg_count) => {
                    let callee = self.peek(arg_count)?.clone();
                    match callee {
                        Value::Function(func) => {
                            if func.arity != arg_count {
                                return Err(VMError::InvalidOperation(
                                    format!("Expected {} arguments but got {}", func.arity, arg_count)
                                ));
                            }

                            // 栈布局: [..., function, arg1, arg2, ...]
                            // 我们需要移除function，只保留参数
                            let stack_offset = self.stack.len() - arg_count - 1;
                            
                            // 移除function对象，参数上移
                            self.stack.remove(stack_offset);
                            
                            self.frames.push(CallFrame {
                                function: func,
                                ip: 0,
                                stack_offset: self.stack.len() - arg_count,
                            });
                            self.current_frame += 1;
                        }
                        _ => return Err(VMError::TypeError("Can only call functions".to_string())),
                    }
                }

                OpCode::Return => {
                    let result = self.pop()?;
                    
                    // 清理当前帧的栈
                    let frame_offset = self.frames[self.current_frame].stack_offset;
                    self.stack.truncate(frame_offset);
                    
                    self.frames.pop();
                    
                    if self.frames.is_empty() {
                        return Ok(());
                    }
                    
                    self.current_frame -= 1;
                    self.push(result)?;
                }

                // 栈操作
                OpCode::Pop => {
                    self.pop()?;
                }

                OpCode::Dup => {
                    let value = self.peek(0)?.clone();
                    self.push(value)?;
                }

                // 数组操作
                OpCode::NewArray(size) => {
                    let mut elements = Vec::with_capacity(size);
                    // 从栈中弹出元素（注意顺序）
                    for _ in 0..size {
                        elements.push(self.pop()?);
                    }
                    // 反转以保持正确顺序
                    elements.reverse();
                    self.push(Value::Array(elements))?;
                }

                OpCode::ArrayGet => {
                    let index = self.pop()?;
                    let array = self.pop()?;
                    
                    let idx = match index {
                        Value::Integer(i) => i,
                        _ => return Err(VMError::TypeError("Array index must be an integer".to_string())),
                    };
                    
                    match array {
                        Value::Array(arr) => {
                            let actual_idx = if idx < 0 {
                                // 负索引：从末尾访问
                                let len = arr.len() as i64;
                                (len + idx) as usize
                            } else {
                                idx as usize
                            };
                            
                            if actual_idx >= arr.len() {
                                return Err(VMError::InvalidOperation(
                                    format!("Array index {} out of bounds (length: {})", idx, arr.len())
                                ));
                            }
                            
                            self.push(arr[actual_idx].clone())?;
                        }
                        _ => return Err(VMError::TypeError("Can only index arrays".to_string())),
                    }
                }

                OpCode::ArraySet => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let array = self.pop()?;
                    
                    let idx = match index {
                        Value::Integer(i) => i,
                        _ => return Err(VMError::TypeError("Array index must be an integer".to_string())),
                    };
                    
                    // 我们需要可变引用来修改数组
                    // 但由于所有权问题，这里需要重新构建数组
                    match array {
                        Value::Array(mut arr) => {
                            let actual_idx = if idx < 0 {
                                let len = arr.len() as i64;
                                (len + idx) as usize
                            } else {
                                idx as usize
                            };
                            
                            if actual_idx >= arr.len() {
                                return Err(VMError::InvalidOperation(
                                    format!("Array index {} out of bounds (length: {})", idx, arr.len())
                                ));
                            }
                            
                            arr[actual_idx] = value.clone();
                            self.push(Value::Array(arr))?;
                            self.push(value)?; // 返回被赋的值
                        }
                        _ => return Err(VMError::TypeError("Can only index arrays".to_string())),
                    }
                }

                OpCode::ArrayLen => {
                    let array = self.pop()?;
                    match array {
                        Value::Array(arr) => {
                            self.push(Value::Integer(arr.len() as i64))?;
                        }
                        _ => return Err(VMError::TypeError("Can only get length of arrays".to_string())),
                    }
                }

                // 其他
                OpCode::Print => {
                    let value = self.pop()?;
                    println!("{}", value.to_string());
                }

                OpCode::Halt => {
                    return Ok(());
                }
            }
        }
    }

    // 辅助方法
    fn push(&mut self, value: Value) -> VMResult<()> {
        if self.stack.len() >= 1024 {
            return Err(VMError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    fn pop(&mut self) -> VMResult<Value> {
        self.stack.pop().ok_or(VMError::StackUnderflow)
    }

    fn peek(&self, distance: usize) -> VMResult<&Value> {
        let len = self.stack.len();
        if distance >= len {
            return Err(VMError::StackUnderflow);
        }
        Ok(&self.stack[len - 1 - distance])
    }

    fn binary_op<F>(&mut self, op: F) -> VMResult<()>
    where
        F: FnOnce(Value, Value) -> VMResult<Value>,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = op(a, b)?;
        self.push(result)
    }

    fn comparison_op<F>(&mut self, op: F) -> VMResult<()>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        let result = match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => op(x as f64, y as f64),
            (Value::Float(x), Value::Float(y)) => op(x, y),
            (Value::Integer(x), Value::Float(y)) => op(x as f64, y),
            (Value::Float(x), Value::Integer(y)) => op(x, y as f64),
            _ => return Err(VMError::TypeError("Cannot compare non-numeric values".to_string())),
        };

        self.push(Value::Boolean(result))
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}