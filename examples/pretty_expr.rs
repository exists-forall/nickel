extern crate nickel;
extern crate pretty_trait;

use std::rc::Rc;

use pretty_trait::println_simple;

use nickel::types::*;
use nickel::expr::*;
use nickel::test_utils::types;
use nickel::test_utils::expr::*;
use nickel::pretty_syntax::names::Names;
use nickel::pretty_syntax::expr::{Place, to_pretty};

fn print_expr(var_names: &mut Names, type_names: &mut Names, expr: Expr<Rc<String>>) {
    println_simple(&to_pretty(var_names, type_names, Place::Root, expr));
}

fn rc_str(s: &str) -> Rc<String> {
    Rc::new(s.to_owned())
}

fn main() {
    let mut var_names = Names::new();
    let mut type_names = Names::new();

    var_names.add_name(rc_str("foo"));
    let foo_var = var(VarUsage::Copy, 2, 2, 0);

    var_names.add_name(rc_str("bar"));
    let bar_var = var(VarUsage::Copy, 2, 2, 1);

    type_names.add_name(rc_str("Foo"));
    let foo_type = types::var(2, 0);

    type_names.add_name(rc_str("Bar"));
    let bar_type = types::var(2, 1);

    println!("Simple variables:");
    print_expr(&mut var_names, &mut type_names, foo_var.clone());
    print_expr(&mut var_names, &mut type_names, bar_var.clone());

    println!();
    println!("By move:");
    print_expr(
        &mut var_names,
        &mut type_names,
        var(VarUsage::Move, 2, 2, 0),
    );
    print_expr(
        &mut var_names,
        &mut type_names,
        var(VarUsage::Move, 2, 2, 1),
    );

    println!();
    println!("Simple pairs:");
    print_expr(
        &mut var_names,
        &mut type_names,
        pair(foo_var.clone(), bar_var.clone()),
    );
    print_expr(
        &mut var_names,
        &mut type_names,
        pair(foo_var.clone(), pair(bar_var.clone(), foo_var.clone())),
    );
    print_expr(
        &mut var_names,
        &mut type_names,
        pair(pair(foo_var.clone(), bar_var.clone()), foo_var.clone()),
    );

    println!();
    println!("Simple applications:");
    print_expr(
        &mut var_names,
        &mut type_names,
        app(foo_var.clone(), bar_var.clone()),
    );
    print_expr(
        &mut var_names,
        &mut type_names,
        app(foo_var.clone(), pair(bar_var.clone(), foo_var.clone())),
    );

    println!();
    println!("With type parameters:");
    print_expr(
        &mut var_names,
        &mut type_names,
        app_forall(
            foo_var.clone(),
            &[foo_type.clone(), bar_type.clone()],
            pair(foo_var.clone(), bar_var.clone()),
        ),
    );

    println!();
    println!("Simple functions");
    print_expr(
        &mut var_names,
        &mut type_names,
        func_named(
            "baz",
            foo_type.clone(),
            app(foo_var.clone(), var(VarUsage::Move, 3, 2, 2)),
        ),
    );
    print_expr(
        &mut var_names,
        &mut type_names,
        func_named(
            "foo",
            types::pair(foo_type.clone(), bar_type.clone()),
            pair(var(VarUsage::Copy, 3, 2, 0), var(VarUsage::Move, 3, 2, 2)),
        ),
    );

    println!();
    println!("Univesally quantified:");
    print_expr(
        &mut var_names,
        &mut type_names,
        func_forall_named(
            &[("a", Kind::Type)],
            "x",
            types::var(3, 2),
            var(VarUsage::Move, 3, 3, 2),
        ),
    );
}
