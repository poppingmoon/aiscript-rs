use indexmap::{IndexMap, map::Entry};

use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node as ast,
    parser::token::{TemplateToken, Token, Tokens, TokensExt},
};

use super::{
    common::{parse_block, parse_params},
    statements::parse_block_or_statement,
    types::{parse_type, parse_type_params},
};

pub fn parse_expr(s: &mut Tokens) -> Result<ast::Expression, AiScriptSyntaxError> {
    parse_pratt(s, 0)
}

enum Prefix {
    Plus,
    Minus,
    Not,
}

impl Prefix {
    fn bp(&self) -> u8 {
        match self {
            Prefix::Plus => 14,
            Prefix::Minus => 14,
            Prefix::Not => 14,
        }
    }
}

enum Infix {
    Dot,
    Hat,
    Asterisk,
    Slash,
    Percent,
    Plus,
    Minus,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Eq2,
    NotEq,
    And2,
    Or2,
}

impl Infix {
    fn lbp(&self) -> u8 {
        match self {
            Infix::Dot => 18,
            Infix::Hat => 17,
            Infix::Asterisk => 12,
            Infix::Slash => 12,
            Infix::Percent => 12,
            Infix::Plus => 10,
            Infix::Minus => 10,
            Infix::Lt => 8,
            Infix::LtEq => 8,
            Infix::Gt => 8,
            Infix::GtEq => 8,
            Infix::Eq2 => 6,
            Infix::NotEq => 6,
            Infix::And2 => 4,
            Infix::Or2 => 2,
        }
    }

    fn rbp(&self) -> u8 {
        match self {
            Infix::Dot => 19,
            Infix::Hat => 16,
            Infix::Asterisk => 13,
            Infix::Slash => 13,
            Infix::Percent => 13,
            Infix::Plus => 11,
            Infix::Minus => 11,
            Infix::Lt => 9,
            Infix::LtEq => 9,
            Infix::Gt => 9,
            Infix::GtEq => 9,
            Infix::Eq2 => 7,
            Infix::NotEq => 7,
            Infix::And2 => 5,
            Infix::Or2 => 3,
        }
    }
}

enum Postfix {
    OpenParen,
    OpenBracket,
}

impl Postfix {
    fn bp(&self) -> u8 {
        match self {
            Postfix::OpenParen => 20,
            Postfix::OpenBracket => 20,
        }
    }
}

