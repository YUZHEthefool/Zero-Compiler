use crate::lexer::token::Token;

// 类型系统定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Void,
    Null,
    Array(Box<Type>),  // 数组类型
    Function(FunctionType),
    Unknown,  // 用于类型推导
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
}

// 函数参数定义
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<Type>,
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }
    
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            // 相同类型
            (a, b) if a == b => true,
            // 数字类型之间兼容
            (a, b) if a.is_numeric() && b.is_numeric() => true,
            // Unknown类型与任何类型兼容
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            // 数组类型需要元素类型兼容
            (Type::Array(a), Type::Array(b)) => a.is_compatible_with(b),
            _ => false,
        }
    }
    
    pub fn get_element_type(&self) -> Option<&Type> {
        match self {
            Type::Array(element_type) => Some(element_type),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // 字面量
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    
    // 数组字面量
    Array {
        elements: Vec<Expr>,
    },
    
    // 二元运算
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    
    // 一元运算
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },
    
    // 函数调用
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    
    // 数组/索引访问
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    
    // 索引赋值
    IndexAssign {
        object: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
    },
    
    // 赋值
    Assign {
        name: String,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // 算术运算符
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    
    // 比较运算符
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    
    // 逻辑运算符
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // 表达式语句
    Expression(Expr),
    
    // 变量声明
    VarDeclaration {
        name: String,
        mutable: bool,
        type_annotation: Option<Type>,
        initializer: Option<Expr>,
    },
    
    // 函数声明
    FnDeclaration {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
    },
    
    // 返回语句
    Return {
        value: Option<Expr>,
    },
    
    // if 语句
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    
    // while 循环
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    
    // for 循环
    For {
        variable: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
    },
    
    // 打印语句
    Print {
        value: Expr,
    },
    
    // 代码块
    Block {
        statements: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            statements: Vec::new(),
        }
    }
    
    pub fn add_statement(&mut self, stmt: Stmt) {
        self.statements.push(stmt);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

// 辅助函数用于创建表达式
impl Expr {
    pub fn integer(value: i64) -> Self {
        Expr::Integer(value)
    }
    
    pub fn float(value: f64) -> Self {
        Expr::Float(value)
    }
    
    pub fn string(value: String) -> Self {
        Expr::String(value)
    }
    
    pub fn boolean(value: bool) -> Self {
        Expr::Boolean(value)
    }
    
    pub fn identifier(name: String) -> Self {
        Expr::Identifier(name)
    }
    
    pub fn array(elements: Vec<Expr>) -> Self {
        Expr::Array { elements }
    }
    
    pub fn binary(left: Expr, operator: BinaryOp, right: Expr) -> Self {
        Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
    
    pub fn unary(operator: UnaryOp, operand: Expr) -> Self {
        Expr::Unary {
            operator,
            operand: Box::new(operand),
        }
    }
    
    pub fn call(callee: Expr, arguments: Vec<Expr>) -> Self {
        Expr::Call {
            callee: Box::new(callee),
            arguments,
        }
    }
    
    pub fn index(object: Expr, index: Expr) -> Self {
        Expr::Index {
            object: Box::new(object),
            index: Box::new(index),
        }
    }
    
    pub fn index_assign(object: Expr, index: Expr, value: Expr) -> Self {
        Expr::IndexAssign {
            object: Box::new(object),
            index: Box::new(index),
            value: Box::new(value),
        }
    }
    
    pub fn assign(name: String, value: Expr) -> Self {
        Expr::Assign {
            name,
            value: Box::new(value),
        }
    }
}