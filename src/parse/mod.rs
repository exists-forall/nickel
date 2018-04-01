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

pub fn type_(s: &str) -> ParseResult<syntax::Type> {
    grammar::TypeParser::new().parse(lex::Lexer::from_str(s))
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

    fn mk_ident(s: &str) -> syntax::Ident {
        syntax::Ident {
            name: s.to_owned(),
            collision_id: 0,
        }
    }

    fn ty_var(s: &str) -> syntax::Type {
        syntax::Type::Var { ident: mk_ident(s) }
    }

    #[test]
    fn test_type() {
        assert_eq!(
            type_("( // embedded whitespace \n )"),
            Ok(syntax::Type::Unit)
        );

        assert_eq!(type_("hello"), Ok(ty_var("hello")));

        assert_eq!(type_("(((((hello)))))"), Ok(ty_var("hello")));

        assert_eq!(
            type_("foo(bar)"),
            Ok(syntax::Type::App {
                constructor: Box::new(ty_var("foo")),
                param: Box::new(ty_var("bar")),
            })
        );

        assert_eq!(
            type_("foo(bar; baz)"),
            Ok(syntax::Type::App {
                constructor: Box::new(syntax::Type::App {
                    constructor: Box::new(ty_var("foo")),
                    param: Box::new(ty_var("bar")),
                }),
                param: Box::new(ty_var("baz")),
            })
        );

        assert_eq!(
            type_("foo(bar; baz;)"),
            Ok(syntax::Type::App {
                constructor: Box::new(syntax::Type::App {
                    constructor: Box::new(ty_var("foo")),
                    param: Box::new(ty_var("bar")),
                }),
                param: Box::new(ty_var("baz")),
            })
        );

        assert_eq!(
            type_("exists {t : *} t"),
            Ok(syntax::Type::Exists {
                param: syntax::TypeParam {
                    ident: mk_ident("t"),
                    kind: types::Kind::Type,
                },
                body: Box::new(ty_var("t")),
            })
        );

        assert_eq!(
            type_("foo -> bar"),
            Ok(syntax::Type::Func {
                params: Vec::new(),
                arg: Box::new(ty_var("foo")),
                ret: Box::new(ty_var("bar")),
            })
        );

        assert_eq!(
            type_("forall {t : *} t -> foo"),
            Ok(syntax::Type::Func {
                params: vec![
                    syntax::TypeParam {
                        ident: mk_ident("t"),
                        kind: types::Kind::Type,
                    },
                ],
                arg: Box::new(ty_var("t")),
                ret: Box::new(ty_var("foo")),
            })
        );

        assert_eq!(
            type_("foo, bar, baz"),
            Ok(syntax::Type::Pair {
                left: Box::new(ty_var("foo")),
                right: Box::new(syntax::Type::Pair {
                    left: Box::new(ty_var("bar")),
                    right: Box::new(ty_var("baz")),
                }),
            })
        );

        // Full example:

        assert_eq!(
            type_("exists {f : (*) -> *} (Functor(f), f(T))"),
            Ok(syntax::Type::Exists {
                param: syntax::TypeParam {
                    ident: mk_ident("f"),
                    kind: types::Kind::Constructor {
                        params: Rc::new(vec![types::Kind::Type]),
                        result: Rc::new(types::Kind::Type),
                    },
                },
                body: Box::new(syntax::Type::Pair {
                    left: Box::new(syntax::Type::App {
                        constructor: Box::new(ty_var("Functor")),
                        param: Box::new(ty_var("f")),
                    }),
                    right: Box::new(syntax::Type::App {
                        constructor: Box::new(ty_var("f")),
                        param: Box::new(ty_var("T")),
                    }),
                }),
            })
        );
    }
}
