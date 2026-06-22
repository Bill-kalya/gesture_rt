use super::lexer::Token;

#[derive(Debug, Clone)]
pub enum ASTNode {
    GestureDefinition {
        name: String,
        body: Vec<ASTNode>,
    },
    ExecuteStatement {
        command: String,
    },
    Literal {
        value: String,
    },
    Number {
        value: f64,
    },
    Comment {
        text: String,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            position: 0,
        }
    }
    
    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Vec<ASTNode>, String> {
        self.tokens = tokens;
        self.position = 0;
        let mut ast = Vec::new();
        
        while self.position < self.tokens.len() {
            if let Some(node) = self.parse_statement()? {
                ast.push(node);
            } else {
                self.position += 1;
            }
        }
        
        Ok(ast)
    }
    
    fn parse_statement(&mut self) -> Result<Option<ASTNode>, String> {
        let current = self.current_token();
        
        match current {
            Some(Token::GestureKeyword) => {
                self.advance(); // Skip 'gesture'
                self.parse_gesture_definition()
            }
            Some(Token::Comment(text)) => {
                self.advance();
                Ok(Some(ASTNode::Comment { text: text.clone() }))
            }
            Some(_) => {
                self.advance();
                Ok(None)
            }
            None => Ok(None),
        }
    }
    
    fn parse_gesture_definition(&mut self) -> Result<Option<ASTNode>, String> {
        // Expect identifier
        let name = match self.current_token() {
            Some(Token::Identifier(name)) => {
                self.advance();
                name.clone()
            }
            _ => {
                return Err("Expected gesture name".to_string());
            }
        };
        
        // Expect {
        match self.current_token() {
            Some(Token::LCurly) => {
                self.advance();
            }
            _ => {
                return Err("Expected '{' after gesture name".to_string());
            }
        }
        
        let mut body = Vec::new();
        
        // Parse body until }
        while let Some(token) = self.current_token() {
            match token {
                Token::RCurly => {
                    self.advance();
                    break;
                }
                Token::ExecuteKeyword => {
                    self.advance();
                    // Parse execute statement
                    match self.current_token() {
                        Some(Token::StringLiteral(command)) => {
                            self.advance();
                            body.push(ASTNode::ExecuteStatement {
                                command: command.clone(),
                            });
                            
                            // Expect ;
                            if let Some(Token::Semicolon) = self.current_token() {
                                self.advance();
                            }
                        }
                        _ => {
                            return Err("Expected string literal after 'execute'".to_string());
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }
        
        Ok(Some(ASTNode::GestureDefinition { name, body }))
    }
    
    fn current_token(&self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            Some(&self.tokens[self.position])
        } else {
            None
        }
    }
    
    fn advance(&mut self) {
        self.position += 1;
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lexer::Lexer;
    
    #[test]
    fn test_parse_gesture() {
        let input = r#"
            gesture swipe_left {
                execute "ALT+TAB";
            }
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        let mut parser = Parser::new();
        let ast = parser.parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        if let ASTNode::GestureDefinition { name, body } = &ast[0] {
            assert_eq!(name, "swipe_left");
            assert_eq!(body.len(), 1);
            if let ASTNode::ExecuteStatement { command } = &body[0] {
                assert_eq!(command, "ALT+TAB");
            } else {
                panic!("Expected ExecuteStatement");
            }
        } else {
            panic!("Expected GestureDefinition");
        }
    }
}