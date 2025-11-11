//! Zero编译器错误处理系统
//! 
//! 这个模块提供了解耦的错误处理架构：
//! 1. 错误定义（纯数据，enum）
//! 2. 错误收集器（收集所有错误）
//! 3. 错误展示器（负责格式化和输出）
//! 4. 错误消息配置（从TOML文件加载）

use std::collections::HashMap;
use std::fmt;
use serde::Deserialize;

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

/// 编译器错误 - 纯数据结构
#[derive(Debug, Clone)]
pub struct CompilerError {
    /// 错误代码（如 "L001", "P002"）
    pub code: String,
    /// 错误位置
    pub location: SourceLocation,
    /// 错误类型（用于查找配置）
    pub error_type: ErrorType,
    /// 动态参数（用于替换模板中的占位符）
    pub params: HashMap<String, String>,
}

impl CompilerError {
    pub fn new(code: impl Into<String>, location: SourceLocation, error_type: ErrorType) -> Self {
        Self {
            code: code.into(),
            location,
            error_type,
            params: HashMap::new(),
        }
    }
    
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

/// 错误类型枚举 - 仅用于分类，不包含具体消息
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    // 词法错误
    LexerUnterminatedString,
    LexerInvalidEscapeSequence,
    LexerInvalidCharacter,
    LexerInvalidNumber,
    LexerInvalidUnicodeEscape,
    
    // 语法错误
    ParserUnexpectedToken,
    ParserUnexpectedEOF,
    ParserInvalidExpression,
    
    // 类型检查错误
    TypeCheckerTypeMismatch,
    TypeCheckerUndefinedVariable,
    
    // 编译器错误
    CompilerError,
    
    // 运行时错误
    RuntimeError,
}

impl ErrorType {
    /// 获取错误代码
    pub fn code(&self) -> &'static str {
        match self {
            Self::LexerUnterminatedString => "L001",
            Self::LexerInvalidEscapeSequence => "L002",
            Self::LexerInvalidCharacter => "L003",
            Self::LexerInvalidNumber => "L004",
            Self::LexerInvalidUnicodeEscape => "L005",
            Self::ParserUnexpectedToken => "P001",
            Self::ParserUnexpectedEOF => "P002",
            Self::ParserInvalidExpression => "P003",
            Self::TypeCheckerTypeMismatch => "T001",
            Self::TypeCheckerUndefinedVariable => "T002",
            Self::CompilerError => "C001",
            Self::RuntimeError => "R001",
        }
    }
    
    /// 获取配置键
    pub fn config_key(&self) -> &'static str {
        match self {
            Self::LexerUnterminatedString => "lexer.L001",
            Self::LexerInvalidEscapeSequence => "lexer.L002",
            Self::LexerInvalidCharacter => "lexer.L003",
            Self::LexerInvalidNumber => "lexer.L004",
            Self::LexerInvalidUnicodeEscape => "lexer.L005",
            Self::ParserUnexpectedToken => "parser.P001",
            Self::ParserUnexpectedEOF => "parser.P002",
            Self::ParserInvalidExpression => "parser.P003",
            Self::TypeCheckerTypeMismatch => "type_checker.T001",
            Self::TypeCheckerUndefinedVariable => "type_checker.T002",
            Self::CompilerError => "compiler.C001",
            Self::RuntimeError => "vm.R001",
        }
    }
}

/// 错误消息配置（从TOML加载）
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMessageConfig {
    pub code: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub suggestion: Option<String>,
    #[serde(default)]
    pub suggestion_hex: Option<String>,
    #[serde(default)]
    pub suggestion_bin: Option<String>,
    #[serde(default)]
    pub suggestion_oct: Option<String>,
    #[serde(default)]
    pub suggestion_exp: Option<String>,
    #[serde(default)]
    pub suggestion_default: Option<String>,
    pub category: String,
}

/// 错误消息注册表
#[derive(Debug)]
pub struct ErrorRegistry {
    messages: HashMap<String, ErrorMessageConfig>,
}

impl ErrorRegistry {
    /// 从TOML配置创建注册表
    pub fn from_toml(toml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct Config {
            lexer: Option<HashMap<String, ErrorMessageConfig>>,
            parser: Option<HashMap<String, ErrorMessageConfig>>,
            type_checker: Option<HashMap<String, ErrorMessageConfig>>,
            compiler: Option<HashMap<String, ErrorMessageConfig>>,
            vm: Option<HashMap<String, ErrorMessageConfig>>,
        }
        
        let config: Config = toml::from_str(toml_str)?;
        let mut messages = HashMap::new();
        
        // 收集所有类别的错误消息
        for (category, map) in [
            ("lexer", config.lexer),
            ("parser", config.parser),
            ("type_checker", config.type_checker),
            ("compiler", config.compiler),
            ("vm", config.vm),
        ] {
            if let Some(map) = map {
                for (key, msg) in map {
                    messages.insert(format!("{}.{}", category, key), msg);
                }
            }
        }
        
        Ok(Self { messages })
    }
    
