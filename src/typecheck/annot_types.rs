use std::rc::Rc;

use types::*;
use expr::*;
use super::context::{Context, Usage};
use super::annot_kinds;
use super::equiv::{equiv_kind, equiv};

#[derive(Clone, Debug)]
pub enum Error<Name> {
    Kind(annot_kinds::Error<Name>),
    KindMismatch {
        context: Context<Name>,
        actual: AnnotType<Kind, Name>,
        expected: Kind,
    },
    Mismatch {
        context: Context<Name>,
        in_expr: Expr<Name>,
        expected: AnnotType<Kind, Name>,
        actual: AnnotType<Kind, Name>,
    },
    ExpectedFunc {
        context: Context<Name>,
        in_expr: Expr<Name>,
        actual: AnnotType<Kind, Name>,
    },
    ExpectedPair {
        context: Context<Name>,
        in_expr: Expr<Name>,
        actual: AnnotType<Kind, Name>,
    },
    ExpectedExists {
        context: Context<Name>,
        in_expr: Expr<Name>,
        actual: AnnotType<Kind, Name>,
    },
    MovedTwice { context: Context<Name>, var: usize },
    NotMoved { context: Context<Name>, var: usize },
    IllegalCopy { context: Context<Name>, var: usize },
    ParameterCountMismatch {
        context: Context<Name>,
        in_expr: Expr<Name>,
        expected_parameters: usize,
        actual_parameters: usize,
    },
}

