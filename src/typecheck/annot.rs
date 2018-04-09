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
    assert_eq!(
        ty.free(),
        ctx.type_index_count(),
        "Cannot annotate a type with {} free variables in a context with {} free variables",
        ty.free(),
        ctx.type_index_count(),
    );
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

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use super::super::equiv::equiv;
    use test_utils::types::*;
    use utils::rc_vec_view::RcVecView;

    fn kind_of<Name: Clone>(ctx: &mut Context<Name>, ty: Type<Name>) -> Result<Kind, Error> {
        let ty_annot = annot_kinds(ctx, ty.clone())?;
        assert!(
            equiv(ty_annot.clone(), ty.clone()),
            "Annotating a type with kinds should not change its content"
        );
        Ok(ty_annot.annot())
    }

    fn assert_kind<Name: Clone>(ctx: &mut Context<Name>, ty: Type<Name>, kind: Kind) {
        assert!(equiv_kind(&kind_of(ctx, ty).unwrap(), &kind));
    }

    fn assert_invalid<Name: Clone>(ctx: &Context<Name>, ty: Type<Name>) {
        // clone ctx to avoid leaving it in an invalid state
        assert!(kind_of(&mut ctx.clone(), ty).is_err());
    }

    #[test]
    #[should_panic]
    fn incompatible_free() {
        let ctx = &mut Context::new();
        let _ = kind_of(ctx, unit(10));
    }

    #[test]
    fn unit_kind() {
        let ctx = &mut Context::new();
        assert_kind(ctx, unit(0), Kind::Type);
        for _ in 0..10 {
            ctx.add_type_kind(Kind::Type);
        }
        assert_kind(ctx, unit(10), Kind::Type);
    }

    #[test]
    fn var_kind() {
        let ctx = &mut Context::new();
        ctx.add_type_kind(Kind::Place);
        ctx.add_type_kind(Kind::Type);
        assert_kind(ctx, var(2, 0), Kind::Place);
        assert_kind(ctx, var(2, 1), Kind::Type);
    }

    #[test]
    fn exists_kind() {
        let ctx = &mut Context::new();
        ctx.add_type_kind(Kind::Type);
        ctx.add_type_kind(Kind::Place);

        assert_kind(ctx, exists(Kind::Version, var(3, 0)), Kind::Type);
        assert_kind(ctx, exists(Kind::Type, var(3, 2)), Kind::Type);

        assert_invalid(ctx, exists(Kind::Type, var(3, 1)));
        assert_invalid(ctx, exists(Kind::Version, var(3, 2)));
    }

    #[test]
    fn func_kind() {
        let ctx = &mut Context::new();
        ctx.add_type_kind(Kind::Type);
        ctx.add_type_kind(Kind::Place);

        assert_kind(ctx, func(var(2, 0), var(2, 0)), Kind::Type);

        assert_invalid(ctx, func(var(2, 1), var(2, 0)));
        assert_invalid(ctx, func(var(2, 0), var(2, 1)));

        assert_kind(
            ctx,
            func_forall(&[Kind::Type, Kind::Place], var(4, 0), var(4, 0)),
            Kind::Type,
        );
        assert_kind(
            ctx,
            func_forall(&[Kind::Type, Kind::Place], var(4, 2), var(4, 2)),
            Kind::Type,
        );

        assert_invalid(
            ctx,
            func_forall(&[Kind::Type, Kind::Place], var(4, 1), var(4, 0)),
        );
        assert_invalid(
            ctx,
            func_forall(&[Kind::Type, Kind::Place], var(4, 0), var(4, 1)),
        );
        assert_invalid(
            ctx,
            func_forall(&[Kind::Type, Kind::Place], var(4, 3), var(4, 0)),
        );
        assert_invalid(
            ctx,
            func_forall(&[Kind::Type, Kind::Place], var(4, 0), var(4, 3)),
        );
    }

    #[test]
    fn pair_kind() {
        let ctx = &mut Context::new();
        ctx.add_type_kind(Kind::Type);
        ctx.add_type_kind(Kind::Place);

        assert_kind(ctx, pair(var(2, 0), var(2, 0)), Kind::Type);

        assert_invalid(ctx, pair(var(2, 1), var(2, 0)));
        assert_invalid(ctx, pair(var(2, 0), var(2, 1)));
    }

    #[test]
    fn app_kind() {
        let ctx = &mut Context::new();
        ctx.add_type_kind(Kind::Type);
        ctx.add_type_kind(Kind::Place);
        ctx.add_type_kind(Kind::Constructor {
            params: RcVecView::new(Rc::new(vec![Kind::Place])),
            result: Rc::new(Kind::Type),
        });
        ctx.add_type_kind(Kind::Constructor {
            params: RcVecView::new(Rc::new(vec![Kind::Type, Kind::Version])),
            result: Rc::new(Kind::Place),
        });

        assert_kind(ctx, app(var(4, 2), var(4, 1)), Kind::Type);

        assert_invalid(ctx, app(var(4, 2), var(4, 0)));

        assert_kind(
            ctx,
            app(var(4, 3), var(4, 0)),
            Kind::Constructor {
                params: RcVecView::new(Rc::new(vec![Kind::Version])),
                result: Rc::new(Kind::Place),
            },
        );

        assert_invalid(ctx, app(var(4, 3), var(4, 1)));
    }
}
