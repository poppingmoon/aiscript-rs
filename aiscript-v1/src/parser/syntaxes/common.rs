use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node as ast,
    parser::token::{Token, Tokens, TokensExt},
};

use super::{expressions::parse_expr, statements::parse_statement, types::parse_type};

pub fn parse_dest(s: &mut Tokens) -> Result<ast::Expression, AiScriptSyntaxError> {
    if let Token::Identifier { value, .. } = s.peek() {
        let name = value.to_string();
        let name_start_pos = s.pop_token().into_pos();
        Ok(ast::Expression::Identifier(
            ast::Identifier {
                loc: ast::Loc {
                    start: name_start_pos,
                    end: s.peek().pos().clone(),
                },
                name,
            }
            .into(),
        ))
    } else {
        parse_expr(s)
    }
}

pub fn parse_params(s: &mut Tokens) -> Result<Vec<ast::Param>, AiScriptSyntaxError> {
    match s.pop_token() {
        Token::OpenParen { .. } => {}
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let mut items = Vec::new();

    loop {
        match s.peek() {
            Token::NewLine { .. } => {
                s.pop_token();
                continue;
            }
            Token::CloseParen { .. } => {
                s.pop_token();
                break;
            }
            _ => {}
        }

        let dest = parse_dest(s)?;

        let (optional, default_expr) = match s.peek() {
            Token::Question { .. } => {
                s.pop_token();
                (true, None)
            }
            Token::Eq { .. } => {
                s.pop_token();
                (false, Some(parse_expr(s)?))
            }
            _ => (false, None),
        };
        let type_ = if let Token::Colon { .. } = s.peek() {
            s.pop_token();
            Some(parse_type(s)?)
        } else {
            None
        };

        items.push(ast::Param {
            dest,
            optional,
            default: default_expr,
            arg_type: type_,
        });

        match s.pop_token() {
            Token::NewLine { .. } | Token::Comma { .. } => {}
            Token::CloseParen { .. } => break,
            Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                pos,
            })?,
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::SeparatorExpected,
                pos: token.into_pos(),
            })?,
        }
    }

    Ok(items)
}

pub fn parse_block(s: &mut Tokens) -> Result<Vec<ast::StatementOrExpression>, AiScriptSyntaxError> {
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

    if let Token::NewLine { .. } = s.peek() {
        s.pop_token();
    }

    if let Token::CloseBrace { .. } = s.peek() {
        s.pop_token();
        return Ok(Vec::new());
    }

    let mut steps = Vec::new();

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
            _ => {}
        }

        steps.push(parse_statement(s)?);

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

    Ok(steps)
}

pub fn parse_label(s: &mut Tokens) -> Result<String, AiScriptSyntaxError> {
    match s.pop_token() {
        Token::Sharp { .. } => {}
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
        Token::Identifier {
            pos,
            has_left_spacing: true,
            ..
        } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::SpaceInLabel,
            pos,
        }),
        Token::Identifier { value, .. } => Ok(value.to_string()),
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        }),
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        }),
    }
}
