use indexmap::IndexMap;

use crate::{error::AiScriptSyntaxError, node as ast};

pub trait Visitor {
    fn visit_node(&mut self, node: ast::Node) -> Result<ast::Node, AiScriptSyntaxError> {
        match node {
            ast::Node::Namespace(namespace) => self
                .visit_namespace(*namespace)
                .map(Into::into)
                .map(ast::Node::Namespace),
            ast::Node::Meta(meta) => self.visit_meta(*meta).map(Into::into).map(ast::Node::Meta),
            ast::Node::Statement(statement) => {
                self.visit_statement(statement).map(ast::Node::Statement)
            }
            ast::Node::Expression(expression) => {
                self.visit_expression(expression).map(ast::Node::Expression)
            }
        }
    }

    fn visit_namespace(
        &mut self,
        namespace: ast::Namespace,
    ) -> Result<ast::Namespace, AiScriptSyntaxError> {
        let namespace = self.callback_namespace(namespace)?;
        Ok(ast::Namespace {
            members: namespace
                .members
                .into_iter()
                .map(|member| match member {
                    ast::DefinitionOrNamespace::Definition(definition) => self
                        .visit_definition(*definition)
                        .map(Into::into)
                        .map(ast::DefinitionOrNamespace::Definition),
                    ast::DefinitionOrNamespace::Namespace(namespace) => self
                        .visit_namespace(*namespace)
                        .map(Into::into)
                        .map(ast::DefinitionOrNamespace::Namespace),
                })
                .collect::<Result<Vec<ast::DefinitionOrNamespace>, AiScriptSyntaxError>>()?,
            ..namespace
        })
    }

    fn visit_meta(&mut self, meta: ast::Meta) -> Result<ast::Meta, AiScriptSyntaxError> {
        self.callback_meta(meta)
    }

    fn visit_statement(
        &mut self,
        statement: ast::Statement,
    ) -> Result<ast::Statement, AiScriptSyntaxError> {
        let statement = self.callback_statement(statement)?;
        Ok(match statement {
            ast::Statement::Definition(definition) => {
                ast::Statement::Definition(self.visit_definition(*definition)?.into())
            }
            ast::Statement::Return(return_) => {
                let return_ = *return_;
                ast::Statement::Return(
                    ast::Return {
                        expr: self.visit_expression(return_.expr)?,
                        ..return_
                    }
                    .into(),
                )
            }
            ast::Statement::Each(each) => ast::Statement::Each(self.visit_each(*each)?.into()),
            ast::Statement::For(for_) => ast::Statement::For(self.visit_for(*for_)?.into()),
            ast::Statement::ForLet(for_let) => {
                ast::Statement::ForLet(self.visit_for_let(*for_let)?.into())
            }
            ast::Statement::Loop(loop_) => ast::Statement::Loop(self.visit_loop(*loop_)?.into()),
            ast::Statement::Break(break_) => {
                let break_ = *break_;
                ast::Statement::Break(
                    ast::Break {
                        expr: break_
                            .expr
                            .map(|expr| self.visit_expression(expr))
                            .map_or(Ok(None), |r| r.map(Some))?,
                        ..break_
                    }
                    .into(),
                )
            }
            ast::Statement::Continue(_) => statement,
            ast::Statement::Assign(assign) => {
                let assign = *assign;
                ast::Statement::Assign(
                    ast::Assign {
                        expr: self.visit_expression(assign.expr)?,
                        dest: self.visit_expression(assign.dest)?,
                        ..assign
                    }
                    .into(),
                )
            }
            ast::Statement::AddAssign(add_assign) => {
                let add_assign = *add_assign;
                ast::Statement::AddAssign(
                    ast::AddAssign {
                        expr: self.visit_expression(add_assign.expr)?,
                        dest: self.visit_expression(add_assign.dest)?,
                        ..add_assign
                    }
                    .into(),
                )
            }
            ast::Statement::SubAssign(sub_assign) => {
                let sub_assign = *sub_assign;
                ast::Statement::SubAssign(
                    ast::SubAssign {
                        expr: self.visit_expression(sub_assign.expr)?,
                        dest: self.visit_expression(sub_assign.dest)?,
                        ..sub_assign
                    }
                    .into(),
                )
            }
        })
    }

