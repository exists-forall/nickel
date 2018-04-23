use std::rc::Rc;
use std::iter::repeat;

use expr::*;
use types::*;

pub fn unit(free_vars: usize, free_types: usize) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Unit {
        free_vars,
        free_types,
    })
}

pub fn var(usage: VarUsage, free_vars: usize, free_types: usize, index: usize) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Var {
        usage,
        free_vars,
        free_types,
        index,
    })
}

pub fn forall(param_count: usize, body: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::ForAll {
        type_params: Rc::new(
            repeat(TypeParam {
                name: Rc::new("".to_owned()),
            }).take(param_count)
                .collect(),
        ),
        body,
    })
}

pub fn forall_named(params: &[&str], body: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::ForAll {
        type_params: Rc::new(
            params
                .iter()
                .cloned()
                .map(|name| TypeParam {
                    name: Rc::new(name.to_owned()),
                })
                .collect(),
        ),
        body,
    })
}

pub fn func(arg_type: Type<Rc<String>>, body: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Func {
        arg_name: Rc::new("".to_owned()),
        arg_type,
        arg_phase: Phase::Dynamic,
        body,
    })
}

pub fn func_forall(
    param_count: usize,
    arg_type: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    forall(param_count, func(arg_type, body))
}

pub fn func_named(
    arg_name: &str,
    arg_type: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Func {
        arg_name: Rc::new(arg_name.to_owned()),
        arg_type,
        arg_phase: Phase::Dynamic,
        body,
    })
}

pub fn func_forall_named(
    params: &[&str],
    arg_name: &str,
    arg_type: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    forall_named(params, func_named(arg_name, arg_type, body))
}

pub fn inst(receiver: Expr<Rc<String>>, type_params: &[Type<Rc<String>>]) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Inst {
        receiver,
        type_params: Rc::new(type_params.to_owned()),
    })
}

pub fn app(callee: Expr<Rc<String>>, arg: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::App { callee, arg })
}

pub fn app_forall(
    callee: Expr<Rc<String>>,
    type_params: &[Type<Rc<String>>],
    arg: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    app(inst(callee, type_params), arg)
}

pub fn pair(left: Expr<Rc<String>>, right: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Pair { left, right })
}

pub fn let_vars(count: usize, val: Expr<Rc<String>>, body: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Let {
        names: Rc::new(repeat(Rc::new("".to_owned())).take(count).collect()),
        val,
        body,
    })
}

pub fn let_vars_named(
    names: &[&str],
    val: Expr<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Let {
        names: Rc::new(names.iter().map(|&name| Rc::new(name.to_owned())).collect()),
        val,
        body,
    })
}

pub fn let_exists(
    type_count: usize,
    val: Expr<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::LetExists {
        type_names: Rc::new(repeat(Rc::new("".to_owned())).take(type_count).collect()),
        val_name: Rc::new("".to_owned()),
        val,
        body,
    })
}

pub fn let_exists_named(
    type_names: &[&str],
    val_name: &str,
    val: Expr<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::LetExists {
        type_names: Rc::new(
            type_names
                .iter()
                .map(|&name| Rc::new(name.to_owned()))
                .collect(),
        ),
        val_name: Rc::new(val_name.to_owned()),
        val,
        body,
    })
}

pub fn make_exists(
    params: &[Type<Rc<String>>],
    type_body: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::MakeExists {
        params: Rc::new(
            params
                .iter()
                .cloned()
                .map(|ty| (Rc::new("".to_owned()), ty))
                .collect(),
        ),
        type_body,
        body,
    })
}

pub fn make_exists_named(
    params: &[(&str, Type<Rc<String>>)],
    type_body: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::MakeExists {
        params: Rc::new(
            params
                .iter()
                .cloned()
                .map(|(name, ty)| (Rc::new(name.to_owned()), ty))
                .collect(),
        ),
        type_body,
        body,
    })
}

pub fn cast(
    type_body: Type<Rc<String>>,
    equivalence: Expr<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    cast_named("", type_body, equivalence, body)
}

pub fn cast_named(
    param_name: &str,
    type_body: Type<Rc<String>>,
    equivalence: Expr<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Cast {
        param: TypeParam {
            name: Rc::new(param_name.to_owned()),
        },
        type_body,
        equivalence,
        body,
    })
}
