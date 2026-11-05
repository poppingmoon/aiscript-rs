use std::sync::Arc;

use crate::{error::AiScriptSyntaxError, node as ast};

pub trait Visitor {
    fn visit_node(&mut self, node: &ast::Node) -> Result<Option<ast::Node>, AiScriptSyntaxError> {
        match node {
            ast::Node::Namespace(namespace) => self
                .visit_namespace(namespace)
                .map(|namespace| namespace.map(Arc::new).map(ast::Node::Namespace)),
            ast::Node::Meta(meta) => self
                .visit_meta(meta)
                .map(|meta| meta.map(Arc::new).map(ast::Node::Meta)),
            ast::Node::Statement(statement) => self
                .visit_statement(statement)
                .map(|statement| statement.map(ast::Node::Statement)),
            ast::Node::Expression(expression) => self
                .visit_expression(expression)
                .map(|expression| expression.map(ast::Node::Expression)),
        }
    }

    fn visit_namespace(
        &mut self,
        namespace: &ast::Namespace,
    ) -> Result<Option<ast::Namespace>, AiScriptSyntaxError> {
        if let Some(namespace) = self.callback_namespace(namespace)? {
            let members = namespace
                .members
                .into_iter()
                .map(|member| {
                    let modified = match &member {
                        ast::DefinitionOrNamespace::Definition(definition) => self
                            .visit_definition(definition)?
                            .map(Arc::new)
                            .map(ast::DefinitionOrNamespace::Definition),
                        ast::DefinitionOrNamespace::Namespace(namespace) => self
                            .visit_namespace(namespace)?
                            .map(Arc::new)
                            .map(ast::DefinitionOrNamespace::Namespace),
                    };
                    Ok(modified.unwrap_or(member))
                })
                .collect::<Result<Vec<ast::DefinitionOrNamespace>, AiScriptSyntaxError>>()?;
            Ok(Some(ast::Namespace {
                members,
                ..namespace
            }))
        } else {
            let members = namespace
                .members
                .iter()
                .map(|member| match member {
                    ast::DefinitionOrNamespace::Definition(definition) => {
                        self.visit_definition(definition).map(|definition| {
                            definition
                                .map(Arc::new)
                                .map(ast::DefinitionOrNamespace::Definition)
                        })
                    }
                    ast::DefinitionOrNamespace::Namespace(namespace) => {
                        self.visit_namespace(namespace).map(|namespace| {
                            namespace
                                .map(Arc::new)
                                .map(ast::DefinitionOrNamespace::Namespace)
                        })
                    }
                })
                .collect::<Result<Vec<Option<ast::DefinitionOrNamespace>>, AiScriptSyntaxError>>(
                )?;
            Ok(if members.iter().any(Option::is_some) {
                let members = members
                    .into_iter()
                    .zip(&namespace.members)
                    .map(|(modified, member)| modified.unwrap_or_else(|| member.clone()))
                    .collect();
                Some(ast::Namespace {
                    loc: namespace.loc.clone(),
                    members,
                    name: namespace.name.clone(),
                })
            } else {
                None
            })
        }
    }

    fn visit_meta(&mut self, meta: &ast::Meta) -> Result<Option<ast::Meta>, AiScriptSyntaxError> {
        self.callback_meta(meta)
    }

