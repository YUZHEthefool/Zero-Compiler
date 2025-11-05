//! Zero编译器错误处理系统
//! 
//! 该模块提供了两种错误报告模式：
//! 1. 简易模式：仅显示行号和列号
//! 2. 详细模式：Rust风格的完整错误信息，包含源码片段和修复建议

use std::fmt;

/// 错误显示模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorMode {
    /// 简易模式：仅显示行号和位置
    Simple,
    /// 详细模式：显示完整的错误层次结构和源码片段
    Detailed,
}

/// 源码位置信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize, offset: usize, length: usize) -> Self {
        Self { line, column, offset, length }
    }
    
    pub fn single(line: usize, column: usize, offset: usize) -> Self {
        Self::new(line, column, offset, 1)
    }
}

/// 错误基类 - 所有错误的共同接口
pub trait CompilerError: fmt::Debug + fmt::Display {
    /// 获取错误位置
    fn location(&self) -> SourceLocation;
    
    /// 获取错误代码（用于文档查询）
    fn error_code(&self) -> &'static str;
    
    /// 获取错误标题
    fn title(&self) -> String;
    
    /// 获取错误详细描述
    fn description(&self) -> String;
    
    /// 获取修复建议
    fn suggestion(&self) -> Option<String> {
        None
    }
    
    /// 格式化为简易模式
    fn format_simple(&self) -> String {
        format!(
            "错误 [{}] 在 {}:{}: {}",
            self.error_code(),
            self.location().line,
            self.location().column,
            self.title()
        )
    }
    
    /// 格式化为详细模式
    fn format_detailed(&self, source: Option<&str>) -> String {
        let mut output = String::new();
        
        // 错误标题
        output.push_str(&format!("\x1b[1;31merror[{}]\x1b[0m: {}\n", self.error_code(), self.title()));
        
        // 位置信息
        let loc = self.location();
        output.push_str(&format!("  \x1b[1;34m-->\x1b[0m {}:{}:{}\n", "<input>", loc.line, loc.column));
        
        // 源码片段（如果提供）
        if let Some(src) = source {
            output.push_str(&self.format_source_snippet(src));
        }
        
        // 详细描述
        output.push_str(&format!("\n{}\n", self.description()));
        
        // 修复建议
        if let Some(suggestion) = self.suggestion() {
            output.push_str(&format!("\n\x1b[1;32m帮助\x1b[0m: {}\n", suggestion));
        }
        
        output
    }
    
    /// 格式化源码片段
    fn format_source_snippet(&self, source: &str) -> String {
        let loc = self.location();
        let lines: Vec<&str> = source.lines().collect();
        
        if loc.line == 0 || loc.line > lines.len() {
            return String::new();
        }
        
        let mut output = String::new();
        let line_num_width = loc.line.to_string().len();
        
        // 显示出错行的前一行（如果存在）
        if loc.line > 1 {
            output.push_str(&format!(
                "{:>width$} | {}\n",
                loc.line - 1,
                lines[loc.line - 2],
                width = line_num_width
            ));
        }
        
        // 显示出错行
        output.push_str(&format!(
            "{:>width$} | {}\n",
            loc.line,
            lines[loc.line - 1],
            width = line_num_width
        ));
        
        // 显示错误指示符
        output.push_str(&format!(
            "{:>width$} | {}{}",
            "",
            " ".repeat(loc.column.saturating_sub(1)),
            "\x1b[1;31m^",
            width = line_num_width
        ));
        
        if loc.length > 1 {
            output.push_str(&"~".repeat(loc.length.saturating_sub(1)));
        }
        output.push_str("\x1b[0m\n");
        
        // 显示出错行的后一行（如果存在）
        if loc.line < lines.len() {
            output.push_str(&format!(
                "{:>width$} | {}\n",
                loc.line + 1,
                lines[loc.line],
                width = line_num_width
            ));
        }
        
        output
    }
}

