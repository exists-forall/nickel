use std::rc::Rc;

use pretty_trait::{block, Conditional, Group, Indent, JoinExt, Pretty, Sep};

use super::super::types::*;
use pretty_syntax::names::Names;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Root,
    QuantifierBody,
    FuncArg,
    FuncRet,
    PairLeft,
    PairRight,
    AppConstructor,
    AppParam,
}

pub fn to_pretty<Name: Clone + Into<Rc<String>>>(
    names: &mut Names,
    place: Place,
    ty: Type<Name>,
) -> Box<Pretty> {
    assert_eq!(names.index_count(), ty.free());

    match ty.to_content() {
        TypeContent::Unit { free: _ } => Box::new("()"),

        TypeContent::Var { free: _, index } => Box::new(names.get_name(index)),

        TypeContent::Quantified {
            quantifier,
            param,
            body,
        } => {
            names.push_scope();
            let name = names.add_name(param.name.into());
            let body_pretty = to_pretty(names, Place::QuantifierBody, body);
            names.pop_scope();

            let param_pretty = Group::new("{".join(block(name)).join("}"));

            let quantifier_name = match quantifier {
                Quantifier::Exists => "exists",
                Quantifier::ForAll => "forall",
            };

            let content_pretty = quantifier_name
                .join(" ")
                .join(param_pretty)
                .join(Sep(1))
                .join(body_pretty);

            match place {
                Place::QuantifierBody => Box::new(content_pretty),

                Place::Root | Place::FuncRet | Place::PairLeft | Place::PairRight => {
                    Box::new(Group::new(content_pretty))
                }

                _ => Box::new(Group::new("(".join(block(content_pretty)).join(")"))),
            }
        }

        TypeContent::Func {
            arg,
            arg_phase,
            ret,
            ret_phase,
        } => {
            let arg_pretty: Box<Pretty> = match arg_phase {
                Phase::Dynamic => Box::new(to_pretty(names, Place::FuncArg, arg)),
                Phase::Static => {
                    let arg_ty_pretty = to_pretty(names, Place::Root, arg);
                    Box::new(Group::new(
                        "(".join(block(Group::new("static".join(Sep(1)).join(arg_ty_pretty))))
                            .join(")"),
                    ))
                }
            };

            let ret_ty_pretty = to_pretty(names, Place::FuncRet, ret);

            let ret_pretty = match ret_phase {
                Phase::Dynamic => Group::new(None.join(ret_ty_pretty)),
                Phase::Static => Group::new(Some("static".join(Sep(1))).join(ret_ty_pretty)),
            };

            let content_pretty = arg_pretty.join(" ->").join(Sep(1)).join(ret_pretty);

            match place {
                Place::FuncRet => Box::new(content_pretty),

                Place::Root | Place::PairLeft | Place::PairRight | Place::QuantifierBody => {
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

                _ => Box::new(Group::new(
                    "(".join(block(content_pretty.join(Conditional::OnlyBroken(","))))
                        .join(")"),
                )),
            }
        }

        TypeContent::App { constructor, param } => {
            let constructor_pretty = to_pretty(names, Place::AppConstructor, constructor);
            let param_pretty = to_pretty(names, Place::AppParam, param);

            let content_pretty = constructor_pretty.join(Indent(Sep(1).join(param_pretty)));

            match place {
                Place::AppConstructor => Box::new(content_pretty),

                Place::Root
                | Place::QuantifierBody
                | Place::FuncArg
                | Place::FuncRet
                | Place::PairLeft
                | Place::PairRight => Box::new(Group::new(content_pretty)),

                _ => Box::new("(".join(block(content_pretty)).join(")")),
            }
        }

        TypeContent::Equiv { orig, dest } => {
            let orig_pretty = to_pretty(names, Place::AppParam, orig);
            let dest_pretty = to_pretty(names, Place::AppParam, dest);

            let content_pretty = "equiv".join(Indent(
                Sep(1).join(orig_pretty).join(Sep(1)).join(dest_pretty),
            ));

            match place {
                Place::Root
                | Place::QuantifierBody
                | Place::FuncArg
                | Place::FuncRet
                | Place::PairLeft
                | Place::PairRight => Box::new(Group::new(content_pretty)),

                _ => Box::new("(".join(block(content_pretty)).join(")")),
            }
        }
    }
}
