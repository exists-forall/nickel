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

            // TODO: Render kind
            let content_pretty = "∃ ".join(name).join(".").join(Sep(1)).join(body_pretty);

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

        TypeContent::Func {
            access: _,
            params,
            arg,
            ret,
        } => {
            // TODO: Render access

            names.push_scope();

            let params_pretty = if params.len() > 0 {
                // TODO: Render kinds
                let names_pretty = delimited(
                    &";".join(Sep(1)),
                    params.iter().map(|param|
                         // This is a mutating operation.
                         // Names are added here!
                        names.add_name(param.name.clone().into())),
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
