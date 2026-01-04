use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node as ast,
    parser::visit::Visitor,
};

struct JumpDestination {
    pub label: Option<String>,
    pub is_statement: bool,
}

struct JumpStatementValidator {
    pub has_ancestor_function: bool,
    pub jump_destinations: Vec<JumpDestination>,
}

impl Visitor for JumpStatementValidator {
    fn visit_each(&mut self, each: ast::Each) -> Result<ast::Each, AiScriptSyntaxError> {
        let items = self.visit_expression(each.items)?;
        self.jump_destinations.push(JumpDestination {
            label: each.label.clone(),
            is_statement: true,
        });
        let each = ast::Each {
            var: self.visit_expression(each.var)?,
            items,
            for_: match *each.for_ {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(ast::StatementOrExpression::Statement)?,
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(ast::StatementOrExpression::Expression)?,
            }
            .into(),
            ..each
        };
        self.jump_destinations.pop();
        Ok(each)
    }

    fn visit_for(&mut self, for_: ast::For) -> Result<ast::For, AiScriptSyntaxError> {
        let times = self.visit_expression(for_.times)?;
        self.jump_destinations.push(JumpDestination {
            label: for_.label.clone(),
            is_statement: true,
        });
        let for_ = ast::For {
            times,
            for_: match *for_.for_ {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(ast::StatementOrExpression::Statement)?,
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(ast::StatementOrExpression::Expression)?,
            }
            .into(),
            ..for_
        };
        self.jump_destinations.pop();
        Ok(for_)
    }

    fn visit_for_let(&mut self, for_let: ast::ForLet) -> Result<ast::ForLet, AiScriptSyntaxError> {
        let from = self.visit_expression(for_let.from)?;
        let to = self.visit_expression(for_let.to)?;
        self.jump_destinations.push(JumpDestination {
            label: for_let.label.clone(),
            is_statement: true,
        });
        let for_let = ast::ForLet {
            from,
            to,
            for_: match *for_let.for_ {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(ast::StatementOrExpression::Statement)?,
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(ast::StatementOrExpression::Expression)?,
            }
            .into(),
            ..for_let
        };
        self.jump_destinations.pop();
        Ok(for_let)
    }

    fn visit_loop(&mut self, loop_: ast::Loop) -> Result<ast::Loop, AiScriptSyntaxError> {
        self.jump_destinations.push(JumpDestination {
            label: loop_.label.clone(),
            is_statement: true,
        });
        let loop_ = ast::Loop {
            statements: loop_
                .statements
                .into_iter()
                .map(|s| match s {
                    ast::StatementOrExpression::Statement(statement) => self
                        .visit_statement(statement)
                        .map(ast::StatementOrExpression::Statement),
                    ast::StatementOrExpression::Expression(expression) => self
                        .visit_expression(expression)
                        .map(ast::StatementOrExpression::Expression),
                })
                .collect::<Result<Vec<ast::StatementOrExpression>, AiScriptSyntaxError>>()?,
            ..loop_
        };
        self.jump_destinations.pop();
        Ok(loop_)
    }

    fn visit_if(&mut self, if_: ast::If) -> Result<ast::If, AiScriptSyntaxError> {
        let cond = self.visit_expression(*if_.cond)?.into();
        let elseif = if_
            .elseif
            .into_iter()
            .map(|elseif| {
                Ok(ast::Elseif {
                    cond: self.visit_expression(elseif.cond)?,
                    ..elseif
                })
            })
            .collect::<Result<Vec<ast::Elseif>, AiScriptSyntaxError>>()?;
        self.jump_destinations.push(JumpDestination {
            label: if_.label.clone(),
            is_statement: false,
        });
        let if_ = ast::If {
            cond,
            then: match *if_.then {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(ast::StatementOrExpression::Statement)?,
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(ast::StatementOrExpression::Expression)?,
            }
            .into(),
            elseif: elseif
                .into_iter()
                .map(|elseif| {
                    Ok(ast::Elseif {
                        then: match elseif.then {
                            ast::StatementOrExpression::Statement(statement) => self
                                .visit_statement(statement)
                                .map(ast::StatementOrExpression::Statement)?,
                            ast::StatementOrExpression::Expression(expression) => self
                                .visit_expression(expression)
                                .map(ast::StatementOrExpression::Expression)?,
                        },
                        ..elseif
                    })
                })
                .collect::<Result<Vec<ast::Elseif>, AiScriptSyntaxError>>()?,
            else_: if_
                .else_
                .map(|else_| match *else_ {
                    ast::StatementOrExpression::Statement(statement) => self
                        .visit_statement(statement)
                        .map(ast::StatementOrExpression::Statement),
                    ast::StatementOrExpression::Expression(expression) => self
                        .visit_expression(expression)
                        .map(ast::StatementOrExpression::Expression),
                })
                .map_or(Ok(None), |r| r.map(Some))?
                .map(Into::into),
            ..if_
        };
        self.jump_destinations.pop();
        Ok(if_)
    }

