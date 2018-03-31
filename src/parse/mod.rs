pub mod syntax;
pub mod grammar;
pub mod lex;

use lalrpop_util::ParseError;

use types;

type ParseResult<T> = Result<T, ParseError<usize, lex::Token, lex::Error>>;

pub fn ident(s: &str) -> ParseResult<syntax::Ident> {
    grammar::IdentParser::new().parse(lex::Lexer::from_str(s))
}

pub fn kind(s: &str) -> ParseResult<types::Kind> {
    grammar::KindParser::new().parse(lex::Lexer::from_str(s))
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

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

    #[test]
    fn test_kind() {
        assert_eq!(kind("*"), Ok(types::Kind::Type));
        assert_eq!(kind("Place"), Ok(types::Kind::Place));
        assert_eq!(kind("Version"), Ok(types::Kind::Version));
        assert_eq!(
            kind(
                "(((( // an embedded comment \n * // another embedded comment \n ))))",
            ),
            Ok(types::Kind::Type)
        );
        assert_eq!(
            kind("(*) -> *"),
            Ok(types::Kind::Constructor {
                params: Rc::new(vec![types::Kind::Type]),
                result: Rc::new(types::Kind::Type),
            })
        );
        assert_eq!(
            kind("(*; Place; Version) -> *"),
            Ok(types::Kind::Constructor {
                params: Rc::new(vec![
                    types::Kind::Type,
                    types::Kind::Place,
                    types::Kind::Version,
                ]),
                result: Rc::new(types::Kind::Type),
            })
        );
        assert_eq!(
            kind("(*; (*) -> *; *;) -> Place"),
            Ok(types::Kind::Constructor {
                params: Rc::new(vec![
                    types::Kind::Type,
                    types::Kind::Constructor {
                        params: Rc::new(vec![types::Kind::Type]),
                        result: Rc::new(types::Kind::Type),
                    },
                    types::Kind::Type,
                ]),
                result: Rc::new(types::Kind::Place),
            })
        );
    }
}
