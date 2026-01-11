use crate::ast::*;
use crate::lexer::{Token, TokenStream};
use crate::utils::{CrabbyError, ErrorLocation};

pub struct Parser {
    tokens: Vec<TokenStream>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenStream>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, CrabbyError> {
        let mut program = Program::new();
        while !self.is_at_end() {
            program.statements.push(self.parse_statement()?);
        }
        Ok(program)
    }

    fn parse_params(&mut self) -> Result<Vec<String>, CrabbyError> {
        self.consume(&Token::LParen, "Expected '(' after function name")?;
        let mut params = Vec::new();

        while !matches!(self.peek().token, Token::RParen) {
            if let Token::Identifier(name) = &self.peek().token {
                params.push(name.clone());
                self.advance();

                if matches!(self.peek().token, Token::Comma) {
                    self.advance();
                }
            } else {
                return Err(self.error("Expected parameter name"));
            }
        }

        self.consume(&Token::RParen, "Expected ')' after parameters")?;
        Ok(params)
    }

    fn parse_statement(&mut self) -> Result<Statement, CrabbyError> {
        match &self.peek().token {
            Token::Loop => self.parse_loop_statement(),
            Token::For => self.parse_for_statement(),
            Token::Import => self.parse_import_statement(),
            Token::Def => self.parse_definition(),
            Token::Function => self.parse_function(),
            Token::Let => self.parse_let_statement(),
            Token::Variable => self.parse_var_statement(),
            Token::Return => {
                self.advance(); // consume 'return'
                let expr = self.parse_expression()?;
                Ok(Statement::Return(Box::new(expr)))
            }
            // Token::Class => self.parse_class_statement(),
            // Token::Trait => self.parse_trait_statement(),
            // Token::Implement => self.parse_impl_statement(),
            // Token::Mutable => parse_mutable_statement(),
            Token::Match => self.parse_match_statement(),
            Token::And => self.parse_and_statement(),
            Token::Enum => self.parse_enum_statement(),
            Token::Struct => self.parse_struct_statement(),
            Token::Where => self.parse_where_statement(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::Async => self.parse_async_statement(),
            // Token::Await => self.parse_await_statement(),
            Token::Identifier(_) => {
                let expr = self.parse_expression()?;

                if matches!(self.peek().token, Token::LBracket) {
                    self.advance(); // consume '['
                    let index = self.parse_expression()?;
                    self.consume(&Token::RBracket, "Expected ']' after array index")?;

                    if matches!(self.peek().token, Token::Equals) {
                        self.advance(); // consume '='
                        let value = self.parse_expression()?;

                        Ok(Statement::ArrayAssign {
                            array: expr,
                            index: Box::new(index),
                            value: Box::new(value),
                        })
                    } else {
                        Ok(Statement::Expression(expr))
                    }
                } else {
                    Ok(Statement::Expression(expr))
                }
            }
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_visibility(&mut self) -> Result<Visibility, CrabbyError> {
        match self.peek().token {
            Token::Public => {
                self.advance();
                Ok(Visibility::Public)
            }
            Token::Protect => {
                self.advance();
                Ok(Visibility::Protect)
            }
            Token::Private => {
                self.advance();
                Ok(Visibility::Private)
            }
            _ => Ok(Visibility::Private),
        }
    }

    fn parse_definition(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'def'

        let name = if let Token::Identifier(name) = &self.peek().token {
            name.clone()
        } else {
            return Err(self.error("Expected function name"));
        };
        self.advance();

        self.consume(&Token::LParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !matches!(self.peek().token, Token::RParen) {
            loop {
                if let Token::Identifier(param) = &self.peek().token {
                    params.push(param.clone());
                    self.advance();
                } else {
                    return Err(self.error("Expected parameter name"));
                }

                if matches!(self.peek().token, Token::RParen) {
                    break;
                }
                self.consume(&Token::Comma, "Expected ',' between parameters")?;
            }
        }
        self.advance(); // consume ')'

        let body = self.parse_block()?;

        Ok(Statement::FunctionDef {
            name,
            params,
            body: Box::new(body),
            return_type: String::new(),
            docstring: String::new(),
            visibility: Visibility::default(),
        })
    }

    fn parse_function(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'fun'

        let name = if let Token::Identifier(name) = &self.peek().token {
            name.clone()
        } else {
            return Err(self.error("Expected function name"));
        };
        self.advance();

        self.consume(&Token::LParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !matches!(self.peek().token, Token::RParen) {
            loop {
                if let Token::Identifier(param) = &self.peek().token {
                    params.push(param.clone());
                    self.advance();
                } else {
                    return Err(self.error("Expected parameter name"));
                }

                if matches!(self.peek().token, Token::RParen) {
                    break;
                }
                self.consume(&Token::Comma, "Expected ',' between parameters")?;
            }
        }
        self.advance(); // consume ')'

        let body = self.parse_block()?;

        Ok(Statement::FunctionFun {
            name,
            params,
            body: Box::new(body),
            return_type: String::new(),
            docstring: String::new(),
            visibility: Visibility::default(),
        })
    }

    fn parse_match_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'match'
        let value = self.parse_expression()?;
        self.consume(&Token::LBrace, "Expected '{' after match expression")?;

        let mut arms = Vec::new();
        while !matches!(self.peek().token, Token::RBrace) {
            if !matches!(self.peek().token, Token::Case) {
                return Err(CrabbyError::MissingCaseKeyword(ErrorLocation {
                    // line: self.tokens.span.line,
                    line: self.peek().span.line,
                    // column: self.tokens.span.column,
                    column: self.peek().span.column,
                    message: "Expected 'case' keyword!".to_string(),
                }));
            }
            self.advance(); // consume 'case'

            let pattern = self.parse_expression()?;
            self.consume(&Token::Arrow, "Expected '=>' after match pattern")?;
            let body = self.parse_expression()?;
            arms.push(MatchArm { pattern, body });

            if matches!(self.peek().token, Token::Comma) {
                self.advance();
            }
        }

        self.consume(&Token::RBrace, "Expected '}' after match arms")?;

        Ok(Statement::Match {
            value: Box::new(value),
            arms,
        })
    }

    // fn parse_macro_statement(&mut self) -> Result<Statement, CrabbyError> {
    //    self.advance(); // consume 'macro'
    //    let name = if let Token::Identifier(name) = &self.peek().token {
    //        name.clone()
    //    } else {
    //        return Err(self.error("Expected macro name"));
    //    };
    //    self.advance();

    //    let params = self.parse_params()?;
    //    let body = self.parse_block()?;

    //    Ok(Statement::Macro {
    //        name,
    //        params: params.join(","),
    //        body: Box::new(Expression::Lambda {
    //            params,
    //            body: Box::new(body),
    //        }),
    //    })
    // }

    pub fn parse_async_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'async'

        if matches!(self.peek().token, Token::Def) {
            self.advance(); // consume 'def'

            let name = if let Token::Identifier(name) = &self.peek().token {
                name.clone()
            } else {
                return Err(self.error("Expected function name after 'async def'"));
            };
            self.advance();

            let params = self.parse_params()?;

            let return_type = if matches!(self.peek().token, Token::CoolerArrow) {
                self.advance(); // consume '->'
                if let Token::Identifier(type_name) = &self.peek().token {
                    Some(type_name.clone())
                } else {
                    return Err(self.error("Expected return type after '->'"));
                }
            } else {
                None
            };

            let body = self.parse_block()?;

            Ok(Statement::AsyncFunction {
                name,
                params,
                body: Box::new(body),
                return_type,
            })
        } else {
            let expr = self.parse_expression()?;
            Ok(Statement::Expression(expr))
        }
    }

    // pub fn parse_await_statement(&mut self) -> Result<Statement, CrabbyError> {
    //    self.advance(); // consume 'await'

    //    let expr = self.parse_expression()?;
    //    Ok(Expression::Await { expr: Box::new(expr)});
    // }

    // fn parse_class_statement(&mut self) -> Result<Statement, CrabbyError> {
    //    self.advance(); // consume 'class'
    // }

    // fn parse_trait_statement(&mut self) -> Result<Statement, CrabbyError> {
    //    self.advance(); // consume 'trait'
    // }

    // fn parse_impl_statement(&mut self) -> Result<Statement, CrabbyError> {
    //     self.advance(); // consume 'impl'
    // }

    // fn parse_mutable_statement(&mut self) -> Result<Statement, CrabbyError> {
    //     self.advance(); // consume 'mut'
    // }

    fn parse_and_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'and'
        let left = self.parse_expression()?;
        self.consume(&Token::And, "Expected 'and' operator")?;
        let right = self.parse_expression()?;

        Ok(Statement::And {
            left: left.to_string(),
            right: right.to_string(),
        })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;

        let then_branch = self.parse_block()?;

        let else_branch = if matches!(self.peek().token, Token::Else) {
            self.advance(); // consume 'else'
            Some(Box::new(self.parse_block()?))
        } else {
            None
        };

        Ok(Statement::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'while'
        let condition = self.parse_expression()?;
        self.consume(&Token::Colon, "Expected ':' after while condition")?;
        let body = self.parse_block()?;

        Ok(Statement::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, CrabbyError> {
        let expr = self.parse_primary()?; // SO point 1/2

        let _ = match &self.peek().token {
            Token::Await => self.parse_expression(),
            _ => self.parse_primary(),
        };

        while matches!(self.peek().token, Token::Dot) {
            self.advance(); // consume dot

            let _method = if let Token::Identifier(name) = &self.peek().token {
                name.clone()
            } else {
                return Err(self.error("Expected method name after dot"));
            };
            self.advance();

            let mut arguments = Vec::new();
            if matches!(self.peek().token, Token::LParen) {
                self.advance();
                while !matches!(self.peek().token, Token::RParen) {
                    arguments.push(self.parse_expression()?);
                    if matches!(self.peek().token, Token::Comma) {
                        self.advance();
                    }
                }
                self.advance();
            }
        }
        Ok(expr)
    }

    fn parse_addition(&mut self) -> Result<Expression, CrabbyError> {
        let mut expr = self.parse_multiplication()?;

        while matches!(self.peek().token, Token::Plus | Token::Minus) {
            let operator = match self.peek().token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.parse_multiplication()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, CrabbyError> {
        let mut expr = self.parse_primary()?;

        while matches!(self.peek().token, Token::Star | Token::Slash | Token::Arrow) {
            let operator = match self.peek().token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Arrow => BinaryOp::MatchOp,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.parse_primary()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, CrabbyError> {
        match &self.peek().token {
            Token::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Integer(n))
            }
            Token::Float(f) => {
                let f = *f;
                self.advance();
                Ok(Expression::Float(f))
            }
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::String(s))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if matches!(self.peek().token, Token::LParen) {
                    self.parse_function_call(name)
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            Token::True => {
                self.advance();
                Ok(Expression::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Boolean(false))
            }
            Token::Range => {
                self.advance(); // consume 'range'
                self.consume(&Token::LParen, "Expected '(' after 'range'")?;
                let count = self.parse_expression()?;
                self.consume(&Token::RParen, "Expected ')' after range count")?;
                Ok(Expression::Range(Box::new(count)))
            }
            Token::Lambda => {
                self.advance(); // consume 'lambda'
                self.consume(&Token::LParen, "Expected '(' after lambda")?;

                let mut params = Vec::new();
                if !matches!(self.peek().token, Token::RParen) {
                    loop {
                        if let Token::Identifier(param) = &self.peek().token {
                            params.push(param.clone());
                            self.advance();
                        } else {
                            return Err(self.error("Expected parameter name"));
                        }

                        if matches!(self.peek().token, Token::RParen) {
                            break;
                        }
                        self.consume(&Token::Comma, "Expected ',' between parameters")?;
                    }
                }
                self.advance(); // consume ')'

                // self.consume(&Token::Colon, "Expected ':' after parameters")?;
                let body = self.parse_block()?;

                Ok(Expression::Lambda {
                    params,
                    body: Box::new(body),
                })
            }
            Token::FString(template) => {
                let template = template.clone();
                self.advance();

                // Parse expressions within {}
                let mut expressions = Vec::new();
                let mut curr_pos = 0;

                while let Some(start) = template[curr_pos..].find('{') {
                    if let Some(end) = template[curr_pos + start + 1..].find('}') {
                        let _expr_str = &template[curr_pos + start + 1..curr_pos + start + 1 + end];
                        let expr = self.parse_expression()?;
                        expressions.push(expr);
                        curr_pos = curr_pos + start + 2 + end;
                    }
                }

                Ok(Expression::FString {
                    template,
                    expressions,
                })
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&Token::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::LBracket => {
                self.advance(); // consume '['
                let mut elements = Vec::new();

                if !matches!(self.peek().token, Token::RBracket) {
                    loop {
                        elements.push(self.parse_expression()?);

                        if !matches!(self.peek().token, Token::Comma) {
                            break;
                        }
                        self.advance(); // consume ','
                    }
                }

                self.consume(&Token::RBracket, "Expected ']' after array elements")?;
                Ok(Expression::Array(elements))
            }
            x => {
                // let expr = self.parse_expression()?; //SO point 2/2

                // if matches!(self.peek().token, Token::LBracket) {
                //     self.advance(); // consume '['
                //     let index = self.parse_expression()?;
                //     self.consume(&Token::RBracket, "Expected ']' after array index")?;

                //     Ok(Expression::Index {
                //         array: Box::new(expr),
                //         index: Box::new(index),
                //     })
                // } else {
                //     Ok(expr)
                // }
                // Ok(Expression::String("Bruh".to_string()))
                Err(CrabbyError::ParserError(ErrorLocation {
                    line: 581,
                    column: 0,
                    message: format!("Unexpected {x:?} at this time."),
                }))
            }
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'let'

        let name = if let Token::Identifier(name) = &self.peek().token {
            name.clone()
        } else {
            return Err(self.error("Expected variable name"));
        };
        self.advance();

        self.consume(&Token::Equals, "Expected '=' after variable name")?;
        let value = self.parse_expression()?;

        Ok(Statement::Let {
            name,
            value: Box::new(value),
        })
    }

    fn parse_var_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'var'

        let name = if let Token::Identifier(name) = &self.peek().token {
            name.clone()
        } else {
            return Err(self.error("Expected variable name"));
        };
        self.advance();

        self.consume(&Token::Equals, "Expected '=' after variable name")?;
        let value = self.parse_expression()?;

        Ok(Statement::Var {
            name,
            value: Box::new(value),
        })
    }

    fn parse_constant_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'const'

        let name = if let Token::Identifier(name) = &self.peek().token {
            name.clone()
        } else {
            return Err(self.error("Expected variable name"));
        };
        self.advance();

        self.consume(&Token::Equals, "Expected '=' after variable name")?;
        let value = self.parse_expression()?;

        Ok(Statement::Const {
            name,
            value: Box::new(value),
        })
    }

    fn parse_loop_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'loop'

        let count = self.parse_expression()?;

        // self.consume(&Token::Colon, "Expected ':' after loop count")?;
        let body = self.parse_block()?;

        Ok(Statement::Loop {
            count: Box::new(count),
            body: Box::new(body),
        })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'for'

        let variable = if let Token::Identifier(name) = &self.peek().token {
            name.clone()
        } else {
            return Err(self.error("Expected variable name after 'for'"));
        };
        self.advance();

        self.consume(&Token::In, "Expected 'in' after variable name")?;

        let iterator = self.parse_expression()?;

        // self.consume(&Token::Colon, "Expected ':' after iterator expression")?;
        let body = self.parse_block()?;

        Ok(Statement::ForIn {
            variable,
            iterator: Box::new(iterator),
            body: Box::new(body),
        })
    }

    fn parse_enum_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'enum'

        let name = if let Token::Identifier(name) = &self.peek().token {
            name.clone()
        } else {
            return Err(self.error("Expected enum name"));
        };
        self.advance();

        let mut where_clause = None;
        if matches!(self.peek().token, Token::Where) {
            self.advance(); // consume 'where'
            where_clause = Some(Box::new(self.parse_expression()?));
        }

        self.consume(&Token::LBrace, "Expected '{' after enum name")?;

        let mut variants = Vec::new();
        while !matches!(self.peek().token, Token::RBrace) {
            let variant_name = if let Token::Identifier(name) = &self.peek().token {
                name.clone()
            } else {
                return Err(self.error("Expected variant name"));
            };
            self.advance();

            let fields = if matches!(self.peek().token, Token::LParen) {
                self.advance(); // consume '('
                let mut fields = Vec::new();

                while !matches!(self.peek().token, Token::RParen) {
                    fields.push(self.parse_expression()?);
                    if matches!(self.peek().token, Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }

                self.consume(&Token::RParen, "Expected ')' after variant fields")?;
                Some(fields)
            } else {
                None
            };

            variants.push(EnumVariant {
                name: variant_name,
                fields,
            });

            if matches!(self.peek().token, Token::Comma) {
                self.advance();
            }
        }

        self.consume(&Token::RBrace, "Expected '}' after enum variants")?;

        Ok(Statement::Enum {
            name,
            variants,
            where_clause,
        })
    }

    fn parse_struct_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'struct'

        let name = if let Token::Identifier(name) = &self.peek().token {
            name.clone()
        } else {
            return Err(self.error("Expected struct name"));
        };
        self.advance();

        let mut where_clause = None;
        if matches!(self.peek().token, Token::Where) {
            self.advance(); // consume 'where'
            where_clause = Some(Box::new(self.parse_expression()?));
        }

        self.consume(&Token::LBrace, "Expected '{' after struct name")?;

        let mut fields = Vec::new();
        while !matches!(self.peek().token, Token::RBrace) {
            let field_name = if let Token::Identifier(name) = &self.peek().token {
                name.clone()
            } else {
                return Err(self.error("Expected field name"));
            };
            self.advance();

            self.consume(&Token::Colon, "Expected ':' after field name")?;
            let type_expr = self.parse_expression()?;

            fields.push(StructField {
                name: field_name,
                type_expr,
            });

            if matches!(self.peek().token, Token::Comma) {
                self.advance();
            }
        }

        self.consume(&Token::RBrace, "Expected '}' after struct fields")?;

        Ok(Statement::Struct {
            name,
            fields,
            where_clause,
        })
    }

    fn parse_where_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'where'
        let condition = self.parse_expression()?;
        let expr = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::Expression(Expression::Where {
            expr: Box::new(expr),
            condition: Box::new(condition),
            body: Box::new(body),
        }))
    }

    fn parse_import_statement(&mut self) -> Result<Statement, CrabbyError> {
        self.advance(); // consume 'import'

        let name = if let Token::Identifier(name) = &self.peek().token {
            name.clone()
        } else {
            return Err(self.error("Expected module name after 'import'"));
        };
        self.advance();

        let source = if matches!(self.peek().token, Token::From) {
            self.advance(); // consume 'from'
            if let Token::String(path) = &self.peek().token {
                Some(path.clone())
            } else {
                return Err(self.error("Expected string literal after 'from'"));
            }
        } else {
            None
        };
        self.advance();

        Ok(Statement::Import { name, source })
    }

    fn parse_function_call(&mut self, name: String) -> Result<Expression, CrabbyError> {
        self.advance(); // consume '('

        let mut arguments = Vec::new();
        if !matches!(self.peek().token, Token::RParen) {
            loop {
                arguments.push(self.parse_expression()?);
                if !matches!(self.peek().token, Token::Comma) {
                    break;
                }
                self.advance(); // consume ','
            }
        }

        self.consume(&Token::RParen, "Expected ')' after arguments")?;

        Ok(Expression::Call {
            function: name,
            arguments,
        })
    }

    fn parse_block(&mut self) -> Result<Statement, CrabbyError> {
        self.consume(&Token::LBrace, "Expected '{' at start of block")?;

        let mut statements = Vec::new();
        while !matches!(self.peek().token, Token::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.consume(&Token::RBrace, "Expected '}' at end of block")?;
        Ok(Statement::Block(statements))
    }

    fn peek(&self) -> &TokenStream {
        if self.is_at_end() {
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[self.current]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn consume(&mut self, expected: &Token, message: &str) -> Result<(), CrabbyError> {
        if self.peek().token == *expected {
            self.advance();
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn error(&self, message: &str) -> CrabbyError {
        let span = if self.is_at_end() {
            &self.tokens[self.tokens.len() - 1].span
        } else {
            &self.peek().span
        };

        CrabbyError::ParserError(ErrorLocation {
            line: span.line,
            column: span.column,
            message: message.to_string(),
        })
    }
}

pub fn parse(tokens: Vec<TokenStream>) -> Result<Program, CrabbyError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