    /// 获取错误消息配置
    pub fn get(&self, key: &str) -> Option<&ErrorMessageConfig> {
        self.messages.get(key)
    }
    
    /// 创建默认注册表（从submodule加载配置）
    pub fn default() -> Self {
        // 优先从submodule加载中文错误消息
        const DEFAULT_CONFIG: &str = include_str!("../../error-msg/locale/zh_CN/error_messages.toml");
        Self::from_toml(DEFAULT_CONFIG).expect("Failed to load error messages from submodule")
    }
    
    /// 从指定语言加载错误消息
    pub fn from_locale(locale: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = format!("error-msg/locale/{}/error_messages.toml", locale);
        let config_str = std::fs::read_to_string(&path)?;
        Self::from_toml(&config_str)
    }
}

impl Default for ErrorRegistry {
    fn default() -> Self {
        Self::default()
    }
}

/// 错误收集器 - 收集编译过程中的所有错误
#[derive(Debug)]
pub struct ErrorCollector {
    errors: Vec<CompilerError>,
    max_errors: usize,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            max_errors: 100,
        }
    }
    
    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.max_errors = max;
        self
    }
    
    /// 添加一个错误
    pub fn add(&mut self, error: CompilerError) {
        if self.errors.len() < self.max_errors {
            self.errors.push(error);
        }
    }
    
    /// 是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// 错误数量
    pub fn count(&self) -> usize {
        self.errors.len()
    }
    
    /// 获取所有错误
    pub fn errors(&self) -> &[CompilerError] {
        &self.errors
    }
    
    /// 清空错误
    pub fn clear(&mut self) {
        self.errors.clear();
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 错误展示器 - 负责格式化和输出错误
pub struct ErrorDisplayer {
    registry: ErrorRegistry,
    mode: ErrorMode,
}

impl ErrorDisplayer {
    pub fn new(mode: ErrorMode) -> Self {
        Self {
            registry: ErrorRegistry::default(),
            mode,
        }
    }
    
    pub fn with_registry(mut self, registry: ErrorRegistry) -> Self {
        self.registry = registry;
        self
    }
    
    /// 格式化单个错误
    pub fn format_error(&self, error: &CompilerError, source: Option<&str>) -> String {
        match self.mode {
            ErrorMode::Simple => self.format_simple(error),
            ErrorMode::Detailed => self.format_detailed(error, source),
        }
    }
    
    /// 格式化所有错误
    pub fn format_errors(&self, errors: &[CompilerError], source: Option<&str>) -> String {
        errors
            .iter()
            .map(|e| self.format_error(e, source))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
    
    /// 简易模式格式化
    fn format_simple(&self, error: &CompilerError) -> String {
        let config = self.registry.get(error.error_type.config_key());
        let title = config
            .map(|c| Self::replace_params(&c.title, &error.params))
            .unwrap_or_else(|| "未知错误".to_string());
        
        format!(
            "错误 [{}] 在 {}:{}: {}",
            error.code,
            error.location.line,
            error.location.column,
            title
        )
    }
    
    /// 详细模式格式化
    fn format_detailed(&self, error: &CompilerError, source: Option<&str>) -> String {
        let config = self.registry.get(error.error_type.config_key());
        
        let title = config
            .map(|c| Self::replace_params(&c.title, &error.params))
            .unwrap_or_else(|| "未知错误".to_string());
        
        let description = config
            .map(|c| Self::replace_params(&c.description, &error.params))
            .unwrap_or_default();
        
        let suggestion = config.and_then(|c| {
            // 根据参数选择合适的建议
            if let Some(value) = error.params.get("value") {
                if value.starts_with("0x") || value.starts_with("0X") {
                    c.suggestion_hex.as_ref()
                } else if value.starts_with("0b") || value.starts_with("0B") {
                    c.suggestion_bin.as_ref()
                } else if value.starts_with("0o") || value.starts_with("0O") {
                    c.suggestion_oct.as_ref()
                } else if value.contains('e') || value.contains('E') {
                    c.suggestion_exp.as_ref()
                } else {
                    c.suggestion.as_ref().or(c.suggestion_default.as_ref())
                }
            } else {
                c.suggestion.as_ref()
            }
        });
        
        let mut output = String::new();
        
        // 错误标题
        output.push_str(&format!("\x1b[1;31merror[{}]\x1b[0m: {}\n", error.code, title));
        
        // 位置信息
        let loc = &error.location;
        output.push_str(&format!("  \x1b[1;34m-->\x1b[0m {}:{}:{}\n", "<input>", loc.line, loc.column));
        
        // 源码片段
        if let Some(src) = source {
            output.push_str(&self.format_source_snippet(src, &error.location));
        }
        
        // 详细描述
        if !description.is_empty() {
            output.push_str(&format!("\n{}\n", description));
        }
        
        // 修复建议
        if let Some(sug) = suggestion {
            let sug = Self::replace_params(sug, &error.params);
            output.push_str(&format!("\n\x1b[1;32m帮助\x1b[0m: {}\n", sug));
        }
        
        output
    }
    
    /// 格式化源码片段
    fn format_source_snippet(&self, source: &str, location: &SourceLocation) -> String {
        let lines: Vec<&str> = source.lines().collect();
        
        if location.line == 0 || location.line > lines.len() {
            return String::new();
        }
        
        let mut output = String::new();
        let line_num_width = location.line.to_string().len();
        
        // 显示出错行的前一行
        if location.line > 1 {
            output.push_str(&format!(
                "{:>width$} | {}\n",
                location.line - 1,
                lines[location.line - 2],
                width = line_num_width
            ));
        }
        
        // 显示出错行
        output.push_str(&format!(
            "{:>width$} | {}\n",
            location.line,
            lines[location.line - 1],
            width = line_num_width
        ));
        
        // 显示错误指示符
        output.push_str(&format!(
            "{:>width$} | {}{}",
            "",
            " ".repeat(location.column.saturating_sub(1)),
            "\x1b[1;31m^",
            width = line_num_width
        ));
        
        if location.length > 1 {
            output.push_str(&"~".repeat(location.length.saturating_sub(1)));
        }
        output.push_str("\x1b[0m\n");
        
        // 显示出错行的后一行
        if location.line < lines.len() {
            output.push_str(&format!(
                "{:>width$} | {}\n",
                location.line + 1,
                lines[location.line],
                width = line_num_width
            ));
        }
        
        output
    }
    
    /// 替换消息模板中的参数
    fn replace_params(template: &str, params: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in params {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }
}

// 为了向后兼容，保留旧的错误类型别名
pub type LexerError = CompilerError;
pub type ParseError = CompilerError;

// 便捷构造函数
impl CompilerError {
    pub fn unterminated_string(line: usize, column: usize, offset: usize) -> Self {
        Self::new(
            "L001",
            SourceLocation::single(line, column, offset),
            ErrorType::LexerUnterminatedString,
        )
    }
    
    pub fn invalid_escape_sequence(sequence: String, line: usize, column: usize, offset: usize) -> Self {
        Self::new(
            "L002",
            SourceLocation::new(line, column, offset, sequence.len()),
            ErrorType::LexerInvalidEscapeSequence,
        )
        .with_param("sequence", sequence)
    }
    
    pub fn invalid_character(ch: char, line: usize, column: usize, offset: usize) -> Self {
        Self::new(
            "L003",
            SourceLocation::single(line, column, offset),
            ErrorType::LexerInvalidCharacter,
        )
        .with_param("character", ch.to_string())
        .with_param("unicode", format!("{:04X}", ch as u32))
    }
    
    pub fn invalid_number(value: String, line: usize, column: usize, offset: usize) -> Self {
        Self::new(
            "L004",
            SourceLocation::new(line, column, offset, value.len()),
            ErrorType::LexerInvalidNumber,
        )
        .with_param("value", value)
    }
    
    pub fn invalid_unicode_escape(sequence: String, line: usize, column: usize, offset: usize) -> Self {
        Self::new(
            "L005",
            SourceLocation::new(line, column, offset, sequence.len()),
            ErrorType::LexerInvalidUnicodeEscape,
        )
        .with_param("sequence", sequence)
    }
    
    pub fn unexpected_token(expected: String, found: String, line: usize, column: usize, offset: usize, length: usize) -> Self {
        Self::new(
            "P001",
            SourceLocation::new(line, column, offset, length),
            ErrorType::ParserUnexpectedToken,
        )
        .with_param("expected", expected)
        .with_param("found", found)
    }
    
    pub fn unexpected_eof(line: usize, column: usize, offset: usize) -> Self {
        Self::new(
            "P002",
            SourceLocation::single(line, column, offset),
            ErrorType::ParserUnexpectedEOF,
        )
    }
    
    pub fn invalid_expression(line: usize, column: usize, offset: usize) -> Self {
        Self::new(
            "P003",
            SourceLocation::single(line, column, offset),
            ErrorType::ParserInvalidExpression,
        )
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let displayer = ErrorDisplayer::new(ErrorMode::Simple);
        write!(f, "{}", displayer.format_error(self, None))
    }
}

impl std::error::Error for CompilerError {}