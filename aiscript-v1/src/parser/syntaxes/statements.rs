use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node as ast,
    parser::token::{Token, Tokens, TokensExt},
};

use super::{
    common::{parse_block, parse_dest, parse_label, parse_params},
    expressions::parse_expr,
    types::{parse_type, parse_type_params},
};

pub fn parse_statement(s: &mut Tokens) -> Result<ast::StatementOrExpression, AiScriptSyntaxError> {
    Ok(match s.peek() {
        Token::VarKeyword { .. } => {
            ast::Statement::Definition(parse_var_def_statement(s)?.into()).into()
        }
        Token::LetKeyword { .. } => {
            ast::Statement::Definition(parse_let_def_statement(s)?.into()).into()
        }
        Token::At { .. } if matches!(s.lookahead(1), Token::Identifier { .. }) => {
            ast::Statement::Definition(parse_fn_def_statement(s)?.into()).into()
        }
        Token::Out { .. } => ast::Expression::Call(parse_out(s)?.into()).into(),
        Token::ReturnKeyword { .. } => ast::Statement::Return(parse_return(s)?.into()).into(),
        Token::OpenSharpBracket { .. } => {
            ast::Statement::Definition(parse_statement_with_attr(s)?.into()).into()
        }
        Token::Sharp { .. } => parse_statement_with_label(s)?,
        Token::EachKeyword { .. } => ast::Statement::Each(parse_each(s)?.into()).into(),
        Token::ForKeyword { .. } => parse_for(s)?.into(),
        Token::LoopKeyword { .. } => ast::Statement::Loop(parse_loop(s)?.into()).into(),
        Token::DoKeyword { .. } => ast::Statement::Loop(parse_do_while(s)?.into()).into(),
        Token::WhileKeyword { .. } => ast::Statement::Loop(parse_while(s)?.into()).into(),
        Token::BreakKeyword { .. } => ast::Statement::Break(parse_break(s)?.into()).into(),
        Token::ContinueKeyword { .. } => ast::Statement::Continue(parse_continue(s)?.into()).into(),
        _ => {
            let expr = parse_expr(s)?;
            match s.peek() {
                Token::Eq { .. } => {
                    let dest = expr;
                    let start_pos = s.pop_token().into_pos();
                    let expr = parse_expr(s)?;
                    ast::Statement::Assign(
                        ast::Assign {
                            loc: ast::Loc {
                                start: start_pos,
                                end: s.peek().pos().clone(),
                            },
                            dest,
                            expr,
                        }
                        .into(),
                    )
                    .into()
                }
                Token::PlusEq { .. } => {
                    let dest = expr;
                    let start_pos = s.pop_token().into_pos();
                    let expr = parse_expr(s)?;
                    ast::Statement::AddAssign(
                        ast::AddAssign {
                            loc: ast::Loc {
                                start: start_pos,
                                end: s.peek().pos().clone(),
                            },
                            dest,
                            expr,
                        }
                        .into(),
                    )
                    .into()
                }
                Token::MinusEq { .. } => {
                    let dest = expr;
                    let start_pos = s.pop_token().into_pos();
                    let expr = parse_expr(s)?;
                    ast::Statement::SubAssign(
                        ast::SubAssign {
                            loc: ast::Loc {
                                start: start_pos,
                                end: s.peek().pos().clone(),
                            },
                            dest,
                            expr,
                        }
                        .into(),
                    )
                    .into()
                }
                _ => expr.into(),
            }
        }
    })
}

pub fn parse_block_or_statement(
    s: &mut Tokens,
) -> Result<ast::StatementOrExpression, AiScriptSyntaxError> {
    if let Token::OpenBrace { pos, .. } = s.peek() {
        let start_pos = pos.clone();
        let statements = parse_block(s)?;
        Ok(ast::Expression::Block(
            ast::Block {
                loc: ast::Loc {
                    start: start_pos,
                    end: s.peek().pos().clone(),
                },
                label: None,
                statements,
            }
            .into(),
        )
        .into())
    } else {
        parse_statement(s)
    }
}

pub fn parse_var_def_statement(s: &mut Tokens) -> Result<ast::Definition, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::VarKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };
    parse_var_def(start_pos, s, true)
}

pub fn parse_let_def_statement(s: &mut Tokens) -> Result<ast::Definition, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::LetKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };
    parse_var_def(start_pos, s, false)
}

