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
    fn visit_identifier(
        &mut self,
        identifier: &ast::Identifier,
    ) -> Result<Option<ast::Identifier>, AiScriptSyntaxError> {
        if RESERVED_WORD.contains(&identifier.name.as_str()) {
            Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord(identifier.name.to_string()),
                pos: identifier.loc.start.clone(),
            })?
        }
        Ok(None)
    }

    fn callback_namespace(
        &mut self,
        namespace: &ast::Namespace,
    ) -> Result<Option<ast::Namespace>, AiScriptSyntaxError> {
        if RESERVED_WORD.contains(&namespace.name.as_str()) {
            Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord(namespace.name.to_string()),
                pos: namespace.loc.start.clone(),
            })?
        }
        Ok(None)
    }

    fn callback_meta(
        &mut self,
        meta: &ast::Meta,
    ) -> Result<Option<ast::Meta>, AiScriptSyntaxError> {
        match meta {
            ast::Meta {
                loc,
                name: Some(name),
                ..
            } if RESERVED_WORD.contains(&name.as_str()) => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord(name.to_string()),
                pos: loc.start.clone(),
            })?,
            _ => {}
        }
        Ok(None)
    }

    fn callback_statement(
        &mut self,
        statement: &ast::Statement,
    ) -> Result<Option<ast::Statement>, AiScriptSyntaxError> {
        match statement {
            ast::Statement::Definition(definition) => {
                DestKeywordValidator.visit_expression(&definition.dest)?;
            }
            ast::Statement::Each(each) => {
                if let Some(label) = &each.label
                    && RESERVED_WORD.contains(&label.as_str())
                {
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ReservedWord(label.to_string()),
                        pos: each.loc.start.clone(),
                    })?
                } else {
                    DestKeywordValidator.visit_expression(&each.var)?;
                }
            }
            ast::Statement::For(for_) => {
                if let Some(label) = &for_.label
                    && RESERVED_WORD.contains(&label.as_str())
                {
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ReservedWord(label.to_string()),
                        pos: for_.loc.start.clone(),
                    })?
                }
            }
            ast::Statement::ForLet(for_let) => {
                if RESERVED_WORD.contains(&for_let.var.as_str()) {
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ReservedWord(for_let.var.to_string()),
                        pos: for_let.loc.start.clone(),
                    })?
                }
            }
            ast::Statement::Loop(loop_) => {
                if let Some(label) = &loop_.label
                    && RESERVED_WORD.contains(&label.as_str())
                {
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ReservedWord(label.to_string()),
                        pos: loop_.loc.start.clone(),
                    })?
                }
            }
            ast::Statement::Break(break_) => {
                if let Some(label) = &break_.label
                    && RESERVED_WORD.contains(&label.as_str())
                {
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ReservedWord(label.to_string()),
                        pos: break_.loc.start.clone(),
                    })?
                }
            }
            ast::Statement::Continue(continue_) => {
                if let Some(label) = &continue_.label
                    && RESERVED_WORD.contains(&label.as_str())
                {
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ReservedWord(label.to_string()),
                        pos: continue_.loc.start.clone(),
                    })?
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn callback_expression(
        &mut self,
        expression: &ast::Expression,
    ) -> Result<Option<ast::Expression>, AiScriptSyntaxError> {
        if let ast::Expression::Fn(fn_) = expression {
            if let Some(type_params) = &fn_.type_params {
                for ast::TypeParam { name, .. } in type_params {
                    if RESERVED_WORD.contains(&name.as_str()) {
                        Err(AiScriptSyntaxError {
                            kind: AiScriptSyntaxErrorKind::ReservedWord(name.to_string()),
                            pos: fn_.loc.start.clone(),
                        })?
                    }
                }
            }
            for ast::Param { dest, .. } in &fn_.params {
                DestKeywordValidator.visit_expression(dest)?;
            }
        }
        Ok(None)
    }

    fn callback_type_source(
        &mut self,
        type_source: &ast::TypeSource,
    ) -> Result<Option<ast::TypeSource>, AiScriptSyntaxError> {
        match type_source {
            ast::TypeSource::NamedTypeSource(named_type_source)
                if RESERVED_WORD.contains(&named_type_source.name.as_str()) =>
            {
                Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::ReservedWord(named_type_source.name.to_string()),
                    pos: named_type_source.loc.start.clone(),
                })?
            }
            ast::TypeSource::FnTypeSource(fn_type_source) => {
                if let Some(type_params) = &fn_type_source.type_params {
                    for ast::TypeParam { name, .. } in type_params {
                        if RESERVED_WORD.contains(&name.as_str()) {
                            Err(AiScriptSyntaxError {
                                kind: AiScriptSyntaxErrorKind::ReservedWord(name.to_string()),
                                pos: fn_type_source.loc.start.clone(),
                            })?
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn callback_attribute(
        &mut self,
        attribute: &ast::Attribute,
    ) -> Result<Option<ast::Attribute>, AiScriptSyntaxError> {
        if RESERVED_WORD.contains(&attribute.name.as_str()) {
            Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord(attribute.name.to_string()),
                pos: attribute.loc.start.clone(),
            })?
        }
        Ok(None)
    }
}

struct DestKeywordValidator;

impl Visitor for DestKeywordValidator {
    fn callback_expression(
        &mut self,
        expression: &ast::Expression,
    ) -> Result<Option<ast::Expression>, AiScriptSyntaxError> {
        match expression {
            ast::Expression::Null(null) => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord("null".to_string()),
                pos: null.loc.start.clone(),
            })?,
            ast::Expression::Bool(bool) => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReservedWord(bool.value.to_string()),
                pos: bool.loc.start.clone(),
            })?,
            _ => {}
        }
        Ok(None)
    }
}

pub fn validate_keyword(nodes: Vec<ast::Node>) -> Result<Vec<ast::Node>, AiScriptSyntaxError> {
    for node in &nodes {
        KeywordValidator.visit_node(node)?;
    }
    Ok(nodes)
}
