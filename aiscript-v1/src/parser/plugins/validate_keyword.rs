use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node as ast,
    parser::visit::Visitor,
};

const RESERVED_WORD: [&str; 39] = [
    "as",
    "async",
    "attr",
    "attribute",
    "await",
    "catch",
    "class",
    // "const",
    "component",
    "constructor",
    // "def",
    "dictionary",
    "enum",
    "export",
    "finally",
    "fn",
    // "func",
    // "function",
    "hash",
    "in",
    "interface",
    "out",
    "private",
    "public",
    "ref",
    "static",
    "struct",
    "table",
    "this",
    "throw",
    "trait",
    "try",
    "undefined",
    "use",
    "using",
    "when",
    "yield",
    "import",
    "is",
    "meta",
    "module",
    "namespace",
    "new",
];

struct KeywordValidator;

impl Visitor for KeywordValidator {
    fn callback_namespace(
        &mut self,
        namespace: ast::Namespace,
    ) -> Result<ast::Namespace, AiScriptSyntaxError> {
        if RESERVED_WORD.contains(&namespace.name.as_str()) {
            Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord(namespace.name),
                pos: namespace.loc.start,
            })
        } else {
            Ok(namespace)
        }
    }

    fn callback_meta(&mut self, meta: ast::Meta) -> Result<ast::Meta, AiScriptSyntaxError> {
        match meta {
            ast::Meta {
                loc,
                name: Some(name),
                ..
            } if RESERVED_WORD.contains(&name.as_str()) => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord(name),
                pos: loc.start,
            })?,
            _ => Ok(meta),
        }
    }

    fn callback_statement(
        &mut self,
        statement: ast::Statement,
    ) -> Result<ast::Statement, AiScriptSyntaxError> {
        match statement {
            ast::Statement::Definition(definition) => {
                let definition = *definition;
                let dest = DestKeywordValidator.visit_expression(definition.dest)?;
                Ok(ast::Statement::Definition(
                    ast::Definition { dest, ..definition }.into(),
                ))
            }
            ast::Statement::Each(each) => {
                let each = *each;
                match each {
                    ast::Each {
                        loc,
                        label: Some(label),
                        ..
                    } if RESERVED_WORD.contains(&label.as_str()) => Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ReservedWord(label),
                        pos: loc.start,
                    }),
                    ast::Each { var, .. } => {
                        let var = DestKeywordValidator.visit_expression(var)?;
                        Ok(ast::Statement::Each(ast::Each { var, ..each }.into()))
                    }
                }
            }
            ast::Statement::For(for_) => match *for_ {
                ast::For {
                    loc,
                    label: Some(label),
                    ..
                } if RESERVED_WORD.contains(&label.as_str()) => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::ReservedWord(label),
                    pos: loc.start,
                }),
                _ => Ok(ast::Statement::For(for_)),
            },
            ast::Statement::ForLet(for_let) => match *for_let {
                ast::ForLet { loc, var, .. } if RESERVED_WORD.contains(&var.as_str()) => {
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ReservedWord(var),
                        pos: loc.start,
                    })
                }
                _ => Ok(ast::Statement::ForLet(for_let)),
            },
            ast::Statement::Loop(loop_) => match *loop_ {
                ast::Loop {
                    loc,
                    label: Some(label),
                    ..
                } if RESERVED_WORD.contains(&label.as_str()) => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::ReservedWord(label),
                    pos: loc.start,
                }),
                _ => Ok(ast::Statement::Loop(loop_)),
            },
            ast::Statement::Break(break_) => match *break_ {
                ast::Break {
                    loc,
                    label: Some(label),
                    ..
                } if RESERVED_WORD.contains(&label.as_str()) => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::ReservedWord(label),
                    pos: loc.start,
                }),
                _ => Ok(ast::Statement::Break(break_)),
            },
            ast::Statement::Continue(continue_) => match *continue_ {
                ast::Continue {
                    loc,
                    label: Some(label),
                    ..
                } if RESERVED_WORD.contains(&label.as_str()) => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::ReservedWord(label),
                    pos: loc.start,
                }),
                _ => Ok(ast::Statement::Continue(continue_)),
            },
            _ => Ok(statement),
        }
    }

    fn callback_expression(
        &mut self,
        expression: ast::Expression,
    ) -> Result<ast::Expression, AiScriptSyntaxError> {
        match expression {
            ast::Expression::Identifier(identifier) => match *identifier {
                ast::Identifier { loc, name } if RESERVED_WORD.contains(&name.as_str()) => {
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ReservedWord(name),
                        pos: loc.start,
                    })
                }
                _ => Ok(ast::Expression::Identifier(identifier)),
            },
            ast::Expression::Fn(fn_) => {
                let fn_ = *fn_;
                let type_params = fn_
                    .type_params
                    .map(|type_params| {
                        type_params
                            .into_iter()
                            .map(|type_param| {
                                if RESERVED_WORD.contains(&type_param.name.as_str()) {
                                    Err(AiScriptSyntaxError {
                                        kind: AiScriptSyntaxErrorKind::ReservedWord(
                                            type_param.name,
                                        ),
                                        pos: fn_.loc.start.clone(),
                                    })
                                } else {
                                    Ok(type_param)
                                }
                            })
                            .collect::<Result<Vec<ast::TypeParam>, AiScriptSyntaxError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?;
                let params = fn_
                    .params
                    .into_iter()
                    .map(|param| {
                        DestKeywordValidator
                            .visit_expression(param.dest)
                            .map(|dest| ast::Param { dest, ..param })
                    })
                    .collect::<Result<Vec<ast::Param>, AiScriptSyntaxError>>()?;
                Ok(ast::Expression::Fn(
                    ast::Fn {
                        type_params,
                        params,
                        ..fn_
                    }
                    .into(),
                ))
            }
            _ => Ok(expression),
        }
    }

    fn callback_type_source(
        &mut self,
        type_source: ast::TypeSource,
    ) -> Result<ast::TypeSource, AiScriptSyntaxError> {
        match type_source {
            ast::TypeSource::NamedTypeSource(ast::NamedTypeSource { loc, name, .. })
                if RESERVED_WORD.contains(&name.as_str()) =>
            {
                Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::ReservedWord(name),
                    pos: loc.start,
                })
            }
            ast::TypeSource::FnTypeSource(fn_type_source) => {
                let type_params = fn_type_source
                    .type_params
                    .map(|type_params| {
                        type_params
                            .into_iter()
                            .map(|type_param| {
                                if RESERVED_WORD.contains(&type_param.name.as_str()) {
                                    Err(AiScriptSyntaxError {
                                        kind: AiScriptSyntaxErrorKind::ReservedWord(
                                            type_param.name,
                                        ),
                                        pos: fn_type_source.loc.start.clone(),
                                    })
                                } else {
                                    Ok(type_param)
                                }
                            })
                            .collect::<Result<Vec<ast::TypeParam>, AiScriptSyntaxError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?;
                Ok(ast::TypeSource::FnTypeSource(ast::FnTypeSource {
                    type_params,
                    ..fn_type_source
                }))
            }
            _ => Ok(type_source),
        }
    }

    fn callback_attribute(
        &mut self,
        attribute: ast::Attribute,
    ) -> Result<ast::Attribute, AiScriptSyntaxError> {
        if RESERVED_WORD.contains(&attribute.name.as_str()) {
            Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord(attribute.name),
                pos: attribute.loc.start,
            })
        } else {
            Ok(attribute)
        }
    }
}

struct DestKeywordValidator;

impl Visitor for DestKeywordValidator {
    fn callback_expression(
        &mut self,
        expression: ast::Expression,
    ) -> Result<ast::Expression, AiScriptSyntaxError> {
        match expression {
            ast::Expression::Null(null) => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord("null".to_string()),
                pos: null.loc.start,
            }),
            ast::Expression::Bool(bool) => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord(bool.value.to_string()),
                pos: bool.loc.start,
            }),
            _ => Ok(expression),
        }
    }
}

pub fn validate_keyword(
    nodes: impl IntoIterator<Item = ast::Node>,
) -> Result<Vec<ast::Node>, AiScriptSyntaxError> {
    nodes
        .into_iter()
        .map(|node| KeywordValidator.visit_node(node))
        .collect()
}