/// 词法分析错误基类
#[derive(Debug, Clone)]
pub enum LexerError {
    UnterminatedString(UnterminatedStringError),
    InvalidEscapeSequence(InvalidEscapeSequenceError),
    InvalidCharacter(InvalidCharacterError),
    InvalidNumber(InvalidNumberError),
    InvalidUnicodeEscape(InvalidUnicodeEscapeError),
}

impl LexerError {
    pub fn location(&self) -> SourceLocation {
        match self {
            Self::UnterminatedString(e) => e.location(),
            Self::InvalidEscapeSequence(e) => e.location(),
            Self::InvalidCharacter(e) => e.location(),
            Self::InvalidNumber(e) => e.location(),
            Self::InvalidUnicodeEscape(e) => e.location(),
        }
    }
    
    pub fn format(&self, mode: ErrorMode, source: Option<&str>) -> String {
        match mode {
            ErrorMode::Simple => match self {
                Self::UnterminatedString(e) => e.format_simple(),
                Self::InvalidEscapeSequence(e) => e.format_simple(),
                Self::InvalidCharacter(e) => e.format_simple(),
                Self::InvalidNumber(e) => e.format_simple(),
                Self::InvalidUnicodeEscape(e) => e.format_simple(),
            },
            ErrorMode::Detailed => match self {
                Self::UnterminatedString(e) => e.format_detailed(source),
                Self::InvalidEscapeSequence(e) => e.format_detailed(source),
                Self::InvalidCharacter(e) => e.format_detailed(source),
                Self::InvalidNumber(e) => e.format_detailed(source),
                Self::InvalidUnicodeEscape(e) => e.format_detailed(source),
            },
        }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format(ErrorMode::Simple, None))
    }
}

impl std::error::Error for LexerError {}

/// 未闭合的字符串错误
#[derive(Debug, Clone)]
pub struct UnterminatedStringError {
    pub location: SourceLocation,
}

impl UnterminatedStringError {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            location: SourceLocation::single(line, column, offset),
        }
    }
}

impl CompilerError for UnterminatedStringError {
    fn location(&self) -> SourceLocation {
        self.location.clone()
    }
    
    fn error_code(&self) -> &'static str {
        "L001"
    }
    
    fn title(&self) -> String {
        "未闭合的字符串字面量".to_string()
    }
    
    fn description(&self) -> String {
        "字符串字面量必须以双引号(\")开始和结束。".to_string()
    }
    
    fn suggestion(&self) -> Option<String> {
        Some("在字符串末尾添加闭合的双引号 \"".to_string())
    }
}

impl fmt::Display for UnterminatedStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

/// 无效的转义序列错误
#[derive(Debug, Clone)]
pub struct InvalidEscapeSequenceError {
    pub location: SourceLocation,
    pub sequence: String,
}

impl InvalidEscapeSequenceError {
    pub fn new(sequence: String, line: usize, column: usize, offset: usize) -> Self {
        Self {
            location: SourceLocation::new(line, column, offset, sequence.len()),
            sequence,
        }
    }
}

impl CompilerError for InvalidEscapeSequenceError {
    fn location(&self) -> SourceLocation {
        self.location.clone()
    }
    
    fn error_code(&self) -> &'static str {
        "L002"
    }
    
    fn title(&self) -> String {
        format!("无效的转义序列: '{}'", self.sequence)
    }
    
    fn description(&self) -> String {
        format!(
            "转义序列 '{}' 不是有效的转义字符。\n有效的转义序列包括: \\n, \\t, \\r, \\\\, \\\", \\', \\0, \\xHH, \\u{{XXXX}}",
            self.sequence
        )
    }
    
    fn suggestion(&self) -> Option<String> {
        Some("检查转义序列的拼写，或使用raw字符串 r\"...\" 来避免转义".to_string())
    }
}

impl fmt::Display for InvalidEscapeSequenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

/// 无效的字符错误
#[derive(Debug, Clone)]
pub struct InvalidCharacterError {
    pub location: SourceLocation,
    pub character: char,
}

