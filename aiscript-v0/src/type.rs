use std::fmt::Display;

use crate::{error::AiScriptSyntaxError, node as ast};

pub enum Type {
    Simple,
    Generic,
    Fn,
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

impl TryFrom<ast::TypeSource> for Type {
    type Error = AiScriptSyntaxError;

    fn try_from(value: ast::TypeSource) -> Result<Self, Self::Error> {
        match value {
            ast::TypeSource::NamedTypeSource(type_source) => match type_source.name.as_str() {
                "null" | "bool" | "num" | "str" | "any" | "void" => Ok(Type::Simple),
                "arr" | "obj" => {
                    if let Some(inner) = type_source.inner {
                        Type::try_from(*inner)?;
                    }
                    Ok(Type::Generic)
                }
                _ => Err(AiScriptSyntaxError::UnknownType(type_source.to_string())),
            },
            ast::TypeSource::FnTypeSource(ast::FnTypeSource { args, result, .. }) => {
                for arg in args {
                    Type::try_from(arg)?;
                }
                Type::try_from(*result)?;
                Ok(Type::Fn)
            }
        }
    }
}
