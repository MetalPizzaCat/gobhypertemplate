pub mod action;
pub mod lexer;
pub mod parser;
pub mod state;

fn main() {
    /*
    Body{
        Text(class =  "bla"){ "some value or rather" .. $variable .. "adad" }
    }


     */
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{lexer::Lexer, parser::Parser, state::State};

    #[test]
    fn test_full_gen_simple() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new("\"hello world\"");
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        assert_eq!(1, tokens.len());
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser.parse_unit()?.unwrap().generate(&mut state)?.unwrap(),
            "hello world".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_full_gen_tag() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new("p(class = \"bla\"){\"hello world\";}");
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser.parse_unit()?.unwrap().generate(&mut state)?.unwrap(),
            "<p class=\"bla\">hello world</p>".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_full_gen_tag_multiple_statements() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new("p(class = \"bla\"){\"hello world\"; \"hehe\";}");
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser.parse_unit()?.unwrap().generate(&mut state)?.unwrap(),
            "<p class=\"bla\">hello worldhehe</p>".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_full_gen_concat() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new("p(){\"hello world\" @ \"k\" @ \"hehe\";}");
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser.parse_unit()?.unwrap().generate(&mut state)?.unwrap(),
            "<p >hello worldkhehe</p>".to_owned()
        );

        Ok(())
    }
}