fn parse_var_def(
    start_pos: ast::Pos,
    s: &mut Tokens,
    mut_: bool,
) -> Result<ast::Definition, AiScriptSyntaxError> {
    let dest = parse_dest(s)?;

    let type_ = if let Token::Colon { .. } = s.peek() {
        s.pop_token();
        Some(parse_type(s)?)
    } else {
        None
    };

    match s.pop_token() {
        Token::Eq { .. } => {}
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

    let expr = parse_expr(s)?;

    Ok(ast::Definition {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        dest,
        var_type: type_,
        expr,
        mut_,
        attr: None,
    })
}

pub fn parse_fn_def_statement(s: &mut Tokens) -> Result<ast::Definition, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::At { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let dest = match s.pop_token() {
        Token::Identifier { pos, value, .. } => ast::Identifier {
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
    };

    let type_params = if let Token::Lt { .. } = s.peek() {
        Some(parse_type_params(s)?)
    } else {
        None
    };

    let params = parse_params(s)?;

    let type_ = if let Token::Colon { .. } = s.peek() {
        s.pop_token();
        Some(parse_type(s)?)
    } else {
        None
    };

    let body = parse_block(s)?;

    let loc = ast::Loc {
        start: start_pos,
        end: s.peek().pos().clone(),
    };

    Ok(ast::Definition {
        loc: loc.clone(),
        dest: ast::Expression::Identifier(dest.into()),
        var_type: None,
        expr: ast::Expression::Fn(
            ast::Fn {
                loc,
                type_params,
                params,
                ret_type: type_,
                children: body,
            }
            .into(),
        ),
        mut_: false,
        attr: None,
    })
}

fn parse_out(s: &mut Tokens) -> Result<ast::Call, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::Out { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };
    let expr = parse_expr(s)?;

    Ok(ast::Call {
        loc: ast::Loc {
            start: start_pos.clone(),
            end: s.peek().pos().clone(),
        },
        target: ast::Expression::Identifier(
            ast::Identifier {
                loc: ast::Loc {
                    start: start_pos.clone(),
                    end: start_pos,
                },
                name: "print".to_string(),
            }
            .into(),
        )
        .into(),
        args: vec![expr],
    })
}

pub fn parse_statement_with_label(
    s: &mut Tokens,
) -> Result<ast::StatementOrExpression, AiScriptSyntaxError> {
    let label = parse_label(s)?;
    match s.pop_token() {
        Token::Colon { .. } => {}
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let statement = parse_statement(s)?;
    Ok(match statement {
        ast::StatementOrExpression::Expression(expression) => {
            ast::StatementOrExpression::Expression(match expression {
                ast::Expression::If(if_) => ast::Expression::If(
                    ast::If {
                        label: Some(label),
                        ..*if_
                    }
                    .into(),
                ),
                ast::Expression::Match(match_) => ast::Expression::Match(
                    ast::Match {
                        label: Some(label),
                        ..*match_
                    }
                    .into(),
                ),
                ast::Expression::Block(block) => ast::Expression::Block(
                    ast::Block {
                        label: Some(label),
                        ..*block
                    }
                    .into(),
                ),
                _ => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::InvalidStatementWithLabel,
                    pos: expression.into_loc().start,
                })?,
            })
        }
        ast::StatementOrExpression::Statement(statement) => {
            ast::StatementOrExpression::Statement(match statement {
                ast::Statement::Each(each) => ast::Statement::Each(
                    ast::Each {
                        label: Some(label),
                        ..*each
                    }
                    .into(),
                ),
                ast::Statement::For(for_) => ast::Statement::For(
                    ast::For {
                        label: Some(label),
                        ..*for_
                    }
                    .into(),
                ),
                ast::Statement::ForLet(for_let) => ast::Statement::ForLet(
                    ast::ForLet {
                        label: Some(label),
                        ..*for_let
                    }
                    .into(),
                ),
                ast::Statement::Loop(loop_) => ast::Statement::Loop(
                    ast::Loop {
                        label: Some(label),
                        ..*loop_
                    }
                    .into(),
                ),
                _ => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::InvalidStatementWithLabel,
                    pos: statement.into_loc().start,
                })?,
            })
        }
    })
}

