use std::collections::HashMap;

use crate::{
    execution::ActionKind,
    lexer::{OperatorKind, ParsingError, SeparatorKind, Token, TokenKind},
};

pub struct Parser<'a> {
    input: &'a [Token<'a>],
    tokens: core::iter::Peekable<core::slice::Iter<'a, Token<'a>>>,
    program: Option<ActionKind<'a>>,

    last_row: usize,
    last_column: usize,
}

macro_rules! is_current_of_kind {
    ($self:expr, $pattern:pat) => {{
        if let Some(tok) = $self.tokens.peek() {
            match tok.get_kind() {
                $pattern => true,
                _ => false,
            }
        } else {
            false
        }
    }};
}

macro_rules! consume_token {
    () => {};
}

macro_rules! expect_token_pattern {
    ($self:expr, $pattern:pat) => {{
        let token = $self.next_token();
        match token.kind() {
            $pattern => Ok(token),
            _ => Err(ParseError::new(
                ParseErrorKind::ExpectedToken,
                token.clone(),
            )),
        }
    }};
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [Token<'a>]) -> Self {
        Parser {
            input,
            tokens: input.iter().peekable(),
            program: None,
            last_row: 0,
            last_column: 0,
        }
    }

    pub fn try_get_as_token_id(&mut self) -> Option<&'a str> {
        if let Some(tok) = self.tokens.peek() {
            match tok.get_kind() {
                TokenKind::Identifier(s) => Some(s),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_current_as_id(&mut self, error: &'static str) -> Result<&'a str, ParsingError> {
        if let Some(tok) = self.tokens.peek() {
            match tok.get_kind() {
                TokenKind::Identifier(s) => return Ok(s),
                _ => {}
            }
        }
        Err(ParsingError::new_with_message(
            crate::lexer::ParsingErrorKind::UnexpectedCharacter,
            error.to_string(),
            self.get_row(),
            self.get_column(),
        ))
    }

    /// Advance the token iterator and record the debug position info
    pub fn next(&mut self) -> Option<&Token<'a>> {
        if let Some(curr) = self.tokens.peek() {
            self.last_row = curr.get_row();
            self.last_column = curr.get_column();
        }
        self.tokens.next()
    }

    pub fn get_column(&mut self) -> usize {
        if let Some(tok) = self.tokens.peek() {
            tok.get_column()
        } else {
            self.last_column
        }
    }

    pub fn get_row(&mut self) -> usize {
        if let Some(tok) = self.tokens.peek() {
            tok.get_row()
        } else {
            self.last_row
        }
    }

    /// Attempt to consume the current token if it's the separator provided otherwise throw an error.
    /// Advances the iterator if it's the expected separator
    pub fn consume_separator(&mut self, sep: SeparatorKind) -> Result<(), ParsingError> {
        if let Some(tok) = self.tokens.peek() {
            match tok.get_kind() {
                TokenKind::Separator(s) => {
                    if sep == s {
                        self.tokens.next();
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
        Err(ParsingError::new_with_message(
            crate::lexer::ParsingErrorKind::UnexpectedCharacter,
            format!("Expected {}", sep.to_char()),
            self.get_row(),
            self.get_column(),
        ))
    }

    /// Attempt to consume the current token if it's the separator provided otherwise throw an error.
    /// Advances the iterator if it's the expected separator
    pub fn consume_operator(&mut self, sep: OperatorKind) -> Result<(), ParsingError> {
        if let Some(tok) = self.tokens.peek() {
            match tok.get_kind() {
                TokenKind::Operator(s) => {
                    if sep == s {
                        self.tokens.next();
                        return Ok(());
                    }
                }
                _ => {
                    println!("{:?}", tok.get_kind());
                }
            }
        }
        Err(ParsingError::new_with_message(
            crate::lexer::ParsingErrorKind::UnexpectedCharacter,
            format!("Expected {}", sep.to_char()),
            self.get_row(),
            self.get_column(),
        ))
    }

    /// Attempt to parse unit of the language
    /// Unit represents the smallest expression possible, such as function call or constant string
    pub fn parse_unit(&mut self) -> Result<Option<ActionKind<'a>>, ParsingError> {
        if let Some(tok) = self.tokens.peek() {
            let act: Option<ActionKind<'_>> = match tok.get_kind() {
                // any identifier will be considered a function call
                // this way any operation like variable operations could be done as separate functions
                TokenKind::Identifier(_) => Some(self.parse_call()?),
                TokenKind::Variable(name) => Some(ActionKind::GetVariable(name)),
                TokenKind::StringConst(s) => Some(ActionKind::ConstString(s)),
                _ => None,
            };
            self.next();
            return Ok(act);
        } else {
            return Err(ParsingError::new_with_message(
                crate::lexer::ParsingErrorKind::UnexpectedCharacter,
                "Expected an expression".to_owned(),
                self.get_row(),
                self.get_column(),
            ));
        }
    }

    pub fn parse_argument(&mut self) -> Result<Option<(&'a str, ActionKind<'a>)>, ParsingError> {
        if let Some(arg_name) = self.try_get_as_token_id() {
            self.next();
            self.consume_operator(OperatorKind::Assign)?;
            return Ok(Some((arg_name, self.parse_unit()?.unwrap())));
        }
        Ok(None)
    }

    pub fn parse_call(&mut self) -> Result<ActionKind<'a>, ParsingError> {
        let id = self.get_current_as_id("Expected function name")?;
        self.next();
        self.consume_separator(SeparatorKind::BracketOpen)?;
        let mut arguments: HashMap<&'a str, ActionKind<'a>> = HashMap::new();
        while let Some((arg, act)) = self.parse_argument()? {
            arguments.insert(arg, act);
            if !is_current_of_kind!(self, TokenKind::Separator(SeparatorKind::Comma)) {
                break;
            }
        }
        self.consume_separator(SeparatorKind::BracketClose)?;

        let mut body: Vec<ActionKind> = Vec::new();
        self.consume_separator(SeparatorKind::BlockOpen)?;
        loop {
            if is_current_of_kind!(self, TokenKind::Separator(SeparatorKind::BlockClose)) {
                break;
            } else if let Some(act) = self.parse_unit()? {
                self.consume_separator(SeparatorKind::Semicolon)?;
                body.push(act);
            }
        }
        // while let Some(act) = self.parse_unit()? {
        //     self.consume_separator(SeparatorKind::Semicolon)?;
        //     body.push(act);
        //     if is_current_of_kind!(self, TokenKind::Separator(SeparatorKind::BlockClose)) {
        //         break;
        //     }
        // }
        self.consume_separator(SeparatorKind::BlockClose)?;

        Ok(ActionKind::FunctionSequence {
            tag_name: id,
            arguments: arguments,
            body: body,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        execution::ActionKind,
        lexer::{OperatorKind, SeparatorKind, Token, TokenKind},
        parser::Parser,
    };

    #[test]
    fn parse_string_consts() -> Result<(), Box<dyn Error>> {
        let tokens = vec![Token::new(
            TokenKind::StringConst("hello world".to_string()),
            0,
            0,
        )];

        let mut parser = Parser::new(&tokens);

        let res = parser.parse_unit()?;

        match res.unwrap() {
            ActionKind::ConstString(_) => return Ok(()),
            _ => panic!("Wrong value"),
        }
    }

    #[test]
    fn parse_empty_call() -> Result<(), Box<dyn Error>> {
        let tokens = vec![
            Token::new(TokenKind::Identifier("Body"), 0, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BracketOpen), 1, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BracketClose), 2, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BlockOpen), 3, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BlockClose), 4, 0),
        ];

        let mut parser = Parser::new(&tokens);

        let res = parser.parse_call()?;

        match res {
            ActionKind::FunctionSequence {
                tag_name,
                arguments,
                body,
            } => return Ok(()),
            _ => panic!("Wrong value"),
        }
    }

    #[test]
    fn parse_empty_with_one_argument() -> Result<(), Box<dyn Error>> {
        let tokens = vec![
            Token::new(TokenKind::Identifier("Body"), 0, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BracketOpen), 1, 0),
            Token::new(TokenKind::Identifier("arg1"), 0, 0),
            Token::new(TokenKind::Operator(OperatorKind::Assign), 0, 0),
            Token::new(TokenKind::StringConst("hello world".to_string()), 0, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BracketClose), 2, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BlockOpen), 3, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BlockClose), 4, 0),
        ];

        let mut parser = Parser::new(&tokens);

        let res = parser.parse_call()?;

        match res {
            ActionKind::FunctionSequence {
                tag_name,
                arguments,
                body,
            } => {
                assert!(matches!(
                    arguments.get("arg1").unwrap(),
                    ActionKind::ConstString(_)
                ));
                return Ok(());
            }
            _ => panic!("Wrong value"),
        }
    }
}
