use indexmap::IndexMap;

use crate::{error::AiScriptError, parser::node as cst};

pub trait Visitor {
    fn visit_node(&self, node: cst::Node) -> Result<cst::Node, AiScriptError> {
        match node {
            cst::Node::Namespace(namespace) => {
                self.visit_namespace(namespace).map(cst::Node::Namespace)
            }
            cst::Node::Meta(meta) => self.visit_meta(meta).map(cst::Node::Meta),
            cst::Node::Statement(statement) => {
                self.visit_statement(statement).map(cst::Node::Statement)
            }
            cst::Node::Expression(expression) => {
                self.visit_expression(expression).map(cst::Node::Expression)
            }
        }
    }

    fn visit_namespace(&self, namespace: cst::Namespace) -> Result<cst::Namespace, AiScriptError> {
        let namespace = self.callback_namespace(namespace)?;
        Ok(cst::Namespace {
            members: namespace
                .members
                .into_iter()
                .map(|member| match member {
                    cst::DefinitionOrNamespace::Definition(definition) => self
                        .visit_statement(cst::Statement::Definition(definition))
                        .map(|statement| {
                            if let cst::Statement::Definition(definition) = statement {
                                cst::DefinitionOrNamespace::Definition(definition)
                            } else {
                                panic!()
                            }
                        }),
                    cst::DefinitionOrNamespace::Namespace(namespace) => self
                        .visit_namespace(namespace)
                        .map(cst::DefinitionOrNamespace::Namespace),
                })
                .collect::<Result<Vec<cst::DefinitionOrNamespace>, AiScriptError>>()?,
            ..namespace
        })
    }

    fn visit_meta(&self, meta: cst::Meta) -> Result<cst::Meta, AiScriptError> {
        let meta = self.callback_meta(meta)?;
        Ok(meta)
    }

    fn visit_statement(&self, statement: cst::Statement) -> Result<cst::Statement, AiScriptError> {
        let statement = self.callback_statement(statement)?;
        Ok(match statement {
            cst::Statement::Definition(definition) => cst::Statement::Definition(cst::Definition {
                expr: self.visit_expression(definition.expr)?,
                ..definition
            }),
            cst::Statement::Return(return_) => cst::Statement::Return(cst::Return {
                expr: self.visit_expression(return_.expr)?,
                ..return_
            }),
            cst::Statement::Attribute(_) => statement,
            cst::Statement::Each(each) => cst::Statement::Each(cst::Each {
                items: self.visit_expression(each.items)?,
                for_: match *each.for_ {
                    cst::StatementOrExpression::Statement(statement) => self
                        .visit_statement(statement)
                        .map(cst::StatementOrExpression::Statement)?,
                    cst::StatementOrExpression::Expression(expression) => self
                        .visit_expression(expression)
                        .map(cst::StatementOrExpression::Expression)?,
                }
                .into(),
                ..each
            }),
            cst::Statement::For(for_) => cst::Statement::For(cst::For {
                from: for_
                    .from
                    .map(|expression| self.visit_expression(expression))
                    .map_or(Ok(None), |r| r.map(Some))?,
                to: for_
                    .to
                    .map(|expression| self.visit_expression(expression))
                    .map_or(Ok(None), |r| r.map(Some))?,
                times: for_
                    .times
                    .map(|expression| self.visit_expression(expression))
                    .map_or(Ok(None), |r| r.map(Some))?,
                for_: match *for_.for_ {
                    cst::StatementOrExpression::Statement(statement) => self
                        .visit_statement(statement)
                        .map(cst::StatementOrExpression::Statement)?,
                    cst::StatementOrExpression::Expression(expression) => self
                        .visit_expression(expression)
                        .map(cst::StatementOrExpression::Expression)?,
                }
                .into(),
                ..for_
            }),
            cst::Statement::Loop(loop_) => cst::Statement::Loop(cst::Loop {
                statements: loop_
                    .statements
                    .into_iter()
                    .map(|s| match s {
                        cst::StatementOrExpression::Statement(statement) => self
                            .visit_statement(statement)
                            .map(cst::StatementOrExpression::Statement),
                        cst::StatementOrExpression::Expression(expression) => self
                            .visit_expression(expression)
                            .map(cst::StatementOrExpression::Expression),
                    })
                    .collect::<Result<Vec<cst::StatementOrExpression>, AiScriptError>>()?,
                ..loop_
            }),
            cst::Statement::Break(_) => statement,
            cst::Statement::Continue(_) => statement,
            cst::Statement::Assign(assign) => cst::Statement::Assign(cst::Assign {
                expr: self.visit_expression(assign.expr)?,
                dest: self.visit_expression(assign.dest)?,
                ..assign
            }),
            cst::Statement::AddAssign(add_assign) => cst::Statement::AddAssign(cst::AddAssign {
                expr: self.visit_expression(add_assign.expr)?,
                dest: self.visit_expression(add_assign.dest)?,
                ..add_assign
            }),
            cst::Statement::SubAssign(sub_assign) => cst::Statement::SubAssign(cst::SubAssign {
                expr: self.visit_expression(sub_assign.expr)?,
                dest: self.visit_expression(sub_assign.dest)?,
                ..sub_assign
            }),
        })
    }