fn parse_each(s: &mut Tokens) -> Result<ast::Each, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::EachKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let has_paren = if let Token::OpenParen { .. } = s.peek() {
        s.pop_token();
        true
    } else {
        false
    };

    match s.pop_token() {
        Token::LetKeyword { .. } => {}
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let dest = parse_dest(s)?;

    match s.pop_token() {
        Token::Comma { .. } => {}
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::SeparatorExpected,
            pos: token.into_pos(),
        })?,
    }

    let items = parse_expr(s)?;

    if has_paren {
        match s.pop_token() {
            Token::CloseParen { .. } => {}
            Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                pos,
            })?,
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                pos: token.into_pos(),
            })?,
        };
    }

    let body = parse_block_or_statement(s)?;

    Ok(ast::Each {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        label: None,
        var: dest,
        items,
        for_: body.into(),
    })
}

fn parse_for(s: &mut Tokens) -> Result<ast::Statement, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::ForKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let has_paren = if let Token::OpenParen { .. } = s.peek() {
        s.pop_token();
        true
    } else {
        false
    };

    Ok(if let Token::LetKeyword { .. } = s.peek() {
        s.pop_token();

        let (ident_pos, name) = match s.pop_token() {
            Token::Identifier { pos, value, .. } => (pos, value),
            Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                pos,
            })?,
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                pos: token.into_pos(),
            })?,
        };

        let from = if let Token::Eq { .. } = s.peek() {
            s.pop_token();
            parse_expr(s)?
        } else {
            ast::Expression::Num(
                ast::Num {
                    loc: ast::Loc {
                        start: ident_pos.clone(),
                        end: ident_pos,
                    },
                    value: 0_f64,
                }
                .into(),
            )
        };

        match s.pop_token() {
            Token::Comma { .. } => {}
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::SeparatorExpected,
                pos: token.into_pos(),
            })?,
        }

        let to = parse_expr(s)?;

        if has_paren {
            match s.pop_token() {
                Token::CloseParen { .. } => {}
                Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                    pos,
                })?,
                token => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                    pos: token.into_pos(),
                })?,
            };
        }

        let body = parse_block_or_statement(s)?;

        ast::Statement::ForLet(
            ast::ForLet {
                loc: ast::Loc {
                    start: start_pos,
                    end: s.peek().pos().clone(),
                },
                label: None,
                var: name.to_string(),
                from,
                to,
                for_: body.into(),
            }
            .into(),
        )
    } else {
        let times = parse_expr(s)?;

        if has_paren {
            match s.pop_token() {
                Token::CloseParen { .. } => {}
                Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                    pos,
                })?,
                token => Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                    pos: token.into_pos(),
                })?,
            };
        }

        let body = parse_block_or_statement(s)?;

        ast::Statement::For(
            ast::For {
                loc: ast::Loc {
                    start: start_pos,
                    end: s.peek().pos().clone(),
                },
                label: None,
                times,
                for_: body.into(),
            }
            .into(),
        )
    })
}

fn parse_return(s: &mut Tokens) -> Result<ast::Return, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::ReturnKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };
    let expr = parse_expr(s)?;

    Ok(ast::Return {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        expr,
    })
}

pub fn parse_statement_with_attr(s: &mut Tokens) -> Result<ast::Definition, AiScriptSyntaxError> {
    let mut attrs = Vec::new();
    while let Token::OpenSharpBracket { .. } = s.peek() {
        attrs.push(parse_attr(s)?);
        match s.pop_token() {
            Token::NewLine { .. } => {}
            Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                pos,
            })?,
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                pos: token.into_pos(),
            })?,
        };
    }

    let statement = parse_statement(s)?;

    if let ast::StatementOrExpression::Statement(ast::Statement::Definition(definition)) = statement
    {
        let definition = *definition;
        let mut attr = definition.attr.unwrap_or_default();
        attr.extend(attrs);
        Ok(ast::Definition {
            attr: Some(attr),
            ..definition
        })
    } else {
        Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::InvalidAttribute,
            pos: statement.into_loc().start,
        })
    }
}

fn parse_attr(s: &mut Tokens) -> Result<ast::Attribute, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::OpenSharpBracket { pos, .. } => pos,
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
        Token::Identifier { value, .. } => value,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let value = if let Token::CloseBracket { .. } = s.peek() {
        let close_pos = s.peek().pos().clone();
        ast::Expression::Bool(
            ast::Bool {
                loc: ast::Loc {
                    start: close_pos.clone(),
                    end: close_pos,
                },
                value: true,
            }
            .into(),
        )
    } else {
        parse_expr(s)?
    };

    match s.pop_token() {
        Token::CloseBracket { .. } => {}
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    Ok(ast::Attribute {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        name: name.to_string(),
        value,
    })
}