impl InvalidCharacterError {
    pub fn new(character: char, line: usize, column: usize, offset: usize) -> Self {
        Self {
            location: SourceLocation::single(line, column, offset),
            character,
        }
    }
}

impl CompilerError for InvalidCharacterError {
    fn location(&self) -> SourceLocation {
        self.location.clone()
    }
    
    fn error_code(&self) -> &'static str {
        "L003"
    }
    
    fn title(&self) -> String {
        format!("意外的字符: '{}'", self.character)
    }
    
    fn description(&self) -> String {
        format!(
            "字符 '{}' (Unicode: U+{:04X}) 在此处不是有效的语法元素。",
            self.character,
            self.character as u32
        )
    }
    
    fn suggestion(&self) -> Option<String> {
        Some("检查是否有拼写错误或多余的字符".to_string())
    }
}

impl fmt::Display for InvalidCharacterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

/// 无效的数字格式错误
#[derive(Debug, Clone)]
pub struct InvalidNumberError {
    pub location: SourceLocation,
    pub value: String,
}

impl InvalidNumberError {
    pub fn new(value: String, line: usize, column: usize, offset: usize) -> Self {
        Self {
            location: SourceLocation::new(line, column, offset, value.len()),
            value,
        }
    }
}

impl CompilerError for InvalidNumberError {
    fn location(&self) -> SourceLocation {
        self.location.clone()
    }
    
    fn error_code(&self) -> &'static str {
        "L004"
    }
    
    fn title(&self) -> String {
        format!("无效的数字字面量: '{}'", self.value)
    }
    
    fn description(&self) -> String {
        let mut desc = format!("'{}' 不是有效的数字格式。\n", self.value);
        desc.push_str("支持的数字格式:\n");
        desc.push_str("  - 十进制: 123, 45.67, 1.2e10\n");
        desc.push_str("  - 十六进制: 0xFF, 0x1A\n");
        desc.push_str("  - 二进制: 0b1010, 0b11\n");
        desc.push_str("  - 八进制: 0o755, 0o17");
        desc
    }
    
    fn suggestion(&self) -> Option<String> {
        if self.value.starts_with("0x") || self.value.starts_with("0X") {
            Some("十六进制数字后必须跟随至少一个十六进制数字 (0-9, A-F)".to_string())
        } else if self.value.starts_with("0b") || self.value.starts_with("0B") {
            Some("二进制数字后必须跟随至少一个二进制数字 (0-1)".to_string())
        } else if self.value.starts_with("0o") || self.value.starts_with("0O") {
            Some("八进制数字后必须跟随至少一个八进制数字 (0-7)".to_string())
        } else if self.value.contains('e') || self.value.contains('E') {
            Some("科学计数法的指数部分必须是有效的整数".to_string())
        } else {
            Some("检查数字格式是否正确".to_string())
        }
    }
}

impl fmt::Display for InvalidNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

/// 无效的Unicode转义序列错误
#[derive(Debug, Clone)]
pub struct InvalidUnicodeEscapeError {
    pub location: SourceLocation,
    pub sequence: String,
}

impl InvalidUnicodeEscapeError {
    pub fn new(sequence: String, line: usize, column: usize, offset: usize) -> Self {
        Self {
            location: SourceLocation::new(line, column, offset, sequence.len()),
            sequence,
        }
    }
}

impl CompilerError for InvalidUnicodeEscapeError {
    fn location(&self) -> SourceLocation {
        self.location.clone()
    }
    
    fn error_code(&self) -> &'static str {
        "L005"
    }
    
    fn title(&self) -> String {
        format!("无效的Unicode转义序列: '{}'", self.sequence)
    }
    
    fn description(&self) -> String {
        format!(
            "'{}' 不是有效的Unicode转义序列。\nUnicode转义序列格式:\n  - \\uXXXX (固定4位十六进制)\n  - \\u{{X}} 到 \\u{{XXXXXX}} (1-6位十六进制，需要花括号)",
            self.sequence
        )
    }
    
    fn suggestion(&self) -> Option<String> {
        Some("确保Unicode码点是有效的十六进制数字，并且在有效范围内 (U+0000 到 U+10FFFF)".to_string())
    }
}

