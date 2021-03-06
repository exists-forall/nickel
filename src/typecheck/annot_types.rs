use types::*;
use expr::*;
use super::context::{Annot, Context, Usage};
use super::equiv::{subphase, subtype};

#[derive(Clone, Debug)]
pub enum Error<Name> {
    Mismatch {
        context: Context<Name>,
        in_expr: Expr<Name>,
        expected: Type<Name>,
        actual: Type<Name>,
    },
    ExpectedFunc {
        context: Context<Name>,
        in_expr: Expr<Name>,
        actual: Type<Name>,
    },
    ExpectedPair {
        context: Context<Name>,
        in_expr: Expr<Name>,
        actual: Type<Name>,
    },
    ExpectedExists {
        context: Context<Name>,
        in_expr: Expr<Name>,
        actual: Type<Name>,
    },
    ExpectedForAll {
        context: Context<Name>,
        in_expr: Expr<Name>,
        actual: Type<Name>,
    },
    ExpectedEquivalence {
        context: Context<Name>,
        in_expr: Expr<Name>,
        actual: Type<Name>,
    },
    MovedTwice {
        context: Context<Name>,
        var: usize,
    },
    NotMoved {
        context: Context<Name>,
        var: usize,
    },
    IllegalCopy {
        context: Context<Name>,
        var: usize,
    },
    ParameterCountMismatch {
        context: Context<Name>,
        in_expr: Expr<Name>,
        expected_parameters: usize,
        actual_parameters: usize,
    },
    UnexpectedDynamic {
        context: Context<Name>,
        in_expr: Expr<Name>,
    },
}

fn is_copyable_primitive<TAnnot: Clone, Name: Clone>(ty: &AnnotType<TAnnot, Name>) -> bool {
    match ty.to_content() {
        TypeContent::Unit { .. } => true,

        TypeContent::Quantified { body, .. } => is_copyable_primitive(&body),

        TypeContent::Func { .. } => true,

        TypeContent::Pair { left, right } => {
            is_copyable_primitive(&left) && is_copyable_primitive(&right)
        }

        _ => false,
    }
}

fn check_moved_in_scope<Name: Clone>(ctx: &Context<Name>) -> Result<(), Error<Name>> {
    for var in ctx.curr_scope_vars() {
        match ctx.var_usage(var) {
            Usage::Unmoved => {
                let ty = ctx.var_type(var);
                if !is_copyable_primitive(ty) {
                    return Err(Error::NotMoved {
                        context: ctx.clone(),
                        var,
                    });
                }
            }
            Usage::Moved => {}
        }
    }
    Ok(())
}

fn intrinsic_signature<Name: Clone + Default>(intrinsic: Intrinsic) -> Type<Name> {
    // NOTE: It may be better to cache this the first time it's computed

    // NOTE: It may be worthwhile to create a tool which permits intrinsic signatures to be declared
    // in Nickel syntax, as opposed to having to directly construct their syntax trees.  This could
    // work either through a build script or by parsing the signatures dynamically at startup-time.

    match intrinsic {
        Intrinsic::ReflEquiv => Type::from_content(TypeContent::Quantified {
            quantifier: Quantifier::ForAll,
            param: TypeParam {
                name: Name::default(),
            },
            body: Type::from_content(TypeContent::Equiv {
                orig: Type::from_content(TypeContent::Var { free: 1, index: 0 }),
                dest: Type::from_content(TypeContent::Var { free: 1, index: 0 }),
            }),
        }),
    }
}

