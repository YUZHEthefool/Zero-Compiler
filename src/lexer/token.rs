#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // 字面量
    Integer,
    Float,
    String,
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
            "float" => Some(TokenType::Float64),
            "string" => Some(TokenType::String64),
            "bool" => Some(TokenType::Bool),
            "void" => Some(TokenType::Void),
            "null" => Some(TokenType::Null),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

impl Token {
    pub fn new(token_type: TokenType, value: String) -> Self {
        Token { token_type, value }
    }
}