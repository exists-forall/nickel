extern crate nickel;
extern crate pretty_trait;

use std::rc::Rc;

use pretty_trait::println_simple;

use nickel::types::*;
use nickel::pretty_syntax::names::Names;
use nickel::pretty_syntax::types::{Place, to_pretty};

fn print_type(names: &mut Names, ty: Type<Rc<String>>) {
    println_simple(&to_pretty(names, Place::Root, ty));
}

fn rc_str(s: &str) -> Rc<String> {
    Rc::new(s.to_owned())
}

fn main() {
    let mut names = Names::new();

    {
        names.push_scope();

        names.add_name(rc_str("foo"));
        let foo = Type::from_content(TypeContent::Var { free: 2, index: 0 });

        names.add_name(rc_str("bar"));
        let bar = Type::from_content(TypeContent::Var { free: 2, index: 1 });

        println!("Simple variables:");
        print_type(&mut names, foo.clone());
        print_type(&mut names, bar.clone());

        println!();
        println!("Simple pairs:");
        print_type(
            &mut names,
            Type::from_content(TypeContent::Pair {
                left: foo.clone(),
                right: bar.clone(),
            }),
        );
        print_type(
            &mut names,
            Type::from_content(TypeContent::Pair {
                left: foo.clone(),
                right: Type::from_content(TypeContent::Pair {
                    left: bar.clone(),
                    right: foo.clone(),
                }),
            }),
        );
        print_type(
            &mut names,
            Type::from_content(TypeContent::Pair {
                left: Type::from_content(TypeContent::Pair {
                    left: foo.clone(),
                    right: bar.clone(),
                }),
                right: foo.clone(),
            }),
        );

        println!();
        println!("Simple functions:");
        print_type(
            &mut names,
            Type::from_content(TypeContent::Func {
                access: FuncAccess::Many,
                params: Rc::new(Vec::new()),
                arg: foo.clone(),
                ret: bar.clone(),
            }),
        );
        print_type(
            &mut names,
            Type::from_content(TypeContent::Func {
                access: FuncAccess::Many,
                params: Rc::new(Vec::new()),
                arg: Type::from_content(TypeContent::Pair {
                    left: foo.clone(),
                    right: bar.clone(),
                }),
                ret: bar.clone(),
            }),
        );
        print_type(
            &mut names,
            Type::from_content(TypeContent::Func {
                access: FuncAccess::Many,
                params: Rc::new(Vec::new()),
                arg: foo.clone(),
                ret: Type::from_content(TypeContent::Pair {
                    left: bar.clone(),
                    right: foo.clone(),
                }),
            }),
        );

        names.pop_scope();
    }

}
