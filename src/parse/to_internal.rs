use std::rc::Rc;

use super::syntax;
use types;
use super::names::{Names, Error};

pub fn convert_type(
    type_names: &mut Names,
    ty: syntax::Type,
) -> Result<types::Type<Rc<String>>, Error> {
    match ty {
        syntax::Type::Unit => Ok(types::Type::from_content(
            types::TypeContent::Unit { free: type_names.index_count() },
        )),

        syntax::Type::Var { ident } => Ok(types::Type::from_content(types::TypeContent::Var {
            free: type_names.index_count(),
            index: type_names.get_index(&ident)?,
        })),

        syntax::Type::Exists { param, body } => {
            type_names.push_scope();

            type_names.add_name(param.ident.clone())?;

            let result = types::Type::from_content(types::TypeContent::Exists {
                param: types::TypeParam {
                    name: Rc::new(param.ident.name),
                    kind: param.kind,
                },
                body: convert_type(type_names, *body)?,
            });

            type_names.pop_scope();

            Ok(result)
        }

        syntax::Type::Func { params, arg, ret } => {
            type_names.push_scope();

            for &syntax::TypeParam { ref ident, kind: _ } in &params {
                type_names.add_name(ident.clone())?;
            }

            let result = types::Type::from_content(types::TypeContent::Func {
                params: Rc::new(
                    params
                        .into_iter()
                        .map(|param| {
                            types::TypeParam {
                                name: Rc::new(param.ident.name),
                                kind: param.kind,
                            }
                        })
                        .collect(),
                ),
                arg: convert_type(type_names, *arg)?,
                ret: convert_type(type_names, *ret)?,
            });

            type_names.pop_scope();

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
