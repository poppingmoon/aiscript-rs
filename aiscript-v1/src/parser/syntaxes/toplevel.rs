use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node as ast,
    parser::token::{Token, Tokens, TokensExt},
};

use super::{
    expressions::parse_static_expr,
    statements::{
        parse_fn_def_statement, parse_let_def_statement, parse_statement,
        parse_statement_with_attr, parse_var_def_statement,
    },
};

pub fn parse_top_level(s: &mut Tokens) -> Result<Vec<ast::Node>, AiScriptSyntaxError> {
    let mut nodes = Vec::new();
    loop {
        match s.peek() {
            Token::NewLine { .. } => {
                s.pop_token();
                continue;
            }
            Token::Eof { .. } => break,
            Token::Colon2 { .. } => {
                nodes.push(ast::Node::Namespace(parse_namespace(s)?.into()));
            }
            Token::Sharp3 { .. } => {
                nodes.push(ast::Node::Meta(parse_meta(s)?.into()));
            }
            _ => {
                nodes.push(parse_statement(s)?.into());
            }
        }

        match s.pop_token() {
            Token::NewLine { .. } | Token::SemiColon { .. } => {
                while let Token::NewLine { .. } | Token::SemiColon { .. } = s.peek() {
                    s.pop_token();
                }
            }
            Token::Eof { .. } => break,
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::MultipleStatements,
                pos: token.into_pos(),
            })?,
        }
    }

    Ok(nodes)
}

fn parse_namespace(s: &mut Tokens) -> Result<ast::Namespace, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::Colon2 { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let name = match s.pop_token() {
        Token::Identifier { value, .. } => value.to_string(),
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    match s.pop_token() {
        Token::OpenBrace { .. } => {}
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let mut members = Vec::new();
    loop {
        match s.peek() {
            Token::NewLine { .. } => {
                s.pop_token();
                continue;
            }
            Token::CloseBrace { .. } => {
                s.pop_token();
                break;
            }
            Token::VarKeyword { .. } => members.push(ast::DefinitionOrNamespace::Definition(
                parse_var_def_statement(s)?.into(),
            )),
            Token::LetKeyword { .. } => members.push(ast::DefinitionOrNamespace::Definition(
                parse_let_def_statement(s)?.into(),
            )),
            Token::At { .. } => members.push(ast::DefinitionOrNamespace::Definition(
                parse_fn_def_statement(s)?.into(),
            )),
            Token::Colon2 { .. } => members.push(ast::DefinitionOrNamespace::Namespace(
                parse_namespace(s)?.into(),
            )),
            Token::OpenSharpBracket { .. } => members.push(ast::DefinitionOrNamespace::Definition(
                parse_statement_with_attr(s)?.into(),
            )),
            _ => {}
        }

        match s.pop_token() {
            Token::NewLine { .. } | Token::SemiColon { .. } => {
                while let Token::NewLine { .. } | Token::SemiColon { .. } = s.peek() {
                    s.pop_token();
                }
            }
            Token::CloseBrace { .. } => break,
            Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                pos,
            })?,
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::MultipleStatements,
                pos: token.into_pos(),
            })?,
        }
    }

    Ok(ast::Namespace {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        name,
        members,
    })
}

fn parse_meta(s: &mut Tokens) -> Result<ast::Meta, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::Sharp3 { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let name = if let Token::Identifier { value, .. } = s.peek() {
        let value = value.to_string();
        s.pop_token();
        Some(value)
    } else {
        None
    };

    let value = parse_static_expr(s)?;

    Ok(ast::Meta {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        name,
        value,
    })
}