pub fn annot_types<Name: Clone + Default>(
    ctx: &mut Context<Name>,
    ex: Expr<Name>,
) -> Result<AnnotExpr<(), Annot<Name>, Name>, Error<Name>> {
    assert_eq!(
        ex.free_vars(),
        ctx.var_index_count(),
        "Cannot annotate an expression with {} free variables in a context with {} free variables",
        ex.free_vars(),
        ctx.var_index_count(),
    );
    assert_eq!(
        ex.free_types(),
        ctx.type_index_count(),
        "Cannot annotate an expression with {} free types in a context with {} free types",
        ex.free_types(),
        ctx.type_index_count(),
    );
    match ex.to_content() {
        ExprContent::Unit {
            free_vars,
            free_types,
        } => Ok(AnnotExpr::from_content_annot(
            Annot {
                phase: Phase::Static,
                ty: Type::from_content(TypeContent::Unit { free: free_types }),
            },
            ExprContent::Unit {
                free_vars,
                free_types,
            },
        )),

        ExprContent::Var {
            usage,
            free_vars,
            free_types,
            index,
        } => {
            match usage {
                VarUsage::Move => {
                    ctx.move_var(index).map_err(|()| Error::MovedTwice {
                        context: ctx.clone(),
                        var: index,
                    })?;
                }
                VarUsage::Copy => {
                    let ty = ctx.var_type(index);
                    if !is_copyable_primitive(ty) {
                        return Err(Error::IllegalCopy {
                            context: ctx.clone(),
                            var: index,
                        });
                    }
                }
            }

            Ok(AnnotExpr::from_content_annot(
                Annot {
                    phase: ctx.var_phase(index),
                    ty: ctx.var_type(index).accomodate_free(ctx.type_index_count()),
                },
                ExprContent::Var {
                    usage,
                    free_vars,
                    free_types,
                    index,
                },
            ))
        }

        ExprContent::ForAll { type_params, body } => {
            ctx.push_scope();
            for param in type_params.iter() {
                ctx.add_type(param.name.clone());
            }
            let body_annot = annot_types(ctx, body)?;
            ctx.pop_scope();

            let mut result_type = body_annot.annot().ty.clone();
            for type_param in type_params.iter().rev() {
                result_type = Type::from_content(TypeContent::Quantified {
                    quantifier: Quantifier::ForAll,
                    param: type_param.clone(),
                    body: result_type,
                })
            }

            Ok(AnnotExpr::from_content_annot(
                Annot {
                    phase: body_annot.annot().phase,
                    ty: result_type,
                },
                ExprContent::ForAll {
                    type_params,
                    body: body_annot,
                },
            ))
        }

        ExprContent::Func {
            arg_name,
            arg_type,
            arg_phase,
            body,
        } => {
            ctx.push_scope();
            ctx.add_var_unmoved(
                arg_name.clone(),
                Annot {
                    phase: arg_phase,
                    ty: arg_type.clone(),
                },
            );
            let body_annot = annot_types(ctx, body)?;
            check_moved_in_scope(ctx)?;
            ctx.pop_scope();

            let Annot {
                phase: ret_phase,
                ty: ret_ty,
            } = body_annot.annot().clone();

            Ok(AnnotExpr::from_content_annot(
                Annot {
                    // All function expressions yield constant function pointers, and are therefore
                    // statically known
                    phase: Phase::Static,
                    ty: Type::from_content(TypeContent::Func {
                        arg: arg_type.clone(),
                        arg_phase,
                        ret: ret_ty,
                        ret_phase,
                    }),
                },
                ExprContent::Func {
                    arg_name,
                    arg_type,
                    arg_phase,
                    body: body_annot,
                },
            ))
        }

        ExprContent::Inst {
            receiver,
            type_params,
        } => {
            let receiver_annot = annot_types(ctx, receiver)?;

            let mut nested_receiver_ty = receiver_annot.annot().ty.clone();
            for _ in 0..type_params.len() {
                if let TypeContent::Quantified {
                    quantifier: Quantifier::ForAll,
                    param: _,
                    body,
                } = nested_receiver_ty.to_content()
                {
                    nested_receiver_ty = body;
                } else {
                    return Err(Error::ExpectedForAll {
                        context: ctx.clone(),
                        in_expr: ex,
                        actual: nested_receiver_ty,
                    });
                }
            }

            let receiver_ty_instantiated = nested_receiver_ty.subst(&type_params);

            Ok(AnnotExpr::from_content_annot(
                Annot {
                    phase: receiver_annot.annot().phase,
                    ty: receiver_ty_instantiated,
                },
                ExprContent::Inst {
                    receiver: receiver_annot,
                    type_params,
                },
            ))
        }

        ExprContent::App { callee, arg } => {
            let callee_annot = annot_types(ctx, callee)?;
            let arg_annot = annot_types(ctx, arg)?;

            if let TypeContent::Func {
                arg,
                arg_phase,
                ret,
                ret_phase,
            } = callee_annot.annot().ty.to_content()
            {
                if !subphase(arg_annot.annot().phase, arg_phase) {
                    return Err(Error::UnexpectedDynamic {
                        context: ctx.clone(),
                        in_expr: ex,
                    });
                }

                if !subtype(arg_annot.annot().ty.clone(), arg.clone()) {
                    return Err(Error::Mismatch {
                        context: ctx.clone(),
                        in_expr: ex,
                        expected: arg,
                        actual: arg_annot.annot().ty.clone(),
                    });
                }

                let result_phase = match (
                    callee_annot.annot().phase,
                    ret_phase,
                    arg_annot.annot().phase,
                ) {
                    (Phase::Static, Phase::Static, Phase::Static) => Phase::Static,
                    _ => Phase::Dynamic,
                };

                Ok(AnnotExpr::from_content_annot(
                    Annot {
                        phase: result_phase,
                        ty: ret,
                    },
                    ExprContent::App {
                        callee: callee_annot,
                        arg: arg_annot,
                    },
                ))
            } else {
                return Err(Error::ExpectedFunc {
                    context: ctx.clone(),
                    in_expr: ex,
                    actual: callee_annot.annot().ty.clone(),
                });
            }
        }

        ExprContent::Pair { left, right } => {
            let left_annot = annot_types(ctx, left)?;
            let right_annot = annot_types(ctx, right)?;

            let result_phase = match (left_annot.annot().phase, right_annot.annot().phase) {
                (Phase::Static, Phase::Static) => Phase::Static,
                _ => Phase::Dynamic,
            };

            Ok(AnnotExpr::from_content_annot(
                Annot {
                    phase: result_phase,
                    ty: Type::from_content(TypeContent::Pair {
                        left: left_annot.annot().ty.clone(),
                        right: right_annot.annot().ty.clone(),
                    }),
                },
                ExprContent::Pair {
                    left: left_annot,
                    right: right_annot,
                },
            ))
        }

        ExprContent::Let { names, val, body } => {
            let val_annot = annot_types(ctx, val)?;

            ctx.push_scope();

            debug_assert!(names.len() > 0);
            let phase = val_annot.annot().phase;
            let mut nested_pairs = val_annot.annot().ty.clone();
            for name in &names[0..names.len() - 1] {
                if let TypeContent::Pair { left, right } = nested_pairs.to_content() {
                    ctx.add_var_unmoved(name.clone(), Annot { phase, ty: left });
                    nested_pairs = right;
                } else {
                    let mut outer_ctx = ctx.clone();
                    outer_ctx.pop_scope();
                    return Err(Error::ExpectedPair {
                        context: outer_ctx,
                        in_expr: ex,
                        actual: nested_pairs,
                    });
                }
            }
            let deepest_right = nested_pairs;
            ctx.add_var_unmoved(
                names.last().unwrap().clone(),
                Annot {
                    phase,
                    ty: deepest_right,
                },
            );

            let body_annot = annot_types(ctx, body)?;

            check_moved_in_scope(ctx)?;
            ctx.pop_scope();

            Ok(AnnotExpr::from_content_annot(
                body_annot.annot().clone(),
                ExprContent::Let {
                    names,
                    val: val_annot,
                    body: body_annot,
                },
            ))
        }

        ExprContent::LetExists {
            type_names,
            val_name,
            val,
            body,
        } => {
            let val_annot = annot_types(ctx, val)?;

            ctx.push_scope();

            let mut nested = val_annot.annot().ty.clone();
            for type_name in type_names.iter() {
                if let TypeContent::Quantified {
                    quantifier: Quantifier::Exists,
                    param: _,
                    body,
                } = nested.to_content()
                {
                    ctx.add_type(type_name.clone());
                    nested = body;
                } else {
                    let mut outer_ctx = ctx.clone();
                    outer_ctx.pop_scope();
                    return Err(Error::ExpectedExists {
                        context: outer_ctx,
                        in_expr: ex,
                        actual: nested,
                    });
                }
            }
            let deepest_body = nested;
            ctx.add_var_unmoved(
                val_name.clone(),
                Annot {
                    phase: val_annot.annot().phase,
                    ty: deepest_body,
                },
            );

            let body_annot = annot_types(ctx, body)?;

            check_moved_in_scope(ctx)?;
            ctx.pop_scope();

            Ok(AnnotExpr::from_content_annot(
                body_annot.annot().clone(),
                ExprContent::LetExists {
                    type_names,
                    val_name,
                    val: val_annot,
                    body: body_annot,
                },
            ))
        }

        ExprContent::MakeExists {
            params,
            type_body,
            body,
        } => {
            let body_annot = annot_types(ctx, body)?;

            let substitutions = params
                .iter()
                .map(|&(_, ref ty)| ty.clone())
                .collect::<Vec<_>>();
            let instantiated_type_body = type_body.subst(&substitutions);

            if !subtype(
                body_annot.annot().ty.clone(),
                instantiated_type_body.clone(),
            ) {
                return Err(Error::Mismatch {
                    context: ctx.clone(),
                    in_expr: ex,
                    actual: body_annot.annot().ty.clone(),
                    expected: instantiated_type_body.clone(),
                });
            }

            let mut result_type = type_body.clone();
            for &(ref name, _) in params.iter().rev() {
                result_type = Type::from_content(TypeContent::Quantified {
                    quantifier: Quantifier::Exists,
                    param: TypeParam { name: name.clone() },
                    body: result_type,
                });
            }

            Ok(AnnotExpr::from_content_annot(
                Annot {
                    phase: body_annot.annot().phase,
                    ty: result_type,
                },
                ExprContent::MakeExists {
                    params,
                    type_body,
                    body: body_annot,
                },
            ))
        }

        ExprContent::Cast {
            param,
            type_body,
            equivalence,
            body,
        } => {
            let equivalence_annot = annot_types(ctx, equivalence)?;
            let body_annot = annot_types(ctx, body)?;

            if let TypeContent::Equiv { orig, dest } = equivalence_annot.annot().ty.to_content() {
                let type_body_orig = type_body.subst(&[orig]);
                let type_body_dest = type_body.subst(&[dest]);

                if !subtype(body_annot.annot().ty.clone(), type_body_orig.clone()) {
                    return Err(Error::Mismatch {
                        context: ctx.clone(),
                        in_expr: ex,
                        expected: type_body_orig,
                        actual: body_annot.annot().ty.clone(),
                    });
                }

                Ok(AnnotExpr::from_content_annot(
                    Annot {
                        // NOTE: this will produce a static expression even if the equivalence token
                        // is not static.  This seems valid, but there is currently nothing else
                        // that works this way.  Should we either artificially restrict cast
                        // expressions to only produce static expressions when their cast tokens or
                        // static, or should we create a more general system for "erasing" the
                        // dynamicness of computationally-irrelevant tokens?
                        phase: body_annot.annot().phase,
                        ty: type_body_dest,
                    },
                    ExprContent::Cast {
                        param,
                        type_body,
                        equivalence: equivalence_annot,
                        body: body_annot,
                    },
                ))
            } else {
                return Err(Error::ExpectedEquivalence {
                    context: ctx.clone(),
                    in_expr: ex,
                    actual: equivalence_annot.annot().ty.clone(),
                });
            }
        }

        ExprContent::Intrinsic {
            free_vars,
            free_types,
            intrinsic,
        } => Ok(AnnotExpr::from_content_annot(
            Annot {
                phase: Phase::Static,
                ty: intrinsic_signature(intrinsic).accomodate_free(free_types),
            },
            ExprContent::Intrinsic {
                free_vars,
                free_types,
                intrinsic,
            },
        )),
    }
}