    fn visit_definition(
        &mut self,
        definition: ast::Definition,
    ) -> Result<ast::Definition, AiScriptSyntaxError> {
        Ok(ast::Definition {
            dest: self.visit_expression(definition.dest)?,
            var_type: definition
                .var_type
                .map(|var_type| self.visit_type_source(var_type))
                .map_or(Ok(None), |r| r.map(Some))?,
            expr: self.visit_expression(definition.expr)?,
            attr: definition
                .attr
                .map(|attr| {
                    attr.into_iter()
                        .map(|attr| self.visit_attribute(attr))
                        .collect::<Result<Vec<ast::Attribute>, AiScriptSyntaxError>>()
                })
                .map_or(Ok(None), |r| r.map(Some))?,
            ..definition
        })
    }

    fn visit_each(&mut self, each: ast::Each) -> Result<ast::Each, AiScriptSyntaxError> {
        Ok(ast::Each {
            var: self.visit_expression(each.var)?,
            items: self.visit_expression(each.items)?,
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
        })
    }

    fn visit_for(&mut self, for_: ast::For) -> Result<ast::For, AiScriptSyntaxError> {
        Ok(ast::For {
            times: self.visit_expression(for_.times)?,
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
        })
    }

    fn visit_for_let(&mut self, for_let: ast::ForLet) -> Result<ast::ForLet, AiScriptSyntaxError> {
        Ok(ast::ForLet {
            from: self.visit_expression(for_let.from)?,
            to: self.visit_expression(for_let.to)?,
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
        })
    }

    fn visit_loop(&mut self, loop_: ast::Loop) -> Result<ast::Loop, AiScriptSyntaxError> {
        Ok(ast::Loop {
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
        })
    }

