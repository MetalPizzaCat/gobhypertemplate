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
        let mut lexer = Lexer::new("\"hello world\";");
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        assert_eq!(2, tokens.len());
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser.parse_body()?.generate(&mut state)?.unwrap(),
            "hello world".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_full_gen_tag() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new("p(class = \"bla\"){\"hello world\";};");
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser.parse_body()?.generate(&mut state)?.unwrap(),
            "<p class=\"bla\">hello world</p>".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_full_gen_tag_multiple() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new(
            "p(class = \"bla\"){\"hello world\";}; p(class = \"bla\"){\"hello world\";};",
        );
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser.parse_body()?.generate(&mut state)?.unwrap(),
            "<p class=\"bla\">hello world</p><p class=\"bla\">hello world</p>".to_owned()
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
    fn test_full_gen_concat_no_func() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new("\"hello world\" @ \"k\";");
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser
                .parse_expression()?
                .unwrap()
                .generate(&mut state)?
                .unwrap(),
            "hello worldk".to_owned()
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

    #[test]
    fn test_full_user_function() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new(
            r#"
		def bla(){ "bla";};
		%bla();
		%bla();
		"#,
        );
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser.parse_body()?.generate(&mut state)?.unwrap(),
            "blabla".to_owned()
        );

        Ok(())
    }

	#[test]
    fn test_full_user_function_multiple() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new(
            r#"
		def bla(){ "bla";};
		def f1(){ p(){"f1";};};
		%bla();
		%f1();
		"#,
        );
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert_eq!(
            parser.parse_body()?.generate(&mut state)?.unwrap(),
            "bla<p >f1</p>".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_full_user_function_decl() -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new(r#"def bla(){ "bla";}"#);
        lexer.tokenize()?;
        let tokens = lexer.get_tokens();
        let mut parser = Parser::new(&tokens);
        let mut state = State::default();
        assert!(
            parser
                .parse_function_definition()?
                .unwrap()
                .generate(&mut state)?
                .is_none()
        );

		assert!(state.get_user_function("bla").is_some());

        Ok(())
    }

	// #[test]
    // fn test_full_user_function_decl_with_arg() -> Result<(), Box<dyn Error>> {
    //     let mut lexer = Lexer::new(r#"def bla(a){ $a;}"#);
    //     lexer.tokenize()?;
    //     let tokens = lexer.get_tokens();
    //     let mut parser = Parser::new(&tokens);
    //     let mut state = State::default();
    //     assert!(
    //         parser
    //             .parse_function_definition()?
    //             .unwrap()
    //             .generate(&mut state)?
    //             .is_none()
    //     );

	// 	assert!(state.get_user_function("bla").is_some());

    //     Ok(())
    // }
}
