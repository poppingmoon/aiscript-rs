use std::collections::HashSet;

use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node as ast,
    parser::visit::Visitor,
    r#type::get_type_by_source,
};

fn validate_type_params(type_params: &[ast::TypeParam]) -> Result<(), AiScriptSyntaxError> {
    let mut type_param_names = HashSet::new();
    for type_param in type_params {
        if type_param_names.contains(&type_param.name) {
            Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::DuplicateTypeParameterName(
                    type_param.name.to_string(),
                ),
                pos: type_param.loc.start.clone(),
            })?
        } else {
            type_param_names.insert(type_param.name.to_string());
        }
    }
    Ok(())
}

struct TypeValidator;

impl Visitor for TypeValidator {
    fn visit_fn(&mut self, fn_: &ast::Fn) -> Result<Option<ast::Fn>, AiScriptSyntaxError> {
        if let Some(type_params) = &fn_.type_params {
            validate_type_params(type_params)?;
            FnTypeValidator {
                type_params: vec![type_params.clone()],
            }
            .visit_fn(fn_)?;
        } else {
            for ast::Param {
                dest,
                default,
                arg_type,
                ..
            } in &fn_.params
            {
                self.visit_expression(dest)?;
                if let Some(default) = default {
                    self.visit_expression(default)?;
                }
                if let Some(arg_type) = arg_type {
                    self.visit_type_source(arg_type)?;
                }
            }
            if let Some(ret_type) = &fn_.ret_type {
                self.visit_type_source(ret_type)?;
            }
            for child in &fn_.children {
                match child {
                    ast::StatementOrExpression::Statement(statement) => {
                        self.visit_statement(statement)?;
                    }
                    ast::StatementOrExpression::Expression(expression) => {
                        self.visit_expression(expression)?;
                    }
                }
            }
        }
        Ok(None)
    }

    fn visit_type_source(
        &mut self,
        type_source: &ast::TypeSource,
    ) -> Result<Option<ast::TypeSource>, AiScriptSyntaxError> {
        get_type_by_source(type_source, &[])?;
        Ok(None)
    }
}

struct FnTypeValidator {
    pub type_params: Vec<Vec<ast::TypeParam>>,
}

impl Visitor for FnTypeValidator {
    fn visit_fn(&mut self, fn_: &ast::Fn) -> Result<Option<ast::Fn>, AiScriptSyntaxError> {
        if let Some(type_params) = &fn_.type_params {
            validate_type_params(type_params)?;
            self.type_params.push(type_params.clone());
        }
        for ast::Param {
            dest,
            default,
            arg_type,
            ..
        } in &fn_.params
        {
            self.visit_expression(dest)?;
            if let Some(default) = default {
                self.visit_expression(default)?;
            }
            if let Some(arg_type) = arg_type {
                self.visit_type_source(arg_type)?;
            }
        }
        if let Some(ret_type) = &fn_.ret_type {
            self.visit_type_source(ret_type)?;
        }
        for child in &fn_.children {
            match child {
                ast::StatementOrExpression::Statement(statement) => {
                    self.visit_statement(statement)?;
                }
                ast::StatementOrExpression::Expression(expression) => {
                    self.visit_expression(expression)?;
                }
            }
        }
        if fn_.type_params.is_some() {
            self.type_params.pop();
        }
        Ok(None)
    }

    fn visit_type_source(
        &mut self,
        type_source: &ast::TypeSource,
    ) -> Result<Option<ast::TypeSource>, AiScriptSyntaxError> {
        get_type_by_source(
            type_source,
            &self
                .type_params
                .iter()
                .flatten()
                .collect::<Vec<&ast::TypeParam>>(),
        )?;
        Ok(None)
    }
}

pub fn validate_type(nodes: Vec<ast::Node>) -> Result<Vec<ast::Node>, AiScriptSyntaxError> {
    for node in &nodes {
        TypeValidator.visit_node(node)?;
    }
    Ok(nodes)
}
