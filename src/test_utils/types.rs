use std::rc::Rc;

use types::*;

pub fn unit(free: usize) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Unit { free })
}

pub fn var(free: usize, index: usize) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Var { free, index })
}

pub fn quantified(quantifier: Quantifier, kind: Kind, body: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Quantified {
        quantifier,
        param: TypeParam {
            name: Rc::new("".to_owned()),
            kind,
        },
        body,
    })
}

pub fn quantified_named(
    quantifier: Quantifier,
    name: &str,
    kind: Kind,
    body: Type<Rc<String>>,
) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Quantified {
        quantifier,
        param: TypeParam {
            name: Rc::new(name.to_owned()),
            kind,
        },
        body,
    })
}

pub fn exists(kind: Kind, body: Type<Rc<String>>) -> Type<Rc<String>> {
    quantified(Quantifier::Exists, kind, body)
}

pub fn exists_named(name: &str, kind: Kind, body: Type<Rc<String>>) -> Type<Rc<String>> {
    quantified_named(Quantifier::Exists, name, kind, body)
}

pub fn forall(kind: Kind, body: Type<Rc<String>>) -> Type<Rc<String>> {
    quantified(Quantifier::ForAll, kind, body)
}

pub fn forall_named(name: &str, kind: Kind, body: Type<Rc<String>>) -> Type<Rc<String>> {
    quantified_named(Quantifier::ForAll, name, kind, body)
}

pub fn func(arg: Type<Rc<String>>, ret: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Func { arg, ret })
}

pub fn func_forall(
    param_kinds: &[Kind],
    arg: Type<Rc<String>>,
    ret: Type<Rc<String>>,
) -> Type<Rc<String>> {
    let mut result = func(arg, ret);
    for kind in param_kinds.iter().rev() {
        result = forall(kind.clone(), result);
    }
    result
}

pub fn func_forall_named(
    params: &[(&str, Kind)],
    arg: Type<Rc<String>>,
    ret: Type<Rc<String>>,
) -> Type<Rc<String>> {
    let mut result = func(arg, ret);
    for (name, kind) in params.iter().rev().cloned() {
        result = forall_named(name, kind, result);
    }
    result
}

pub fn pair(left: Type<Rc<String>>, right: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Pair { left, right })
}

pub fn app(constructor: Type<Rc<String>>, param: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::App { constructor, param })
}
