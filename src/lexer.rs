use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ParsingErrorKind {
    UnknownCharacter,
    InvalidIntegerConstant,
    InvalidNumberConstant,
    UnexpectedCharacter,
}

#[derive(Debug)]

pub struct ParsingError {
    error_type: ParsingErrorKind,
    message: Option<String>,
    row: usize,
    column: usize,
}

impl ParsingError {
    pub fn new(error_type: ParsingErrorKind, row: usize, column: usize) -> Self {
        Self {
            error_type,
            message: None,
            row,
            column,
        }
    }

    pub fn new_with_message(
        error_type: ParsingErrorKind,
        message: String,
        row: usize,
        column: usize,
    ) -> Self {
        Self {
            error_type,
            message: Some(message),
            row,
            column,
        }
    }
}

impl Error for ParsingError {}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}: {} Row: {} Column:{} ",
            self.error_type,
            self.message.clone().unwrap_or_else(|| "".to_owned()),
            self.row,
            self.column
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum SeparatorKind {
    BlockOpen,
    BlockClose,
    BracketOpen,
    BracketClose,
    Comma,
}
impl SeparatorKind {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            '(' => Some(SeparatorKind::BracketOpen),
            ')' => Some(SeparatorKind::BracketClose),
            '{' => Some(SeparatorKind::BlockOpen),
            '}' => Some(SeparatorKind::BlockClose),
            ',' => Some(SeparatorKind::Comma),
            _ => None,
        }
    }

    pub fn to_char(&self) -> char {
        match &self {
            SeparatorKind::BlockOpen => '{',
            SeparatorKind::BlockClose => '}',
            SeparatorKind::BracketOpen => '(',
            SeparatorKind::BracketClose => ')',
            SeparatorKind::Comma => ',',
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperatorKind {
    Assign,
    StringConcat,
}

impl OperatorKind {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            '=' => Some(OperatorKind::Assign),
            '@' => Some(OperatorKind::StringConcat),
            _ => None,
        }
    }

    pub fn to_char(&self) -> char {
        match &self {
            OperatorKind::Assign => '=',
            OperatorKind::StringConcat => '@',
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenKind<'a> {
    // any text that isn't enclosed in string, such as tag contents or tag fields
    Identifier(&'a str),
    // This is using php style variable names to make it easier
    Variable(&'a str),
    // any quote enclosed string
    StringConst(String),
    Separator(SeparatorKind),
    Operator(OperatorKind),
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    kind: TokenKind<'a>,
    row: usize,
    column: usize,
}

pub struct Lexer<'a> {
    code: &'a str,
    tokens: Vec<Token<'a>>,
    chars_indices: core::iter::Peekable<core::str::CharIndices<'a>>,
    column: usize,
    row: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind<'a>, row: usize, column: usize) -> Self {
        Self { kind, row, column }
    }

    pub fn get_kind(&self) -> TokenKind<'a> {
        self.kind.clone()
    }

    pub fn get_row(&self) -> usize {
        self.row
    }

    pub fn get_column(&self) -> usize {
        self.column
    }
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str) -> Self {
        Self {
            code,
            tokens: Vec::new(),
            chars_indices: code.char_indices().peekable(),
            row: 0,
            column: 0,
        }
    }

    /// Advance primary iterator and return the character. Adjusts column and row values.
    /// If string is over returns None
    fn next_char(&mut self) -> Option<char> {
        self.chars_indices
            .next()
            .inspect(|(idx, ch)| {
                if *ch == '\n' {
                    self.row += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
                }
            })
            .map(|(_, ch)| ch)
    }

    pub fn peek_char(&mut self) -> Option<char> {
        self.chars_indices.peek().map(|(_, ch)| ch).copied()
    }

    fn next_char_if<F>(&mut self, func: F) -> Option<char>
    where
        F: Fn(char) -> bool,
    {
        if func(self.chars_indices.peek().map(|(_, ch)| *ch)?) {
            self.next_char()
        } else {
            None
        }
    }

    pub fn skip_char_while<F>(&mut self, f: F)
    where
        F: Fn(char) -> bool,
    {
        while self.next_char_if(&f).is_some() {}
    }

    /// Advance code iterator by given amount or until the end of string
    fn advance_by(&mut self, amount: usize) {
        let mut i: usize = 0;
        while i < amount && self.chars_indices.next().is_some() {
            i += 1
        }
    }

    /// Check if next sequence of characters is the same as given string without advancing primary iterator
    pub fn is_string(&self, value: &'static str) -> bool {
        let mut tmp = self.chars_indices.clone();

        for i in value.chars() {
            let curr = tmp.next();
            if curr.is_none() || curr.unwrap().1 != i {
                return false;
            }
        }

        // check if we reached end of the line or the next character is not suitable
        return !tmp.next().is_some_and(|ch| ch.1.is_alphanumeric());
    }

    pub fn tokenize_id(&mut self) -> Option<Token<'a>> {
        if !self
            .peek_char()
            .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
        {
            return None;
        }

        let mut it = self.chars_indices.clone();

        let mut str_len: usize = 0;
        if let Some((start, _)) = it.peek().cloned() {
            while it
                .next_if(|(_, c)| c.is_alphanumeric() || *c == '_')
                .is_some()
            {
                str_len += 1;
            }
            let tok = Token::new(
                TokenKind::Identifier(&self.code[start..(start + str_len)]),
                self.row,
                self.column,
            );
            self.advance_by(str_len);
            return Some(tok);
        }

        None
    }

    pub fn tokenize_string(&mut self) -> Option<Token<'a>> {
        if !self.peek_char().is_some_and(|c| c == '"') {
            return None;
        }
        let mut it = self.chars_indices.clone();
        it.next();
        let mut const_str = String::new();
        let mut offset: usize = 1;
        while let Some((i, ch)) = it.next_if(|(_, ch)| *ch != '"') {
            if it.peek().is_some()
                && let Some(spec) = Self::convert_special(&self.code[i..(i + 2)])
            {
                it.next();
                const_str.push(spec);
                offset += 1;
            } else {
                const_str.push(ch);
            }
        }
        if it.peek().is_none_or(|(_, c)| *c != '"') {
            return None;
        }
        let tok = Token::new(TokenKind::StringConst(const_str), self.row, self.column);
        self.advance_by(offset + 1);

        Some(tok)
    }

    fn tokenize_separator(&mut self) -> Option<Token<'a>> {
        if let Some(ch) = self.peek_char().clone()
            && let Some(sep) = SeparatorKind::from_char(ch)
        {
            let tok = Token::new(TokenKind::Separator(sep), self.row, self.column);
            self.next_char();
            Some(tok)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Result<(), ParsingError> {
        self.skip_char_while(char::is_whitespace);
        let token = self
            .tokenize_separator()
            .or_else(|| self.tokenize_id())
            .or_else(|| self.tokenize_string())
            .ok_or(ParsingError::new(
                ParsingErrorKind::UnknownCharacter,
                self.row,
                self.column,
            ))?;
        self.tokens.push(token);
        Ok(())
    }

    pub const fn convert_special(ch: &str) -> Option<char> {
        match ch.as_bytes() {
            b"\\n" => Some('\n'),
            b"\\\"" => Some('"'),
            b"\\'" => Some('\''),
            b"\\t" => Some('\t'),
            b"\\\\" => Some('\\'),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, SeparatorKind, TokenKind};

    #[test]
    fn test_advance_by() {
        let code = "main()";
        let mut lexer = Lexer::new(code);
        lexer.skip_char_while(char::is_whitespace);
        lexer.advance_by("main".len());
        assert_eq!(lexer.chars_indices.peek().unwrap().1, '(');
    }

    #[test]
    fn test_dont_skip_char() {
        let code = "main()";
        let mut lexer = Lexer::new(code);
        assert!(lexer.tokenize_id().is_some());
        lexer.skip_char_while(char::is_whitespace);
        assert_eq!(lexer.chars_indices.peek().unwrap().1, '(');

        let res = lexer.tokenize_separator();
        assert!(matches!(
            res.unwrap().kind,
            TokenKind::Separator(SeparatorKind::BracketOpen)
        ));
    }

    #[test]
    fn test_id_advance() {
        let code = "main()";
        let mut lexer = Lexer::new(code);
        lexer.skip_char_while(char::is_whitespace);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert_eq!(lexer.chars_indices.peek().unwrap().1, '(');
    }

    #[test]
    fn test_id() {
        let code = "var1";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert!(matches!(res.unwrap().kind, TokenKind::Identifier("var1")));
    }

    #[test]
    fn test_id_extra() {
        let code = "var1=";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert!(matches!(res.unwrap().kind, TokenKind::Identifier("var1")));
    }

    #[test]
    fn test_id_extra2() {
        let code = " main(";
        let mut lexer = Lexer::new(code);
        lexer.skip_char_while(char::is_whitespace);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert!(matches!(res.unwrap().kind, TokenKind::Identifier("main")));
    }

    #[test]
    fn test_id_boundary() {
        let code = "va_r1()";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert!(matches!(res.unwrap().kind, TokenKind::Identifier("va_r1")));
    }

    #[test]
    fn test_string_full() {
        let code = r#""inner""#;
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_string();
        assert!(res.is_some());
        match res.unwrap().kind {
            TokenKind::StringConst(s) => assert_eq!(s, "inner".to_owned()),
            _ => panic!(),
        }
    }

    #[test]
    fn test_string_with_special() {
        let code = r#""in\"n\"er""#;
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_string();
        assert!(res.is_some());
        match res.unwrap().kind {
            TokenKind::StringConst(s) => assert_eq!(s, "in\"n\"er".to_owned()),
            _ => panic!(),
        }
    }

    #[test]
    fn test_string_incomplete() {
        let code = r#""inner"#;
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_string();
        assert!(res.is_none());
    }
}
