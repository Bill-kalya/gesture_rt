use std::collections::HashMap;

/// Standard library of predefined gestures
pub struct StdLib;

impl StdLib {
    pub fn new() -> Self {
        Self
    }
    
    /// Get standard gesture definitions
    pub fn get_standard_gestures() -> HashMap<&'static str, &'static str> {
        let mut gestures = HashMap::new();
        gestures.insert(
            "swipe_left",
            r#"
                gesture swipe_left {
                    execute "ALT+TAB";
                }
            "#
        );
        gestures.insert(
            "swipe_right",
            r#"
                gesture swipe_right {
                    execute "CTRL+TAB";
                }
            "#
        );
        gestures.insert(
            "swipe_up",
            r#"
                gesture swipe_up {
                    execute "VOLUME_UP";
                }
            "#
        );
        gestures.insert(
            "swipe_down",
            r#"
                gesture swipe_down {
                    execute "VOLUME_DOWN";
                }
            "#
        );
        gestures.insert(
            "pinch",
            r#"
                gesture pinch {
                    execute "CTRL+C";
                }
            "#
        );
        gestures.insert(
            "fist",
            r#"
                gesture fist {
                    execute "CTRL+V";
                }
            "#
        );
        gestures
    }
    
    /// Load standard library into interpreter
    pub fn load_into_interpreter(
        interpreter: &mut super::interpreter::Interpreter,
    ) -> Result<(), String> {
        let lib = Self::get_standard_gestures();
        let mut parser = super::parser::Parser::new();
        let mut lexer = super::lexer::Lexer::new("");
        
        for (name, script) in lib {
            lexer = super::lexer::Lexer::new(script);
            let tokens = lexer.tokenize();
            let ast = parser.parse(tokens)?;
            interpreter.load_script(ast)?;
        }
        
        Ok(())
    }
}

impl Default for StdLib {
    fn default() -> Self {
        Self::new()
    }
}