// Like raw annot_kinds, but wraps errors appropriately
fn annot_kinds<Name: Clone>(
    ctx: &mut Context<Name>,
    ty: Type<Name>,
) -> Result<AnnotType<Kind, Name>, Error<Name>> {
    annot_kinds::annot_kinds(ctx, ty).map_err(Error::Kind)
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

fn check_kind<Name: Clone>(
    ctx: &Context<Name>,
    ty: &AnnotType<Kind, Name>,
    expected: &Kind,
) -> Result<(), Error<Name>> {
    if equiv_kind(&ty.annot(), expected) {
        Ok(())
    } else {
        Err(Error::KindMismatch {
            context: ctx.clone(),
            expected: expected.clone(),
            actual: ty.clone(),
        })
    }
}

pub fn annot_types<Name: Clone>(
    ctx: &mut Context<Name>,
    ex: Expr<Name>,
) -> Result<AnnotExpr<Kind, AnnotType<Kind, Name>, Name>, Error<Name>> {
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
        } => {
            Ok(AnnotExpr::from_content_annot(
                AnnotType::from_content_annot(
                    Kind::Type,
                    TypeContent::Unit { free: free_types },
                ),
                ExprContent::Unit {
                    free_vars,
                    free_types,
                },
            ))
        }

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
                ctx.var_type(index).accomodate_free(ctx.type_index_count()),
                ExprContent::Var {
                    usage,
                    free_vars,
                    free_types,
                    index,
                },
            ))
        }

        ExprContent::ForAll { type_param, body } => {
            ctx.push_scope();

            ctx.add_type_kind(type_param.name.clone(), type_param.kind.clone());
            let body_annot = annot_types(ctx, body)?;

            ctx.pop_scope();

            Ok(AnnotExpr::from_content_annot(
                AnnotType::from_content_annot(
                    Kind::Type,
                    TypeContent::Quantified {
                        quantifier: Quantifier::ForAll,
                        param: type_param.clone(),
                        body: body_annot.annot(),
                    },
                ),
                ExprContent::ForAll {
                    type_param,
                    body: body_annot,
                },
            ))
        }

        ExprContent::Func {
            type_params,
            arg_name,
            arg_type,
            body,
        } => {
            ctx.push_scope();

            for param in type_params.iter() {
                ctx.add_type_kind(param.name.clone(), param.kind.clone());
            }

            let arg_type_annot = annot_kinds(ctx, arg_type)?;
            check_kind(ctx, &arg_type_annot, &Kind::Type)?;
            ctx.add_var_unmoved(arg_name.clone(), arg_type_annot.clone());

            let body_annot = annot_types(ctx, body)?;

            check_moved_in_scope(ctx)?;
            ctx.pop_scope();

            Ok(AnnotExpr::from_content_annot(
                AnnotType::from_content_annot(
                    Kind::Type,
                    TypeContent::Func {
                        params: type_params.clone(),
                        arg: arg_type_annot.clone(),
                        ret: body_annot.annot(),
                    },
                ),
                ExprContent::Func {
                    type_params,
                    arg_name,
                    arg_type: arg_type_annot,
                    body: body_annot,
                },
            ))
        }

        ExprContent::App {
            callee,
            type_params,
            arg,
        } => {
            let callee_annot = annot_types(ctx, callee)?;
            let arg_annot = annot_types(ctx, arg)?;

            if let TypeContent::Func { params, arg, ret } = callee_annot.annot().to_content() {
                if params.len() != type_params.len() {
                    return Err(Error::ParameterCountMismatch {
                        context: ctx.clone(),
                        in_expr: ex,
                        expected_parameters: params.len(),
                        actual_parameters: type_params.len(),
                    });
                }
                let mut type_params_annot = Vec::with_capacity(type_params.len());
                for type_param in type_params.iter() {
                    let type_param_annot = annot_kinds(ctx, type_param.clone())?;
                    type_params_annot.push(type_param_annot);
                }
                for (expected, actual) in params.iter().zip(type_params_annot.iter()) {
                    check_kind(ctx, actual, &expected.kind)?;
                }
                let arg_type_instantiated = arg.subst(&type_params_annot);
                let ret_type_instantiated = ret.subst(&type_params_annot);
                if !equiv(arg_type_instantiated.clone(), arg_annot.annot()) {
                    return Err(Error::Mismatch {
                        context: ctx.clone(),
                        in_expr: ex,
                        expected: arg_type_instantiated,
                        actual: arg_annot.annot(),
                    });
                }
                Ok(AnnotExpr::from_content_annot(
                    ret_type_instantiated,
                    ExprContent::App {
                        callee: callee_annot,
                        type_params: Rc::new(type_params_annot),
                        arg: arg_annot,
                    },
                ))
            } else {
                return Err(Error::ExpectedFunc {
                    context: ctx.clone(),
                    in_expr: ex,
                    actual: callee_annot.annot(),
                });
            }
        }

        ExprContent::Pair { left, right } => {
            let left_annot = annot_types(ctx, left)?;
            let right_annot = annot_types(ctx, right)?;
            Ok(AnnotExpr::from_content_annot(
                AnnotType::from_content_annot(
                    Kind::Type,
                    TypeContent::Pair {
                        left: left_annot.annot(),
                        right: right_annot.annot(),
                    },
                ),
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
            let mut nested_pairs = val_annot.annot();
            for name in &names[0..names.len() - 1] {
                if let TypeContent::Pair { left, right } = nested_pairs.to_content() {
                    ctx.add_var_unmoved(name.clone(), left);
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
            ctx.add_var_unmoved(names.last().unwrap().clone(), deepest_right);

            let body_annot = annot_types(ctx, body)?;

            check_moved_in_scope(ctx)?;
            ctx.pop_scope();

            Ok(AnnotExpr::from_content_annot(
                body_annot.annot(),
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

            let mut nested = val_annot.annot();
            for type_name in type_names.iter() {
                if let TypeContent::Quantified {
                    quantifier: Quantifier::Exists,
                    param,
                    body,
                } = nested.to_content()
                {
                    ctx.add_type_kind(type_name.clone(), param.kind);
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
            ctx.add_var_unmoved(val_name.clone(), deepest_body);

            let body_annot = annot_types(ctx, body)?;

            check_moved_in_scope(ctx)?;
            ctx.pop_scope();

            Ok(AnnotExpr::from_content_annot(
                body_annot.annot(),
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
            let mut params_annot = Vec::with_capacity(params.len());
            for (name, ty) in params.iter().cloned() {
                let ty_annot = annot_kinds(ctx, ty.clone())?;
                params_annot.push((name, ty_annot));
            }

            ctx.push_scope();
            for &(ref name, ref ty_annot) in params_annot.iter() {
                ctx.add_type_kind(name.clone(), ty_annot.annot());
            }
            let type_body_annot = annot_kinds(ctx, type_body)?;
            ctx.pop_scope();

            let body_annot = annot_types(ctx, body)?;

            let substitutions = params_annot
                .iter()
                .map(|&(_, ref ty)| ty.clone())
                .collect::<Vec<_>>();
            let instantiated_type_body = type_body_annot.subst(&substitutions);

            if !equiv(instantiated_type_body.clone(), body_annot.annot()) {
                return Err(Error::Mismatch {
                    context: ctx.clone(),
                    in_expr: ex,
                    actual: body_annot.annot(),
                    expected: instantiated_type_body.clone(),
                });
            }

            let mut result_type = type_body_annot.clone();
            for (name, ty_annot) in params_annot.iter().rev().cloned() {
                result_type = AnnotType::from_content_annot(
                    Kind::Type,
                    TypeContent::Quantified {
                        quantifier: Quantifier::Exists,
                        param: TypeParam {
                            name,
                            kind: ty_annot.annot(),
                        },
                        body: result_type,
                    },
                );
            }

            Ok(AnnotExpr::from_content_annot(
                result_type,
                ExprContent::MakeExists {
                    params: Rc::new(params_annot),
                    type_body: type_body_annot,
                    body: body_annot,
                },
            ))
        }
    }
}
