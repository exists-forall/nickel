use types::*;

pub fn equiv_kind(kind1: &Kind, kind2: &Kind) -> bool {
    // Currently, because kinds contain no variables with display name information, they are
    // equivalent iff they are syntactically identical.  This may change in the future.
    kind1 == kind2
}

pub fn equiv<TAnnot1: Clone, TAnnot2: Clone, Name1: Clone, Name2: Clone>(
    ty1: AnnotType<TAnnot1, Name1>,
    ty2: AnnotType<TAnnot2, Name2>,
) -> bool {
    assert_eq!(
        ty1.free(),
        ty2.free(),
        "Cannot compare types with a different number of free variables"
    );
    match (ty1.to_content(), ty2.to_content()) {
        (TypeContent::Unit { free: _ }, TypeContent::Unit { free: _ }) => true,

        (TypeContent::Var {
             index: index1,
             free: _,
         },
         TypeContent::Var {
             index: index2,
             free: _,
         }) => index1 == index2,

        (TypeContent::Exists {
             param: param1,
             body: body1,
         },
         TypeContent::Exists {
             param: param2,
             body: body2,
         }) => equiv_kind(&param1.kind, &param2.kind) && equiv(body1, body2),

        (TypeContent::Func {
             params: params1,
             arg: arg1,
             ret: ret1,
         },
         TypeContent::Func {
             params: params2,
             arg: arg2,
             ret: ret2,
         }) => {
            if params1.len() != params2.len() {
                return false;
            }
            for (param1, param2) in params1.iter().zip(params2.iter()) {
                if !equiv_kind(&param1.kind, &param2.kind) {
                    return false;
                }
            }
            equiv(arg1, arg2) && equiv(ret1, ret2)
        }

        (TypeContent::Pair {
             left: left1,
             right: right1,
         },
         TypeContent::Pair {
             left: left2,
             right: right2,
         }) => equiv(left1, left2) && equiv(right1, right2),

        (TypeContent::App {
             constructor: constructor1,
             param: param1,
         },
         TypeContent::App {
             constructor: constructor2,
             param: param2,
         }) => equiv(constructor1, constructor2) && equiv(param1, param2),

        (_, _) => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use test_utils::types::*;

    #[test]
    #[should_panic]
    fn incompatible_free() {
        equiv(unit(0), unit(1));
    }

    #[test]
    fn equiv_unit() {
        assert!(equiv(unit(0), unit(0)));
        assert!(equiv(unit(5), unit(5)));
    }

    #[test]
    fn equiv_var() {
        assert!(equiv(var(1, 0), var(1, 0)));
        assert!(equiv(var(5, 2), var(5, 2)));
        assert!(!equiv(var(5, 2), var(5, 3)));
    }

    #[test]
    fn equiv_exists() {
        assert!(equiv(
            exists_named("x", Kind::Type, var(2, 0)),
            exists_named("x", Kind::Type, var(2, 0)),
        ));

        assert!(equiv(
            exists_named("name", Kind::Type, var(2, 0)),
            exists_named("different_name", Kind::Type, var(2, 0)),
        ));

        assert!(!equiv(
            exists_named("x", Kind::Type, var(2, 0)),
            exists_named("x", Kind::Place, var(2, 0)),
        ));

        assert!(!equiv(
            exists_named("x", Kind::Type, var(2, 0)),
            exists_named("x", Kind::Type, var(2, 1)),
        ));
    }

    #[test]
    fn equiv_func() {
        assert!(equiv(
            func(var(2, 0), var(2, 1)),
            func(var(2, 0), var(2, 1)),
        ));

        assert!(!equiv(
            func(var(3, 0), var(3, 2)),
            func(var(3, 1), var(3, 2)),
        ));

        assert!(!equiv(
            func(var(3, 0), var(3, 1)),
            func(var(3, 0), var(3, 2)),
        ));
    }

    #[test]
    fn equiv_forall() {
        assert!(equiv(
            func_forall_named(
                &[("T", Kind::Type), ("U", Kind::Place)],
                var(2, 0),
                var(2, 1),
            ),
            func_forall_named(
                &[("T", Kind::Type), ("U", Kind::Place)],
                var(2, 0),
                var(2, 1),
            ),
        ));

        assert!(equiv(
            func_forall_named(
                &[("T", Kind::Type), ("U", Kind::Place)],
                var(2, 0),
                var(2, 1),
            ),
            func_forall_named(
                &[("V", Kind::Type), ("W", Kind::Place)],
                var(2, 0),
                var(2, 1),
            ),
        ));

        assert!(!equiv(
            func_forall_named(
                &[("T", Kind::Type), ("U", Kind::Place)],
                var(2, 0),
                var(2, 1),
            ),
            func_forall_named(
                &[("T", Kind::Type), ("U", Kind::Place), ("V", Kind::Type)],
                var(3, 0),
                var(3, 1),
            ),
        ));

        assert!(!equiv(
            func_forall_named(
                &[("T", Kind::Type), ("U", Kind::Version)],
                var(2, 0),
                var(2, 1),
            ),
            func_forall_named(
                &[("T", Kind::Place), ("U", Kind::Version)],
                var(2, 0),
                var(2, 1),
            ),
        ));

        assert!(!equiv(
            func_forall_named(
                &[("T", Kind::Type), ("U", Kind::Version)],
                var(3, 0),
                var(3, 2),
            ),
            func_forall_named(
                &[("T", Kind::Place), ("U", Kind::Version)],
                var(3, 1),
                var(3, 2),
            ),
        ));

        assert!(!equiv(
            func_forall_named(
                &[("T", Kind::Type), ("U", Kind::Version)],
                var(3, 0),
                var(3, 1),
            ),
            func_forall_named(
                &[("T", Kind::Place), ("U", Kind::Version)],
                var(3, 0),
                var(3, 2),
            ),
        ));
    }

    #[test]
    fn equiv_pair() {
        assert!(equiv(
            pair(var(2, 0), var(2, 1)),
            pair(var(2, 0), var(2, 1)),
        ));

        assert!(!equiv(
            pair(var(3, 0), var(3, 2)),
            pair(var(3, 1), var(3, 2)),
        ));

        assert!(!equiv(
            pair(var(3, 0), var(3, 1)),
            pair(var(3, 0), var(3, 2)),
        ));
    }

    #[test]
    fn equiv_app() {
        assert!(equiv(app(var(2, 0), var(2, 1)), app(var(2, 0), var(2, 1))));

        assert!(!equiv(app(var(3, 0), var(3, 2)), app(var(3, 1), var(3, 2))));

        assert!(!equiv(app(var(3, 0), var(3, 1)), app(var(3, 0), var(3, 2))));
    }
}
