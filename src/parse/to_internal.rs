use std::rc::Rc;

use super::syntax;
use types;
use expr;
use super::names::{Names, Error};

#[derive(Clone, Debug)]
pub struct Context {
    pub var_names: Names,
    pub type_names: Names,
}

fn add_type_params(type_names: &mut Names, params: &[syntax::TypeParam]) -> Result<(), Error> {
    for &syntax::TypeParam { ref ident, kind: _ } in params {
        type_names.add_name(ident.clone())?;
    }
    Ok(())
}

fn convert_type_params(params: Vec<syntax::TypeParam>) -> Rc<Vec<types::TypeParam<Rc<String>>>> {
    Rc::new(
        params
            .into_iter()
            .map(|param| {
                types::TypeParam {
                    name: Rc::new(param.ident.name),
                    kind: param.kind,
                }
            })
            .collect(),
    )
}

pub fn convert_type(
    type_names: &mut Names,
    ty: syntax::Type,
) -> Result<types::Type<Rc<String>>, Error> {
    match ty {
        syntax::Type::Unit => {
            Ok(types::Type::from_content(
                types::TypeContent::Unit { free: type_names.index_count() },
            ))
        }

        syntax::Type::Var { ident } => {
            Ok(types::Type::from_content(types::TypeContent::Var {
                free: type_names.index_count(),
                index: type_names.get_index(&ident)?,
            }))
        }

        syntax::Type::Quantified {
            quantifier,
            param,
            body,
        } => {
            type_names.push_scope();

            type_names.add_name(param.ident.clone())?;

            let result = types::Type::from_content(types::TypeContent::Quantified {
                quantifier,
                param: types::TypeParam {
                    name: Rc::new(param.ident.name),
                    kind: param.kind,
                },
                body: convert_type(type_names, *body)?,
            });

            type_names.pop_scope();

            Ok(result)
        }

        syntax::Type::Func { arg, ret } => {
            let result = types::Type::from_content(types::TypeContent::Func {
                arg: convert_type(type_names, *arg)?,
                ret: convert_type(type_names, *ret)?,
            });

            Ok(result)
        }

        syntax::Type::Pair { left, right } => {
            Ok(types::Type::from_content(types::TypeContent::Pair {
                left: convert_type(type_names, *left)?,
                right: convert_type(type_names, *right)?,
            }))
        }

        syntax::Type::App { constructor, param } => {
            Ok(types::Type::from_content(types::TypeContent::App {
                constructor: convert_type(type_names, *constructor)?,
                param: convert_type(type_names, *param)?,
            }))
        }
    }
}

pub fn convert_expr(ctx: &mut Context, ex: syntax::Expr) -> Result<expr::Expr<Rc<String>>, Error> {
    match ex {
        syntax::Expr::Unit => {
            Ok(expr::Expr::from_content(expr::ExprContent::Unit {
                free_vars: ctx.var_names.index_count(),
                free_types: ctx.type_names.index_count(),
            }))
        }

        syntax::Expr::Var { usage, ident } => {
            Ok(expr::Expr::from_content(expr::ExprContent::Var {
                usage,
                free_vars: ctx.var_names.index_count(),
                free_types: ctx.type_names.index_count(),
                index: ctx.var_names.get_index(&ident)?,
            }))
        }

        syntax::Expr::ForAll { type_params, body } => {
            ctx.type_names.push_scope();

            add_type_params(&mut ctx.type_names, &type_params)?;

            let result = expr::Expr::from_content(expr::ExprContent::ForAll {
                type_params: convert_type_params(type_params),
                body: convert_expr(ctx, *body)?,
            });

            ctx.type_names.pop_scope();

            Ok(result)
        }

        syntax::Expr::Func {
            arg_name,
            arg_type,
            body,
        } => {
            ctx.var_names.push_scope();

            ctx.var_names.add_name(arg_name.clone())?;

            let result = expr::Expr::from_content(expr::ExprContent::Func {
                arg_name: Rc::new(arg_name.name),
                arg_type: convert_type(&mut ctx.type_names, arg_type)?,
                body: convert_expr(ctx, *body)?,
            });

            ctx.var_names.pop_scope();

            Ok(result)
        }

        syntax::Expr::Inst {
            receiver,
            type_params,
        } => {
            let mut converted_params = Vec::with_capacity(type_params.len());
            for ty in type_params {
                converted_params.push(convert_type(&mut ctx.type_names, ty)?);
            }

            Ok(expr::Expr::from_content(expr::ExprContent::Inst {
                receiver: convert_expr(ctx, *receiver)?,
                type_params: Rc::new(converted_params),
            }))
        }

        syntax::Expr::App { callee, arg } => {
            Ok(expr::Expr::from_content(expr::ExprContent::App {
                callee: convert_expr(ctx, *callee)?,
                arg: convert_expr(ctx, *arg)?,
            }))
        }

        syntax::Expr::Pair { left, right } => {
            Ok(expr::Expr::from_content(expr::ExprContent::Pair {
                left: convert_expr(ctx, *left)?,
                right: convert_expr(ctx, *right)?,
            }))
        }

        syntax::Expr::Let { names, val, body } => {
            let converted_val = convert_expr(ctx, *val)?;

            ctx.var_names.push_scope();

            for name in &names {
                ctx.var_names.add_name(name.clone())?;
            }

            let result = expr::Expr::from_content(expr::ExprContent::Let {
                names: Rc::new(names.into_iter().map(|name| Rc::new(name.name)).collect()),
                val: converted_val,
                body: convert_expr(ctx, *body)?,
            });

            ctx.var_names.pop_scope();

            Ok(result)
        }

        syntax::Expr::LetExists {
            type_names,
            val_name,
            val,
            body,
        } => {
            let converted_val = convert_expr(ctx, *val)?;

            ctx.var_names.push_scope();
            ctx.type_names.push_scope();

            for type_name in &type_names {
                ctx.type_names.add_name(type_name.clone())?;
            }

            ctx.var_names.add_name(val_name.clone())?;

            let result = expr::Expr::from_content(expr::ExprContent::LetExists {
                type_names: Rc::new(
                    type_names
                        .into_iter()
                        .map(|name| Rc::new(name.name))
                        .collect(),
                ),
                val_name: Rc::new(val_name.name),
                val: converted_val,
                body: convert_expr(ctx, *body)?,
            });

            ctx.var_names.pop_scope();
            ctx.type_names.pop_scope();

            Ok(result)
        }

        syntax::Expr::MakeExists {
            params,
            type_body,
            body,
        } => {
            let mut param_idents = Vec::with_capacity(params.len());
            let mut converted_params = Vec::with_capacity(params.len());
            for (ident, ty) in params {
                param_idents.push(ident.clone());
                converted_params.push((
                    Rc::new(ident.name),
                    convert_type(&mut ctx.type_names, ty)?,
                ));
            }

            ctx.type_names.push_scope();
            for ident in param_idents {
                ctx.type_names.add_name(ident)?;
            }
            let converted_type_body = convert_type(&mut ctx.type_names, type_body)?;
            ctx.type_names.pop_scope();

            Ok(expr::Expr::from_content(expr::ExprContent::MakeExists {
                params: Rc::new(converted_params),
                type_body: converted_type_body,
                body: convert_expr(ctx, *body)?,
            }))
        }
    }
}
