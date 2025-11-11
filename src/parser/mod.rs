use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp, Type, Parameter};
use crate::lexer::token::{Token, TokenType, Position};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: TokenType,
    },
    UnexpectedEOF,
    InvalidExpression,
}

type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn current_token(&self) -> Token {
        self.tokens.get(self.current)
            .cloned()
            .unwrap_or_else(|| {
                let pos = Position::new(0, 0, 0);
                Token::new(TokenType::EOF, String::new(), pos.clone(), pos)
            })
    }

    fn peek(&self, offset: usize) -> Token {
        self.tokens.get(self.current + offset)
            .cloned()
            .unwrap_or_else(|| {
                let pos = Position::new(0, 0, 0);
                Token::new(TokenType::EOF, String::new(), pos.clone(), pos)
            })
    }

    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1).unwrap()
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.current_token().token_type == token_type
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> ParseResult<Token> {
        if self.check(token_type) {
            Ok(self.advance().clone())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: message.to_string(),
                found: self.current_token().token_type.clone(),
            })
        }
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut program = Program::new();

        while !self.check(TokenType::EOF) {
            let stmt = self.declaration()?;
            program.add_statement(stmt);
        }

        Ok(program)
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        if self.match_token(&[TokenType::Let, TokenType::Var]) {
            self.var_declaration()
        } else if self.match_token(&[TokenType::Fn]) {
            self.fn_declaration()
        } else if self.match_token(&[TokenType::Struct]) {
            self.struct_declaration()
        } else if self.match_token(&[TokenType::Type]) {
            self.type_alias_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let is_mutable = self.tokens.get(self.current.saturating_sub(1))
            .map(|t| t.token_type == TokenType::Var)
            .unwrap_or(false);

        let name_token = self.consume(TokenType::Identifier, "Expected variable name")?;
        let name = name_token.value.clone();

        // 解析可选的类型注解
        let type_annotation = if self.match_token(&[TokenType::Colon]) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration")?;

        Ok(Stmt::VarDeclaration {
            name,
            mutable: is_mutable,
            type_annotation,
            initializer,
        })
    }

    fn fn_declaration(&mut self) -> ParseResult<Stmt> {
        let name_token = self.consume(TokenType::Identifier, "Expected function name")?;
        let name = name_token.value.clone();

        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = self.consume(TokenType::Identifier, "Expected parameter name")?;
                
                // 解析可选的类型注解
                let type_annotation = if self.match_token(&[TokenType::Colon]) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                
                parameters.push(Parameter {
                    name: param_name.value.clone(),
                    type_annotation,
                });

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;
        
        // 解析可选的返回类型
        let return_type = if self.match_token(&[TokenType::Arrow]) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.consume(TokenType::LeftBrace, "Expected '{' before function body")?;

        let mut body = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
            body.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after function body")?;

        Ok(Stmt::FnDeclaration {
            name,
            parameters,
            return_type,
            body,
        })
    }
    
    fn struct_declaration(&mut self) -> ParseResult<Stmt> {
        let name_token = self.consume(TokenType::Identifier, "Expected struct name")?;
        let name = name_token.value.clone();
        
        self.consume(TokenType::LeftBrace, "Expected '{' after struct name")?;
        
        let mut fields = Vec::new();
        
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
            let field_name_token = self.consume(TokenType::Identifier, "Expected field name")?;
            let field_name = field_name_token.value.clone();
            
            self.consume(TokenType::Colon, "Expected ':' after field name")?;
            
            let field_type = self.parse_type()?;
            
            fields.push(crate::ast::StructField {
                name: field_name,
                field_type,
            });
            
            // 允许可选的逗号
            if self.match_token(&[TokenType::Comma]) {
                // 继续
            } else {
                break;
            }
        }
        
        self.consume(TokenType::RightBrace, "Expected '}' after struct fields")?;
        self.consume(TokenType::Semicolon, "Expected ';' after struct declaration")?;
        
        Ok(Stmt::StructDeclaration { name, fields })
    }
    
    fn type_alias_declaration(&mut self) -> ParseResult<Stmt> {
        let name_token = self.consume(TokenType::Identifier, "Expected type alias name")?;
        let name = name_token.value.clone();
        
        self.consume(TokenType::Equal, "Expected '=' after type alias name")?;
        
        // 检查是否是匿名结构体
        let target_type = if self.match_token(&[TokenType::Struct]) {
            self.consume(TokenType::LeftBrace, "Expected '{' after 'struct'")?;
            
            let mut fields = Vec::new();
            
            while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
                let field_name_token = self.consume(TokenType::Identifier, "Expected field name")?;
                let field_name = field_name_token.value.clone();
                
                self.consume(TokenType::Colon, "Expected ':' after field name")?;
                
                let field_type = self.parse_type()?;
                
                fields.push(crate::ast::StructField {
                    name: field_name,
                    field_type,
                });
                
                // 允许可选的逗号
                if self.match_token(&[TokenType::Comma]) {
                    // 继续
                } else {
                    break;
                }
            }
            
            self.consume(TokenType::RightBrace, "Expected '}' after struct fields")?;
            
            Type::Struct(crate::ast::StructType {
                name: format!("anonymous_{}", name),
                fields,
            })
        } else {
            // 普通类型别名
            let type_name_token = self.consume(TokenType::Identifier, "Expected type name")?;
            Type::Named(type_name_token.value.clone())
        };
        
        self.consume(TokenType::Semicolon, "Expected ';' after type alias")?;
        
        Ok(Stmt::TypeAlias { name, target_type })
    }
    
    fn parse_type(&mut self) -> ParseResult<Type> {
        // 检查数组类型 [element_type]
        if self.check(TokenType::LeftBracket) {
            self.advance(); // 消费 '['
            let element_type = self.parse_type()?;
            self.consume(TokenType::RightBracket, "Expected ']' after array element type")?;
            return Ok(Type::Array(Box::new(element_type)));
        }
        
        // 检查匿名结构体类型
        if self.match_token(&[TokenType::Struct]) {
            self.consume(TokenType::LeftBrace, "Expected '{' after 'struct'")?;
            
            let mut fields = Vec::new();
            
            while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
                let field_name_token = self.consume(TokenType::Identifier, "Expected field name")?;
                let field_name = field_name_token.value.clone();
                
                self.consume(TokenType::Colon, "Expected ':' after field name")?;
                
                let field_type = self.parse_type()?;
                
                fields.push(crate::ast::StructField {
                    name: field_name,
                    field_type,
                });
                
                if self.match_token(&[TokenType::Comma]) {
                    // 继续
                } else {
                    break;
                }
            }
            
            self.consume(TokenType::RightBrace, "Expected '}' after struct fields")?;
            
            return Ok(Type::Struct(crate::ast::StructType {
                name: "anonymous".to_string(),
                fields,
            }));
        }
        
        let token = self.current_token();
        match token.token_type {
            TokenType::Int => {
                self.advance();
                Ok(Type::Int)
            }
            TokenType::Float64 => {
                self.advance();
                Ok(Type::Float)
            }
            TokenType::String64 => {
                self.advance();
                Ok(Type::String)
            }
            TokenType::Bool => {
                self.advance();
                Ok(Type::Bool)
            }
            TokenType::Void => {
                self.advance();
                Ok(Type::Void)
            }
            TokenType::Null => {
                self.advance();
                Ok(Type::Null)
            }
            TokenType::Identifier => {
                // 用户定义的类型（结构体名或类型别名）
                let type_name = token.value.clone();
                self.advance();
                Ok(Type::Named(type_name))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "type name".to_string(),
                found: token.token_type.clone(),
            }),
        }
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_token(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_token(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_token(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(&[TokenType::LeftBrace]) {
            self.block_statement()
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';' after return value")?;

        Ok(Stmt::Return { value })
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        let condition = self.expression()?;

        self.consume(TokenType::LeftBrace, "Expected '{' after if condition")?;

        let mut then_branch = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
            then_branch.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after then branch")?;

        let else_branch = if self.match_token(&[TokenType::Else]) {
            self.consume(TokenType::LeftBrace, "Expected '{' after else")?;

            let mut else_stmts = Vec::new();
            while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
                else_stmts.push(self.declaration()?);
            }

            self.consume(TokenType::RightBrace, "Expected '}' after else branch")?;
            Some(else_stmts)
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        let condition = self.expression()?;

        self.consume(TokenType::LeftBrace, "Expected '{' after while condition")?;

        let mut body = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
            body.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after while body")?;

        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> ParseResult<Stmt> {
        let var_token = self.consume(TokenType::Identifier, "Expected variable name")?;
        let variable = var_token.value.clone();

        self.consume(TokenType::In, "Expected 'in' after loop variable")?;

        let start = self.expression()?;

        self.consume(TokenType::DotDot, "Expected '..' in range")?;

        let end = self.expression()?;

        self.consume(TokenType::LeftBrace, "Expected '{' after for range")?;

        let mut body = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
            body.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after for body")?;

        Ok(Stmt::For {
            variable,
            start,
            end,
            body,
        })
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'print'")?;
        let value = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after print value")?;
        self.consume(TokenType::Semicolon, "Expected ';' after print statement")?;

        Ok(Stmt::Print { value })
    }

    fn block_statement(&mut self) -> ParseResult<Stmt> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block")?;

        Ok(Stmt::Block { statements })
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Equal]) {
            match expr {
                Expr::Identifier(name) => {
                    let value = self.assignment()?;
                    return Ok(Expr::assign(name, value));
                }
                Expr::Index { object, index } => {
                    let value = self.assignment()?;
                    return Ok(Expr::index_assign(*object, *index, value));
                }
                Expr::FieldAccess { object, field } => {
                    let value = self.assignment()?;
                    return Ok(Expr::field_assign(*object, field, value));
                }
                _ => {}
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::Or]) {
            let right = self.and()?;
            expr = Expr::binary(expr, BinaryOp::Or, right);
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let right = self.equality()?;
            expr = Expr::binary(expr, BinaryOp::And, right);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let op = match self.tokens.get(self.current.saturating_sub(1))
                .map(|t| &t.token_type)
                .unwrap() {
                TokenType::EqualEqual => BinaryOp::Equal,
                TokenType::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expr::binary(expr, op, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = match self.tokens.get(self.current.saturating_sub(1))
                .map(|t| &t.token_type)
                .unwrap() {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::binary(expr, op, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let op = match self.tokens.get(self.current.saturating_sub(1))
                .map(|t| &t.token_type)
                .unwrap() {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expr::binary(expr, op, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let op = match self.tokens.get(self.current.saturating_sub(1))
                .map(|t| &t.token_type)
                .unwrap() {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expr::binary(expr, op, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let op = match self.tokens.get(self.current.saturating_sub(1))
                .map(|t| &t.token_type)
                .unwrap() {
                TokenType::Bang => UnaryOp::Not,
                TokenType::Minus => UnaryOp::Negate,
                _ => unreachable!(),
            };
            let operand = self.unary()?;
            return Ok(Expr::unary(op, operand));
        }

        self.call()
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::LeftBracket]) {
                let index = self.expression()?;
                self.consume(TokenType::RightBracket, "Expected ']' after index")?;
                expr = Expr::index(expr, index);
            } else if self.match_token(&[TokenType::Dot]) {
                // 字段访问
                let field_token = self.consume(TokenType::Identifier, "Expected field name after '.'")?;
                let field = field_token.value.clone();
                expr = Expr::field_access(expr, field);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut arguments = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;

        Ok(Expr::call(callee, arguments))
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::boolean(true));
        }

        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::boolean(false));
        }

        if self.match_token(&[TokenType::Integer]) {
            let value = self.tokens.get(self.current.saturating_sub(1))
                .unwrap().value.parse::<i64>().unwrap();
            return Ok(Expr::integer(value));
        }

        if self.match_token(&[TokenType::Float]) {
            let value = self.tokens.get(self.current.saturating_sub(1))
                .unwrap().value.parse::<f64>().unwrap();
            return Ok(Expr::float(value));
        }

        if self.match_token(&[TokenType::String]) {
            let value = self.tokens.get(self.current.saturating_sub(1))
                .unwrap().value.clone();
            return Ok(Expr::string(value));
        }

        if self.match_token(&[TokenType::Identifier]) {
            let name = self.tokens.get(self.current.saturating_sub(1))
                .unwrap().value.clone();
            
            // 检查是否是结构体字面量 StructName { field: value, ... }
            if self.check(TokenType::LeftBrace) {
                self.advance(); // 消费 '{'
                
                let mut fields = Vec::new();
                
                while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
                    let field_name_token = self.consume(TokenType::Identifier, "Expected field name")?;
                    let field_name = field_name_token.value.clone();
                    
                    self.consume(TokenType::Colon, "Expected ':' after field name")?;
                    
                    let field_value = self.expression()?;
                    
                    fields.push((field_name, field_value));
                    
                    if self.match_token(&[TokenType::Comma]) {
                        // 继续
                    } else {
                        break;
                    }
                }
                
                self.consume(TokenType::RightBrace, "Expected '}' after struct fields")?;
                
                return Ok(Expr::struct_literal(name, fields));
            }
            
            return Ok(Expr::identifier(name));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        // 数组字面量 [elem1, elem2, ...]
        if self.match_token(&[TokenType::LeftBracket]) {
            let mut elements = Vec::new();
            
            if !self.check(TokenType::RightBracket) {
                loop {
                    elements.push(self.expression()?);
                    
                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                }
            }
            
            self.consume(TokenType::RightBracket, "Expected ']' after array elements")?;
            return Ok(Expr::array(elements));
        }

        Err(ParseError::InvalidExpression)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_variable_declaration() {
        let mut lexer = Lexer::new("let x = 42;".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_function() {
        let mut lexer = Lexer::new("fn add(a, b) { return a + b; }".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_expression() {
        let mut lexer = Lexer::new("2 + 3 * 4;".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.statements.len(), 1);
    }
}