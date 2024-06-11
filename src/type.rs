use crate::{error::AiScriptError, node as ast};

pub struct TSimple {
    pub name: String,
}

pub struct TGeneric {
    pub name: String,
    pub inners: Vec<Type>,
}

pub struct TFn {
    pub args: Vec<Type>,
    pub result: Box<Type>,
}

pub enum Type {
    Simple(TSimple),
    Generic(TGeneric),
    Fn(TFn),
}

pub fn get_type_by_source(type_source: ast::TypeSource) -> Result<Type, AiScriptError> {
    match type_source {
        ast::TypeSource::NamedTypeSource(type_source) => match type_source.name.as_str() {
            "null" | "bool" | "num" | "str" | "any" | "void" => Ok(Type::Simple(TSimple {
                name: type_source.name,
            })),
            "arr" | "obj" => Ok(Type::Generic(TGeneric {
                name: type_source.name,
                inners: vec![type_source.inner.map_or_else(
                    || {
                        Ok(Type::Simple(TSimple {
                            name: "any".to_string(),
                        }))
                    },
                    |inner| get_type_by_source(*inner),
                )?],
            })),
            _ => todo!(),
        },
        ast::TypeSource::FnTypeSource(type_source) => Ok(Type::Fn(TFn {
            args: type_source
                .args
                .into_iter()
                .map(get_type_by_source)
                .collect::<Result<Vec<Type>, AiScriptError>>()?,
            result: get_type_by_source(*type_source.result)?.into(),
        })),
    }
}
