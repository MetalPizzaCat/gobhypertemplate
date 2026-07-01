use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    lexer::OperatorKind,
    state::{ExecutionError, State},
};

#[derive(Debug)]
pub enum ActionKind {
    /// Final unit of code that represents a constant string
    ConstString(String),
    /// A function that will generate an html tag without any children
    Function {
        tag_name: String,
        arguments: HashMap<String, ActionKind>,
        body: Option<Box<ActionKind>>,
    },
    /// Function which has a body that can contain multiple children such as div
    Sequence(Vec<ActionKind>),
    GetVariable(String),
    BinaryOperation {
        op: OperatorKind,
        left: Box<ActionKind>,
        right: Box<ActionKind>,
    },
    UserFunctionCall {
        function_name: String,
        arguments: HashMap<String, ActionKind>,
    },
    UserFunctionDeclaration {
        function_name: String,
        arguments: Vec<String>,
        body: Rc<ActionKind>,
    },
}

impl<'a> Display for ActionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ActionKind::ConstString(s) => write!(f, "CONST({s})"),
            ActionKind::Function {
                tag_name,
                arguments,
                body,
            } => {
                write!(
                    f,
                    "FUNCTION(tag: {tag_name}, arguments: {}, body : {})",
                    arguments
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<String>>()
                        .join(","),
                    if let Some(b) = body {
                        b.to_string()
                    } else {
                        "none".to_owned()
                    }
                )
            }
            ActionKind::Sequence(seq) => {
                write!(
                    f,
                    "SEQ({})",
                    seq.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(";")
                )
            }
            ActionKind::GetVariable(s) => write!(f, "VAR({s})"),
            ActionKind::BinaryOperation { op, left, right } => write!(
                f,
                "OPERATION(op: {:?}, left: {}, right: {})",
                op,
                left.to_string(),
                right.to_string()
            ),
            ActionKind::UserFunctionCall {
                function_name,
                arguments,
            } => write!(
                f,
                "FUNC_CALL(name : {function_name}, args : \"{}\")",
                arguments
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<String>>()
                    .join(","),
            ),
            ActionKind::UserFunctionDeclaration {
                function_name,
                arguments,
                body,
            } => write!(
                f,
                "DECLARATION(name : {function_name}, args : {}, body : {}",
                arguments.join(","),
                body.to_string()
            ),
        }
    }
}

impl<'a> ActionKind {
    pub fn generate(&self, state: &mut State) -> Result<Option<String>, ExecutionError> {
        match &self {
            ActionKind::ConstString(s) => Ok(Some(s.to_string())),
            ActionKind::Function {
                tag_name,
                arguments,
                body,
            } => {
                let mut result = format!("<{tag_name} ");
                for (arg, val) in arguments {
                    result += &format!(
                        "{arg}=\"{}\"",
                        val.generate(state)?.unwrap_or_else(|| String::new())
                    );
                }
                result += ">";
                if let Some(body) = body {
                    if let Some(res) = body.generate(state)? {
                        result += &res;
                    }
                }
                Ok(Some(format!("{result}</{tag_name}>")))
            }
            ActionKind::Sequence(seq) => {
                let mut result = String::new();
                for item in seq {
                    if let Some(statement) = item.generate(state)? {
                        result += &statement;
                    }
                }
                Ok(Some(result))
            }
            ActionKind::BinaryOperation { op, left, right } => match op {
                OperatorKind::Assign => todo!(),
                OperatorKind::StringConcat => Ok(Some(format!(
                    "{}{}",
                    left.generate(state)?.unwrap_or_else(|| String::new()),
                    right.generate(state)?.unwrap_or_else(|| String::new()),
                ))),
            },
            ActionKind::UserFunctionCall {
                function_name,
                arguments,
            } => {
                let mut args: HashMap<String, String> = HashMap::new();
                for (arg_name, arg_val) in arguments {
                    args.insert(
                        arg_name.to_string(),
                        arg_val.generate(state)?.unwrap_or_else(|| String::new()),
                    );
                }

                state.execute_user_function(function_name, args)
            }
            ActionKind::UserFunctionDeclaration {
                function_name,
                arguments,
                body,
            } => {
                state.add_user_function(function_name.clone(), arguments.clone(), body.clone());
                Ok(None)
            }
            ActionKind::GetVariable(name) => Ok(state.get_variable_value(name)),
          
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::action::ActionKind;
    use crate::state::State;

    #[test]
    fn test_string_generation() {
        let act = ActionKind::ConstString("hello world".to_owned());
        let mut state: State = State::default();
        assert_eq!(
            act.generate(&mut state).unwrap().unwrap(),
            "hello world".to_owned()
        );
    }

    #[test]
    fn test_tag_generation() {
        let act = ActionKind::Function {
            tag_name: "div".to_owned(),
            arguments: HashMap::new(),
            body: None,
        };
        let mut state: State = State::default();
        assert_eq!(
            act.generate(&mut state).unwrap().unwrap(),
            "<div ></div>".to_owned()
        );
    }

    #[test]
    fn test_tag_generation_with_arguments() {
        let mut args: HashMap<String, ActionKind> = HashMap::new();
        args.insert("class".to_owned(), ActionKind::ConstString("amazing".to_owned()));
        let act = ActionKind::Function {
            tag_name: "div".to_owned(),
            arguments: args,
            body: None,
        };

        let mut state: State = State::default();
        assert_eq!(
            act.generate(&mut state).unwrap().unwrap(),
            "<div class=\"amazing\"></div>".to_owned()
        );
    }

    #[test]
    fn test_tag_generation_body_p() {
        let act = ActionKind::Sequence(vec![ActionKind::ConstString("hello world".to_owned())]);
        let mut state: State = State::default();
        assert_eq!(
            act.generate(&mut state).unwrap().unwrap(),
            "hello world".to_owned()
        );
    }
}
