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
    fn visit_each(&mut self, each: &ast::Each) -> Result<Option<ast::Each>, AiScriptSyntaxError> {
        self.visit_expression(&each.items)?;
        self.jump_destinations.push(JumpDestination {
            label: each.label.clone(),
            is_statement: true,
        });
        self.visit_expression(&each.var)?;
        match &each.for_ {
            ast::StatementOrExpression::Statement(statement) => {
                self.visit_statement(statement)?;
            }
            ast::StatementOrExpression::Expression(expression) => {
                self.visit_expression(expression)?;
            }
        };
        self.jump_destinations.pop();
        Ok(None)
    }

    fn visit_for(&mut self, for_: &ast::For) -> Result<Option<ast::For>, AiScriptSyntaxError> {
        self.visit_expression(&for_.times)?;
        self.jump_destinations.push(JumpDestination {
            label: for_.label.clone(),
            is_statement: true,
        });
        match &for_.for_ {
            ast::StatementOrExpression::Statement(statement) => {
                self.visit_statement(statement)?;
            }
            ast::StatementOrExpression::Expression(expression) => {
                self.visit_expression(expression)?;
            }
        };
        self.jump_destinations.pop();
        Ok(None)
    }

    fn visit_for_let(
        &mut self,
        for_let: &ast::ForLet,
    ) -> Result<Option<ast::ForLet>, AiScriptSyntaxError> {
        self.visit_expression(&for_let.from)?;
        self.visit_expression(&for_let.to)?;
        self.jump_destinations.push(JumpDestination {
            label: for_let.label.clone(),
            is_statement: true,
        });
        match &for_let.for_ {
            ast::StatementOrExpression::Statement(statement) => {
                self.visit_statement(statement)?;
            }
            ast::StatementOrExpression::Expression(expression) => {
                self.visit_expression(expression)?;
            }
        };
        self.jump_destinations.pop();
        Ok(None)
    }

    fn visit_loop(&mut self, loop_: &ast::Loop) -> Result<Option<ast::Loop>, AiScriptSyntaxError> {
        self.jump_destinations.push(JumpDestination {
            label: loop_.label.clone(),
            is_statement: true,
        });
        for s in &loop_.statements {
            match s {
                ast::StatementOrExpression::Statement(statement) => {
                    self.visit_statement(statement)?;
                }
                ast::StatementOrExpression::Expression(expression) => {
                    self.visit_expression(expression)?;
                }
            };
        }
        self.jump_destinations.pop();
        Ok(None)
    }

    fn visit_if(&mut self, if_: &ast::If) -> Result<Option<ast::If>, AiScriptSyntaxError> {
        self.visit_expression(&if_.cond)?;
        for ast::Elseif { cond, .. } in &if_.elseif {
            self.visit_expression(cond)?;
        }
        self.jump_destinations.push(JumpDestination {
            label: if_.label.clone(),
            is_statement: false,
        });
        match &if_.then {
            ast::StatementOrExpression::Statement(statement) => {
                self.visit_statement(statement)?;
            }
            ast::StatementOrExpression::Expression(expression) => {
                self.visit_expression(expression)?;
            }
        };
        for ast::Elseif { then, .. } in &if_.elseif {
            match then {
                ast::StatementOrExpression::Statement(statement) => {
                    self.visit_statement(statement)?;
                }
                ast::StatementOrExpression::Expression(expression) => {
                    self.visit_expression(expression)?;
                }
            };
        }
        if let Some(else_) = &if_.else_ {
            match else_ {
                ast::StatementOrExpression::Statement(statement) => {
                    self.visit_statement(statement)?;
                }
                ast::StatementOrExpression::Expression(expression) => {
                    self.visit_expression(expression)?;
                }
            };
        }
        self.jump_destinations.pop();
        Ok(None)
    }

    fn visit_fn(&mut self, fn_: &ast::Fn) -> Result<Option<ast::Fn>, AiScriptSyntaxError> {
        let has_ancestor_function = self.has_ancestor_function;
        let jump_destinations = std::mem::take(&mut self.jump_destinations);
        for ast::Param {
            dest,
            default,
            arg_type,
            ..
        } in &fn_.params
        {
            if let Some(default) = default {
                self.visit_expression(default)?;
            }
            self.has_ancestor_function = true;
            self.visit_expression(dest)?;
            if let Some(arg_type) = arg_type {
                self.visit_type_source(arg_type)?;
            }
            self.has_ancestor_function = has_ancestor_function;
        }
        self.has_ancestor_function = true;
        if let Some(ret_type) = &fn_.ret_type {
            self.visit_type_source(ret_type)?;
        }
        for child in &fn_.children {
            match child {
                ast::StatementOrExpression::Statement(statement) => {
                    self.visit_statement(statement)?;
                }
                ast::StatementOrExpression::Expression(expression) => {
                    self.visit_expression(expression)?;
                }
            };
        }
        self.has_ancestor_function = has_ancestor_function;
        self.jump_destinations = jump_destinations;
        Ok(None)
    }

    fn visit_match(
        &mut self,
        match_: &ast::Match,
    ) -> Result<Option<ast::Match>, AiScriptSyntaxError> {
        self.visit_expression(&match_.about)?;
        for ast::QA { q, .. } in &match_.qs {
            self.visit_expression(q)?;
        }
        self.jump_destinations.push(JumpDestination {
            label: match_.label.clone(),
            is_statement: false,
        });
        for ast::QA { a, .. } in &match_.qs {
            match a {
                ast::StatementOrExpression::Statement(statement) => {
                    self.visit_statement(statement)?;
                }
                ast::StatementOrExpression::Expression(expression) => {
                    self.visit_expression(expression)?;
                }
            };
        }
        if let Some(default) = &match_.default {
            match default {
                ast::StatementOrExpression::Statement(statement) => {
                    self.visit_statement(statement)?;
                }
                ast::StatementOrExpression::Expression(expression) => {
                    self.visit_expression(expression)?;
                }
            };
        }
        self.jump_destinations.pop();
        Ok(None)
    }

    fn visit_block(
        &mut self,
        block: &ast::Block,
    ) -> Result<Option<ast::Block>, AiScriptSyntaxError> {
        self.jump_destinations.push(JumpDestination {
            label: block.label.clone(),
            is_statement: false,
        });
        for child in &block.statements {
            match child {
                ast::StatementOrExpression::Statement(statement) => {
                    self.visit_statement(statement)?;
                }
                ast::StatementOrExpression::Expression(expression) => {
                    self.visit_expression(expression)?;
                }
            };
        }
        self.jump_destinations.pop();
        Ok(None)
    }

    fn callback_statement(
        &mut self,
        statement: &ast::Statement,
    ) -> Result<Option<ast::Statement>, AiScriptSyntaxError> {
        match statement {
            ast::Statement::Return(return_) if !self.has_ancestor_function => {
                Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                    pos: return_.loc.start.clone(),
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
                                    pos: break_.loc.start.clone(),
                                })
                            } else {
                                Ok(None)
                            };
                        }
                    }
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UndefinedLabel(label.to_string()),
                        pos: break_.loc.start.clone(),
                    })
                } else {
                    for ancestor in self.jump_destinations.iter().rev() {
                        if ancestor.is_statement {
                            return if break_.expr.is_some() {
                                Err(AiScriptSyntaxError {
                                    kind: AiScriptSyntaxErrorKind::BreakToStatementWithValue,
                                    pos: break_.loc.start.clone(),
                                })
                            } else {
                                Ok(None)
                            };
                        }
                    }
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UnlabeledBreakOutsideLoop,
                        pos: break_.loc.start.clone(),
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
                                Ok(None)
                            } else {
                                Err(AiScriptSyntaxError {
                                    kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
                                    pos: continue_.loc.start.clone(),
                                })
                            };
                        }
                    }
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::UndefinedLabel(label.to_string()),
                        pos: continue_.loc.start.clone(),
                    })
                } else {
                    for ancestor in self.jump_destinations.iter().rev() {
                        if ancestor.is_statement {
                            return Ok(None);
                        }
                    }
                    Err(AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
                        pos: continue_.loc.start.clone(),
                    })
                }
            }
            _ => Ok(None),
        }
    }
}

pub fn validate_jump_statement(
    nodes: Vec<ast::Node>,
) -> Result<Vec<ast::Node>, AiScriptSyntaxError> {
    for node in &nodes {
        JumpStatementValidator {
            has_ancestor_function: false,
            jump_destinations: Vec::new(),
        }
        .visit_node(node)?;
    }
    Ok(nodes)
}
