use super::equiv::equiv_kind;
use super::context::Context;
use types::*;

#[derive(Clone, Debug)]
pub enum Error {
    KindMismatch,
    TypeMismatch,
    MovedTwice,
    NotMoved,
}

pub fn annot_kinds<Name: Clone>(
    ctx: &mut Context<Name>,
    ty: Type<Name>,
) -> Result<AnnotType<Kind, Name>, Error> {
    match ty.to_content() {
        TypeContent::Unit { free } => {
            Ok(AnnotType::from_content_annot(
                Kind::Type,
                TypeContent::Unit { free },
            ))
        }

        TypeContent::Var { free, index } => {
            Ok(AnnotType::from_content_annot(
                ctx.type_kind(index).clone(),
                TypeContent::Var { free, index },
            ))
        }

        TypeContent::Exists { param, body } => {
            ctx.push_scope();
            ctx.add_type_kind(param.kind.clone());
            let body_annot = annot_kinds(ctx, body)?;
            ctx.pop_scope();

            if equiv_kind(&body_annot.annot(), &Kind::Type) {
                Ok(AnnotType::from_content_annot(
                    Kind::Type,
                    TypeContent::Exists {
                        param,
                        body: body_annot,
                    },
                ))
            } else {
                Err(Error::KindMismatch)
            }
        }

        TypeContent::Func { params, arg, ret } => {
            ctx.push_scope();
            for param in params.iter().cloned() {
                ctx.add_type_kind(param.kind);
            }
            let arg_annot = annot_kinds(ctx, arg)?;
            let ret_annot = annot_kinds(ctx, ret)?;
            ctx.pop_scope();

            if equiv_kind(&arg_annot.annot(), &Kind::Type) &&
                equiv_kind(&ret_annot.annot(), &Kind::Type)
            {
                Ok(AnnotType::from_content_annot(
                    Kind::Type,
                    TypeContent::Func {
                        params,
                        arg: arg_annot,
                        ret: ret_annot,
                    },
                ))
            } else {
                Err(Error::KindMismatch)
            }
        }

        TypeContent::Pair { left, right } => {
            let left_annot = annot_kinds(ctx, left)?;
            let right_annot = annot_kinds(ctx, right)?;

            if equiv_kind(&left_annot.annot(), &Kind::Type) &&
                equiv_kind(&right_annot.annot(), &Kind::Type)
            {
                Ok(AnnotType::from_content_annot(
                    Kind::Type,
                    TypeContent::Pair {
                        left: left_annot,
                        right: right_annot,
                    },
                ))
            } else {
                Err(Error::KindMismatch)
            }
        }

        TypeContent::App { constructor, param } => {
            let constructor_annot = annot_kinds(ctx, constructor)?;
            let param_annot = annot_kinds(ctx, param)?;

            let app_kind =
                if let Kind::Constructor { params, result } = constructor_annot.annot() {
                    debug_assert!(params.len() > 0);
                    if !equiv_kind(&params[0], &param_annot.annot()) {
                        return Err(Error::KindMismatch);
                    }
                    if params.len() == 1 {
                        (&result as &Kind).clone()
                    } else {
                        Kind::Constructor {
                            params: params.slice(1..params.len()),
                            result,
                        }
                    }
                } else {
                    return Err(Error::KindMismatch);
                };

            Ok(AnnotType::from_content_annot(
                app_kind,
                TypeContent::App {
                    constructor: constructor_annot,
                    param: param_annot,
                },
            ))
        }
    }
}
