use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    action::ActionKind,
    lexer::{Lexer, ParsingError},
    parser::Parser,
};

use std::error::Error;

#[derive(Debug)]
pub enum ExecutionErrorKind {
    UnknownFunction,
    UnknownVariable,
}

#[derive(Debug)]

pub struct ExecutionError {
    kind: ExecutionErrorKind,
}

impl ExecutionError {
    pub fn new(kind: ExecutionErrorKind) -> Self {
        Self { kind }
    }
}

impl Error for ExecutionError {}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Clone)]
pub struct UserFunction {
    arguments: Vec<String>,
    body: Rc<ActionKind>,
}

pub struct Module {
    body: ActionKind,
    functions: HashMap<String, UserFunction>,
}

#[derive(Default)]
pub struct State {
    variables: Vec<HashMap<String, String>>,
    functions: HashMap<String, UserFunction>,
}

impl State {
    pub fn create_new_variable_scope(&mut self, vars: HashMap<String, String>) {
        self.variables.push(vars);
    }

    pub fn pop_variable_scope(&mut self) {
        self.variables.pop();
    }

    pub fn get_variable_value(&self, name: &str) -> Option<String> {
        for scope in self.variables.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        return None;
    }

    pub fn execute_function(
        &mut self,
        body: &ActionKind,
        args: HashMap<String, String>,
    ) -> Result<Option<String>, ExecutionError> {
        self.create_new_variable_scope(args);
        let res = body.generate(self);
        self.pop_variable_scope();
        res
    }

    pub fn execute_user_function(
        &mut self,
        name: &str,
        args: HashMap<String, String>,
    ) -> Result<Option<String>, ExecutionError> {
        if let Some(f) = self.get_user_function(name).cloned() {
            self.create_new_variable_scope(args);
            let res = f.body.generate(self);
            self.pop_variable_scope();
            res
        } else {
            Err(ExecutionError::new(ExecutionErrorKind::UnknownFunction))
        }
    }

    pub fn add_user_function(&mut self, name: String, args: Vec<String>, body: Rc<ActionKind>) {
        self.functions.insert(
            name,
            UserFunction {
                arguments: args,
                body,
            },
        );
    }

    /// Load module from given code and execute it, returning the string.
    /// The process of executing will also add all functions into the state allowing them to be used
    ///
    /// Note that this will unconditionally load the module, so calling it on same module twice will cause a duplication error
    pub fn load_module(&mut self, code: &str) -> Result<ActionKind, ParsingError> {
        let mut lexer: Lexer = Lexer::new(code);
        lexer.tokenize()?;
        let tokens: Vec<crate::lexer::Token> = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        parser.parse_body()
    }

    pub fn load_module_from_file(&mut self, path: &str) -> Result<ActionKind, Box<dyn Error>> {
        let code = std::fs::read_to_string(path)?;
        Ok(self.load_module(code.as_ref())?)
    }

    pub fn get_user_function(&self, name: &str) -> Option<&UserFunction> {
        self.functions.get(name)
    }
}
