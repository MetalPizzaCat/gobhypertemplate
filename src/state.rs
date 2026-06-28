use std::{collections::HashMap, error::Error, fmt::Display, rc::Rc};

use crate::action::ActionKind;

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
pub struct UserFunction<'a> {
    arguments: Vec<&'a str>,
    body: Rc<Box<ActionKind<'a>>>,
}

#[derive(Default)]
pub struct State<'a> {
    variables: Vec<HashMap<&'a str, String>>,
    functions: HashMap<&'a str, UserFunction<'a>>,
}

impl<'a> State<'a> {
    pub fn create_new_variable_scope(&mut self, vars: HashMap<&'a str, String>) {
        self.variables.push(vars);
    }

    pub fn pop_variable_scope(&mut self) {
        self.variables.pop();
    }

    pub fn execute_function(
        &mut self,
        body: &ActionKind<'a>,
        args: HashMap<&'a str, String>,
    ) -> Result<Option<String>, ExecutionError> {
        self.create_new_variable_scope(args);
        let res = body.generate(self);
        self.pop_variable_scope();
        res
    }

    pub fn execute_user_function(
        &mut self,
        name: &str,
        args: HashMap<&'a str, String>,
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

    pub fn add_user_function(
        &mut self,
        name: &'a str,
        args: Vec<&'a str>,
        body: Rc<Box<ActionKind<'a>>>,
    ) {
        self.functions.insert(
            name,
            UserFunction {
                arguments: args,
                body,
            },
        );
    }

    pub fn get_user_function(&self, name: &str) -> Option<&UserFunction<'a>> {
        self.functions.get(name)
    }
}
