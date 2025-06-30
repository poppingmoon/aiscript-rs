use crate::{
    error::AiScriptError,
    parser::{node as cst, visit::Visitor},
    r#type::Type,
};

#[derive(Debug, PartialEq, Clone)]
struct TypeValidator;

impl Visitor for TypeValidator {
    fn callback_statement(
        &self,
        statement: cst::Statement,
    ) -> Result<cst::Statement, crate::error::AiScriptError> {
        if let cst::Statement::Definition(definition) = &statement
            && let Some(var_type) = definition.var_type.as_ref()
        {
            Type::try_from(var_type.clone())?;
        };
        Ok(statement)
    }

    fn callback_expression(
        &self,
        expression: cst::Expression,
    ) -> Result<cst::Expression, crate::error::AiScriptError> {
        if let cst::Expression::Fn(fn_) = &expression
            && let Some(ret_type) = fn_.ret_type.as_ref()
        {
            Type::try_from(ret_type.clone())?;
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
