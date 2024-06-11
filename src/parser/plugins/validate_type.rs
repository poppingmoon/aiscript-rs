use crate::{
    error::AiScriptError,
    parser::{node as cst, visit::Visitor},
    r#type::get_type_by_source,
};

#[derive(Debug, PartialEq, Clone)]
struct TypeValidator;

impl Visitor for TypeValidator {
    fn callback_statement(
        &self,
        statement: cst::Statement,
    ) -> Result<cst::Statement, crate::error::AiScriptError> {
        if let cst::Statement::Definition(cst::Definition {
            var_type: Some(var_type),
            ..
        }) = &statement
        {
            get_type_by_source(var_type.clone())?;
        };
        Ok(statement)
    }

    fn callback_expression(
        &self,
        expression: cst::Expression,
    ) -> Result<cst::Expression, crate::error::AiScriptError> {
        if let cst::Expression::Fn(cst::Fn_ {
            ret_type: Some(ret_type),
            ..
        }) = &expression
        {
            get_type_by_source(ret_type.clone())?;
        };
        Ok(expression)
    }
}

pub fn validate_type(
    nodes: impl IntoIterator<Item = cst::Node>,
) -> Result<Vec<cst::Node>, AiScriptError> {
    nodes
        .into_iter()
        .map(|node| TypeValidator.visit_node(node))
        .collect()
}
