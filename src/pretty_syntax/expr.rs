use std::rc::Rc;
use pretty_trait::{Pretty, JoinExt, Group, Sep, Conditional, delimited, block, Indent, Seq};

use super::super::expr::*;
use pretty_syntax::types;
use pretty_syntax::names::Names;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Root,
    AbsBody,
    InstReceiver,
    AppCallee,
    PairLeft,
    PairRight,
    LetBody,
    MakeExistsBody,
    ForAllBody,
}

pub fn to_pretty<Name: Clone + Into<Rc<String>>>(
    var_names: &mut Names,
    type_names: &mut Names,
    place: Place,
    ex: Expr<Name>,
) -> Box<Pretty> {
    assert_eq!(var_names.index_count(), ex.free_vars());
    assert_eq!(type_names.index_count(), ex.free_types());

    match ex.to_content() {
        ExprContent::Unit {
            free_vars: _,
            free_types: _,
        } => Box::new("()"),

        ExprContent::Var {
            usage,
            free_vars: _,
            free_types: _,
            index,
        } => {
            match usage {
                VarUsage::Move => {
                    Box::new(Group::new(
                        "move".join(Sep(1)).join(var_names.get_name(index)),
                    ))
                }
                VarUsage::Copy => Box::new(var_names.get_name(index)),
            }
        }

        ExprContent::ForAll { type_params, body } => {
            type_names.push_scope();

            let type_params_pretty = Seq(
                type_params
                    .iter()
                    .map(|param| {
                        // This is a mutating operation.
                        // Names are added here!
                        let name = type_names.add_name(param.name.clone().into());
                        let kind_pretty =
                            types::kind_to_pretty(types::KindPlace::Root, &param.kind);
                        Sep(1).join(Group::new(
                            "{"
                                .join(block(name.join(" :").join(Sep(1)).join(kind_pretty)))
                                .join("}"),
                        ))
                    })
                    .collect(),
            );

            let body_pretty = to_pretty(var_names, type_names, Place::ForAllBody, body);

            type_names.pop_scope();

            let content_pretty = Group::new("forall".join(type_params_pretty))
                .join(Sep(1))
                .join(body_pretty);

            match place {
                Place::Root | Place::AbsBody | Place::PairLeft | Place::PairRight |
                Place::LetBody | Place::MakeExistsBody => Box::new(Group::new(content_pretty)),

                Place::ForAllBody => Box::new(content_pretty),

                _ => Box::new(Group::new("(".join(block(content_pretty)).join(")"))),
            }
        }

        ExprContent::Func {
            arg_name,
            arg_type,
            body,
        } => {
            var_names.push_scope();

            let arg_name_pretty = var_names.add_name(arg_name.clone().into());
            let arg_type_pretty = types::to_pretty(type_names, types::Place::Root, arg_type);

            let arg_pretty = Group::new(
                "("
                    .join(block(arg_name_pretty.join(" :").join(Sep(1)).join(
                        arg_type_pretty,
                    ))).join(")"),
            );

            let body_pretty = to_pretty(var_names, type_names, Place::AbsBody, body);

            var_names.pop_scope();

            let content_pretty = "func".join(Sep(1)).join(arg_pretty).join(" ->").join(
                Indent(
                    Sep(1).join(body_pretty),
                ),
            );

            match place {
                Place::Root | Place::AbsBody | Place::PairLeft | Place::PairRight |
                Place::LetBody | Place::MakeExistsBody | Place::ForAllBody => Box::new(Group::new(
                    content_pretty,
                )),

                _ => Box::new(Group::new("(".join(block(content_pretty)).join(")"))),
            }
        }

        ExprContent::Inst {
            receiver,
            type_params,
        } => {
            let receiver_pretty = to_pretty(var_names, type_names, Place::InstReceiver, receiver);

            let params_pretty = Seq(
                type_params
                    .iter()
                    .map(|param| {
                        Sep(0).join(Group::new(
                            "{"
                                .join(block(types::to_pretty(
                                    type_names,
                                    types::Place::Root,
                                    param.clone(),
                                ))).join("}"),
                        ))
                    })
                    .collect(),
            );

            let content_pretty = receiver_pretty.join(params_pretty);

            match place {
                Place::InstReceiver => Box::new(content_pretty),
                _ => Box::new(Group::new(content_pretty)),
            }
        }

        ExprContent::App { callee, arg } => {
            let callee_pretty = to_pretty(var_names, type_names, Place::AppCallee, callee);

            let arg_pretty = to_pretty(var_names, type_names, Place::Root, arg);

            Box::new(Group::new(
                callee_pretty.join("(").join(block(arg_pretty)).join(")"),
            ))
        }

        ExprContent::Pair { left, right } => {
            let left_pretty = to_pretty(var_names, type_names, Place::PairLeft, left);
            let right_pretty = to_pretty(var_names, type_names, Place::PairRight, right);

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

        ExprContent::Let { names, val, body } => {
            let val_pretty = to_pretty(var_names, type_names, Place::Root, val);

            var_names.push_scope();

            let names_pretty = delimited(
                &",".join(Sep(1)),
                names.iter().map(|name| {
                    // This is a mutating operation.
                    // Names are added here!
                    var_names.add_name(name.clone().into())
                }),
            ).join(Conditional::OnlyBroken(","));

            let body_pretty = to_pretty(var_names, type_names, Place::LetBody, body);

            var_names.pop_scope();

            let binding_pretty = Group::new(
                Group::new(names_pretty.join(Sep(1).join("=")))
                    .join(Sep(1))
                    .join(val_pretty),
            );

            let content_pretty = Group::new(
                "let"
                    .join(Conditional::OnlyUnbroken(" "))
                    .join(block(binding_pretty))
                    .join(Conditional::OnlyUnbroken(" "))
                    .join("in"),
            ).join(Sep(1))
                .join(body_pretty);

            match place {
                Place::LetBody => Box::new(content_pretty),

                Place::Root | Place::AbsBody | Place::PairLeft | Place::PairRight |
                Place::MakeExistsBody | Place::ForAllBody => Box::new(Group::new(content_pretty)),

                _ => Box::new("(".join(block(content_pretty)).join(")")),
            }
        }

        ExprContent::LetExists {
            type_names: exists_type_names,
            val_name,
            val,
            body,
        } => {
            let val_pretty = to_pretty(var_names, type_names, Place::Root, val);

            var_names.push_scope();
            type_names.push_scope();

            let type_names_pretty = delimited(
                &";".join(Sep(1)),
                exists_type_names.iter().map(|name| {
                    // This is a mutating operation.
                    // Names are added here!
                    type_names.add_name(name.clone().into())
                }),
            ).join(Conditional::OnlyBroken(";"));

            let val_name_pretty = var_names.add_name(val_name.clone().into());

            let body_pretty = to_pretty(var_names, type_names, Place::LetBody, body);

            var_names.pop_scope();
            type_names.pop_scope();

            let binding_pretty = Group::new(
                Group::new("{".join(block(type_names_pretty)).join("}"))
                    .join(Sep(1))
                    .join(Group::new(val_name_pretty.join(Sep(1)).join("=")))
                    .join(Sep(1))
                    .join(val_pretty),
            );

            let content_pretty = Group::new(
                "let_exists"
                    .join(Conditional::OnlyUnbroken(" "))
                    .join(block(binding_pretty))
                    .join(Conditional::OnlyUnbroken(" "))
                    .join("in"),
            ).join(Sep(1))
                .join(body_pretty);

            match place {
                Place::LetBody => Box::new(content_pretty),

                Place::Root | Place::AbsBody | Place::PairLeft | Place::PairRight |
                Place::MakeExistsBody | Place::ForAllBody => Box::new(Group::new(content_pretty)),

                _ => Box::new("(".join(block(content_pretty)).join(")")),
            }
        }

        ExprContent::MakeExists {
            params,
            type_body,
            body,
        } => {
            let param_types_pretty = params
                .iter()
                .map(|&(_, ref ty)| {
                    types::to_pretty(type_names, types::Place::Root, ty.clone())
                })
                .collect::<Vec<_>>();

            type_names.push_scope();

            let params_pretty = Group::new(
                "{"
                    .join(block(
                        delimited(
                            &";".join(Sep(1)),
                            params.iter().zip(param_types_pretty).map(
                                |(param, ty_pretty)| {
                                    // This is a mutating operation
                                    // Names are added here!
                                    let name_pretty = type_names.add_name(param.0.clone().into());
                                    Group::new(name_pretty.join(" =").join(Sep(1)).join(ty_pretty))
                                },
                            ),
                        ).join(Conditional::OnlyBroken(";")),
                    )).join("}"),
            );

            let type_body_pretty = types::to_pretty(type_names, types::Place::Root, type_body);

            type_names.pop_scope();

            let body_pretty = to_pretty(var_names, type_names, Place::MakeExistsBody, body);

            let content_pretty = Group::new(
                "make_exists"
                    .join(Conditional::OnlyUnbroken(" "))
                    .join(block(Group::new(
                        params_pretty.join(Sep(1)).join(type_body_pretty),
                    )))
                    .join(Conditional::OnlyUnbroken(" "))
                    .join("of"),
            ).join(Sep(1))
                .join(body_pretty);

            match place {
                Place::MakeExistsBody => Box::new(content_pretty),

                Place::Root | Place::AbsBody | Place::PairLeft | Place::PairRight |
                Place::LetBody | Place::ForAllBody => Box::new(Group::new(content_pretty)),

                _ => Box::new(Group::new("(".join(block(content_pretty)).join(")"))),
            }
        }
    }
}