fn parse_prefix(
    start_pos: ast::Pos,
    op: Prefix,
    s: &mut Tokens,
) -> Result<ast::Expression, AiScriptSyntaxError> {
    if let Token::BackSlash { .. } = s.peek() {
        s.pop_token();
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

    let expr = parse_pratt(s, op.bp())?;

    Ok(match op {
        Prefix::Plus => {
            if let ast::Expression::Num(num) = expr {
                ast::Expression::Num(
                    ast::Num {
                        loc: ast::Loc {
                            start: start_pos,
                            end: num.loc.end,
                        },
                        value: num.value,
                    }
                    .into(),
                )
            } else {
                ast::Expression::Plus(
                    ast::Plus {
                        loc: ast::Loc {
                            start: start_pos,
                            end: s.peek().pos().clone(),
                        },
                        expr: expr.into(),
                    }
                    .into(),
                )
            }
        }
        Prefix::Minus => {
            if let ast::Expression::Num(num) = expr {
                ast::Expression::Num(
                    ast::Num {
                        loc: ast::Loc {
                            start: start_pos,
                            end: num.loc.end,
                        },
                        value: -num.value,
                    }
                    .into(),
                )
            } else {
                ast::Expression::Minus(
                    ast::Minus {
                        loc: ast::Loc {
                            start: start_pos,
                            end: s.peek().pos().clone(),
                        },
                        expr: expr.into(),
                    }
                    .into(),
                )
            }
        }
        Prefix::Not => ast::Expression::Not(
            ast::Not {
                loc: ast::Loc {
                    start: start_pos,
                    end: s.peek().pos().clone(),
                },
                expr: expr.into(),
            }
            .into(),
        ),
    })
}

fn parse_infix(
    start_pos: ast::Pos,
    op: Infix,
    s: &mut Tokens,
    left: ast::Expression,
) -> Result<ast::Expression, AiScriptSyntaxError> {
    if let Token::BackSlash { .. } = s.peek() {
        s.pop_token();
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

    Ok(match op {
        Infix::Dot => {
            let name = parse_object_key(s)?.0;

            ast::Expression::Prop(
                ast::Prop {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    target: left.into(),
                    name,
                }
                .into(),
            )
        }
        Infix::Hat => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Pow(
                ast::Pow {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::Asterisk => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Mul(
                ast::Mul {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::Slash => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Div(
                ast::Div {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::Percent => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Rem(
                ast::Rem {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::Plus => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Add(
                ast::Add {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::Minus => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Sub(
                ast::Sub {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::Lt => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Lt(
                ast::Lt {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::LtEq => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Lteq(
                ast::Lteq {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::Gt => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Gt(
                ast::Gt {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::GtEq => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Gteq(
                ast::Gteq {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::Eq2 => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Eq(
                ast::Eq {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::NotEq => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Neq(
                ast::Neq {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::And2 => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::And(
                ast::And {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
        Infix::Or2 => {
            let right = parse_pratt(s, op.rbp())?;

            ast::Expression::Or(
                ast::Or {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    left: left.into(),
                    right: right.into(),
                }
                .into(),
            )
        }
    })
}

fn parse_postfix(
    start_pos: ast::Pos,
    op: Postfix,
    s: &mut Tokens,
    expr: ast::Expression,
) -> Result<ast::Expression, AiScriptSyntaxError> {
    Ok(match op {
        Postfix::OpenParen => ast::Expression::Call(parse_call(start_pos, s, expr)?.into()),
        Postfix::OpenBracket => {
            let index = parse_expr(s)?;
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

            ast::Expression::Index(
                ast::Index {
                    loc: ast::Loc {
                        start: start_pos,
                        end: s.peek().pos().clone(),
                    },
                    target: expr.into(),
                    index: index.into(),
                }
                .into(),
            )
        }
    })
}

fn parse_atom(s: &mut Tokens) -> Result<ast::Expression, AiScriptSyntaxError> {
    Ok(match s.pop_token() {
        Token::IfKeyword { pos, .. } => ast::Expression::If(parse_if(pos, s)?.into()),
        Token::At { pos, .. } => ast::Expression::Fn(parse_fn_expr(pos, s)?.into()),
        Token::MatchKeyword { pos, .. } => ast::Expression::Match(parse_match(pos, s)?.into()),
        Token::EvalKeyword { pos, .. } => ast::Expression::Block(parse_eval(pos, s)?.into()),
        Token::ExistsKeyword { pos, .. } => ast::Expression::Exists(parse_exists(pos, s)?.into()),
        Token::Template { pos, children, .. } => {
            let mut children = children.into_iter().peekable();
            let mut values = Vec::new();

            while let Some(element) = children.next() {
                match element {
                    TemplateToken::TemplateStringElement { pos, value } => {
                        let end_pos = children.peek().map_or_else(
                            || s.peek().pos(),
                            |token| match token {
                                TemplateToken::TemplateStringElement { pos, .. }
                                | TemplateToken::TemplateExprElement { pos, .. } => pos,
                            },
                        );
                        values.push(ast::Expression::Str(
                            ast::Str {
                                loc: ast::Loc {
                                    start: pos,
                                    end: end_pos.clone(),
                                },
                                value: value.to_string(),
                            }
                            .into(),
                        ))
                    }
                    TemplateToken::TemplateExprElement { mut children, .. } => {
                        children.reverse();
                        if let Token::NewLine { .. } = children.peek() {
                            children.pop_token();
                        }
                        let expr = parse_expr(&mut children)?;
                        if let Token::NewLine { .. } = children.peek() {
                            children.pop_token();
                        }
                        match children.pop_token() {
                            Token::Eof { .. } => {}
                            token => Err(AiScriptSyntaxError {
                                kind: AiScriptSyntaxErrorKind::UnexpectedToken(
                                    token.kind().to_string(),
                                ),
                                pos: token.into_pos(),
                            })?,
                        }
                        values.push(expr);
                    }
                }
            }

            ast::Expression::Tmpl(
                ast::Tmpl {
                    loc: ast::Loc {
                        start: pos,
                        end: s.peek().pos().clone(),
                    },
                    tmpl: values,
                }
                .into(),
            )
        }
        Token::StringLiteral { pos, value, .. } => ast::Expression::Str(
            ast::Str {
                loc: ast::Loc {
                    start: pos,
                    end: s.peek().pos().clone(),
                },
                value: value.concat(),
            }
            .into(),
        ),
        Token::NumberLiteral { pos, value, .. } => match value.parse() {
            Ok(value) => ast::Expression::Num(
                ast::Num {
                    loc: ast::Loc {
                        start: pos,
                        end: s.peek().pos().clone(),
                    },
                    value,
                }
                .into(),
            ),
            Err(e) => Err(AiScriptSyntaxError {
                kind: e.into(),
                pos,
            })?,
        },
        Token::TrueKeyword { pos, .. } => ast::Expression::Bool(
            ast::Bool {
                loc: ast::Loc {
                    start: pos,
                    end: s.peek().pos().clone(),
                },
                value: true,
            }
            .into(),
        ),
        Token::FalseKeyword { pos, .. } => ast::Expression::Bool(
            ast::Bool {
                loc: ast::Loc {
                    start: pos,
                    end: s.peek().pos().clone(),
                },
                value: false,
            }
            .into(),
        ),
        Token::NullKeyword { pos, .. } => ast::Expression::Null(
            ast::Null {
                loc: ast::Loc {
                    start: pos,
                    end: s.peek().pos().clone(),
                },
            }
            .into(),
        ),
        Token::OpenBrace { pos, .. } => ast::Expression::Obj(parse_object(pos, s, false)?.into()),
        Token::OpenBracket { pos, .. } => ast::Expression::Arr(parse_array(pos, s, false)?.into()),
        Token::Identifier { pos, value, .. } => {
            ast::Expression::Identifier(parse_reference(pos, value, s)?.into())
        }
        Token::OpenParen { .. } => {
            let expr = parse_expr(s)?;
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
            expr
        }
        Token::Sharp { .. } => parse_expr_with_label(s)?,
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

pub fn parse_static_expr(s: &mut Tokens) -> Result<ast::Expression, AiScriptSyntaxError> {
    Ok(match s.pop_token() {
        Token::StringLiteral { pos, value, .. } => ast::Expression::Str(
            ast::Str {
                loc: ast::Loc {
                    start: pos,
                    end: s.peek().pos().clone(),
                },
                value: value.concat(),
            }
            .into(),
        ),
        Token::NumberLiteral { pos, value, .. } => match value.parse() {
            Ok(value) => ast::Expression::Num(
                ast::Num {
                    loc: ast::Loc {
                        start: pos,
                        end: s.peek().pos().clone(),
                    },
                    value,
                }
                .into(),
            ),
            Err(e) => Err(AiScriptSyntaxError {
                kind: e.into(),
                pos,
            })?,
        },
        Token::TrueKeyword { pos, .. } => ast::Expression::Bool(
            ast::Bool {
                loc: ast::Loc {
                    start: pos,
                    end: s.peek().pos().clone(),
                },
                value: true,
            }
            .into(),
        ),
        Token::FalseKeyword { pos, .. } => ast::Expression::Bool(
            ast::Bool {
                loc: ast::Loc {
                    start: pos,
                    end: s.peek().pos().clone(),
                },
                value: false,
            }
            .into(),
        ),
        Token::NullKeyword { pos, .. } => ast::Expression::Null(
            ast::Null {
                loc: ast::Loc {
                    start: pos,
                    end: s.peek().pos().clone(),
                },
            }
            .into(),
        ),
        Token::OpenBrace { pos, .. } => ast::Expression::Obj(parse_object(pos, s, true)?.into()),
        Token::OpenBracket { pos, .. } => ast::Expression::Arr(parse_array(pos, s, true)?.into()),
        Token::OpenParen { .. } => {
            let expr = parse_static_expr(s)?;
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
            expr
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

fn parse_call(
    start_pos: ast::Pos,
    s: &mut Tokens,
    target: ast::Expression,
) -> Result<ast::Call, AiScriptSyntaxError> {
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

        items.push(parse_expr(s)?);

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

    Ok(ast::Call {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        target: target.into(),
        args: items,
    })
}

fn parse_expr_with_label(s: &mut Tokens) -> Result<ast::Expression, AiScriptSyntaxError> {
    let label = match s.pop_token() {
        Token::Identifier {
            pos,
            has_left_spacing: true,
            ..
        } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::SpaceInLabel,
            pos,
        })?,
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

    Ok(match parse_expr(s)? {
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
        expression => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::InvalidExpressionWithLabel,
            pos: expression.into_loc().start,
        })?,
    })
}

fn parse_if(start_pos: ast::Pos, s: &mut Tokens) -> Result<ast::If, AiScriptSyntaxError> {
    let cond = parse_expr(s)?;
    let then = parse_block_or_statement(s)?;

    if let Token::NewLine { .. } = s.peek()
        && let Token::ElifKeyword { .. } | Token::ElseKeyword { .. } = s.lookahead(1)
    {
        s.pop_token();
    }

    let mut elseif = Vec::new();
    while let Token::ElifKeyword { .. } = s.peek() {
        s.pop_token();
        let elif_cond = parse_expr(s)?;
        let elif_then = parse_block_or_statement(s)?;
        elseif.push(ast::Elseif {
            cond: elif_cond,
            then: elif_then,
        });
        if let Token::NewLine { .. } = s.peek()
            && let Token::ElifKeyword { .. } | Token::ElseKeyword { .. } = s.lookahead(1)
        {
            s.pop_token();
        }
    }

    let else_ = if let Token::ElseKeyword { .. } = s.peek() {
        s.pop_token();
        Some(parse_block_or_statement(s)?.into())
    } else {
        None
    };

    Ok(ast::If {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        label: None,
        cond: cond.into(),
        then: then.into(),
        elseif,
        else_,
    })
}

fn parse_fn_expr(start_pos: ast::Pos, s: &mut Tokens) -> Result<ast::Fn, AiScriptSyntaxError> {
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

    Ok(ast::Fn {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        type_params,
        params,
        ret_type: type_,
        children: body,
    })
}

fn parse_match(start_pos: ast::Pos, s: &mut Tokens) -> Result<ast::Match, AiScriptSyntaxError> {
    let about = parse_expr(s)?;

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

    let mut qs = Vec::new();
    let x = loop {
        match s.pop_token() {
            Token::NewLine { .. } => {}
            Token::CaseKeyword { .. } => {
                let q = parse_expr(s)?;
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
                let a = parse_block_or_statement(s)?;
                qs.push(ast::QA { q, a });

                match s.pop_token() {
                    Token::NewLine { .. } | Token::Comma { .. } => {}
                    Token::CloseBrace { .. } => break None,
                    token => Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::SeparatorExpected,
                        pos: token.into_pos(),
                    })?,
                }
            }
            Token::DefaultKeyword { .. } => {
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
                let x = parse_block_or_statement(s)?;

                match s.peek() {
                    Token::NewLine { .. } => {
                        s.pop_token();
                    }
                    Token::Comma { .. } => {
                        s.pop_token();
                        if let Token::NewLine { .. } = s.peek() {
                            s.pop_token();
                        }
                    }
                    _ => {}
                }
                match s.pop_token() {
                    Token::CloseBrace { .. } => {}
                    Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                        pos,
                    })?,
                    token => Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                        pos: token.into_pos(),
                    })?,
                };

                break Some(x.into());
            }
            Token::CloseBrace { .. } => break None,
            Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                pos,
            })?,
            token => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                pos: token.into_pos(),
            })?,
        }
    };

    Ok(ast::Match {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        label: None,
        about: about.into(),
        qs,
        default: x,
    })
}

fn parse_eval(start_pos: ast::Pos, s: &mut Tokens) -> Result<ast::Block, AiScriptSyntaxError> {
    let statements = parse_block(s)?;

    Ok(ast::Block {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        label: None,
        statements,
    })
}

fn parse_exists(start_pos: ast::Pos, s: &mut Tokens) -> Result<ast::Exists, AiScriptSyntaxError> {
    let identifier = match s.pop_token() {
        Token::Identifier { pos, value, .. } => parse_reference(pos, value, s)?,
        Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            pos,
        })?,
        token => Err(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
            pos: token.into_pos(),
        })?,
    };

    Ok(ast::Exists {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        identifier,
    })
}

fn parse_reference(
    start_pos: ast::Pos,
    initial_value: &str,
    s: &mut Tokens,
) -> Result<ast::Identifier, AiScriptSyntaxError> {
    let mut segs = vec![initial_value];
    loop {
        match s.peek() {
            Token::Colon {
                pos,
                has_left_spacing: true,
            } => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::SpaceInReference,
                pos: pos.clone(),
            })?,
            Token::Colon { .. } => {
                s.pop_token();
                match s.pop_token() {
                    Token::Identifier {
                        pos,
                        has_left_spacing: true,
                        ..
                    } => Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::SpaceInReference,
                        pos,
                    })?,
                    Token::Identifier { value, .. } => segs.push(value),
                    Token::Eof { pos, .. } => Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                        pos,
                    })?,
                    token => Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UnexpectedToken(token.kind().to_string()),
                        pos: token.into_pos(),
                    })?,
                }
            }
            token => {
                break Ok(ast::Identifier {
                    loc: ast::Loc {
                        start: start_pos,
                        end: token.pos().clone(),
                    },
                    name: segs.join(":"),
                });
            }
        }
    }
}

fn parse_object(
    start_pos: ast::Pos,
    s: &mut Tokens,
    is_static: bool,
) -> Result<ast::Obj, AiScriptSyntaxError> {
    let mut map = IndexMap::new();
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

        let (k, pos) = parse_object_key(s)?;

        match map.entry(k) {
            Entry::Occupied(entry) => Err(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::DuplicateKey(entry.key().to_string()),
                pos,
            })?,
            Entry::Vacant(entry) => {
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

                let v = if is_static {
                    parse_static_expr
                } else {
                    parse_expr
                }(s)?;

                entry.insert(v);
            }
        }

        match s.pop_token() {
            Token::NewLine { .. } | Token::Comma { .. } => {}
            Token::CloseBrace { .. } => break,
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

    Ok(ast::Obj {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        value: map,
    })
}

fn parse_object_key(s: &mut Tokens) -> Result<(String, ast::Pos), AiScriptSyntaxError> {
    Ok(match s.pop_token() {
        Token::Identifier { pos, value, .. } => (value.to_string(), pos),
        Token::StringLiteral { pos, value, .. } => (value.concat(), pos),
        Token::NullKeyword { pos, .. } => ("null".to_string(), pos),
        Token::TrueKeyword { pos, .. } => ("true".to_string(), pos),
        Token::FalseKeyword { pos, .. } => ("false".to_string(), pos),
        Token::EachKeyword { pos, .. } => ("each".to_string(), pos),
        Token::ForKeyword { pos, .. } => ("for".to_string(), pos),
        Token::LoopKeyword { pos, .. } => ("loop".to_string(), pos),
        Token::DoKeyword { pos, .. } => ("do".to_string(), pos),
        Token::WhileKeyword { pos, .. } => ("while".to_string(), pos),
        Token::BreakKeyword { pos, .. } => ("break".to_string(), pos),
        Token::ContinueKeyword { pos, .. } => ("continue".to_string(), pos),
        Token::MatchKeyword { pos, .. } => ("match".to_string(), pos),
        Token::CaseKeyword { pos, .. } => ("case".to_string(), pos),
        Token::DefaultKeyword { pos, .. } => ("default".to_string(), pos),
        Token::IfKeyword { pos, .. } => ("if".to_string(), pos),
        Token::ElifKeyword { pos, .. } => ("elif".to_string(), pos),
        Token::ElseKeyword { pos, .. } => ("else".to_string(), pos),
        Token::ReturnKeyword { pos, .. } => ("return".to_string(), pos),
        Token::EvalKeyword { pos, .. } => ("eval".to_string(), pos),
        Token::VarKeyword { pos, .. } => ("var".to_string(), pos),
        Token::LetKeyword { pos, .. } => ("let".to_string(), pos),
        Token::ExistsKeyword { pos, .. } => ("exists".to_string(), pos),
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

fn parse_array(
    start_pos: ast::Pos,
    s: &mut Tokens,
    is_static: bool,
) -> Result<ast::Arr, AiScriptSyntaxError> {
    let mut value = Vec::new();
    loop {
        match s.peek() {
            Token::NewLine { .. } => {
                s.pop_token();
                continue;
            }
            Token::CloseBracket { .. } => {
                s.pop_token();
                break;
            }
            _ => {}
        }

        value.push(if is_static {
            parse_static_expr
        } else {
            parse_expr
        }(s)?);

        match s.pop_token() {
            Token::NewLine { .. } | Token::Comma { .. } => {}
            Token::CloseBracket { .. } => break,
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

    Ok(ast::Arr {
        loc: ast::Loc {
            start: start_pos,
            end: s.peek().pos().clone(),
        },
        value,
    })
}

fn parse_pratt(s: &mut Tokens, min_bp: u8) -> Result<ast::Expression, AiScriptSyntaxError> {
    let prefix = match s.peek() {
        Token::Plus { .. } => Some(Prefix::Plus),
        Token::Minus { .. } => Some(Prefix::Minus),
        Token::Not { .. } => Some(Prefix::Not),
        _ => None,
    };
    let mut left = if let Some(prefix) = prefix {
        parse_prefix(s.pop_token().into_pos(), prefix, s)?
    } else {
        parse_atom(s)?
    };

    loop {
        if let Token::BackSlash { .. } = s.peek() {
            s.pop_token();
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

        let token = s.peek();

        let postfix = match token {
            Token::OpenParen { .. } => Some(Postfix::OpenParen),
            Token::OpenBracket { .. } => Some(Postfix::OpenBracket),
            _ => None,
        };
        if let Some(postfix) = postfix {
            if postfix.bp() < min_bp {
                break;
            }

            let (Token::OpenParen {
                has_left_spacing: true,
                ..
            }
            | Token::OpenBracket {
                has_left_spacing: true,
                ..
            }) = token
            else {
                left = parse_postfix(s.pop_token().into_pos(), postfix, s, left)?;
                continue;
            };
        }

        let infix = match token {
            Token::Dot { .. } => Some(Infix::Dot),
            Token::Hat { .. } => Some(Infix::Hat),
            Token::Asterisk { .. } => Some(Infix::Asterisk),
            Token::Slash { .. } => Some(Infix::Slash),
            Token::Percent { .. } => Some(Infix::Percent),
            Token::Plus { .. } => Some(Infix::Plus),
            Token::Minus { .. } => Some(Infix::Minus),
            Token::Lt { .. } => Some(Infix::Lt),
            Token::LtEq { .. } => Some(Infix::LtEq),
            Token::Gt { .. } => Some(Infix::Gt),
            Token::GtEq { .. } => Some(Infix::GtEq),
            Token::Eq2 { .. } => Some(Infix::Eq2),
            Token::NotEq { .. } => Some(Infix::NotEq),
            Token::And2 { .. } => Some(Infix::And2),
            Token::Or2 { .. } => Some(Infix::Or2),
            _ => None,
        };

        if let Some(infix) = infix
            && infix.lbp() >= min_bp
        {
            left = parse_infix(s.pop_token().into_pos(), infix, s, left)?;
            continue;
        }

        break;
    }

    Ok(left)
}
