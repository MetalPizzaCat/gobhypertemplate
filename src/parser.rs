use std::collections::HashMap;

use crate::{
    action::ActionKind,
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

/// Check if current token matches the provided value and if it does, advance to the next symbol discarding current value
/// Otherwise a parsing value is returned
macro_rules! consume_token {
    ($self:expr, $a:pat, $err:expr) => {
        if let Some(tok) = $self.tokens.peek()
            && matches!(tok.get_kind(), $a)
        {
            $self.tokens.next();
            Ok(())
        } else {
            Err(ParsingError::new_with_message(
                crate::lexer::ParsingErrorKind::UnexpectedCharacter,
                $err,
                $self.get_row(),
                $self.get_column(),
            ))
        }
    };
}

/// Attempt to get the inner value of the kind field of the token
/// If it doesn't match the provided type it will throw an error.
///
/// # Example
/// ```no_run
/// get_token_value_of_kind(self, TokenKind::Separator(SeparatorKind::BracketOpen), "Expected '('".to_owned())
/// ```
macro_rules! get_token_value_of_kind {
    ($self:expr, $t:path, $err:expr) => {
        if let Some(tok) = $self.tokens.peek() {
            match tok.get_kind() {
                $t(s) => Ok(s),
                _ => Err(ParsingError::new_with_message(
                    crate::lexer::ParsingErrorKind::UnexpectedCharacter,
                    $err,
                    $self.get_row(),
                    $self.get_column(),
                )),
            }
        } else {
            Err(ParsingError::new_with_message(
                crate::lexer::ParsingErrorKind::UnexpectedCharacter,
                $err,
                $self.get_row(),
                $self.get_column(),
            ))
        }
    };
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
            if act.is_some() {
                self.next();
            }
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
            consume_token!(
                self,
                TokenKind::Operator(OperatorKind::Assign),
                "Expected '='".to_owned()
            )?;
            return Ok(Some((arg_name, self.parse_unit()?.unwrap())));
        }
        Ok(None)
    }

    pub fn get_binary_operation_priority(&mut self) -> i32 {
        if let Some(t) = self.tokens.peek() {
            t.get_priority()
        } else {
            -1
        }
    }

    pub fn parse_binary_right_side(
        &mut self,
        priority: i32,
        left: ActionKind<'a>,
    ) -> Result<Option<ActionKind<'a>>, ParsingError> {
        let mut left = left;
        loop {
            let tok_priority: i32 = self.get_binary_operation_priority();

            if tok_priority < priority {
                return Ok(Some(left));
            }

            let op = get_token_value_of_kind!(
                self,
                TokenKind::Operator,
                "Expected an operator".to_owned()
            )?;
            self.next();

            // we start by trying to parse anything that might be on the right
            if let Some(mut right_val) = self.parse_unit()? {
                let next_priority = self.get_binary_operation_priority();
                // however we also need to check if next operation should be calculated before current one
                if tok_priority < next_priority {
                    // in which case we do and just replace previous parsed value with this
                    if let Some(right_next) =
                        self.parse_binary_right_side(tok_priority + 1, right_val)?
                    {
                        right_val = right_next
                    } else {
                        return Ok(None);
                    }
                }
                // regardless we have to combine existing data into a new operator and continue with the parsing
                left = ActionKind::BinaryOperation {
                    op: op,
                    left: Box::new(left),
                    right: Box::new(right_val),
                };
            } else {
                // and if we don't find anything we conclude that it is not a binary expression
                // this WILL mess up rest of the parser due to consumption of the operator
                // but that's by design, an operator should not just exist freely
                return Ok(None);
            }
        }
    }

    pub fn parse_expression(&mut self) -> Result<Option<ActionKind<'a>>, ParsingError> {
        let left = self.parse_unit()?;
        if let Some(left) = left {
            return self.parse_binary_right_side(0, left);
        } else {
            Ok(None)
        }
    }

    pub fn parse_call(&mut self) -> Result<ActionKind<'a>, ParsingError> {
        let id = get_token_value_of_kind!(
            self,
            TokenKind::Identifier,
            "Expected function name".to_owned()
        )?;
        self.next();
        consume_token!(
            self,
            TokenKind::Separator(SeparatorKind::BracketOpen),
            "Expected '('".to_owned()
        )?;
        let mut arguments: HashMap<&'a str, ActionKind<'a>> = HashMap::new();
        while let Some((arg, act)) = self.parse_argument()? {
            arguments.insert(arg, act);
            if !is_current_of_kind!(self, TokenKind::Separator(SeparatorKind::Comma)) {
                break;
            }
        }
        consume_token!(
            self,
            TokenKind::Separator(SeparatorKind::BracketClose),
            "Expected ')'".to_owned()
        )?;

        let mut body: Vec<ActionKind> = Vec::new();
        consume_token!(
            self,
            TokenKind::Separator(SeparatorKind::BlockOpen),
            "Expected '{'".to_owned()
        )?;
        // loop {
        //     if let Some(act) = self.parse_unit()? {
        //         self.consume_separator(SeparatorKind::Semicolon)?;
        //         body.push(act);
        //     } else if is_current_of_kind!(self, TokenKind::Separator(SeparatorKind::BlockClose)) {
        //         break;
        //     }
        // }
        while let Some(act) = self.parse_expression()? {
            consume_token!(
                self,
                TokenKind::Separator(SeparatorKind::Semicolon),
                "Expected ';' after the end of the expression".to_owned()
            )?;
            body.push(act);
            if is_current_of_kind!(self, TokenKind::Separator(SeparatorKind::BlockClose)) {
                break;
            }
        }

        consume_token!(
            self,
            TokenKind::Separator(SeparatorKind::BlockClose),
            "Expected '}'".to_owned()
        )?;

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
        action::ActionKind,
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
    fn parse_empty_call_with_one_statement() -> Result<(), Box<dyn Error>> {
        let tokens = vec![
            Token::new(TokenKind::Identifier("Body"), 0, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BracketOpen), 1, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BracketClose), 2, 0),
            Token::new(TokenKind::Separator(SeparatorKind::BlockOpen), 3, 0),
            Token::new(TokenKind::StringConst("hello world".to_string()), 0, 0),
            Token::new(TokenKind::Separator(SeparatorKind::Semicolon), 3, 0),
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
                assert_eq!(body.len(), 1);
                return Ok(());
            }
            _ => panic!("Wrong value"),
        }
    }

    #[test]
    fn parse_expression() -> Result<(), Box<dyn Error>> {
        let tokens = vec![
            Token::new(TokenKind::StringConst("hello world".to_string()), 0, 0),
            Token::new(TokenKind::Operator(OperatorKind::StringConcat), 0, 0),
            Token::new(TokenKind::StringConst("hello world".to_string()), 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        let res = parser.parse_expression()?.unwrap();
        match res {
            ActionKind::BinaryOperation { op, left, right } => return Ok(()),
            _ => panic!(""),
        }
    }


    #[test]
    fn parse_expression_multiple() -> Result<(), Box<dyn Error>> {
        let tokens = vec![
            Token::new(TokenKind::StringConst("hello".to_string()), 0, 0),
            Token::new(TokenKind::Operator(OperatorKind::StringConcat), 0, 0),
            Token::new(TokenKind::StringConst("world".to_string()), 0, 0),
            Token::new(TokenKind::Operator(OperatorKind::StringConcat), 0, 0),
            Token::new(TokenKind::StringConst("!".to_string()), 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        let res = parser.parse_expression()?.unwrap();
        let s = res.to_string();
        match res {
            ActionKind::BinaryOperation { op, left, right } => match *left {
                ActionKind::BinaryOperation { op, left, right } => return Ok(()),
                _ => panic!("expected nested expr: {s}"),
            },
            _ => panic!("expected expr: {s}"),
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
