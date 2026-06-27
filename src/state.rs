use std::collections::HashMap;

use crate::action::ActionKind;

#[derive(Default)]
pub struct State {
    variables: Vec<HashMap<String, String>>,
}

impl State {
    pub fn create_new_variable_scope(&mut self, vars: HashMap<String, String>) {
        self.variables.push(vars);
    }

    pub fn pop_variable_scope(&mut self) {
        self.variables.pop();
    }

    pub fn execute_function(&mut self, body: ActionKind, args: HashMap<String, String>) -> String {
        self.create_new_variable_scope(args);
        let res = body.generate(self);
        self.pop_variable_scope();
        res
    }
}
