use std::collections::HashMap;

use crate::state::State;

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
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::execution::ActionKind;
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
