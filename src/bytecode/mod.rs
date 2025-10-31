pub mod serializer;

/// Zero语言的字节码指令集
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // 常量加载
    LoadConst(usize),      // 加载常量池中的值
    LoadNull,              // 加载null值
    
    // 变量操作
    LoadLocal(usize),      // 加载局部变量
    StoreLocal(usize),     // 存储局部变量
    LoadGlobal(usize),     // 加载全局变量
    StoreGlobal(usize),    // 存储全局变量
    
    // 算术运算
    Add,                   // 加法
    Subtract,              // 减法
    Multiply,              // 乘法
    Divide,                // 除法
    Modulo,                // 取模
    Negate,                // 取负
    
    // 比较运算
    Equal,                 // 相等
    NotEqual,              // 不相等
    Greater,               // 大于
    GreaterEqual,          // 大于等于
    Less,                  // 小于
    LessEqual,             // 小于等于
    
    // 逻辑运算
    Not,                   // 逻辑非
    And,                   // 逻辑与
    Or,                    // 逻辑或
    
    // 控制流
    Jump(usize),           // 无条件跳转
    JumpIfFalse(usize),    // 条件跳转（假）
    JumpIfTrue(usize),     // 条件跳转（真）
    Loop(usize),           // 循环跳转
    
    // 函数相关
    Call(usize),           // 函数调用（参数数量）
    Return,                // 返回
    
    // 数组操作
    NewArray(usize),       // 创建新数组（参数：元素数量）
    ArrayGet,              // 获取数组元素 (array, index -> value)
    ArraySet,              // 设置数组元素 (array, index, value -> value)
    ArrayLen,              // 获取数组长度 (array -> length)
    
    // 栈操作
    Pop,                   // 弹出栈顶
    Dup,                   // 复制栈顶
    
    // 其他
    Print,                 // 打印
    Halt,                  // 停止执行
}

/// 常量值类型
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),     // 数组值
    Function(Function),
    Null,
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Function(_) => "<function>".to_string(),
            Value::Null => "null".to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            Value::Array(arr) => !arr.is_empty(),
            _ => true,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }
    
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }
    
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

/// 函数对象
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,          // 参数数量
    pub chunk: Chunk,           // 函数字节码
    pub locals_count: usize,    // 局部变量数量
}

/// 字节码块
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<OpCode>,      // 指令序列
    pub constants: Vec<Value>,  // 常量池
    pub lines: Vec<usize>,      // 行号信息（用于错误报告）
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// 添加指令
    pub fn write(&mut self, op: OpCode, line: usize) {
        self.code.push(op);
        self.lines.push(line);
    }

    /// 添加常量到常量池
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// 获取指令数量
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// 反汇编（用于调试）
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for (offset, op) in self.code.iter().enumerate() {
            self.disassemble_instruction(offset, op);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize, op: &OpCode) {
        print!("{:04} ", offset);
        
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        match op {
            OpCode::LoadConst(idx) => {
                println!("LoadConst {} '{:?}'", idx, self.constants.get(*idx));
            }
            OpCode::LoadLocal(idx) => println!("LoadLocal {}", idx),
            OpCode::StoreLocal(idx) => println!("StoreLocal {}", idx),
            OpCode::LoadGlobal(idx) => println!("LoadGlobal {}", idx),
            OpCode::StoreGlobal(idx) => println!("StoreGlobal {}", idx),
            OpCode::Jump(offset) => println!("Jump -> {}", offset),
            OpCode::JumpIfFalse(offset) => println!("JumpIfFalse -> {}", offset),
            OpCode::JumpIfTrue(offset) => println!("JumpIfTrue -> {}", offset),
            OpCode::Loop(offset) => println!("Loop -> {}", offset),
            OpCode::Call(arity) => println!("Call({})", arity),
            OpCode::NewArray(size) => println!("NewArray({})", size),
            _ => println!("{:?}", op),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}