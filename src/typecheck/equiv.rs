use types::*;

fn equiv_kind(kind1: &Kind, kind2: &Kind) -> bool {
    // Currently, because kinds have no names, they are equivalent iff they are syntactically
    // identical.  This may change in the future.
    kind1 == kind2
}

pub fn equiv<Name: Clone>(ty1: Type<Name>, ty2: Type<Name>) -> bool {
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
