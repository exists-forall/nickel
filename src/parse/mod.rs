pub mod syntax;
pub mod grammar;
pub mod lex;

use lalrpop_util::ParseError;

pub fn ident(s: &str) -> Result<syntax::Ident, ParseError<usize, lex::Token, lex::Error>> {
    grammar::IdentParser::new().parse(lex::Lexer::from_str(s))
}

#[cfg(test)]
mod test {
    use super::*;
    use super::syntax::Ident;

    fn name(s: &str) -> Result<String, ParseError<usize, lex::Token, lex::Error>> {
        grammar::RawNameParser::new().parse(lex::Lexer::from_str(s))
    }

    fn ws(s: &str) -> Result<(), ParseError<usize, lex::Token, lex::Error>> {
        grammar::WhitespaceParser::new().parse(lex::Lexer::from_str(s))
    }

    #[test]
    fn unquoted_name() {
        assert_eq!(name("hello"), Ok("hello".to_owned()));
        assert_eq!(name("HeLlO_wOrLd"), Ok("HeLlO_wOrLd".to_owned()));
        assert_eq!(name("_foo_bar_42_baz0"), Ok("_foo_bar_42_baz0".to_owned()));

        assert!(name("42").is_err());
        assert!(name("-hello").is_err());
        assert!(name("hello world").is_err());
    }

    #[test]
    fn quoted_name() {
        assert_eq!(name("`hello`"), Ok("hello".to_owned()));
        assert_eq!(name("`hello world`"), Ok("hello world".to_owned()));
        assert_eq!(name("`hello\\\\world`"), Ok("hello\\world".to_owned()));
        assert_eq!(name("`hello\\`world`"), Ok("hello`world".to_owned()));

        assert!(name("` ` `").is_err());
    }

    #[test]
    fn whitespace() {
        assert!(ws("").is_ok());
        assert!(ws("  \t \n    \r \x0B  \n \n \t").is_ok());

        assert!(ws("// a comment").is_ok());
        assert!(ws("   // a comment \n \t \n // another comment  \n   ").is_ok());

        assert!(ws(" - ").is_err());
        assert!(ws(" hello ").is_err());
        assert!(ws(" // a comment \n not a comment").is_err());
    }

    #[test]
    fn no_collision_ident() {
        assert_eq!(
            ident("foo"),
            Ok(Ident {
                name: "foo".to_owned(),
                collision_id: 0,
            })
        );

        assert_eq!(
            ident("`hello \\` world`"),
            Ok(Ident {
                name: "hello ` world".to_owned(),
                collision_id: 0,
            })
        );
    }

    #[test]
    fn collision_ident() {
        assert_eq!(
            ident("foo#42"),
            Ok(Ident {
                name: "foo".to_owned(),
                collision_id: 42,
            })
        );

        assert_eq!(
            ident("foo // comment 1 \n # // comment 2 \n 42"),
            Ok(Ident {
                name: "foo".to_owned(),
                collision_id: 42,
            })
        );

        assert_eq!(
            ident("`quoted ident`#005"),
            Ok(Ident {
                name: "quoted ident".to_owned(),
                collision_id: 5,
            })
        );

        assert!(ident("foo#bar").is_err());
    }
}
