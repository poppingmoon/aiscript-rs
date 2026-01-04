use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node as ast,
    parser::token::{Token, Tokens, TokensExt},
};

pub fn parse_type(s: &mut Tokens) -> Result<ast::TypeSource, AiScriptSyntaxError> {
    parse_union_type(s)
}

pub fn parse_type_params(s: &mut Tokens) -> Result<Vec<ast::TypeParam>, AiScriptSyntaxError> {
    match s.pop_token() {
        Token::Lt { .. } => {}
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

    let mut items = vec![parse_type_param(s)?];
    loop {
        match s.pop_token() {
            Token::NewLine { .. } => {
                if let Token::Gt { .. } = s.peek() {
                    s.pop_token();
                    break;
                }
            }
            Token::Comma { .. } => match s.peek() {
                Token::NewLine { .. } => {
                    s.pop_token();
                    if let Token::Gt { .. } = s.peek() {
                        s.pop_token();
                        break;
                    }
                }
                Token::Gt { .. } => {
                    s.pop_token();
                    break;
                }
                _ => {}
            },
            Token::Gt { .. } => break,
            Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                pos,
            })?,
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                pos: token.into_pos(),
            })?,
        }
        let item = parse_type_param(s)?;
        items.push(item);
    }

    Ok(items)
}

pub fn parse_type_param(s: &mut Tokens) -> Result<ast::TypeParam, AiScriptSyntaxError> {
    Ok(match s.pop_token() {
        Token::Identifier { pos, value, .. } => ast::TypeParam {
            loc: ast::Loc {
                start: pos,
                end: s.peek().pos().clone(),
            },
            name: value.to_string(),
        },
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    })
}

fn parse_union_type(s: &mut Tokens) -> Result<ast::TypeSource, AiScriptSyntaxError> {
    let first = parse_union_type_inner(s)?;

    Ok(if let Token::Or { .. } = s.peek() {
        let start_pos = match &first {
            ast::TypeSource::NamedTypeSource(ast::NamedTypeSource { loc, .. })
            | ast::TypeSource::FnTypeSource(ast::FnTypeSource { loc, .. })
            | ast::TypeSource::UnionTypeSource(ast::UnionTypeSource { loc, .. }) => {
                loc.start.clone()
            }
        };

        let mut inners = vec![first];
        loop {
            s.pop_token();
            inners.push(parse_union_type_inner(s)?);
            let Token::Or { .. } = s.peek() else {
                break ast::TypeSource::UnionTypeSource(ast::UnionTypeSource {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    inners,
                });
            };
        }
    } else {
        first
    })
}

fn parse_union_type_inner(s: &mut Tokens) -> Result<ast::TypeSource, AiScriptSyntaxError> {
    Ok(match s.pop_token() {
        Token::At { pos, .. } => ast::TypeSource::FnTypeSource(parse_fn_type(pos, s)?),
        Token::Identifier { pos, value, .. } => {
            ast::TypeSource::NamedTypeSource(parse_named_type(pos, value.to_string(), s)?)
        }
        Token::NullKeyword { pos, .. } => {
            ast::TypeSource::NamedTypeSource(parse_named_type(pos, "null".to_string(), s)?)
        }
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    })
}

fn parse_fn_type(
    start_pos: ast::Pos,
    s: &mut Tokens,
) -> Result<ast::FnTypeSource, AiScriptSyntaxError> {
    let type_params = if let Token::Lt { .. } = s.peek() {
        Some(parse_type_params(s)?)
    } else {
        None
    };

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

    let mut params = Vec::new();
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

        let type_ = parse_type(s)?;
        params.push(type_);

        match s.pop_token() {
            Token::Comma { .. } => {
                s.pop_token();
            }
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

    match s.pop_token() {
        Token::Arrow { .. } => {}
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let result_type = parse_type(s)?;

    Ok(ast::FnTypeSource {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        type_params,
        params,
        result: result_type.into(),
    })
}

fn parse_named_type(
    start_pos: ast::Pos,
    name: String,
    s: &mut Tokens,
) -> Result<ast::NamedTypeSource, AiScriptSyntaxError> {
    let inner = if let Token::Lt { .. } = s.peek() {
        s.pop_token();
        let inner = parse_type(s)?;
        match s.pop_token() {
            Token::Gt { .. } => {}
            Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                pos,
            })?,
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                pos: token.into_pos(),
            })?,
        };
        Some(inner.into())
    } else {
        None
    };

    Ok(ast::NamedTypeSource {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        name,
        inner,
    })
}
