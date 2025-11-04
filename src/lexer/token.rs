use std::fmt;

/// 位置信息，用于追踪Token在源代码中的位置
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Position { line, column, offset }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Token类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // 字面量
    Integer,
    Float,
    String,
    Char,
    Identifier,
    
    // 关键字
    Let,
    Var,
    Fn,
    Return,
    If,
    Else,
    While,
    For,
    In,
    True,
    False,
    Print,
    
    // 类型关键字
    Int,
    Int64,
    Float64,
    String64,
    Bool,
    Void,
    Null,
    
    // 运算符
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    
    // 复合赋值运算符
    PlusEqual,      // +=
    MinusEqual,     // -=
    StarEqual,      // *=
    SlashEqual,     // /=
    PercentEqual,   // %=
    
    // 比较运算符
    Equal,          // =
    EqualEqual,     // ==
    Bang,           // !
    BangEqual,      // !=
    Less,           // <
    LessEqual,      // <=
    Greater,        // >
    GreaterEqual,   // >=
    
    // 逻辑运算符
    And,        // &&
    Or,         // ||
    
    // 分隔符
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Comma,          // ,
    Semicolon,      // ;
    Colon,          // :
    Dot,            // .
    DotDot,         // ..
    Arrow,          // ->
    
    // 科学计数法（将被预处理器转换）
    ScientificExponent,
    
    // 特殊
    EOF,
    Unknown,
}

impl TokenType {
    pub fn get_keyword(word: &str) -> Option<TokenType> {
        match word {
            "let" => Some(TokenType::Let),
            "var" => Some(TokenType::Var),
            "fn" => Some(TokenType::Fn),
            "return" => Some(TokenType::Return),
            "if" => Some(TokenType::If),
            "else" => Some(TokenType::Else),
            "while" => Some(TokenType::While),
            "for" => Some(TokenType::For),
            "in" => Some(TokenType::In),
            "true" => Some(TokenType::True),
            "false" => Some(TokenType::False),
            "print" => Some(TokenType::Print),
            // 类型关键字
            "int" => Some(TokenType::Int),
            "int64" => Some(TokenType::Int64),
            "float" => Some(TokenType::Float64),
            "string" => Some(TokenType::String64),
            "bool" => Some(TokenType::Bool),
            "void" => Some(TokenType::Void),
            "null" => Some(TokenType::Null),
            _ => None,
        }
    }
}

/// Token结构，包含类型、值和位置信息
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub start_pos: Position,
    pub end_pos: Position,
}

impl Token {
    pub fn new(token_type: TokenType, value: String, start_pos: Position, end_pos: Position) -> Self {
        Token { 
            token_type, 
            value,
            start_pos,
            end_pos,
        }
    }
    
    pub fn simple(token_type: TokenType, value: String) -> Self {
        let pos = Position::new(0, 0, 0);
        Token::new(token_type, value, pos.clone(), pos)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}('{}') at {}",
            self.token_type, self.value, self.start_pos
        )
    }
}