    fn visit_expression(
        &self,
        expression: cst::Expression,
    ) -> Result<cst::Expression, AiScriptError> {
        let expression = self.callback_expression(expression)?;
        Ok(match expression {
            cst::Expression::Not(not) => cst::Expression::Not(cst::Not {
                expr: self.visit_expression(*not.expr)?.into(),
                ..not
            }),
            cst::Expression::And(and) => cst::Expression::And(cst::And {
                left: self.visit_expression(*and.left)?.into(),
                right: self.visit_expression(*and.right)?.into(),
                ..and
            }),
            cst::Expression::Or(or) => cst::Expression::Or(cst::Or {
                left: self.visit_expression(*or.left)?.into(),
                right: self.visit_expression(*or.right)?.into(),
                ..or
            }),
            cst::Expression::If(if_) => cst::Expression::If(cst::If {
                cond: self.visit_expression(*if_.cond)?.into(),
                then: match *if_.then {
                    cst::StatementOrExpression::Statement(statement) => self
                        .visit_statement(statement)
                        .map(cst::StatementOrExpression::Statement)?,
                    cst::StatementOrExpression::Expression(expression) => self
                        .visit_expression(expression)
                        .map(cst::StatementOrExpression::Expression)?,
                }
                .into(),
                elseif: if_
                    .elseif
                    .into_iter()
                    .map(|elseif| {
                        Ok(cst::Elseif {
                            cond: self.visit_expression(elseif.cond)?,
                            then: match elseif.then {
                                cst::StatementOrExpression::Statement(statement) => self
                                    .visit_statement(statement)
                                    .map(cst::StatementOrExpression::Statement)?,
                                cst::StatementOrExpression::Expression(expression) => self
                                    .visit_expression(expression)
                                    .map(cst::StatementOrExpression::Expression)?,
                            },
                        })
                    })
                    .collect::<Result<Vec<cst::Elseif>, AiScriptError>>()?,
                else_: if_
                    .else_
                    .map(|else_| match *else_ {
                        cst::StatementOrExpression::Statement(statement) => self
                            .visit_statement(statement)
                            .map(cst::StatementOrExpression::Statement),
                        cst::StatementOrExpression::Expression(expression) => self
                            .visit_expression(expression)
                            .map(cst::StatementOrExpression::Expression),
                    })
                    .map_or(Ok(None), |r| r.map(Some))?
                    .map(Into::into),
                ..if_
            }),
            cst::Expression::Fn(fn_) => cst::Expression::Fn(cst::Fn_ {
                children: fn_
                    .children
                    .into_iter()
                    .map(|child| match child {
                        cst::StatementOrExpression::Statement(statement) => self
                            .visit_statement(statement)
                            .map(cst::StatementOrExpression::Statement),
                        cst::StatementOrExpression::Expression(expression) => self
                            .visit_expression(expression)
                            .map(cst::StatementOrExpression::Expression),
                    })
                    .collect::<Result<Vec<cst::StatementOrExpression>, AiScriptError>>()?,
                chain: fn_
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..fn_
            }),
            cst::Expression::Match(match_) => cst::Expression::Match(cst::Match {
                about: self.visit_expression(*match_.about)?.into(),
                qs: match_
                    .qs
                    .into_iter()
                    .map(|cst::QA { q, a }| {
                        Ok(cst::QA {
                            q: self.visit_expression(q)?,
                            a: match a {
                                cst::StatementOrExpression::Statement(statement) => self
                                    .visit_statement(statement)
                                    .map(cst::StatementOrExpression::Statement)?,
                                cst::StatementOrExpression::Expression(expression) => self
                                    .visit_expression(expression)
                                    .map(cst::StatementOrExpression::Expression)?,
                            },
                        })
                    })
                    .collect::<Result<Vec<cst::QA>, AiScriptError>>()?,
                default: match_
                    .default
                    .map(|default| match *default {
                        cst::StatementOrExpression::Statement(statement) => self
                            .visit_statement(statement)
                            .map(cst::StatementOrExpression::Statement),
                        cst::StatementOrExpression::Expression(expression) => self
                            .visit_expression(expression)
                            .map(cst::StatementOrExpression::Expression),
                    })
                    .map_or(Ok(None), |r| r.map(Some))?
                    .map(Into::into),
                chain: match_
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..match_
            }),
            cst::Expression::Block(block) => cst::Expression::Block(cst::Block {
                statements: block
                    .statements
                    .into_iter()
                    .map(|child| match child {
                        cst::StatementOrExpression::Statement(statement) => self
                            .visit_statement(statement)
                            .map(cst::StatementOrExpression::Statement),
                        cst::StatementOrExpression::Expression(expression) => self
                            .visit_expression(expression)
                            .map(cst::StatementOrExpression::Expression),
                    })
                    .collect::<Result<Vec<cst::StatementOrExpression>, AiScriptError>>()?,
                chain: block
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..block
            }),
            cst::Expression::Exists(exists) => cst::Expression::Exists(cst::Exists {
                identifier: self
                    .visit_expression(cst::Expression::Identifier(exists.identifier))
                    .map(|expression| {
                        if let cst::Expression::Identifier(identifier) = expression {
                            identifier
                        } else {
                            panic!()
                        }
                    })?,
                chain: exists
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..exists
            }),
            cst::Expression::Tmpl(tmpl) => cst::Expression::Tmpl(cst::Tmpl {
                tmpl: tmpl
                    .tmpl
                    .into_iter()
                    .map(|tmpl| match tmpl {
                        cst::StringOrExpression::String(_) => Ok(tmpl),
                        cst::StringOrExpression::Expression(expression) => self
                            .visit_expression(expression)
                            .map(cst::StringOrExpression::Expression),
                    })
                    .collect::<Result<Vec<cst::StringOrExpression>, AiScriptError>>()?,
                chain: tmpl
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..tmpl
            }),
            cst::Expression::Str(str) => cst::Expression::Str(cst::Str {
                chain: str
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..str
            }),
            cst::Expression::Num(num) => cst::Expression::Num(cst::Num {
                chain: num
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..num
            }),
            cst::Expression::Bool(bool) => cst::Expression::Bool(cst::Bool {
                chain: bool
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..bool
            }),
            cst::Expression::Null(null) => cst::Expression::Null(cst::Null {
                chain: null
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..null
            }),
            cst::Expression::Obj(obj) => cst::Expression::Obj(cst::Obj {
                value: obj
                    .value
                    .into_iter()
                    .map(|(key, expression)| Ok((key, self.visit_expression(expression)?)))
                    .collect::<Result<IndexMap<String, cst::Expression>, AiScriptError>>()?,
                chain: obj
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..obj
            }),
            cst::Expression::Arr(arr) => cst::Expression::Arr(cst::Arr {
                value: arr
                    .value
                    .into_iter()
                    .map(|expression| self.visit_expression(expression))
                    .collect::<Result<Vec<cst::Expression>, AiScriptError>>()?,
                chain: arr
                    .chain
                    .map(|chain| {
                        chain
                            .into_iter()
                            .map(|chain_member| self.visit_chain_member(chain_member))
                            .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
                ..arr
            }),
            cst::Expression::Identifier(identifier) => {
                cst::Expression::Identifier(cst::Identifier {
                    chain: identifier
                        .chain
                        .map(|chain| {
                            chain
                                .into_iter()
                                .map(|chain_member| self.visit_chain_member(chain_member))
                                .collect::<Result<Vec<cst::ChainMember>, AiScriptError>>()
                        })
                        .map_or(Ok(None), |r| r.map(Some))?,
                    ..identifier
                })
            }
            cst::Expression::Call(call) => cst::Expression::Call(cst::Call {
                target: self.visit_expression(*call.target)?.into(),
                args: call
                    .args
                    .into_iter()
                    .map(|expression| self.visit_expression(expression))
                    .collect::<Result<Vec<cst::Expression>, AiScriptError>>()?,
                ..call
            }),
            cst::Expression::Index(index) => cst::Expression::Index(cst::Index {
                target: self.visit_expression(*index.target)?.into(),
                index: self.visit_expression(*index.index)?.into(),
                ..index
            }),
            cst::Expression::Prop(prop) => cst::Expression::Prop(cst::Prop {
                target: self.visit_expression(*prop.target)?.into(),
                ..prop
            }),
        })
    }

    fn visit_chain_member(
        &self,
        chain_member: cst::ChainMember,
    ) -> Result<cst::ChainMember, AiScriptError> {
        let chain_member = self.callback_chain_member(chain_member)?;
        Ok(match chain_member {
            cst::ChainMember::CallChain(call_chain) => {
                cst::ChainMember::CallChain(cst::CallChain {
                    args: call_chain
                        .args
                        .into_iter()
                        .map(|expression| self.visit_expression(expression))
                        .collect::<Result<Vec<cst::Expression>, AiScriptError>>()?,
                    ..call_chain
                })
            }
            cst::ChainMember::IndexChain(index_chain) => {
                cst::ChainMember::IndexChain(cst::IndexChain {
                    index: self.visit_expression(index_chain.index)?,
                    ..index_chain
                })
            }
            cst::ChainMember::PropChain(_) => chain_member,
        })
    }

    fn callback_namespace(
        &self,
        namespace: cst::Namespace,
    ) -> Result<cst::Namespace, AiScriptError> {
        Ok(namespace)
    }

    fn callback_meta(&self, meta: cst::Meta) -> Result<cst::Meta, AiScriptError> {
        Ok(meta)
    }

    fn callback_statement(
        &self,
        statement: cst::Statement,
    ) -> Result<cst::Statement, AiScriptError> {
        Ok(statement)
    }

    fn callback_expression(
        &self,
        expression: cst::Expression,
    ) -> Result<cst::Expression, AiScriptError> {
        Ok(expression)
    }

    fn callback_chain_member(
        &self,
        chain_member: cst::ChainMember,
    ) -> Result<cst::ChainMember, AiScriptError> {
        Ok(chain_member)
    }
}
