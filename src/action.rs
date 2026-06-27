use std::{collections::HashMap, fmt::Display};

use crate::{lexer::OperatorKind, state::State};

#[derive(Debug)]
pub enum ActionKind<'a> {
    /// Final unit of code that represents a constant string
    ConstString(String),
    /// A function that will generate an html tag without any children
    Function {
        tag_name: &'a str,
        arguments: HashMap<&'a str, ActionKind<'a>>,
        body: Option<Box<ActionKind<'a>>>,
    },
    /// Function which has a body that can contain multiple children such as div
    FunctionSequence {
        tag_name: &'a str,
        arguments: HashMap<&'a str, ActionKind<'a>>,
        body: Vec<ActionKind<'a>>,
    },
    GetVariable(&'a str),
    BinaryOperation {
        op: OperatorKind,
        left: Box<ActionKind<'a>>,
        right: Box<ActionKind<'a>>,
    },
}

impl<'a> Display for ActionKind<'a> {
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
            ActionKind::FunctionSequence {
                tag_name,
                arguments,
                body,
            } => write!(
                f,
                "FUNCTION(tag: {tag_name}, arguments: \"{}\", body : \"{}\")",
                arguments
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<String>>()
                    .join(","),
                body.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(";")
            ),
            ActionKind::GetVariable(s) => write!(f, "VAR({s})"),
            ActionKind::BinaryOperation { op, left, right } => write!(
                f,
                "OPERATION(op: {:?}, left: {}, right: {})",
                op,
                left.to_string(),
                right.to_string()
            ),
        }
    }
}

impl<'a> ActionKind<'a> {
    pub fn generate(&self, state: &mut State) -> String {
        match &self {
            ActionKind::ConstString(s) => s.to_string(),
            ActionKind::Function {
                tag_name,
                arguments,
                body,
            } => {
                let mut result = format!("<{tag_name} ");
                for (arg, val) in arguments {
                    result += &format!("{arg}=\"{}\"", val.generate(state));
                }
                result += ">";
                if let Some(body) = body {
                    result += &body.generate(state);
                }
                format!("{result}</{tag_name}>")
            }
            ActionKind::FunctionSequence {
                tag_name,
                arguments,
                body,
            } => {
                let mut result = format!("<{tag_name} ");
                for (arg, val) in arguments {
                    result += &format!("{arg}=\"{}\"", val.generate(state));
                }
                result += ">";
                for seq in body {
                    result += &seq.generate(state);
                }
                format!("{result}</{tag_name}>")
            }
            ActionKind::BinaryOperation { op, left, right } => match op {
                OperatorKind::Assign => todo!(),
                OperatorKind::StringConcat => {
                    return format!("{}{}", left.generate(state), right.generate(state));
                }
            },
            _ => todo!(),
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
        assert_eq!(act.generate(&mut state), "hello world".to_owned());
    }

    #[test]
    fn test_tag_generation() {
        let act = ActionKind::Function {
            tag_name: "div",
            arguments: HashMap::new(),
            body: None,
        };
        let mut state: State = State::default();
        assert_eq!(act.generate(&mut state), "<div ></div>".to_owned());
    }

    #[test]
    fn test_tag_generation_with_arguments() {
        let mut args: HashMap<&str, ActionKind> = HashMap::new();
        args.insert("class", ActionKind::ConstString("amazing".to_owned()));
        let act = ActionKind::Function {
            tag_name: "div",
            arguments: args,
            body: None,
        };

        let mut state: State = State::default();
        assert_eq!(
            act.generate(&mut state),
            "<div class=\"amazing\"></div>".to_owned()
        );
    }

    #[test]
    fn test_tag_generation_body_p() {
        let act = ActionKind::FunctionSequence {
            tag_name: "p",
            arguments: HashMap::new(),
            body: vec![ActionKind::ConstString("hello world".to_owned())],
        };
        let mut state: State = State::default();
        assert_eq!(act.generate(&mut state), "<p >hello world</p>".to_owned());
    }
}
