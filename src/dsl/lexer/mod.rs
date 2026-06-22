#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    GestureKeyword,
    Identifier(String),
    ExecuteKeyword,
    StringLiteral(String),
    NumberLiteral(f64),
    LCurly,
    RCurly,
    LParen,
    RParen,
    Semicolon,
    Colon,
    Comma,
    Equal,
    Comment(String),
    Eof,
}

pub struct Lexer {
    input: String,
    chars: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            chars: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while self.position < self.chars.len() {
            let ch = self.chars[self.position];
            
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.advance();
                }
                '{' => {
                    tokens.push(Token::LCurly);
                    self.advance();
                }
                '}' => {
                    tokens.push(Token::RCurly);
                    self.advance();
                }
                '(' => {
                    tokens.push(Token::LParen);
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::RParen);
                    self.advance();
                }
                ';' => {
                    tokens.push(Token::Semicolon);
                    self.advance();
                }
                ':' => {
                    tokens.push(Token::Colon);
                    self.advance();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                '=' => {
                    tokens.push(Token::Equal);
                    self.advance();
                }
                '"' => {
                    let string = self.read_string();
                    tokens.push(Token::StringLiteral(string));
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        let comment = self.read_line_comment();
                        tokens.push(Token::Comment(comment));
                    } else {
                        // Error
                        self.advance();
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = self.read_identifier();
                    match ident.as_str() {
                        "gesture" => tokens.push(Token::GestureKeyword),
                        "execute" => tokens.push(Token::ExecuteKeyword),
                        _ => tokens.push(Token::Identifier(ident)),
                    }
                }
                '0'..='9' | '.' => {
                    let num = self.read_number();
                    tokens.push(Token::NumberLiteral(num));
                }
                _ => {
                    // Skip unknown characters
                    self.advance();
                }
            }
        }
        
        tokens.push(Token::Eof);
        tokens
    }
    
    fn advance(&mut self) {
        if self.position < self.chars.len() {
            self.position += 1;
            self.column += 1;
        }
    }
    
    fn peek_next(&self) -> Option<char> {
        if self.position + 1 < self.chars.len() {
            Some(self.chars[self.position + 1])
        } else {
            None
        }
    }
    
    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while self.position < self.chars.len() {
            let ch = self.chars[self.position];
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.position].to_string()
    }
    
    fn read_string(&mut self) -> String {
        self.advance(); // Skip opening quote
        let start = self.position;
        while self.position < self.chars.len() && self.chars[self.position] != '"' {
            self.advance();
        }
        let string = self.input[start..self.position].to_string();
        self.advance(); // Skip closing quote
        string
    }
    
    fn read_number(&mut self) -> f64 {
        let start = self.position;
        let mut has_decimal = false;
        
        while self.position < self.chars.len() {
            let ch = self.chars[self.position];
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_decimal {
                has_decimal = true;
                self.advance();
            } else {
                break;
            }
        }
        
        let num_str = &self.input[start..self.position];
        num_str.parse().unwrap_or(0.0)
    }
    
    fn read_line_comment(&mut self) -> String {
        self.advance(); // Skip first '/'
        self.advance(); // Skip second '/'
        let start = self.position;
        while self.position < self.chars.len() && self.chars[self.position] != '\n' {
            self.advance();
        }
        self.input[start..self.position].trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lexer_gesture_definition() {
        let input = "gesture swipe_left { execute \"ALT+TAB\"; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert!(tokens.contains(&Token::GestureKeyword));
        assert!(tokens.contains(&Token::Identifier("swipe_left".to_string())));
        assert!(tokens.contains(&Token::ExecuteKeyword));
        assert!(tokens.contains(&Token::StringLiteral("ALT+TAB".to_string())));
        assert!(tokens.contains(&Token::LCurly));
        assert!(tokens.contains(&Token::RCurly));
        assert!(tokens.contains(&Token::Semicolon));
    }
}