    fn visit_expression(
        &mut self,
        expression: ast::Expression,
    ) -> Result<ast::Expression, AiScriptSyntaxError> {
        let expression = self.callback_expression(expression)?;
        Ok(match expression {
            ast::Expression::If(if_) => ast::Expression::If(self.visit_if(*if_)?.into()),
            ast::Expression::Fn(fn_) => ast::Expression::Fn(self.visit_fn(*fn_)?.into()),
            ast::Expression::Match(match_) => {
                ast::Expression::Match(self.visit_match(*match_)?.into())
            }
            ast::Expression::Block(block) => {
                ast::Expression::Block(self.visit_block(*block)?.into())
            }
            ast::Expression::Exists(exists) => {
                let exists = *exists;
                ast::Expression::Exists(
                    ast::Exists {
                        identifier: self.visit_identifier(exists.identifier)?,
                        ..exists
                    }
                    .into(),
                )
            }
            ast::Expression::Tmpl(tmpl) => ast::Expression::Tmpl(
                {
                    let tmpl = *tmpl;
                    ast::Tmpl {
                        tmpl: tmpl
                            .tmpl
                            .into_iter()
                            .map(|item| self.visit_expression(item))
                            .collect::<Result<Vec<ast::Expression>, AiScriptSyntaxError>>()?,
                        ..tmpl
                    }
                }
                .into(),
            ),
            ast::Expression::Str(_) => expression,
            ast::Expression::Num(_) => expression,
            ast::Expression::Bool(_) => expression,
            ast::Expression::Null(_) => expression,
            ast::Expression::Obj(obj) => {
                let obj = *obj;
                ast::Expression::Obj(ast::Obj {
                value: obj
                    .value
                    .into_iter()
                    .map(|(key, expression)| Ok((key, self.visit_expression(expression)?)))
                    .collect::<Result<IndexMap<String, ast::Expression>, AiScriptSyntaxError>>()?,
                ..obj
            }.into())
            }
            ast::Expression::Arr(arr) => {
                let arr = *arr;
                ast::Expression::Arr(
                    ast::Arr {
                        value: arr
                            .value
                            .into_iter()
                            .map(|expression| self.visit_expression(expression))
                            .collect::<Result<Vec<ast::Expression>, AiScriptSyntaxError>>()?,
                        ..arr
                    }
                    .into(),
                )
            }
            ast::Expression::Plus(plus) => {
                let plus = *plus;
                ast::Expression::Plus(
                    ast::Plus {
                        expr: self.visit_expression(*plus.expr)?.into(),
                        ..plus
                    }
                    .into(),
                )
            }
            ast::Expression::Minus(minus) => {
                let minus = *minus;
                ast::Expression::Minus(
                    ast::Minus {
                        expr: self.visit_expression(*minus.expr)?.into(),
                        ..minus
                    }
                    .into(),
                )
            }
            ast::Expression::Not(not) => {
                let not = *not;
                ast::Expression::Not(
                    ast::Not {
                        expr: self.visit_expression(*not.expr)?.into(),
                        ..not
                    }
                    .into(),
                )
            }
            ast::Expression::Pow(pow) => {
                let pow = *pow;
                ast::Expression::Pow(
                    ast::Pow {
                        left: self.visit_expression(*pow.left)?.into(),
                        right: self.visit_expression(*pow.right)?.into(),
                        ..pow
                    }
                    .into(),
                )
            }
            ast::Expression::Mul(mul) => {
                let mul = *mul;
                ast::Expression::Mul(
                    ast::Mul {
                        left: self.visit_expression(*mul.left)?.into(),
                        right: self.visit_expression(*mul.right)?.into(),
                        ..mul
                    }
                    .into(),
                )
            }
            ast::Expression::Div(div) => {
                let div = *div;
                ast::Expression::Div(
                    ast::Div {
                        left: self.visit_expression(*div.left)?.into(),
                        right: self.visit_expression(*div.right)?.into(),
                        ..div
                    }
                    .into(),
                )
            }
            ast::Expression::Rem(rem) => {
                let rem = *rem;
                ast::Expression::Rem(
                    ast::Rem {
                        left: self.visit_expression(*rem.left)?.into(),
                        right: self.visit_expression(*rem.right)?.into(),
                        ..rem
                    }
                    .into(),
                )
            }
            ast::Expression::Add(add) => {
                let add = *add;
                ast::Expression::Add(
                    ast::Add {
                        left: self.visit_expression(*add.left)?.into(),
                        right: self.visit_expression(*add.right)?.into(),
                        ..add
                    }
                    .into(),
                )
            }
            ast::Expression::Sub(sub) => {
                let sub = *sub;
                ast::Expression::Sub(
                    ast::Sub {
                        left: self.visit_expression(*sub.left)?.into(),
                        right: self.visit_expression(*sub.right)?.into(),
                        ..sub
                    }
                    .into(),
                )
            }
            ast::Expression::Lt(lt) => {
                let lt = *lt;
                ast::Expression::Lt(
                    ast::Lt {
                        left: self.visit_expression(*lt.left)?.into(),
                        right: self.visit_expression(*lt.right)?.into(),
                        ..lt
                    }
                    .into(),
                )
            }
            ast::Expression::Lteq(lteq) => {
                let lteq = *lteq;
                ast::Expression::Lteq(
                    ast::Lteq {
                        left: self.visit_expression(*lteq.left)?.into(),
                        right: self.visit_expression(*lteq.right)?.into(),
                        ..lteq
                    }
                    .into(),
                )
            }
            ast::Expression::Gt(gt) => {
                let gt = *gt;
                ast::Expression::Gt(
                    ast::Gt {
                        left: self.visit_expression(*gt.left)?.into(),
                        right: self.visit_expression(*gt.right)?.into(),
                        ..gt
                    }
                    .into(),
                )
            }
            ast::Expression::Gteq(gteq) => {
                let gteq = *gteq;
                ast::Expression::Gteq(
                    ast::Gteq {
                        left: self.visit_expression(*gteq.left)?.into(),
                        right: self.visit_expression(*gteq.right)?.into(),
                        ..gteq
                    }
                    .into(),
                )
            }
            ast::Expression::Eq(eq) => {
                let eq = *eq;
                ast::Expression::Eq(
                    ast::Eq {
                        left: self.visit_expression(*eq.left)?.into(),
                        right: self.visit_expression(*eq.right)?.into(),
                        ..eq
                    }
                    .into(),
                )
            }
            ast::Expression::Neq(neq) => {
                let neq = *neq;
                ast::Expression::Neq(
                    ast::Neq {
                        left: self.visit_expression(*neq.left)?.into(),
                        right: self.visit_expression(*neq.right)?.into(),
                        ..neq
                    }
                    .into(),
                )
            }
            ast::Expression::And(and) => {
                let and = *and;
                ast::Expression::And(
                    ast::And {
                        left: self.visit_expression(*and.left)?.into(),
                        right: self.visit_expression(*and.right)?.into(),
                        ..and
                    }
                    .into(),
                )
            }
            ast::Expression::Or(or) => {
                let or = *or;
                ast::Expression::Or(
                    ast::Or {
                        left: self.visit_expression(*or.left)?.into(),
                        right: self.visit_expression(*or.right)?.into(),
                        ..or
                    }
                    .into(),
                )
            }
            ast::Expression::Identifier(identifier) => {
                ast::Expression::Identifier(self.visit_identifier(*identifier)?.into())
            }
            ast::Expression::Call(call) => {
                let call = *call;
                ast::Expression::Call(
                    ast::Call {
                        target: self.visit_expression(*call.target)?.into(),
                        args: call
                            .args
                            .into_iter()
                            .map(|expression| self.visit_expression(expression))
                            .collect::<Result<Vec<ast::Expression>, AiScriptSyntaxError>>()?,
                        ..call
                    }
                    .into(),
                )
            }
            ast::Expression::Index(index) => {
                let index = *index;
                ast::Expression::Index(
                    ast::Index {
                        target: self.visit_expression(*index.target)?.into(),
                        index: self.visit_expression(*index.index)?.into(),
                        ..index
                    }
                    .into(),
                )
            }
            ast::Expression::Prop(prop) => {
                let prop = *prop;
                ast::Expression::Prop(
                    ast::Prop {
                        target: self.visit_expression(*prop.target)?.into(),
                        ..prop
                    }
                    .into(),
                )
            }
        })
    }

