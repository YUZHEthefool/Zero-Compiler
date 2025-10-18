pub mod token;

use token::{Token, TokenType};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Lexer {
            input: chars,
            position: 0,
            current_char,
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.current_char == Some('/') && self.peek(1) == Some('/') {
            while self.current_char.is_some() && self.current_char != Some('\n') {
                self.advance();
            }
            self.advance();
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        let mut has_dot = false;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot && self.peek(1).map_or(false, |c| c.is_ascii_digit()) {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let value: String = self.input[start..self.position].iter().collect();
        
        if has_dot {
            Token::new(TokenType::Float, value)
        } else {
            Token::new(TokenType::Integer, value)
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let value: String = self.input[start..self.position].iter().collect();
        let token_type = TokenType::get_keyword(&value).unwrap_or(TokenType::Identifier);
        
        Token::new(token_type, value)
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // 跳过开始的引号
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch == '"' {
                break;
            }
            if ch == '\\' {
                self.advance(); // 跳过转义字符
            }
            self.advance();
        }

        let value: String = self.input[start..self.position].iter().collect();
        self.advance(); // 跳过结束的引号

        Token::new(TokenType::String, value)
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            self.skip_whitespace();

            if self.current_char == Some('/') && self.peek(1) == Some('/') {
                self.skip_comment();
                continue;
            }

            break;
        }

        match self.current_char {
            None => Token::new(TokenType::EOF, String::new()),
            Some(ch) => {
                if ch.is_ascii_digit() {
                    return self.read_number();
                }

                if ch.is_alphabetic() || ch == '_' {
                    return self.read_identifier();
                }

                if ch == '"' {
                    return self.read_string();
                }

                let token = match ch {
                    '+' => Token::new(TokenType::Plus, ch.to_string()),
                    '-' => Token::new(TokenType::Minus, ch.to_string()),
                    '*' => Token::new(TokenType::Star, ch.to_string()),
                    '/' => Token::new(TokenType::Slash, ch.to_string()),
                    '%' => Token::new(TokenType::Percent, ch.to_string()),
                    '=' => {
                        if self.peek(1) == Some('=') {
                            self.advance();
                            Token::new(TokenType::EqualEqual, "==".to_string())
                        } else {
                            Token::new(TokenType::Equal, ch.to_string())
                        }
                    }
                    '!' => {
                        if self.peek(1) == Some('=') {
                            self.advance();
                            Token::new(TokenType::BangEqual, "!=".to_string())
                        } else {
                            Token::new(TokenType::Bang, ch.to_string())
                        }
                    }
                    '<' => {
                        if self.peek(1) == Some('=') {
                            self.advance();
                            Token::new(TokenType::LessEqual, "<=".to_string())
                        } else {
                            Token::new(TokenType::Less, ch.to_string())
                        }
                    }
                    '>' => {
                        if self.peek(1) == Some('=') {
                            self.advance();
                            Token::new(TokenType::GreaterEqual, ">=".to_string())
                        } else {
                            Token::new(TokenType::Greater, ch.to_string())
                        }
                    }
                    '&' => {
                        if self.peek(1) == Some('&') {
                            self.advance();
                            Token::new(TokenType::And, "&&".to_string())
                        } else {
                            Token::new(TokenType::Unknown, ch.to_string())
                        }
                    }
                    '|' => {
                        if self.peek(1) == Some('|') {
                            self.advance();
                            Token::new(TokenType::Or, "||".to_string())
                        } else {
                            Token::new(TokenType::Unknown, ch.to_string())
                        }
                    }
                    '(' => Token::new(TokenType::LeftParen, ch.to_string()),
                    ')' => Token::new(TokenType::RightParen, ch.to_string()),
                    '{' => Token::new(TokenType::LeftBrace, ch.to_string()),
                    '}' => Token::new(TokenType::RightBrace, ch.to_string()),
                    '[' => Token::new(TokenType::LeftBracket, ch.to_string()),
                    ']' => Token::new(TokenType::RightBracket, ch.to_string()),
                    ',' => Token::new(TokenType::Comma, ch.to_string()),
                    ';' => Token::new(TokenType::Semicolon, ch.to_string()),
                    ':' => Token::new(TokenType::Colon, ch.to_string()),
                    '.' => {
                        if self.peek(1) == Some('.') {
                            self.advance();
                            Token::new(TokenType::DotDot, "..".to_string())
                        } else {
                            Token::new(TokenType::Dot, ch.to_string())
                        }
                    }
                    _ => Token::new(TokenType::Unknown, ch.to_string()),
                };

                self.advance();
                token
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token();
            let is_eof = matches!(token.token_type, TokenType::EOF);
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_numbers() {
        let mut lexer = Lexer::new("42 3.14".to_string());
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Integer);
        assert_eq!(tokens[0].value, "42");
        assert_eq!(tokens[1].token_type, TokenType::Float);
        assert_eq!(tokens[1].value, "3.14");
    }

    #[test]
    fn test_lexer_keywords() {
        let mut lexer = Lexer::new("let var fn if else".to_string());
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Var);
        assert_eq!(tokens[2].token_type, TokenType::Fn);
        assert_eq!(tokens[3].token_type, TokenType::If);
        assert_eq!(tokens[4].token_type, TokenType::Else);
    }

    #[test]
    fn test_lexer_operators() {
        let mut lexer = Lexer::new("+ - * / == != < > <= >=".to_string());
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Star);
        assert_eq!(tokens[3].token_type, TokenType::Slash);
        assert_eq!(tokens[4].token_type, TokenType::EqualEqual);
        assert_eq!(tokens[5].token_type, TokenType::BangEqual);
    }
}