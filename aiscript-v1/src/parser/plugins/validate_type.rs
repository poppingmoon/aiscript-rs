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
    fn visit_fn(&mut self, fn_: ast::Fn) -> Result<ast::Fn, AiScriptSyntaxError> {
        if let Some(type_params) = &fn_.type_params {
            validate_type_params(type_params)?;
            FnTypeValidator {
                type_params: vec![type_params.clone()],
            }
            .visit_fn(fn_)
        } else {
            Ok(ast::Fn {
                params: fn_
                    .params
                    .into_iter()
                    .map(|param| {
                        Ok(ast::Param {
                            dest: self.visit_expression(param.dest)?,
                            default: param
                                .default
                                .map(|default| self.visit_expression(default))
                                .map_or(Ok(None), |r| r.map(Some))?,
                            arg_type: param
                                .arg_type
                                .map(|arg_type| self.visit_type_source(arg_type))
                                .map_or(Ok(None), |r| r.map(Some))?,
                            ..param
                        })
                    })
                    .collect::<Result<Vec<ast::Param>, AiScriptSyntaxError>>()?,
                ret_type: fn_
                    .ret_type
                    .map(|ret_type| self.visit_type_source(ret_type))
                    .map_or(Ok(None), |r| r.map(Some))?,
                children: fn_
                    .children
                    .into_iter()
                    .map(|child| match child {
                        ast::StatementOrExpression::Statement(statement) => self
                            .visit_statement(statement)
                            .map(ast::StatementOrExpression::Statement),
                        ast::StatementOrExpression::Expression(expression) => self
                            .visit_expression(expression)
                            .map(ast::StatementOrExpression::Expression),
                    })
                    .collect::<Result<Vec<ast::StatementOrExpression>, AiScriptSyntaxError>>()?,
                ..fn_
            })
        }
    }

    fn visit_type_source(
        &mut self,
        type_source: ast::TypeSource,
    ) -> Result<ast::TypeSource, AiScriptSyntaxError> {
        get_type_by_source(type_source.clone(), &[])?;
        Ok(type_source)
    }
}

struct FnTypeValidator {
    pub type_params: Vec<Vec<ast::TypeParam>>,
}

impl Visitor for FnTypeValidator {
    fn visit_fn(&mut self, fn_: ast::Fn) -> Result<ast::Fn, AiScriptSyntaxError> {
        if let Some(type_params) = &fn_.type_params {
            validate_type_params(type_params)?;
            self.type_params.push(type_params.clone());
        }
        let fn_ = ast::Fn {
            params: fn_
                .params
                .into_iter()
                .map(|param| {
                    Ok(ast::Param {
                        dest: self.visit_expression(param.dest)?,
                        default: param
                            .default
                            .map(|default| self.visit_expression(default))
                            .map_or(Ok(None), |r| r.map(Some))?,
                        arg_type: param
                            .arg_type
                            .map(|arg_type| self.visit_type_source(arg_type))
                            .map_or(Ok(None), |r| r.map(Some))?,
                        ..param
                    })
                })
                .collect::<Result<Vec<ast::Param>, AiScriptSyntaxError>>()?,
            ret_type: fn_
                .ret_type
                .map(|ret_type| self.visit_type_source(ret_type))
                .map_or(Ok(None), |r| r.map(Some))?,
            children: fn_
                .children
                .into_iter()
                .map(|child| match child {
                    ast::StatementOrExpression::Statement(statement) => self
                        .visit_statement(statement)
                        .map(ast::StatementOrExpression::Statement),
                    ast::StatementOrExpression::Expression(expression) => self
                        .visit_expression(expression)
                        .map(ast::StatementOrExpression::Expression),
                })
                .collect::<Result<Vec<ast::StatementOrExpression>, AiScriptSyntaxError>>()?,
            ..fn_
        };
        if fn_.type_params.is_some() {
            self.type_params.pop();
        }
        Ok(fn_)
    }

    fn visit_type_source(
        &mut self,
        type_source: ast::TypeSource,
    ) -> Result<ast::TypeSource, AiScriptSyntaxError> {
        get_type_by_source(
            type_source.clone(),
            &self
                .type_params
                .iter()
                .flatten()
                .collect::<Vec<&ast::TypeParam>>(),
        )?;
        Ok(type_source)
    }
}

pub fn validate_type(
    nodes: impl IntoIterator<Item = ast::Node>,
) -> Result<Vec<ast::Node>, AiScriptSyntaxError> {
    nodes
        .into_iter()
        .map(|node| TypeValidator.visit_node(node))
        .collect()
}
