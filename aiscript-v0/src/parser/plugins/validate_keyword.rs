use crate::{
    error::{AiScriptError, AiScriptSyntaxError},
    parser::{node as cst, visit::Visitor},
};

const RESERVED_WORD: [&str; 29] = [
    "null",
    "true",
    "false",
    "each",
    "for",
    "loop",
    "break",
    "continue",
    "match",
    "if",
    "elif",
    "else",
    "return",
    "eval",
    "var",
    "let",
    "exists",
    // future
    "fn",
    "namespace",
    "meta",
    "attr",
    "attribute",
    "static",
    "class",
    "struct",
    "module",
    "while",
    "import",
    "export",
    // "const",
    // "def",
    // "func",
    // "function",
    // "ref",
    // "out",
];

#[derive(Debug, PartialEq, Clone)]
struct KeywordValidator;

impl Visitor for KeywordValidator {
    fn callback_namespace(
        &self,
        namespace: cst::Namespace,
    ) -> Result<cst::Namespace, AiScriptError> {
        if RESERVED_WORD.contains(&namespace.name.as_str()) {
            Err(AiScriptSyntaxError::reserved_word(namespace.name))?
        } else {
            Ok(namespace)
        }
    }

    fn callback_meta(&self, meta: cst::Meta) -> Result<cst::Meta, AiScriptError> {
        match meta {
            cst::Meta {
                name: Some(name), ..
            } if RESERVED_WORD.contains(&name.as_str()) => {
                Err(AiScriptSyntaxError::reserved_word(name))?
            }
            _ => Ok(meta),
        }
    }

    fn callback_statement(
        &self,
        statement: cst::Statement,
    ) -> Result<cst::Statement, AiScriptError> {
        match &statement {
            cst::Statement::Definition(definition) => Some(&definition.name),
            cst::Statement::Attribute(attribute) => Some(&attribute.name),
            _ => None,
        }
        .map(|name| {
            if RESERVED_WORD.contains(&name.as_str()) {
                Err(AiScriptSyntaxError::reserved_word(name))?
            } else {
                Ok(())
            }
        })
        .unwrap_or(Ok(()))
        .map(|_| statement)
    }

    fn callback_expression(
        &self,
        expression: cst::Expression,
    ) -> Result<cst::Expression, AiScriptError> {
        match &expression {
            cst::Expression::Identifier(identifier) => {
                if RESERVED_WORD.contains(&identifier.name.as_str()) {
                    Err(AiScriptSyntaxError::reserved_word(&identifier.name))?
                } else {
                    Ok(expression)
                }
            }
            cst::Expression::Fn(fn_) => {
                for arg in &fn_.args {
                    if RESERVED_WORD.contains(&arg.name.as_str()) {
                        Err(AiScriptSyntaxError::reserved_word(&arg.name))?
                    }
                }
                Ok(expression)
            }
            _ => Ok(expression),
        }
    }

    fn callback_chain_member(
        &self,
        chain_member: cst::ChainMember,
    ) -> Result<cst::ChainMember, AiScriptError> {
        match &chain_member {
            cst::ChainMember::PropChain(cst::PropChain { name, .. }) => {
                if RESERVED_WORD.contains(&name.as_str()) {
                    Err(AiScriptSyntaxError::reserved_word(name))?
                } else {
                    Ok(chain_member)
                }
            }
            _ => Ok(chain_member),
        }
    }
}

pub fn validate_keyword(
    nodes: impl IntoIterator<Item = cst::Node>,
) -> Result<Vec<cst::Node>, AiScriptError> {
    nodes
        .into_iter()
        .map(|node| KeywordValidator.visit_node(node))
        .collect()
}
