use std::rc::Rc;

use types::*;

pub fn unit(free: usize) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Unit { free })
}

pub fn var(free: usize, index: usize) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Var { free, index })
}

pub fn quantified(quantifier: Quantifier, body: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Quantified {
        quantifier,
        param: TypeParam {
            name: Rc::new("".to_owned()),
        },
        body,
    })
}

pub fn quantified_named(
    quantifier: Quantifier,
    name: &str,
    body: Type<Rc<String>>,
) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Quantified {
        quantifier,
        param: TypeParam {
            name: Rc::new(name.to_owned()),
        },
        body,
    })
}

pub fn exists(body: Type<Rc<String>>) -> Type<Rc<String>> {
    quantified(Quantifier::Exists, body)
}

pub fn exists_named(name: &str, body: Type<Rc<String>>) -> Type<Rc<String>> {
    quantified_named(Quantifier::Exists, name, body)
}

pub fn forall(body: Type<Rc<String>>) -> Type<Rc<String>> {
    quantified(Quantifier::ForAll, body)
}

pub fn forall_named(name: &str, body: Type<Rc<String>>) -> Type<Rc<String>> {
    quantified_named(Quantifier::ForAll, name, body)
}

pub fn func(arg: Type<Rc<String>>, ret: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Func {
        arg,
        arg_phase: Phase::Dynamic,
        ret,
        ret_phase: Phase::Dynamic,
    })
}

pub fn func_forall(
    param_count: usize,
    arg: Type<Rc<String>>,
    ret: Type<Rc<String>>,
) -> Type<Rc<String>> {
    let mut result = func(arg, ret);
    for _ in 0..param_count {
        result = forall(result);
    }
    result
}

pub fn func_forall_named(
    params: &[&str],
    arg: Type<Rc<String>>,
    ret: Type<Rc<String>>,
) -> Type<Rc<String>> {
    let mut result = func(arg, ret);
    for name in params.iter().rev() {
        result = forall_named(name, result);
    }
    result
}

pub fn pair(left: Type<Rc<String>>, right: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Pair { left, right })
}

pub fn app(constructor: Type<Rc<String>>, param: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::App { constructor, param })
}

pub fn equiv_ty(orig: Type<Rc<String>>, dest: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Equiv { orig, dest })
}

pub fn size(ty: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Size { ty })
}
