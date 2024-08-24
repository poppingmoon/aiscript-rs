use crate::{error::AiScriptError, errors::AiScriptSyntaxError, parser::node as cst};

pub fn set_attribute(
    nodes: impl IntoIterator<Item = cst::Node>,
) -> Result<Vec<cst::Node>, AiScriptError> {
    let mut result = Vec::<cst::Node>::new();
    let mut statements = Vec::<cst::StatementOrExpression>::new();

    for node in nodes {
        match node {
            cst::Node::Statement(statement) => {
                statements.push(cst::StatementOrExpression::Statement(statement))
            }
            cst::Node::Expression(expression) => {
                statements.push(cst::StatementOrExpression::Expression(expression))
            }
            _ => {
                if !statements.is_empty() {
                    let mut nodes = set_attribute_statement_or_expression(statements.clone())?
                        .into_iter()
                        .map(Into::into)
                        .collect::<Vec<cst::Node>>();
                    result.append(&mut nodes);
                    statements.clear();
                }
                result.push(node);
            }
        };
    }
    if !statements.is_empty() {
        let mut nodes = set_attribute_statement_or_expression(statements.clone())?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<cst::Node>>();
        result.append(&mut nodes);
    }

    Ok(result)
}

fn set_attribute_statement_or_expression(
    nodes: impl IntoIterator<Item = cst::StatementOrExpression>,
) -> Result<Vec<cst::StatementOrExpression>, AiScriptError> {
    let mut result = Vec::<cst::StatementOrExpression>::new();
    let mut stocked_attrs = Vec::<cst::Attribute>::new();

    for node in nodes {
        match node {
            cst::StatementOrExpression::Statement(cst::Statement::Attribute(attribute)) => {
                stocked_attrs.push(attribute);
            }
            cst::StatementOrExpression::Statement(cst::Statement::Definition(definition)) => {
                let mut attr = definition.attr.unwrap_or_default();
                attr.extend(stocked_attrs.splice(.., []));
                let definition = cst::Definition {
                    attr: Some(attr),
                    expr: if let cst::Expression::Fn(fn_) = definition.expr {
                        cst::Expression::Fn(cst::Fn_ {
                            children: set_attribute_statement_or_expression(fn_.children)?,
                            ..fn_
                        })
                    } else {
                        definition.expr
                    },
                    ..definition
                };
                result.push(cst::StatementOrExpression::Statement(
                    cst::Statement::Definition(definition),
                ));
            }
            _ => {
                if !stocked_attrs.is_empty() {
                    Err(AiScriptSyntaxError::Attribute)?
                }
                let node = match node {
                    cst::StatementOrExpression::Expression(expression) => {
                        cst::StatementOrExpression::Expression(match expression {
                            cst::Expression::Fn(fn_) => cst::Expression::Fn(cst::Fn_ {
                                children: set_attribute_statement_or_expression(fn_.children)?,
                                ..fn_
                            }),
                            cst::Expression::Block(block) => cst::Expression::Block(cst::Block {
                                statements: set_attribute_statement_or_expression(
                                    block.statements,
                                )?,
                                ..block
                            }),
                            _ => expression,
                        })
                    }
                    _ => node,
                };
                result.push(node);
            }
        }
    }
    if !stocked_attrs.is_empty() {
        Err(AiScriptSyntaxError::Attribute)?
    }

    Ok(result)
}