    fn visit_fn(&mut self, fn_: ast::Fn) -> Result<ast::Fn, AiScriptSyntaxError> {
        let has_ancestor_function = self.has_ancestor_function;
        let jump_destinations = self.jump_destinations.drain(..).collect();
        let params = fn_
            .params
            .into_iter()
            .map(|param| {
                let default = param
                    .default
                    .map(|default| self.visit_expression(default))
                    .map_or(Ok(None), |r| r.map(Some))?;
                self.has_ancestor_function = true;
                let param = ast::Param {
                    dest: self.visit_expression(param.dest)?,
                    default,
                    arg_type: param
                        .arg_type
                        .map(|arg_type| self.visit_type_source(arg_type))
                        .map_or(Ok(None), |r| r.map(Some))?,
                    ..param
                };
                self.has_ancestor_function = has_ancestor_function;
                Ok(param)
            })
            .collect::<Result<Vec<ast::Param>, AiScriptSyntaxError>>()?;
        self.has_ancestor_function = true;
        let fn_ = ast::Fn {
            params,
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
        self.has_ancestor_function = has_ancestor_function;
        self.jump_destinations = jump_destinations;
        Ok(fn_)
    }

    fn visit_match(&mut self, match_: ast::Match) -> Result<ast::Match, AiScriptSyntaxError> {
        let about = self.visit_expression(*match_.about)?.into();
        let qs = match_
            .qs
            .into_iter()
            .map(|qa| {
                Ok(ast::QA {
                    q: self.visit_expression(qa.q)?,
                    ..qa
                })
            })
            .collect::<Result<Vec<ast::QA>, AiScriptSyntaxError>>()?;
        self.jump_destinations.push(JumpDestination {
            label: match_.label.clone(),
            is_statement: false,
        });
        let match_ = ast::Match {
            about,
            qs: qs
                .into_iter()
                .map(|qa| {
                    Ok(ast::QA {
                        a: match qa.a {
                            ast::StatementOrExpression::Statement(statement) => self
                                .visit_statement(statement)
                                .map(ast::StatementOrExpression::Statement)?,
                            ast::StatementOrExpression::Expression(expression) => self
                                .visit_expression(expression)
                                .map(ast::StatementOrExpression::Expression)?,
                        },
                        ..qa
                    })
                })
                .collect::<Result<Vec<ast::QA>, AiScriptSyntaxError>>()?,
            default: match_
                .default
                .map(|default| match *default {
                    ast::StatementOrExpression::Statement(statement) => self
                        .visit_statement(statement)
                        .map(ast::StatementOrExpression::Statement),
                    ast::StatementOrExpression::Expression(expression) => self
                        .visit_expression(expression)
                        .map(ast::StatementOrExpression::Expression),
                })
                .map_or(Ok(None), |r| r.map(Some))?
                .map(Into::into),
            ..match_
        };
        self.jump_destinations.pop();
        Ok(match_)
    }

    fn visit_block(&mut self, block: ast::Block) -> Result<ast::Block, AiScriptSyntaxError> {
        self.jump_destinations.push(JumpDestination {
            label: block.label.clone(),
            is_statement: false,
        });
        let block = ast::Block {
            statements: block
                .statements
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
            ..block
        };
        self.jump_destinations.pop();
        Ok(block)
    }

    fn callback_statement(
        &mut self,
        statement: ast::Statement,
    ) -> Result<ast::Statement, AiScriptSyntaxError> {
        match statement {
            ast::Statement::Return(return_) if !self.has_ancestor_function => {
                Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                    pos: return_.loc.start,
                })
            }
            ast::Statement::Break(break_) => {
                if let Some(label) = &break_.label {
                    for ancestor in self.jump_destinations.iter().rev() {
                        if let Some(ancestor_label) = &ancestor.label
                            && ancestor_label == label
                        {
                            return if break_.expr.is_some() && ancestor.is_statement {
                                Err(AiScriptSyntaxError {
                                    kind: AiScriptSyntaxErrorKind::BreakToStatementWithValue,
                                    pos: break_.loc.start,
                                })
                            } else {
                                return Ok(ast::Statement::Break(break_));
                            };
                        }
                    }
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UndefinedLabel(label.to_string()),
                        pos: break_.loc.start,
                    })
                } else {
                    for ancestor in self.jump_destinations.iter().rev() {
                        if ancestor.is_statement {
                            return if break_.expr.is_some() {
                                Err(AiScriptSyntaxError {
                                    kind: AiScriptSyntaxErrorKind::BreakToStatementWithValue,
                                    pos: break_.loc.start,
                                })
                            } else {
                                Ok(ast::Statement::Break(break_))
                            };
                        }
                    }
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UnlabeledBreakOutsideLoop,
                        pos: break_.loc.start,
                    })
                }
            }
            ast::Statement::Continue(continue_) => {
                if let Some(label) = &continue_.label {
                    for ancestor in self.jump_destinations.iter().rev() {
                        if let Some(ancestor_label) = &ancestor.label
                            && ancestor_label == label
                        {
                            return if ancestor.is_statement {
                                Ok(ast::Statement::Continue(continue_))
                            } else {
                                Err(AiScriptSyntaxError {
                                    kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
                                    pos: continue_.loc.start,
                                })
                            };
                        }
                    }
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UndefinedLabel(label.to_string()),
                        pos: continue_.loc.start,
                    })
                } else {
                    for ancestor in self.jump_destinations.iter().rev() {
                        if ancestor.is_statement {
                            return Ok(ast::Statement::Continue(continue_));
                        }
                    }
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
                        pos: continue_.loc.start,
                    })
                }
            }
            _ => Ok(statement),
        }
    }
}

pub fn validate_jump_statement(
    nodes: impl IntoIterator<Item = ast::Node>,
) -> Result<Vec<ast::Node>, AiScriptSyntaxError> {
    nodes
        .into_iter()
        .map(|node| {
            JumpStatementValidator {
                has_ancestor_function: false,
                jump_destinations: Vec::new(),
            }
            .visit_node(node)
        })
        .collect()
}