    fn visit_statement(
        &mut self,
        statement: &ast::Statement,
    ) -> Result<Option<ast::Statement>, AiScriptSyntaxError> {
        let result = self.callback_statement(statement)?;
        let statement = result.as_ref().unwrap_or(statement);
        Ok(match statement {
            ast::Statement::Definition(definition) => self
                .visit_definition(definition)?
                .map(Arc::new)
                .map(ast::Statement::Definition),
            ast::Statement::Return(return_) => {
                let expr = self.visit_expression(&return_.expr)?;
                expr.map(|expr| {
                    ast::Statement::Return(Arc::new(ast::Return {
                        loc: return_.loc.clone(),
                        expr,
                    }))
                })
            }
            ast::Statement::Each(each) => self
                .visit_each(each)?
                .map(Arc::new)
                .map(ast::Statement::Each),
            ast::Statement::For(for_) => {
                self.visit_for(for_)?.map(Arc::new).map(ast::Statement::For)
            }
            ast::Statement::ForLet(for_let) => self
                .visit_for_let(for_let)?
                .map(Arc::new)
                .map(ast::Statement::ForLet),
            ast::Statement::Loop(loop_) => self
                .visit_loop(loop_)?
                .map(Arc::new)
                .map(ast::Statement::Loop),
            ast::Statement::Break(break_) => {
                let expr = break_
                    .expr
                    .as_ref()
                    .map(|expr| self.visit_expression(expr))
                    .transpose()?
                    .flatten();
                expr.map(|expr| {
                    ast::Statement::Break(Arc::new(ast::Break {
                        loc: break_.loc.clone(),
                        label: break_.label.clone(),
                        expr: Some(expr),
                    }))
                })
            }
            ast::Statement::Continue(_) => None,
            ast::Statement::Assign(assign) => {
                let expr = self.visit_expression(&assign.expr)?;
                let dest = self.visit_expression(&assign.dest)?;
                if expr.is_some() || dest.is_some() {
                    Some(ast::Statement::Assign(Arc::new(ast::Assign {
                        loc: assign.loc.clone(),
                        dest: dest.unwrap_or_else(|| assign.dest.clone()),
                        expr: expr.unwrap_or_else(|| assign.expr.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Statement::AddAssign(add_assign) => {
                let expr = self.visit_expression(&add_assign.expr)?;
                let dest = self.visit_expression(&add_assign.dest)?;
                if expr.is_some() || dest.is_some() {
                    Some(ast::Statement::AddAssign(Arc::new(ast::AddAssign {
                        loc: add_assign.loc.clone(),
                        dest: dest.unwrap_or_else(|| add_assign.dest.clone()),
                        expr: expr.unwrap_or_else(|| add_assign.expr.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Statement::SubAssign(sub_assign) => {
                let expr = self.visit_expression(&sub_assign.expr)?;
                let dest = self.visit_expression(&sub_assign.dest)?;
                if expr.is_some() || dest.is_some() {
                    Some(ast::Statement::SubAssign(Arc::new(ast::SubAssign {
                        loc: sub_assign.loc.clone(),
                        dest: dest.unwrap_or_else(|| sub_assign.dest.clone()),
                        expr: expr.unwrap_or_else(|| sub_assign.expr.clone()),
                    })))
                } else {
                    None
                }
            }
        }
        .or(result))
    }

    fn visit_definition(
        &mut self,
        definition: &ast::Definition,
    ) -> Result<Option<ast::Definition>, AiScriptSyntaxError> {
        let dest = self.visit_expression(&definition.dest)?;
        let var_type = definition
            .var_type
            .as_ref()
            .map(|var_type| self.visit_type_source(var_type))
            .transpose()?
            .flatten();
        let expr = self.visit_expression(&definition.expr)?;
        let attr = definition
            .attr
            .as_ref()
            .map(|attr| {
                attr.iter()
                    .map(|attr| self.visit_attribute(attr))
                    .collect::<Result<Vec<Option<ast::Attribute>>, AiScriptSyntaxError>>()
            })
            .transpose()?;
        Ok(
            if dest.is_some()
                || var_type.is_some()
                || expr.is_some()
                || attr
                    .as_ref()
                    .is_some_and(|attr| attr.iter().any(Option::is_some))
            {
                Some(ast::Definition {
                    loc: definition.loc.clone(),
                    dest: dest.unwrap_or_else(|| definition.dest.clone()),
                    var_type: var_type.or_else(|| definition.var_type.clone()),
                    expr: expr.unwrap_or_else(|| definition.expr.clone()),
                    mut_: definition.mut_,
                    attr: attr.map(|attr| {
                        attr.into_iter()
                            .zip(definition.attr.as_ref().unwrap())
                            .map(|(modified, attr)| modified.unwrap_or_else(|| attr.clone()))
                            .collect()
                    }),
                })
            } else {
                None
            },
        )
    }

    fn visit_each(&mut self, each: &ast::Each) -> Result<Option<ast::Each>, AiScriptSyntaxError> {
        let var = self.visit_expression(&each.var)?;
        let items = self.visit_expression(&each.items)?;
        let for_ = match &each.for_ {
            ast::StatementOrExpression::Statement(statement) => self
                .visit_statement(statement)?
                .map(ast::StatementOrExpression::Statement),
            ast::StatementOrExpression::Expression(expression) => self
                .visit_expression(expression)?
                .map(ast::StatementOrExpression::Expression),
        };
        Ok(if var.is_some() || items.is_some() || for_.is_some() {
            Some(ast::Each {
                loc: each.loc.clone(),
                label: each.label.clone(),
                var: var.unwrap_or_else(|| each.var.clone()),
                items: items.unwrap_or_else(|| each.items.clone()),
                for_: for_.unwrap_or_else(|| each.for_.clone()),
            })
        } else {
            None
        })
    }

    fn visit_for(&mut self, for_: &ast::For) -> Result<Option<ast::For>, AiScriptSyntaxError> {
        let times = self.visit_expression(&for_.times)?;
        let for_for = match &for_.for_ {
            ast::StatementOrExpression::Statement(statement) => self
                .visit_statement(statement)?
                .map(ast::StatementOrExpression::Statement),
            ast::StatementOrExpression::Expression(expression) => self
                .visit_expression(expression)?
                .map(ast::StatementOrExpression::Expression),
        };
        Ok(if times.is_some() || for_for.is_some() {
            Some(ast::For {
                loc: for_.loc.clone(),
                label: for_.label.clone(),
                times: times.unwrap_or_else(|| for_.times.clone()),
                for_: for_for.unwrap_or_else(|| for_.for_.clone()),
            })
        } else {
            None
        })
    }

    fn visit_for_let(
        &mut self,
        for_let: &ast::ForLet,
    ) -> Result<Option<ast::ForLet>, AiScriptSyntaxError> {
        let from = self.visit_expression(&for_let.from)?;
        let to = self.visit_expression(&for_let.to)?;
        let for_ = match &for_let.for_ {
            ast::StatementOrExpression::Statement(statement) => self
                .visit_statement(statement)?
                .map(ast::StatementOrExpression::Statement),
            ast::StatementOrExpression::Expression(expression) => self
                .visit_expression(expression)?
                .map(ast::StatementOrExpression::Expression),
        };
        Ok(if from.is_some() || to.is_some() || for_.is_some() {
            Some(ast::ForLet {
                loc: for_let.loc.clone(),
                label: for_let.label.clone(),
                var: for_let.var.clone(),
                from: from.unwrap_or_else(|| for_let.from.clone()),
                to: to.unwrap_or_else(|| for_let.to.clone()),
                for_: for_.unwrap_or_else(|| for_let.for_.clone()),
            })
        } else {
            None
        })
    }

    fn visit_loop(&mut self, loop_: &ast::Loop) -> Result<Option<ast::Loop>, AiScriptSyntaxError> {
        let statements = loop_
            .statements
            .iter()
            .map(|s| match s {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(|statement| statement.map(ast::StatementOrExpression::Statement)),
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(|expression| expression.map(ast::StatementOrExpression::Expression)),
            })
            .collect::<Result<Vec<Option<ast::StatementOrExpression>>, AiScriptSyntaxError>>()?;
        Ok(if statements.iter().any(Option::is_some) {
            Some(ast::Loop {
                loc: loop_.loc.clone(),
                label: loop_.label.clone(),
                statements: statements
                    .into_iter()
                    .zip(&loop_.statements)
                    .map(|(modified, statement)| modified.unwrap_or_else(|| statement.clone()))
                    .collect(),
            })
        } else {
            None
        })
    }

    fn visit_expression(
        &mut self,
        expression: &ast::Expression,
    ) -> Result<Option<ast::Expression>, AiScriptSyntaxError> {
        let result = self.callback_expression(expression)?;
        let expression = result.as_ref().unwrap_or(expression);
        Ok(match expression {
            ast::Expression::If(if_) => self.visit_if(if_)?.map(Arc::new).map(ast::Expression::If),
            ast::Expression::Fn(fn_) => self.visit_fn(fn_)?.map(Arc::new).map(ast::Expression::Fn),
            ast::Expression::Match(match_) => self
                .visit_match(match_)?
                .map(Arc::new)
                .map(ast::Expression::Match),
            ast::Expression::Block(block) => self
                .visit_block(block)?
                .map(Arc::new)
                .map(ast::Expression::Block),
            ast::Expression::Exists(exists) => {
                let identifier = self.visit_identifier(&exists.identifier)?;
                identifier.map(|identifier| {
                    ast::Expression::Exists(Arc::new(ast::Exists {
                        loc: exists.loc.clone(),
                        identifier,
                    }))
                })
            }
            ast::Expression::Tmpl(tmpl) => {
                let tmpl_tmpl = tmpl
                    .tmpl
                    .iter()
                    .map(|item| self.visit_expression(item))
                    .collect::<Result<Vec<Option<ast::Expression>>, AiScriptSyntaxError>>()?;
                if tmpl_tmpl.iter().any(Option::is_some) {
                    Some(ast::Expression::Tmpl(Arc::new(ast::Tmpl {
                        loc: tmpl.loc.clone(),
                        tmpl: tmpl_tmpl
                            .into_iter()
                            .zip(&tmpl.tmpl)
                            .map(|(modified, item)| modified.unwrap_or_else(|| item.clone()))
                            .collect(),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Str(_) => None,
            ast::Expression::Num(_) => None,
            ast::Expression::Bool(_) => None,
            ast::Expression::Null(_) => None,
            ast::Expression::Obj(obj) => {
                let value = obj
                    .value
                    .values()
                    .map(|expression| self.visit_expression(expression))
                    .collect::<Result<Vec<Option<ast::Expression>>, AiScriptSyntaxError>>()?;
                if value.iter().any(Option::is_some) {
                    Some(ast::Expression::Obj(Arc::new(ast::Obj {
                        loc: obj.loc.clone(),
                        value: value
                            .into_iter()
                            .zip(&obj.value)
                            .map(|(modified, (key, expression))| {
                                (key.clone(), modified.unwrap_or_else(|| expression.clone()))
                            })
                            .collect(),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Arr(arr) => {
                let value = arr
                    .value
                    .iter()
                    .map(|expression| self.visit_expression(expression))
                    .collect::<Result<Vec<Option<ast::Expression>>, AiScriptSyntaxError>>()?;
                if value.iter().any(Option::is_some) {
                    Some(ast::Expression::Arr(Arc::new(ast::Arr {
                        loc: arr.loc.clone(),
                        value: value
                            .into_iter()
                            .zip(&arr.value)
                            .map(|(modified, expression)| {
                                modified.unwrap_or_else(|| expression.clone())
                            })
                            .collect(),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Plus(plus) => {
                let expr = self.visit_expression(&plus.expr)?;
                expr.map(|expr| {
                    ast::Expression::Plus(Arc::new(ast::Plus {
                        loc: plus.loc.clone(),
                        expr,
                    }))
                })
            }
            ast::Expression::Minus(minus) => {
                let expr = self.visit_expression(&minus.expr)?;
                expr.map(|expr| {
                    ast::Expression::Minus(Arc::new(ast::Minus {
                        loc: minus.loc.clone(),
                        expr,
                    }))
                })
            }
            ast::Expression::Not(not) => {
                let expr = self.visit_expression(&not.expr)?;
                expr.map(|expr| {
                    ast::Expression::Not(Arc::new(ast::Not {
                        loc: not.loc.clone(),
                        expr,
                    }))
                })
            }
            ast::Expression::Pow(pow) => {
                let left = self.visit_expression(&pow.left)?;
                let right = self.visit_expression(&pow.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Pow(Arc::new(ast::Pow {
                        loc: pow.loc.clone(),
                        left: left.unwrap_or_else(|| pow.left.clone()),
                        right: right.unwrap_or_else(|| pow.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Mul(mul) => {
                let left = self.visit_expression(&mul.left)?;
                let right = self.visit_expression(&mul.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Mul(Arc::new(ast::Mul {
                        loc: mul.loc.clone(),
                        left: left.unwrap_or_else(|| mul.left.clone()),
                        right: right.unwrap_or_else(|| mul.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Div(div) => {
                let left = self.visit_expression(&div.left)?;
                let right = self.visit_expression(&div.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Div(Arc::new(ast::Div {
                        loc: div.loc.clone(),
                        left: left.unwrap_or_else(|| div.left.clone()),
                        right: right.unwrap_or_else(|| div.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Rem(rem) => {
                let left = self.visit_expression(&rem.left)?;
                let right = self.visit_expression(&rem.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Rem(Arc::new(ast::Rem {
                        loc: rem.loc.clone(),
                        left: left.unwrap_or_else(|| rem.left.clone()),
                        right: right.unwrap_or_else(|| rem.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Add(add) => {
                let left = self.visit_expression(&add.left)?;
                let right = self.visit_expression(&add.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Add(Arc::new(ast::Add {
                        loc: add.loc.clone(),
                        left: left.unwrap_or_else(|| add.left.clone()),
                        right: right.unwrap_or_else(|| add.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Sub(sub) => {
                let left = self.visit_expression(&sub.left)?;
                let right = self.visit_expression(&sub.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Sub(Arc::new(ast::Sub {
                        loc: sub.loc.clone(),
                        left: left.unwrap_or_else(|| sub.left.clone()),
                        right: right.unwrap_or_else(|| sub.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Lt(lt) => {
                let left = self.visit_expression(&lt.left)?;
                let right = self.visit_expression(&lt.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Lt(Arc::new(ast::Lt {
                        loc: lt.loc.clone(),
                        left: left.unwrap_or_else(|| lt.left.clone()),
                        right: right.unwrap_or_else(|| lt.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Lteq(lteq) => {
                let left = self.visit_expression(&lteq.left)?;
                let right = self.visit_expression(&lteq.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Lteq(Arc::new(ast::Lteq {
                        loc: lteq.loc.clone(),
                        left: left.unwrap_or_else(|| lteq.left.clone()),
                        right: right.unwrap_or_else(|| lteq.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Gt(gt) => {
                let left = self.visit_expression(&gt.left)?;
                let right = self.visit_expression(&gt.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Gt(Arc::new(ast::Gt {
                        loc: gt.loc.clone(),
                        left: left.unwrap_or_else(|| gt.left.clone()),
                        right: right.unwrap_or_else(|| gt.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Gteq(gteq) => {
                let left = self.visit_expression(&gteq.left)?;
                let right = self.visit_expression(&gteq.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Gteq(Arc::new(ast::Gteq {
                        loc: gteq.loc.clone(),
                        left: left.unwrap_or_else(|| gteq.left.clone()),
                        right: right.unwrap_or_else(|| gteq.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Eq(eq) => {
                let left = self.visit_expression(&eq.left)?;
                let right = self.visit_expression(&eq.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Eq(Arc::new(ast::Eq {
                        loc: eq.loc.clone(),
                        left: left.unwrap_or_else(|| eq.left.clone()),
                        right: right.unwrap_or_else(|| eq.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Neq(neq) => {
                let left = self.visit_expression(&neq.left)?;
                let right = self.visit_expression(&neq.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Neq(Arc::new(ast::Neq {
                        loc: neq.loc.clone(),
                        left: left.unwrap_or_else(|| neq.left.clone()),
                        right: right.unwrap_or_else(|| neq.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::And(and) => {
                let left = self.visit_expression(&and.left)?;
                let right = self.visit_expression(&and.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::And(Arc::new(ast::And {
                        loc: and.loc.clone(),
                        left: left.unwrap_or_else(|| and.left.clone()),
                        right: right.unwrap_or_else(|| and.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Or(or) => {
                let left = self.visit_expression(&or.left)?;
                let right = self.visit_expression(&or.right)?;
                if left.is_some() || right.is_some() {
                    Some(ast::Expression::Or(Arc::new(ast::Or {
                        loc: or.loc.clone(),
                        left: left.unwrap_or_else(|| or.left.clone()),
                        right: right.unwrap_or_else(|| or.right.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Identifier(identifier) => self
                .visit_identifier(identifier)?
                .map(Arc::new)
                .map(ast::Expression::Identifier),
            ast::Expression::Call(call) => {
                let target = self.visit_expression(&call.target)?;
                let args = call
                    .args
                    .iter()
                    .map(|expression| self.visit_expression(expression))
                    .collect::<Result<Vec<Option<ast::Expression>>, AiScriptSyntaxError>>()?;
                if target.is_some() || args.iter().any(Option::is_some) {
                    Some(ast::Expression::Call(Arc::new(ast::Call {
                        loc: call.loc.clone(),
                        target: target.unwrap_or_else(|| call.target.clone()),
                        args: args
                            .into_iter()
                            .zip(&call.args)
                            .map(|(modified, expression)| {
                                modified.unwrap_or_else(|| expression.clone())
                            })
                            .collect(),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Index(index) => {
                let target = self.visit_expression(&index.target)?;
                let index_index = self.visit_expression(&index.index)?;
                if target.is_some() || index_index.is_some() {
                    Some(ast::Expression::Index(Arc::new(ast::Index {
                        loc: index.loc.clone(),
                        target: target.unwrap_or_else(|| index.target.clone()),
                        index: index_index.unwrap_or_else(|| index.index.clone()),
                    })))
                } else {
                    None
                }
            }
            ast::Expression::Prop(prop) => {
                let target = self.visit_expression(&prop.target)?;
                target.map(|target| {
                    ast::Expression::Prop(Arc::new(ast::Prop {
                        loc: prop.loc.clone(),
                        target,
                        name: prop.name.clone(),
                    }))
                })
            }
        }
        .or(result))
    }

    fn visit_if(&mut self, if_: &ast::If) -> Result<Option<ast::If>, AiScriptSyntaxError> {
        let cond = self.visit_expression(&if_.cond)?;
        let then = match &if_.then {
            ast::StatementOrExpression::Statement(statement) => self
                .visit_statement(statement)?
                .map(ast::StatementOrExpression::Statement),
            ast::StatementOrExpression::Expression(expression) => self
                .visit_expression(expression)?
                .map(ast::StatementOrExpression::Expression),
        };
        let elseif = if_
            .elseif
            .iter()
            .map(|elseif| {
                let cond = self.visit_expression(&elseif.cond)?;
                let then = match &elseif.then {
                    ast::StatementOrExpression::Statement(statement) => self
                        .visit_statement(statement)?
                        .map(ast::StatementOrExpression::Statement),
                    ast::StatementOrExpression::Expression(expression) => self
                        .visit_expression(expression)?
                        .map(ast::StatementOrExpression::Expression),
                };
                Ok(if cond.is_some() || then.is_some() {
                    Some(ast::Elseif {
                        cond: cond.unwrap_or_else(|| elseif.cond.clone()),
                        then: then.unwrap_or_else(|| elseif.then.clone()),
                    })
                } else {
                    None
                })
            })
            .collect::<Result<Vec<Option<ast::Elseif>>, AiScriptSyntaxError>>()?;
        let else_ = if_
            .else_
            .as_ref()
            .map(|else_| match else_ {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(|statement| statement.map(ast::StatementOrExpression::Statement)),
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(|expression| expression.map(ast::StatementOrExpression::Expression)),
            })
            .transpose()?
            .flatten();
        Ok(
            if cond.is_some()
                || then.is_some()
                || elseif.iter().any(Option::is_some)
                || else_.is_some()
            {
                Some(ast::If {
                    loc: if_.loc.clone(),
                    label: if_.label.clone(),
                    cond: cond.unwrap_or_else(|| if_.cond.clone()),
                    then: then.unwrap_or_else(|| if_.then.clone()),
                    elseif: elseif
                        .into_iter()
                        .zip(&if_.elseif)
                        .map(|(modified, elseif)| modified.unwrap_or_else(|| elseif.clone()))
                        .collect(),
                    else_: else_.or_else(|| if_.else_.clone()),
                })
            } else {
                None
            },
        )
    }

    fn visit_fn(&mut self, fn_: &ast::Fn) -> Result<Option<ast::Fn>, AiScriptSyntaxError> {
        let params = fn_
            .params
            .iter()
            .map(|param| {
                let dest = self.visit_expression(&param.dest)?;
                let default = param
                    .default
                    .as_ref()
                    .map(|default| self.visit_expression(default))
                    .transpose()?
                    .flatten();
                let arg_type = param
                    .arg_type
                    .as_ref()
                    .map(|arg_type| self.visit_type_source(arg_type))
                    .transpose()?
                    .flatten();
                Ok(
                    if dest.is_some() || default.is_some() || arg_type.is_some() {
                        Some(ast::Param {
                            dest: dest.unwrap_or_else(|| param.dest.clone()),
                            default: default.or_else(|| param.default.clone()),
                            optional: param.optional,
                            arg_type: arg_type.or_else(|| param.arg_type.clone()),
                        })
                    } else {
                        None
                    },
                )
            })
            .collect::<Result<Vec<Option<ast::Param>>, AiScriptSyntaxError>>()?;
        let ret_type = fn_
            .ret_type
            .as_ref()
            .map(|ret_type| self.visit_type_source(ret_type))
            .transpose()?
            .flatten();
        let children = fn_
            .children
            .iter()
            .map(|child| match child {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(|statement| statement.map(ast::StatementOrExpression::Statement)),
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(|expression| expression.map(ast::StatementOrExpression::Expression)),
            })
            .collect::<Result<Vec<Option<ast::StatementOrExpression>>, AiScriptSyntaxError>>()?;
        Ok(
            if params.iter().any(Option::is_some)
                || ret_type.is_some()
                || children.iter().any(Option::is_some)
            {
                Some(ast::Fn {
                    loc: fn_.loc.clone(),
                    type_params: fn_.type_params.clone(),
                    params: params
                        .into_iter()
                        .zip(&fn_.params)
                        .map(|(modified, param)| modified.unwrap_or_else(|| param.clone()))
                        .collect(),
                    ret_type: ret_type.or_else(|| fn_.ret_type.clone()),
                    children: children
                        .into_iter()
                        .zip(&fn_.children)
                        .map(|(modified, child)| modified.unwrap_or_else(|| child.clone()))
                        .collect(),
                })
            } else {
                None
            },
        )
    }

    fn visit_match(
        &mut self,
        match_: &ast::Match,
    ) -> Result<Option<ast::Match>, AiScriptSyntaxError> {
        let about = self.visit_expression(&match_.about)?;
        let qs = match_
            .qs
            .iter()
            .map(|qa| {
                let q = self.visit_expression(&qa.q)?;
                let a = match &qa.a {
                    ast::StatementOrExpression::Statement(statement) => self
                        .visit_statement(statement)?
                        .map(ast::StatementOrExpression::Statement),
                    ast::StatementOrExpression::Expression(expression) => self
                        .visit_expression(expression)?
                        .map(ast::StatementOrExpression::Expression),
                };
                Ok(if q.is_some() || a.is_some() {
                    Some(ast::QA {
                        q: q.unwrap_or_else(|| qa.q.clone()),
                        a: a.unwrap_or_else(|| qa.a.clone()),
                    })
                } else {
                    None
                })
            })
            .collect::<Result<Vec<Option<ast::QA>>, AiScriptSyntaxError>>()?;
        let default = match_
            .default
            .as_ref()
            .map(|default| match default {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(|statement| statement.map(ast::StatementOrExpression::Statement)),
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(|expression| expression.map(ast::StatementOrExpression::Expression)),
            })
            .transpose()?
            .flatten();
        Ok(
            if about.is_some() || qs.iter().any(Option::is_some) || default.is_some() {
                Some(ast::Match {
                    loc: match_.loc.clone(),
                    label: match_.label.clone(),
                    about: about.unwrap_or_else(|| match_.about.clone()),
                    qs: qs
                        .into_iter()
                        .zip(&match_.qs)
                        .map(|(modified, qa)| modified.unwrap_or_else(|| qa.clone()))
                        .collect(),
                    default: default.or_else(|| match_.default.clone()),
                })
            } else {
                None
            },
        )
    }

    fn visit_block(
        &mut self,
        block: &ast::Block,
    ) -> Result<Option<ast::Block>, AiScriptSyntaxError> {
        let statements = block
            .statements
            .iter()
            .map(|child| match child {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(|statement| statement.map(ast::StatementOrExpression::Statement)),
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(|expression| expression.map(ast::StatementOrExpression::Expression)),
            })
            .collect::<Result<Vec<Option<ast::StatementOrExpression>>, AiScriptSyntaxError>>()?;
        Ok(if statements.iter().any(Option::is_some) {
            Some(ast::Block {
                loc: block.loc.clone(),
                label: block.label.clone(),
                statements: statements
                    .into_iter()
                    .zip(&block.statements)
                    .map(|(modified, child)| modified.unwrap_or_else(|| child.clone()))
                    .collect(),
            })
        } else {
            None
        })
    }

    fn visit_identifier(
        &mut self,
        _identifier: &ast::Identifier,
    ) -> Result<Option<ast::Identifier>, AiScriptSyntaxError> {
        Ok(None)
    }

    fn visit_type_source(
        &mut self,
        type_source: &ast::TypeSource,
    ) -> Result<Option<ast::TypeSource>, AiScriptSyntaxError> {
        if let Some(type_source) = self.callback_type_source(type_source)? {
            Ok(Some(match type_source {
                ast::TypeSource::NamedTypeSource(named_type_source) => {
                    let named_type_source: ast::NamedTypeSource = (*named_type_source).clone();
                    let inner = named_type_source
                        .inner
                        .map(|inner| {
                            self.visit_type_source(&inner)
                                .map(|modified| modified.unwrap_or(inner))
                        })
                        .transpose()?;
                    ast::TypeSource::NamedTypeSource(Arc::new(ast::NamedTypeSource {
                        inner,
                        ..named_type_source
                    }))
                }
                ast::TypeSource::FnTypeSource(fn_type_source) => {
                    let fn_type_source: ast::FnTypeSource = (*fn_type_source).clone();
                    let params = fn_type_source
                        .params
                        .into_iter()
                        .map(|param| {
                            self.visit_type_source(&param)
                                .map(|modified| modified.unwrap_or(param))
                        })
                        .collect::<Result<Vec<ast::TypeSource>, AiScriptSyntaxError>>()?;
                    let result = self
                        .visit_type_source(&fn_type_source.result)?
                        .unwrap_or(fn_type_source.result);
                    ast::TypeSource::FnTypeSource(Arc::new(ast::FnTypeSource {
                        params,
                        result,
                        ..fn_type_source
                    }))
                }
                ast::TypeSource::UnionTypeSource(union_type_source) => {
                    let union_type_source: ast::UnionTypeSource = (*union_type_source).clone();
                    let inners = union_type_source
                        .inners
                        .into_iter()
                        .map(|inner| {
                            self.visit_type_source(&inner)
                                .map(|modified| modified.unwrap_or(inner))
                        })
                        .collect::<Result<Vec<ast::TypeSource>, AiScriptSyntaxError>>()?;
                    ast::TypeSource::UnionTypeSource(Arc::new(ast::UnionTypeSource {
                        inners,
                        ..union_type_source
                    }))
                }
            }))
        } else {
            Ok(match type_source {
                ast::TypeSource::NamedTypeSource(named_type_source) => {
                    let inner = named_type_source
                        .inner
                        .as_ref()
                        .map(|inner| self.visit_type_source(inner))
                        .transpose()
                        .map(Option::flatten)?;
                    if inner.is_some() {
                        Some(ast::TypeSource::NamedTypeSource(Arc::new(
                            ast::NamedTypeSource {
                                loc: named_type_source.loc.clone(),
                                name: named_type_source.name.clone(),
                                inner,
                            },
                        )))
                    } else {
                        None
                    }
                }
                ast::TypeSource::FnTypeSource(fn_type_source) => {
                    let params = fn_type_source
                        .params
                        .iter()
                        .map(|param| self.visit_type_source(param))
                        .collect::<Result<Vec<Option<ast::TypeSource>>, AiScriptSyntaxError>>()?;
                    let result = self.visit_type_source(&fn_type_source.result)?;
                    if params.iter().any(Option::is_some) || result.is_some() {
                        Some(ast::TypeSource::FnTypeSource(Arc::new(ast::FnTypeSource {
                            loc: fn_type_source.loc.clone(),
                            type_params: fn_type_source.type_params.clone(),
                            params: params
                                .into_iter()
                                .zip(&fn_type_source.params)
                                .map(|(modified, param)| modified.unwrap_or_else(|| param.clone()))
                                .collect(),
                            result: result.unwrap_or_else(|| fn_type_source.result.clone()),
                        })))
                    } else {
                        None
                    }
                }
                ast::TypeSource::UnionTypeSource(union_type_source) => {
                    let inners = union_type_source
                        .inners
                        .iter()
                        .map(|inner| self.visit_type_source(inner))
                        .collect::<Result<Vec<Option<ast::TypeSource>>, AiScriptSyntaxError>>()?;
                    if inners.iter().any(Option::is_some) {
                        Some(ast::TypeSource::UnionTypeSource(Arc::new(
                            ast::UnionTypeSource {
                                loc: union_type_source.loc.clone(),
                                inners: inners
                                    .into_iter()
                                    .zip(&union_type_source.inners)
                                    .map(|(modified, inner)| {
                                        modified.unwrap_or_else(|| inner.clone())
                                    })
                                    .collect(),
                            },
                        )))
                    } else {
                        None
                    }
                }
            })
        }
    }

    fn visit_attribute(
        &mut self,
        attribute: &ast::Attribute,
    ) -> Result<Option<ast::Attribute>, AiScriptSyntaxError> {
        self.callback_attribute(attribute)
    }

    fn callback_namespace(
        &mut self,
        _namespace: &ast::Namespace,
    ) -> Result<Option<ast::Namespace>, AiScriptSyntaxError> {
        Ok(None)
    }

    fn callback_meta(
        &mut self,
        _meta: &ast::Meta,
    ) -> Result<Option<ast::Meta>, AiScriptSyntaxError> {
        Ok(None)
    }

    fn callback_statement(
        &mut self,
        _statement: &ast::Statement,
    ) -> Result<Option<ast::Statement>, AiScriptSyntaxError> {
        Ok(None)
    }

    fn callback_expression(
        &mut self,
        _expression: &ast::Expression,
    ) -> Result<Option<ast::Expression>, AiScriptSyntaxError> {
        Ok(None)
    }

    fn callback_type_source(
        &mut self,
        _type_source: &ast::TypeSource,
    ) -> Result<Option<ast::TypeSource>, AiScriptSyntaxError> {
        Ok(None)
    }

    fn callback_attribute(
        &mut self,
        _attribute: &ast::Attribute,
    ) -> Result<Option<ast::Attribute>, AiScriptSyntaxError> {
        Ok(None)
    }
}