    fn visit_if(&mut self, if_: ast::If) -> Result<ast::If, AiScriptSyntaxError> {
        Ok(ast::If {
            cond: self.visit_expression(*if_.cond)?.into(),
            then: match *if_.then {
                ast::StatementOrExpression::Statement(statement) => self
                    .visit_statement(statement)
                    .map(ast::StatementOrExpression::Statement)?,
                ast::StatementOrExpression::Expression(expression) => self
                    .visit_expression(expression)
                    .map(ast::StatementOrExpression::Expression)?,
            }
            .into(),
            elseif: if_
                .elseif
                .into_iter()
                .map(|elseif| {
                    Ok(ast::Elseif {
                        cond: self.visit_expression(elseif.cond)?,
                        then: match elseif.then {
                            ast::StatementOrExpression::Statement(statement) => self
                                .visit_statement(statement)
                                .map(ast::StatementOrExpression::Statement)?,
                            ast::StatementOrExpression::Expression(expression) => self
                                .visit_expression(expression)
                                .map(ast::StatementOrExpression::Expression)?,
                        },
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
        })
    }

    fn visit_fn(&mut self, fn_: ast::Fn) -> Result<ast::Fn, AiScriptSyntaxError> {
        Ok(ast::Fn {
            params: fn_
                .params
                .into_iter()
                .map(|param| {
                    Ok(ast::Param {
                        dest: self.visit_expression(param.dest)?,
                        default: param
                            .default
                            .map(|default| self.visit_expression(default))
                            .map_or(Ok(None), |r| r.map(Some))?,
                        arg_type: param
                            .arg_type
                            .map(|arg_type| self.visit_type_source(arg_type))
                            .map_or(Ok(None), |r| r.map(Some))?,
                        ..param
                    })
                })
                .collect::<Result<Vec<ast::Param>, AiScriptSyntaxError>>()?,
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
        })
    }

    fn visit_match(&mut self, match_: ast::Match) -> Result<ast::Match, AiScriptSyntaxError> {
        Ok(ast::Match {
            about: self.visit_expression(*match_.about)?.into(),
            qs: match_
                .qs
                .into_iter()
                .map(|qa| {
                    Ok(ast::QA {
                        q: self.visit_expression(qa.q)?,
                        a: match qa.a {
                            ast::StatementOrExpression::Statement(statement) => self
                                .visit_statement(statement)
                                .map(ast::StatementOrExpression::Statement)?,
                            ast::StatementOrExpression::Expression(expression) => self
                                .visit_expression(expression)
                                .map(ast::StatementOrExpression::Expression)?,
                        },
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
        })
    }

    fn visit_block(&mut self, block: ast::Block) -> Result<ast::Block, AiScriptSyntaxError> {
        Ok(ast::Block {
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
        })
    }

    fn visit_identifier(
        &mut self,
        identifier: ast::Identifier,
    ) -> Result<ast::Identifier, AiScriptSyntaxError> {
        Ok(identifier)
    }

    fn visit_type_source(
        &mut self,
        type_source: ast::TypeSource,
    ) -> Result<ast::TypeSource, AiScriptSyntaxError> {
        let type_source = self.callback_type_source(type_source)?;
        Ok(match type_source {
            ast::TypeSource::NamedTypeSource(named_type_source) => {
                ast::TypeSource::NamedTypeSource(ast::NamedTypeSource {
                    inner: named_type_source
                        .inner
                        .map(|inner| self.visit_type_source(*inner))
                        .map_or(Ok(None), |r| r.map(Some))?
                        .map(Into::into),
                    ..named_type_source
                })
            }
            ast::TypeSource::FnTypeSource(fn_type_source) => {
                ast::TypeSource::FnTypeSource(ast::FnTypeSource {
                    params: fn_type_source
                        .params
                        .into_iter()
                        .map(|param| self.visit_type_source(param))
                        .collect::<Result<Vec<ast::TypeSource>, AiScriptSyntaxError>>()?,
                    result: self.visit_type_source(*fn_type_source.result)?.into(),
                    ..fn_type_source
                })
            }
            ast::TypeSource::UnionTypeSource(union_type_source) => {
                ast::TypeSource::UnionTypeSource(ast::UnionTypeSource {
                    inners: union_type_source
                        .inners
                        .into_iter()
                        .map(|inner| self.visit_type_source(inner))
                        .collect::<Result<Vec<ast::TypeSource>, AiScriptSyntaxError>>()?,
                    ..union_type_source
                })
            }
        })
    }

    fn visit_attribute(
        &mut self,
        attribute: ast::Attribute,
    ) -> Result<ast::Attribute, AiScriptSyntaxError> {
        self.callback_attribute(attribute)
    }

    fn callback_namespace(
        &mut self,
        namespace: ast::Namespace,
    ) -> Result<ast::Namespace, AiScriptSyntaxError> {
        Ok(namespace)
    }

    fn callback_meta(&mut self, meta: ast::Meta) -> Result<ast::Meta, AiScriptSyntaxError> {
        Ok(meta)
    }

    fn callback_statement(
        &mut self,
        statement: ast::Statement,
    ) -> Result<ast::Statement, AiScriptSyntaxError> {
        Ok(statement)
    }

    fn callback_expression(
        &mut self,
        expression: ast::Expression,
    ) -> Result<ast::Expression, AiScriptSyntaxError> {
        Ok(expression)
    }

    fn callback_type_source(
        &mut self,
        type_source: ast::TypeSource,
    ) -> Result<ast::TypeSource, AiScriptSyntaxError> {
        Ok(type_source)
    }

    fn callback_attribute(
        &mut self,
        attribute: ast::Attribute,
    ) -> Result<ast::Attribute, AiScriptSyntaxError> {
        Ok(attribute)
    }
}
