use std::fmt::Display;

use crate::{error::AiScriptSyntaxError, node as ast};

pub enum Type {
    Simple(TSimple),
    Generic(TGeneric),
    Fn(TFn),
}

impl Display for ast::NamedTypeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ast::NamedTypeSource { name, inner, .. } = self;
        write!(f, "{name}")?;
        if let Some(inner) = inner {
            write!(f, "<{inner}>")?;
        }
        Ok(())
    }
}

impl Display for ast::FnTypeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ast::FnTypeSource { args, result, .. } = self;
        write!(
            f,
            "@({}) {{ {result} }}",
            args.iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Display for ast::TypeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ast::TypeSource::NamedTypeSource(type_source) => type_source.fmt(f),
            ast::TypeSource::FnTypeSource(type_source) => type_source.fmt(f),
        }
    }
}

pub fn get_type_by_source(type_source: ast::TypeSource) -> Result<Type, AiScriptSyntaxError> {
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
            _ => Err(AiScriptSyntaxError::UnknownType(type_source.to_string())),
        },
        ast::TypeSource::FnTypeSource(type_source) => Ok(Type::Fn(TFn {
            args: type_source
                .args
                .into_iter()
                .map(get_type_by_source)
                .collect::<Result<Vec<Type>, AiScriptSyntaxError>>()?,
            result: get_type_by_source(*type_source.result)?.into(),
        })),
    }
}
