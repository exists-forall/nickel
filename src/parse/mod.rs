pub mod syntax;
pub mod grammar;

use lalrpop_util::ParseError;

pub fn name(s: &str) -> Result<String, ParseError<usize, grammar::Token, &'static str>> {
    grammar::NameParser::new().parse(s)
}

pub fn ident(s: &str) -> Result<syntax::Ident, ParseError<usize, grammar::Token, &'static str>> {
    grammar::IdentParser::new().parse(s)
}

#[cfg(test)]
mod test {
    use super::*;
    use super::syntax::Ident;

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
        let ws = grammar::WsParser::new();

        assert!(ws.parse("").is_ok());
        assert!(ws.parse("  \t \n    \r \x0B  \n \n \t").is_ok());

        assert!(ws.parse("// a comment").is_ok());
        assert!(
            ws.parse("   // a comment \n \t \n // another comment  \n   ")
                .is_ok()
        );

        assert!(ws.parse(" - ").is_err());
        assert!(ws.parse(" hello ").is_err());
        assert!(ws.parse(" // a comment \n not a comment").is_err());
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
