pub mod syntax;
pub mod grammar;
pub mod lex;
pub mod names;
pub mod to_internal;

use lalrpop_util::ParseError;

type ParseResult<T> = Result<T, ParseError<usize, lex::Token, lex::Error>>;

pub fn ident(s: &str) -> ParseResult<syntax::Ident> {
    grammar::IdentParser::new().parse(lex::Lexer::from_str(s))
}

pub fn type_(s: &str) -> ParseResult<syntax::Type> {
    grammar::TypeParser::new().parse(lex::Lexer::from_str(s))
}

pub fn expr(s: &str) -> ParseResult<syntax::Expr> {
    grammar::ExprParser::new().parse(lex::Lexer::from_str(s))
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use super::syntax::Ident;
    use expr;
    use types;
    use test_utils::parse_syntax::*;

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

        assert!(ws("-- a comment").is_ok());
        assert!(ws("   -- a comment \n \t \n -- another comment  \n   ").is_ok());

        assert!(ws(" - ").is_err());
        assert!(ws(" hello ").is_err());
        assert!(ws(" -- a comment \n not a comment").is_err());
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
            ident("foo -- comment 1 \n # -- comment 2 \n 42"),
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

    fn ty_var(s: &str) -> syntax::Type {
        syntax::Type::Var { ident: mk_ident(s) }
    }

    #[test]
    fn test_type() {
        assert_eq!(
            type_("( -- embedded whitespace \n )"),
            Ok(syntax::Type::Unit)
        );

        assert_eq!(type_("hello"), Ok(ty_var("hello")));

        assert_eq!(type_("(((((hello)))))"), Ok(ty_var("hello")));

        assert_eq!(
            type_("foo bar"),
            Ok(syntax::Type::App {
                constructor: Box::new(ty_var("foo")),
                param: Box::new(ty_var("bar")),
            })
        );

        assert_eq!(
            type_("foo bar baz"),
            Ok(syntax::Type::App {
                constructor: Box::new(syntax::Type::App {
                    constructor: Box::new(ty_var("foo")),
                    param: Box::new(ty_var("bar")),
                }),
                param: Box::new(ty_var("baz")),
            })
        );

        assert_eq!(
            type_("foo bar baz"),
            Ok(syntax::Type::App {
                constructor: Box::new(syntax::Type::App {
                    constructor: Box::new(ty_var("foo")),
                    param: Box::new(ty_var("bar")),
                }),
                param: Box::new(ty_var("baz")),
            })
        );

        assert_eq!(
            type_("exists {t} t"),
            Ok(syntax::Type::Quantified {
                quantifier: types::Quantifier::Exists,
                param: syntax::TypeParam {
                    ident: mk_ident("t"),
                },
                body: Box::new(ty_var("t")),
            })
        );

        assert_eq!(
            type_("foo -> bar"),
            Ok(syntax::Type::Func {
                arg: Box::new(ty_var("foo")),
                arg_phase: types::Phase::Dynamic,
                ret: Box::new(ty_var("bar")),
                ret_phase: types::Phase::Dynamic,
            })
        );

        assert_eq!(
            type_("(static foo) -> bar"),
            Ok(syntax::Type::Func {
                arg: Box::new(ty_var("foo")),
                arg_phase: types::Phase::Static,
                ret: Box::new(ty_var("bar")),
                ret_phase: types::Phase::Dynamic,
            })
        );

        assert_eq!(
            type_("foo -> static bar"),
            Ok(syntax::Type::Func {
                arg: Box::new(ty_var("foo")),
                arg_phase: types::Phase::Dynamic,
                ret: Box::new(ty_var("bar")),
                ret_phase: types::Phase::Static,
            })
        );

        assert_eq!(
            type_("(static foo) -> static bar"),
            Ok(syntax::Type::Func {
                arg: Box::new(ty_var("foo")),
                arg_phase: types::Phase::Static,
                ret: Box::new(ty_var("bar")),
                ret_phase: types::Phase::Static,
            })
        );

        assert_eq!(
            type_("forall {t} t -> foo"),
            Ok(syntax::Type::Quantified {
                quantifier: types::Quantifier::ForAll,
                param: syntax::TypeParam {
                    ident: mk_ident("t"),
                },
                body: Box::new(syntax::Type::Func {
                    arg: Box::new(ty_var("t")),
                    arg_phase: types::Phase::Dynamic,
                    ret: Box::new(ty_var("foo")),
                    ret_phase: types::Phase::Dynamic,
                }),
            })
        );

        assert_eq!(
            type_("forall {T} {U} {V} (T, U, V)"),
            Ok(syntax::Type::Quantified {
                quantifier: types::Quantifier::ForAll,
                param: syntax::TypeParam {
                    ident: mk_ident("T"),
                },
                body: Box::new(syntax::Type::Quantified {
                    quantifier: types::Quantifier::ForAll,
                    param: syntax::TypeParam {
                        ident: mk_ident("U"),
                    },
                    body: Box::new(syntax::Type::Quantified {
                        quantifier: types::Quantifier::ForAll,
                        param: syntax::TypeParam {
                            ident: mk_ident("V"),
                        },
                        body: Box::new(syntax::Type::Pair {
                            left: Box::new(ty_var("T")),
                            right: Box::new(syntax::Type::Pair {
                                left: Box::new(ty_var("U")),
                                right: Box::new(ty_var("V")),
                            }),
                        }),
                    }),
                }),
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

        assert_eq!(
            type_("foo, bar, baz,"),
            Ok(syntax::Type::Pair {
                left: Box::new(ty_var("foo")),
                right: Box::new(syntax::Type::Pair {
                    left: Box::new(ty_var("bar")),
                    right: Box::new(ty_var("baz")),
                }),
            })
        );

        assert_eq!(
            type_("equiv a b"),
            Ok(syntax::Type::Equiv {
                orig: Box::new(ty_var("a")),
                dest: Box::new(ty_var("b")),
            })
        );

        assert_eq!(
            type_("forall {T} T -> F T"),
            Ok(syntax::Type::Quantified {
                quantifier: types::Quantifier::ForAll,
                param: syntax::TypeParam {
                    ident: mk_ident("T"),
                },
                body: Box::new(syntax::Type::Func {
                    arg: Box::new(ty_var("T")),
                    arg_phase: types::Phase::Dynamic,
                    ret: Box::new(syntax::Type::App {
                        constructor: Box::new(ty_var("F")),
                        param: Box::new(ty_var("T")),
                    }),
                    ret_phase: types::Phase::Dynamic,
                }),
            })
        );

        assert_eq!(
            type_("forall {T} F T -> T"),
            Ok(syntax::Type::Quantified {
                quantifier: types::Quantifier::ForAll,
                param: syntax::TypeParam {
                    ident: mk_ident("T"),
                },
                body: Box::new(syntax::Type::Func {
                    arg: Box::new(syntax::Type::App {
                        constructor: Box::new(ty_var("F")),
                        param: Box::new(ty_var("T")),
                    }),
                    arg_phase: types::Phase::Dynamic,
                    ret: Box::new(ty_var("T")),
                    ret_phase: types::Phase::Dynamic,
                }),
            })
        );

        // Full example:

        assert_eq!(
            type_("exists {f} (Functor f, f T)"),
            Ok(syntax::Type::Quantified {
                quantifier: types::Quantifier::Exists,
                param: syntax::TypeParam {
                    ident: mk_ident("f"),
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

    fn ex_var(s: &str) -> syntax::Expr {
        syntax::Expr::Var {
            usage: expr::VarUsage::Copy,
            ident: mk_ident(s),
        }
    }

    fn ex_move_var(s: &str) -> syntax::Expr {
        syntax::Expr::Var {
            usage: expr::VarUsage::Move,
            ident: mk_ident(s),
        }
    }

    #[test]
    fn test_expr() {
        assert_eq!(
            expr("( -- embedded whitespace \n )"),
            Ok(syntax::Expr::Unit),
        );

        assert_eq!(expr("hello"), Ok(ex_var("hello")));

        assert_eq!(expr("move hello"), Ok(ex_move_var("hello")));

        assert_eq!(expr("((((hello))))"), Ok(ex_var("hello")));

        assert_eq!(
            expr("hello(move world)"),
            Ok(syntax::Expr::App {
                callee: Box::new(ex_var("hello")),
                arg: Box::new(ex_move_var("world")),
            })
        );

        assert_eq!(
            expr("hello{T}(move world)"),
            Ok(syntax::Expr::App {
                callee: Box::new(syntax::Expr::Inst {
                    receiver: Box::new(ex_var("hello")),
                    type_params: vec![ty_var("T")],
                }),
                arg: Box::new(ex_move_var("world")),
            })
        );

        assert_eq!(
            expr("hello{T}{U}(move world)"),
            Ok(syntax::Expr::App {
                callee: Box::new(syntax::Expr::Inst {
                    receiver: Box::new(ex_var("hello")),
                    type_params: vec![ty_var("T"), ty_var("U")],
                }),
                arg: Box::new(ex_move_var("world")),
            })
        );

        assert_eq!(
            expr("func (x : T) -> move x"),
            Ok(syntax::Expr::Func {
                arg_name: mk_ident("x"),
                arg_type: ty_var("T"),
                arg_phase: types::Phase::Dynamic,
                body: Box::new(ex_move_var("x")),
            })
        );

        assert_eq!(
            expr("func (static x : T) -> move x"),
            Ok(syntax::Expr::Func {
                arg_name: mk_ident("x"),
                arg_type: ty_var("T"),
                arg_phase: types::Phase::Static,
                body: Box::new(ex_move_var("x")),
            })
        );

        assert_eq!(
            expr("forall {T} func (x : T) -> move x"),
            Ok(syntax::Expr::ForAll {
                type_params: vec![
                    syntax::TypeParam {
                        ident: mk_ident("T"),
                    },
                ],
                body: Box::new(syntax::Expr::Func {
                    arg_name: mk_ident("x"),
                    arg_type: ty_var("T"),
                    arg_phase: types::Phase::Dynamic,
                    body: Box::new(ex_move_var("x")),
                }),
            })
        );

        assert_eq!(
            expr("forall {T} {U} func (x : T) -> move x"),
            Ok(syntax::Expr::ForAll {
                type_params: vec![
                    syntax::TypeParam {
                        ident: mk_ident("T"),
                    },
                    syntax::TypeParam {
                        ident: mk_ident("U"),
                    },
                ],
                body: Box::new(syntax::Expr::Func {
                    arg_name: mk_ident("x"),
                    arg_type: ty_var("T"),
                    arg_phase: types::Phase::Dynamic,
                    body: Box::new(ex_move_var("x")),
                }),
            })
        );

        assert_eq!(
            expr("let x = move y in move x"),
            Ok(syntax::Expr::Let {
                names: vec![mk_ident("x")],
                val: Box::new(ex_move_var("y")),
                body: Box::new(ex_move_var("x")),
            })
        );

        assert_eq!(
            expr("let x, y = move z in ()"),
            Ok(syntax::Expr::Let {
                names: vec![mk_ident("x"), mk_ident("y")],
                val: Box::new(ex_move_var("z")),
                body: Box::new(syntax::Expr::Unit),
            })
        );

        assert_eq!(
            expr("let x, y, = move z in ()"),
            Ok(syntax::Expr::Let {
                names: vec![mk_ident("x"), mk_ident("y")],
                val: Box::new(ex_move_var("z")),
                body: Box::new(syntax::Expr::Unit),
            })
        );

        assert_eq!(
            expr("let_exists {T} x = move y in move x"),
            Ok(syntax::Expr::LetExists {
                type_names: vec![mk_ident("T")],
                val_name: mk_ident("x"),
                val: Box::new(ex_move_var("y")),
                body: Box::new(ex_move_var("x")),
            })
        );

        assert_eq!(
            expr("let_exists {T} {U} x = move y in move x"),
            Ok(syntax::Expr::LetExists {
                type_names: vec![mk_ident("T"), mk_ident("U")],
                val_name: mk_ident("x"),
                val: Box::new(ex_move_var("y")),
                body: Box::new(ex_move_var("x")),
            })
        );

        assert_eq!(
            expr("make_exists {T = Foo} T of move x"),
            Ok(syntax::Expr::MakeExists {
                params: vec![(mk_ident("T"), ty_var("Foo"))],
                type_body: ty_var("T"),
                body: Box::new(ex_move_var("x")),
            })
        );

        assert_eq!(
            expr("make_exists {T = Foo} {U = Bar} T -> U of move f"),
            Ok(syntax::Expr::MakeExists {
                params: vec![
                    (mk_ident("T"), ty_var("Foo")),
                    (mk_ident("U"), ty_var("Bar")),
                ],
                type_body: syntax::Type::Func {
                    arg: Box::new(ty_var("T")),
                    arg_phase: types::Phase::Dynamic,
                    ret: Box::new(ty_var("U")),
                    ret_phase: types::Phase::Dynamic,
                },
                body: Box::new(ex_move_var("f")),
            })
        );

        assert_eq!(
            expr("foo, bar, baz"),
            Ok(syntax::Expr::Pair {
                left: Box::new(ex_var("foo")),
                right: Box::new(syntax::Expr::Pair {
                    left: Box::new(ex_var("bar")),
                    right: Box::new(ex_var("baz")),
                }),
            })
        );

        assert_eq!(
            expr("foo, bar, baz,"),
            Ok(syntax::Expr::Pair {
                left: Box::new(ex_var("foo")),
                right: Box::new(syntax::Expr::Pair {
                    left: Box::new(ex_var("bar")),
                    right: Box::new(ex_var("baz")),
                }),
            })
        );
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum ConvError {
        Parse(ParseError<usize, lex::Token, lex::Error>),
        Names(names::Error),
    }

    // Parse a type and convert it to an internal representation
    fn conv_ty(free_types: &[&str], s: &str) -> Result<types::Type<Rc<String>>, ConvError> {
        let mut type_names = names::Names::new();
        for ty in free_types {
            type_names.add_name(mk_ident(ty)).map_err(ConvError::Names)?;
        }

        let result =
            to_internal::convert_type(&mut type_names, type_(s).map_err(ConvError::Parse)?)
                .map_err(ConvError::Names)?;

        assert_eq!(result.free(), free_types.len());

        Ok(result)
    }

    #[test]
    fn convert_type() {
        use test_utils::types::*;

        assert_eq!(conv_ty(&[], "()"), Ok(unit(0)));
        assert_eq!(conv_ty(&["foo", "bar"], "()"), Ok(unit(2)));

        assert_eq!(conv_ty(&["foo", "bar"], "foo"), Ok(var(2, 0)));
        assert_eq!(conv_ty(&["foo", "bar"], "bar"), Ok(var(2, 1)));

        assert_eq!(
            conv_ty(&[], "forall {T} T"),
            Ok(quantified_named(types::Quantifier::ForAll, "T", var(1, 0)))
        );

        assert_eq!(
            conv_ty(&["T"], "forall {U} (T, U)"),
            Ok(quantified_named(
                types::Quantifier::ForAll,
                "U",
                pair(var(2, 0), var(2, 1))
            ))
        );

        assert_eq!(
            conv_ty(&[], "exists {T} T"),
            Ok(quantified_named(types::Quantifier::Exists, "T", var(1, 0)))
        );

        assert_eq!(
            conv_ty(&["T"], "exists {U} (T, U)"),
            Ok(quantified_named(
                types::Quantifier::Exists,
                "U",
                pair(var(2, 0), var(2, 1))
            ))
        );

        assert_eq!(conv_ty(&["F", "X"], "F X"), Ok(app(var(2, 0), var(2, 1))));

        assert_eq!(
            conv_ty(&["A", "B"], "A -> B"),
            Ok(func(var(2, 0), var(2, 1)))
        );

        assert_eq!(
            conv_ty(&["A", "B", "C", "D"], "A B -> C D"),
            Ok(func(app(var(4, 0), var(4, 1)), app(var(4, 2), var(4, 3))))
        );

        assert_eq!(
            conv_ty(&["A", "B"], "equiv A B"),
            Ok(equiv_ty(var(2, 0), var(2, 1)))
        );
    }

    // Parse an expression and convert it to an internal representation
    fn conv(
        free_vars: &[&str],
        free_types: &[&str],
        s: &str,
    ) -> Result<expr::Expr<Rc<String>>, ConvError> {
        let mut var_names = names::Names::new();
        for var in free_vars {
            var_names.add_name(mk_ident(var)).map_err(ConvError::Names)?;
        }

        let mut type_names = names::Names::new();
        for ty in free_types {
            type_names.add_name(mk_ident(ty)).map_err(ConvError::Names)?;
        }

        let result = to_internal::convert_expr(
            &mut to_internal::Context {
                var_names,
                type_names,
            },
            expr(s).map_err(ConvError::Parse)?,
        ).map_err(ConvError::Names)?;

        assert_eq!(result.free_vars(), free_vars.len());
        assert_eq!(result.free_types(), free_types.len());

        Ok(result)
    }

    #[test]
    fn convert_expr() {
        use test_utils::expr as ex;
        use test_utils::types as ty;
        use expr::VarUsage as Usage;

        assert_eq!(conv(&[], &[], "()"), Ok(ex::unit(0, 0)));

        assert_eq!(
            conv(&["foo"], &[], "foo"),
            Ok(ex::var(Usage::Copy, 1, 0, 0))
        );

        assert_eq!(
            conv(&["foo"], &[], "move foo"),
            Ok(ex::var(Usage::Move, 1, 0, 0))
        );

        assert_eq!(
            conv(&[], &["T"], "func (x : T) -> move x"),
            Ok(ex::func_named(
                "x",
                ty::var(1, 0),
                ex::var(Usage::Move, 1, 1, 0),
            ))
        );

        assert_eq!(
            conv(&[], &[], "forall {T} func (x : T) -> move x"),
            Ok(ex::func_forall_named(
                &["T"],
                "x",
                ty::var(1, 0),
                ex::var(Usage::Move, 1, 1, 0),
            ))
        );

        assert_eq!(
            conv(&["foo", "bar"], &[], "foo(move bar)"),
            Ok(ex::app(
                ex::var(Usage::Copy, 2, 0, 0),
                ex::var(Usage::Move, 2, 0, 1),
            ))
        );

        assert_eq!(
            conv(&["foo", "bar"], &["T", "U"], "foo{T}{U}(move bar)"),
            Ok(ex::app_forall(
                ex::var(Usage::Copy, 2, 2, 0),
                &[ty::var(2, 0), ty::var(2, 1)],
                ex::var(Usage::Move, 2, 2, 1),
            ))
        );

        assert_eq!(
            conv(&["foo", "bar", "baz"], &[], "foo, bar, baz"),
            Ok(ex::pair(
                ex::var(Usage::Copy, 3, 0, 0),
                ex::pair(ex::var(Usage::Copy, 3, 0, 1), ex::var(Usage::Copy, 3, 0, 2),),
            ))
        );

        assert_eq!(
            conv(&[], &[], "let x = () in move x"),
            Ok(ex::let_vars_named(
                &["x"],
                ex::unit(0, 0),
                ex::var(Usage::Move, 1, 0, 0),
            ))
        );

        assert_eq!(
            conv(&[], &[], "let x, y, z = () in (x, y, z)"),
            Ok(ex::let_vars_named(
                &["x", "y", "z"],
                ex::unit(0, 0),
                ex::pair(
                    ex::var(Usage::Copy, 3, 0, 0),
                    ex::pair(ex::var(Usage::Copy, 3, 0, 1), ex::var(Usage::Copy, 3, 0, 2),),
                ),
            ))
        );

        assert_eq!(
            conv(
                &["foo", "bar"],
                &[],
                "let_exists {T} {U} x = move foo in bar{T}{U}(move x)",
            ),
            Ok(ex::let_exists_named(
                &["T", "U"],
                "x",
                ex::var(Usage::Move, 2, 0, 0),
                ex::app_forall(
                    ex::var(Usage::Copy, 3, 2, 1),
                    &[ty::var(2, 0), ty::var(2, 1)],
                    ex::var(Usage::Move, 3, 2, 2),
                ),
            ))
        );

        assert_eq!(
            conv(
                &["foo", "bar"],
                &["Foo", "Bar"],
                "make_exists {T = Foo} {U = Bar} (T, U) of (move foo, move bar)",
            ),
            Ok(ex::make_exists_named(
                &[("T", ty::var(2, 0)), ("U", ty::var(2, 1))],
                ty::pair(ty::var(4, 2), ty::var(4, 3)),
                ex::pair(ex::var(Usage::Move, 2, 2, 0), ex::var(Usage::Move, 2, 2, 1),),
            ))
        );

        assert_eq!(
            conv(
                &["foo"],
                &["Foo", "F"],
                "func (wrap : forall {T} T -> F T) -> wrap{Foo}(foo)"
            ),
            Ok(ex::func_named(
                "wrap",
                ty::func_forall_named(&["T"], ty::var(3, 2), ty::app(ty::var(3, 1), ty::var(3, 2))),
                ex::app_forall(
                    ex::var(Usage::Copy, 2, 2, 1),
                    &[ty::var(2, 0)],
                    ex::var(Usage::Copy, 2, 2, 0)
                )
            ))
        );

        assert_eq!(
            conv(
                &["foo", "token"],
                &["Foo"],
                "cast {T} (Foo, T) by token of foo"
            ),
            Ok(ex::cast_named(
                "T",
                ty::pair(ty::var(2, 0), ty::var(2, 1)),
                ex::var(Usage::Copy, 2, 1, 1),
                ex::var(Usage::Copy, 2, 1, 0)
            ))
        );
    }
}
