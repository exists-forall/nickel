use std::rc::Rc;

use pretty_trait::{Pretty, JoinExt, Group, Sep, Conditional, delimited};

use super::super::types::*;
use pretty_syntax::names::Names;
use pretty_syntax::block::block;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Root,
    ExistsBody,
    FuncArg,
    FuncRet,
    PairLeft,
    PairRight,
    AppLeft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KindPlace {
    Root,
    ConstructorResult,
}

pub fn kind_to_pretty(place: KindPlace, kind: &Kind) -> Box<Pretty> {
    match kind {
        &Kind::Type => Box::new("*"),
        &Kind::Place => Box::new("Place"),
        &Kind::Version => Box::new("Version"),
        &Kind::Constructor {
            ref params,
            ref result,
        } => {
            let params_pretty = delimited(
                &";".join(Sep(1)),
                params.iter().map(
                    |param| kind_to_pretty(KindPlace::Root, param),
                ),
            ).join(Conditional::OnlyBroken(";"));

            let result_pretty = kind_to_pretty(KindPlace::ConstructorResult, result);

            let content_pretty = Group::new("(".join(block(params_pretty)).join(")"))
                .join(" →")
                .join(Sep(1))
                .join(result_pretty);

            match place {
                KindPlace::Root => Box::new(Group::new(content_pretty)),
                KindPlace::ConstructorResult => Box::new(content_pretty),
            }
        }
    }
}

pub fn to_pretty<Name: Clone + Into<Rc<String>>>(
    names: &mut Names,
    place: Place,
    ty: Type<Name>,
) -> Box<Pretty> {
    assert_eq!(names.index_count(), ty.free());

    match ty.to_content() {
        TypeContent::Var { free: _, index } => Box::new(names.get_name(index)),

        TypeContent::Exists { param, body } => {
            names.push_scope();
            let name = names.add_name(param.name.into());
            let body_pretty = to_pretty(names, Place::ExistsBody, body);
            names.pop_scope();

            let kind_pretty = kind_to_pretty(KindPlace::Root, &param.kind);
            let param_pretty = Group::new(
                "{"
                    .join(block(name.join(" :").join(Sep(1)).join(kind_pretty)))
                    .join("}"),
            );

            let content_pretty = "∃ ".join(param_pretty).join(".").join(Sep(1)).join(
                body_pretty,
            );

            match place {
                Place::ExistsBody => Box::new(content_pretty),

                Place::Root | Place::FuncRet | Place::PairLeft | Place::PairRight => Box::new(
                    Group::new(
                        content_pretty,
                    ),
                ),

                _ => Box::new(Group::new("(".join(block(content_pretty)).join(")"))),
            }
        }

        TypeContent::Func { params, arg, ret } => {
            names.push_scope();

            let params_pretty = if params.len() > 0 {
                // TODO: Render kinds
                let names_pretty = delimited(
                    &";".join(Sep(1)),
                    params.iter().map(|param| {
                        // This is a mutating operation.
                        // Names are added here!
                        let name = names.add_name(param.name.clone().into());
                        let kind_pretty = kind_to_pretty(KindPlace::Root, &param.kind);
                        Group::new(name.join(" :").join(Sep(1)).join(kind_pretty))
                    }),
                ).join(Conditional::OnlyBroken(";"));

                Some(
                    Group::new("∀ {".join(block(names_pretty)).join("}")).join(Sep(1)),
                )
            } else {
                None
            };

            let arg_pretty = to_pretty(names, Place::FuncArg, arg);
            let ret_pretty = to_pretty(names, Place::FuncRet, ret);

            names.pop_scope();

            let content_pretty = params_pretty
                .join(arg_pretty)
                .join(" →")
                .join(Sep(1))
                .join(ret_pretty);

            match place {
                Place::FuncRet => Box::new(content_pretty),

                Place::Root | Place::PairLeft | Place::PairRight => {
                    Box::new(Group::new(content_pretty))
                }

                _ => Box::new(Group::new("(".join(block(content_pretty)).join(")"))),
            }
        }

        TypeContent::Pair { left, right } => {
            let left_pretty = to_pretty(names, Place::PairLeft, left);
            let right_pretty = to_pretty(names, Place::PairRight, right);

            let content_pretty = left_pretty.join(",").join(Sep(1)).join(right_pretty);

            match place {
                Place::PairRight => Box::new(content_pretty),

                Place::Root => Box::new(Group::new(
                    content_pretty.join(Conditional::OnlyBroken(",")),
                )),

                _ => {
                    Box::new(Group::new(
                        "("
                            .join(block(content_pretty.join(Conditional::OnlyBroken(","))))
                            .join(")"),
                    ))
                }
            }
        }

        TypeContent::App { constructor, param } => {
            let constructor_pretty = to_pretty(names, Place::AppLeft, constructor);
            let param_pretty = to_pretty(names, Place::Root, param);

            let content_pretty = constructor_pretty
                .join(Sep(0))
                .join("(")
                .join(block(param_pretty))
                .join(")");

            match place {
                Place::AppLeft => Box::new(content_pretty),

                Place::Root | Place::ExistsBody | Place::FuncArg | Place::FuncRet |
                Place::PairLeft | Place::PairRight => Box::new(Group::new(content_pretty)),
            }
        }
    }
}