impl fmt::Display for InvalidUnicodeEscapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

/// 语法分析错误
#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken(UnexpectedTokenError),
    UnexpectedEOF(UnexpectedEOFError),
    InvalidExpression(InvalidExpressionError),
}

impl ParseError {
    pub fn location(&self) -> SourceLocation {
        match self {
            Self::UnexpectedToken(e) => e.location(),
            Self::UnexpectedEOF(e) => e.location(),
            Self::InvalidExpression(e) => e.location(),
        }
    }
    
    pub fn format(&self, mode: ErrorMode, source: Option<&str>) -> String {
        match mode {
            ErrorMode::Simple => match self {
                Self::UnexpectedToken(e) => e.format_simple(),
                Self::UnexpectedEOF(e) => e.format_simple(),
                Self::InvalidExpression(e) => e.format_simple(),
            },
            ErrorMode::Detailed => match self {
                Self::UnexpectedToken(e) => e.format_detailed(source),
                Self::UnexpectedEOF(e) => e.format_detailed(source),
                Self::InvalidExpression(e) => e.format_detailed(source),
            },
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format(ErrorMode::Simple, None))
    }
}

impl std::error::Error for ParseError {}

/// 意外的token错误
#[derive(Debug, Clone)]
pub struct UnexpectedTokenError {
    pub location: SourceLocation,
    pub expected: String,
    pub found: String,
}

impl UnexpectedTokenError {
    pub fn new(expected: String, found: String, line: usize, column: usize, offset: usize, length: usize) -> Self {
        Self {
            location: SourceLocation::new(line, column, offset, length),
            expected,
            found,
        }
    }
}

impl CompilerError for UnexpectedTokenError {
    fn location(&self) -> SourceLocation {
        self.location.clone()
    }
    
    fn error_code(&self) -> &'static str {
        "P001"
    }
    
    fn title(&self) -> String {
        format!("意外的token")
    }
    
    fn description(&self) -> String {
        format!("期望 {}, 但发现 {}", self.expected, self.found)
    }
    
    fn suggestion(&self) -> Option<String> {
        Some(format!("在此处添加或修改为 {}", self.expected))
    }
}

impl fmt::Display for UnexpectedTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

/// 意外的文件结束错误
#[derive(Debug, Clone)]
pub struct UnexpectedEOFError {
    pub location: SourceLocation,
}

impl UnexpectedEOFError {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            location: SourceLocation::single(line, column, offset),
        }
    }
}

impl CompilerError for UnexpectedEOFError {
    fn location(&self) -> SourceLocation {
        self.location.clone()
    }
    
    fn error_code(&self) -> &'static str {
        "P002"
    }
    
    fn title(&self) -> String {
        "意外的文件结束".to_string()
    }
    
    fn description(&self) -> String {
        "在解析完成之前遇到了文件结束。".to_string()
    }
    
    fn suggestion(&self) -> Option<String> {
        Some("检查是否有未闭合的括号、引号或代码块".to_string())
    }
}

impl fmt::Display for UnexpectedEOFError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

/// 无效的表达式错误
#[derive(Debug, Clone)]
pub struct InvalidExpressionError {
    pub location: SourceLocation,
}

impl InvalidExpressionError {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            location: SourceLocation::single(line, column, offset),
        }
    }
}

impl CompilerError for InvalidExpressionError {
    fn location(&self) -> SourceLocation {
        self.location.clone()
    }
    
    fn error_code(&self) -> &'static str {
        "P003"
    }
    
    fn title(&self) -> String {
        "无效的表达式".to_string()
    }
    
    fn description(&self) -> String {
        "此处需要一个有效的表达式。".to_string()
    }
    
    fn suggestion(&self) -> Option<String> {
        Some("检查表达式语法是否正确".to_string())
    }
}

impl fmt::Display for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}