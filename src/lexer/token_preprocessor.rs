use super::token::{Token, TokenType};

/// 推断的数值类型
#[derive(Debug, Clone, PartialEq)]
pub enum InferredNumericType {
    Int64,
    Float64,
}

/// 科学计数法分析器
pub struct ScientificNotationAnalyzer;

impl ScientificNotationAnalyzer {
    /// 分析科学计数法表达式并推断类型
    pub fn analyze(value: &str) -> InferredNumericType {
        // 解析科学计数法 (例如: "1e10", "3.14e-5", "2.5e+3")
        if let Some((base, exp)) = value.split_once(|c| c == 'e' || c == 'E') {
            let has_decimal = base.contains('.');
            
            // 如果基数有小数点，一定是浮点数
            if has_decimal {
                return InferredNumericType::Float64;
            }
            
            // 解析指数
            if let Ok(exponent) = exp.parse::<i32>() {
                // 如果指数为负，结果一定是小数
                if exponent < 0 {
                    return InferredNumericType::Float64;
                }
                
                // 尝试计算结果范围
                if let Ok(base_num) = base.parse::<i64>() {
                    // 检查是否会溢出i64
                    let result = (base_num as f64) * 10_f64.powi(exponent);
                    if result > i64::MAX as f64 || result < i64::MIN as f64 {
                        return InferredNumericType::Float64;
                    }
                    
                    // 如果指数太大，可能是浮点数
                    if exponent > 18 {
                        return InferredNumericType::Float64;
                    }
                    
                    return InferredNumericType::Int64;
                }
            }
        }
        
        // 默认为浮点数
        InferredNumericType::Float64
    }
}

/// Token预处理器
pub struct TokenPreprocessor;

impl TokenPreprocessor {
    /// 预处理token列表，转换科学计数法
    pub fn preprocess(tokens: Vec<Token>) -> Vec<Token> {
        tokens
            .into_iter()
            .map(|token| Self::preprocess_token(token))
            .collect()
    }

    /// 预处理单个token
    fn preprocess_token(token: Token) -> Token {
        match token.token_type {
            TokenType::ScientificExponent => {
                let inferred_type = ScientificNotationAnalyzer::analyze(&token.value);
                let new_type = match inferred_type {
                    InferredNumericType::Int64 => TokenType::Integer,
                    InferredNumericType::Float64 => TokenType::Float,
                };
                
                Token::new(new_type, token.value, token.start_pos, token.end_pos)
            }
            _ => token,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scientific_notation_int() {
        assert_eq!(
            ScientificNotationAnalyzer::analyze("1e10"),
            InferredNumericType::Int64
        );
        assert_eq!(
            ScientificNotationAnalyzer::analyze("2e5"),
            InferredNumericType::Int64
        );
    }

    #[test]
    fn test_scientific_notation_float() {
        assert_eq!(
            ScientificNotationAnalyzer::analyze("3.14e-5"),
            InferredNumericType::Float64
        );
        assert_eq!(
            ScientificNotationAnalyzer::analyze("1e-10"),
            InferredNumericType::Float64
        );
        assert_eq!(
            ScientificNotationAnalyzer::analyze("2.5e3"),
            InferredNumericType::Float64
        );
    }

    #[test]
    fn test_large_exponent() {
        assert_eq!(
            ScientificNotationAnalyzer::analyze("1e20"),
            InferredNumericType::Float64
        );
    }
}