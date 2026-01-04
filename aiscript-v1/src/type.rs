use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node as ast,
};

pub enum Type {
    Simple {
        name: String,
    },
    Generic {
        name: String,
        inners: Vec<Type>,
    },
    Fn {
        params: Vec<Type>,
        result: Box<Type>,
    },
    Param {
        name: String,
    },
    Union {
        inners: Vec<Type>,
    },
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Simple { name } => write!(f, "{name}"),
            Type::Generic { name, inners } => write!(
                f,
                "{name} <{}>",
                inners
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Type::Fn { params, result } => write!(
                f,
                "@({}) {result}",
                params
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Type::Param { name } => write!(f, "{name}"),
            Type::Union { inners } => write!(
                f,
                "{}",
                inners
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(" | ")
            ),
        }
    }
}

impl std::fmt::Display for ast::NamedTypeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ast::NamedTypeSource { name, inner, .. } = self;
        write!(f, "{name}")?;
        if let Some(inner) = inner {
            write!(f, "<{inner}>")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ast::FnTypeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ast::FnTypeSource {
            params: args,
            result,
            ..
        } = self;
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

impl std::fmt::Display for ast::UnionTypeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.inners
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(" | ")
        )
    }
}

impl std::fmt::Display for ast::TypeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ast::TypeSource::NamedTypeSource(type_source) => type_source.fmt(f),
            ast::TypeSource::FnTypeSource(type_source) => type_source.fmt(f),
            ast::TypeSource::UnionTypeSource(type_source) => type_source.fmt(f),
        }
    }
}

pub fn get_type_by_source(
    type_source: ast::TypeSource,
    type_params: &[&ast::TypeParam],
) -> Result<Type, AiScriptSyntaxError> {
    match type_source {
        ast::TypeSource::NamedTypeSource(named_type_source) => {
            if let Some(type_param) = type_params
                .iter()
                .find(|param| param.name == named_type_source.name)
                && named_type_source.inner.is_none()
            {
                return Ok(Type::Param {
                    name: type_param.name.clone(),
                });
            }
            match named_type_source.name.as_str() {
                "null" | "bool" | "num" | "str" | "error" | "never" | "any" | "void"
                    if named_type_source.inner.is_none() =>
                {
                    Ok(Type::Simple {
                        name: named_type_source.name,
                    })
                }
                "arr" | "obj" => {
                    let inner_type = named_type_source
                        .inner
                        .map(|inner| get_type_by_source(*inner, type_params))
                        .unwrap_or_else(|| {
                            Ok(Type::Simple {
                                name: "any".to_string(),
                            })
                        })?;
                    Ok(Type::Generic {
                        name: named_type_source.name,
                        inners: vec![inner_type],
                    })
                }
                _ => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::UnknownType(named_type_source.to_string()),
                    pos: named_type_source.loc.start,
                }),
            }
        }
        ast::TypeSource::FnTypeSource(fn_type_source) => {
            let fn_type_params = fn_type_source.type_params.unwrap_or_default();
            let mut fn_type_params = fn_type_params.iter().collect::<Vec<&ast::TypeParam>>();
            fn_type_params.extend_from_slice(type_params);
            let param_types = fn_type_source
                .params
                .into_iter()
                .map(|param| get_type_by_source(param, &fn_type_params))
                .collect::<Result<Vec<Type>, AiScriptSyntaxError>>()?;
            Ok(Type::Fn {
                params: param_types,
                result: get_type_by_source(*fn_type_source.result, &fn_type_params)?.into(),
            })
        }
        ast::TypeSource::UnionTypeSource(union_type_source) => {
            let inner_types = union_type_source
                .inners
                .into_iter()
                .map(|inner| get_type_by_source(inner, type_params))
                .collect::<Result<Vec<Type>, AiScriptSyntaxError>>()?;
            Ok(Type::Union {
                inners: inner_types,
            })
        }
    }
}