fn parse_loop(s: &mut Tokens) -> Result<ast::Loop, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::LoopKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };
    let statements = parse_block(s)?;

    Ok(ast::Loop {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        label: None,
        statements,
    })
}

fn parse_do_while(s: &mut Tokens) -> Result<ast::Loop, AiScriptSyntaxError> {
    let do_start_pos = match s.pop_token() {
        Token::DoKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };
    let body = parse_block_or_statement(s)?;
    let while_pos = match s.pop_token() {
        Token::WhileKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };
    let cond = parse_expr(s)?;
    let end_pos = s.peek().pos().clone();

    Ok(ast::Loop {
        loc: ast::Loc {
            start: do_start_pos,
            end: end_pos.clone(),
        },
        label: None,
        statements: vec![
            body,
            ast::Expression::If(
                ast::If {
                    loc: ast::Loc {
                        start: while_pos.clone(),
                        end: end_pos.clone(),
                    },
                    label: None,
                    cond: ast::Expression::Not(
                        ast::Not {
                            loc: ast::Loc {
                                start: while_pos,
                                end: end_pos.clone(),
                            },
                            expr: cond.into(),
                        }
                        .into(),
                    )
                    .into(),
                    then: ast::StatementOrExpression::Statement(ast::Statement::Break(
                        ast::Break {
                            loc: ast::Loc {
                                start: end_pos.clone(),
                                end: end_pos,
                            },
                            label: None,
                            expr: None,
                        }
                        .into(),
                    ))
                    .into(),
                    elseif: Vec::new(),
                    else_: None,
                }
                .into(),
            )
            .into(),
        ],
    })
}

fn parse_while(s: &mut Tokens) -> Result<ast::Loop, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::WhileKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };
    let cond = parse_expr(s)?;
    let cond_end_pos = s.peek().pos().clone();
    let body = parse_block_or_statement(s)?;

    Ok(ast::Loop {
        loc: ast::Loc {
            start: start_pos.clone(),
            end: s.peek().pos().clone(),
        },
        label: None,
        statements: vec![
            ast::Expression::If(
                ast::If {
                    loc: ast::Loc {
                        start: start_pos.clone(),
                        end: cond_end_pos.clone(),
                    },
                    label: None,
                    cond: ast::Expression::Not(
                        ast::Not {
                            loc: ast::Loc {
                                start: start_pos,
                                end: cond_end_pos.clone(),
                            },
                            expr: cond.into(),
                        }
                        .into(),
                    )
                    .into(),
                    then: ast::StatementOrExpression::Statement(ast::Statement::Break(
                        ast::Break {
                            loc: ast::Loc {
                                start: cond_end_pos.clone(),
                                end: cond_end_pos,
                            },
                            label: None,
                            expr: None,
                        }
                        .into(),
                    ))
                    .into(),
                    elseif: Vec::new(),
                    else_: None,
                }
                .into(),
            )
            .into(),
            body,
        ],
    })
}

fn parse_break(s: &mut Tokens) -> Result<ast::Break, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::BreakKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let (label, expr) = if let Token::Sharp { .. } = s.peek() {
        let label = parse_label(s)?;

        let expr = match s.peek() {
            Token::Colon { .. } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnlabeledBreakWithExpression,
                pos: start_pos.clone(),
            })?,
            Token::Eof { .. }
            | Token::NewLine { .. }
            | Token::WhileKeyword { .. }
            | Token::ElifKeyword { .. }
            | Token::ElseKeyword { .. }
            | Token::Comma { .. }
            | Token::SemiColon { .. }
            | Token::CloseBrace { .. } => None,
            _ => Some(parse_expr(s)?),
        };

        (Some(label), expr)
    } else {
        (None, None)
    };

    Ok(ast::Break {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        label,
        expr,
    })
}

fn parse_continue(s: &mut Tokens) -> Result<ast::Continue, AiScriptSyntaxError> {
    let start_pos = match s.pop_token() {
        Token::ContinueKeyword { pos, .. } => pos,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    let label = if let Token::Sharp { .. } = s.peek() {
        Some(parse_label(s)?)
    } else {
        None
    };

    Ok(ast::Continue {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        label,
    })
}
