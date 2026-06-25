use std::collections::HashMap;

pub struct State {}

pub struct Function {
    tag_name: String,
    arguments: HashMap<String, String>,
}

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
                let mut result = format!("<{tag_name} >");
                for (arg, val) in arguments {
                    result += &format!("{arg}=\"{}\"", val.generate(state));
                }
                if let Some(body) = body {
                    result += &body.generate(state);
                }
                format!("</{result}>")
            }
            ActionKind::FunctionSequence {
                tag_name,
                arguments,
                body,
            } => {
                let mut result = format!("<{tag_name} >");
                for (arg, val) in arguments {
                    result += &format!("{arg}=\"{}\"", val.generate(state));
                }
                for seq in body {
                    result += &seq.generate(state);
                }
                format!("</{result}>")
            }
            _ => todo!(),
        }
    }
}
