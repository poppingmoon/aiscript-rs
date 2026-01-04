//! AiScript interpreter

use std::{
    collections::HashMap,
    iter::{repeat, repeat_with, zip},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use futures::{
    Future, FutureExt,
    future::{BoxFuture, try_join_all},
};
use indexmap::IndexMap;
use tokio::sync::Mutex;

use crate::{
    error::{
        AiScriptError, AiScriptNamespaceError, AiScriptNamespaceErrorKind, AiScriptRuntimeError,
    },
    node as ast,
};

use self::{
    builder::InterpreterBuilder,
    frame::{AssignmentOperator, Frame},
    lib::std::std,
    primitive_props::get_prim_prop,
    scope::Scope,
    stack::{StackExt, ValueStackExt},
    util::{expect_any, node_to_value},
    value::{Attr, V, VFn, VFnParam, VObj, Value, unwrap_labeled_break, unwrap_ret},
    variable::Variable,
};

pub mod builder;
mod frame;
mod lib;
mod primitive_props;
pub mod scope;
mod stack;
pub mod util;
pub mod value;
mod variable;

#[derive(Clone, Default)]
pub struct Interpreter {
    pub step_count: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    pub scope: Scope,
    abort_handlers: Arc<Mutex<tokio::task::JoinSet<Result<(), AiScriptError>>>>,
    err: Option<Arc<dyn (Fn(AiScriptError) -> BoxFuture<'static, ()>) + Sync + Send + 'static>>,
    max_step: Option<usize>,
}

impl std::fmt::Debug for Interpreter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interpreter")
            .field("step_count", &self.step_count)
            .field("stop", &self.stop)
            .field("scope", &self.scope)
            .field("max_step", &self.max_step)
            .finish()
    }
}

impl Interpreter {
    pub fn new(
        consts: impl IntoIterator<Item = (String, Value)>,
        in_: Option<impl Fn(String) -> BoxFuture<'static, String> + Sync + Send + Clone + 'static>,
        out: Option<impl Fn(Value) -> BoxFuture<'static, ()> + Sync + Send + Clone + 'static>,
        err: Option<impl Fn(AiScriptError) -> BoxFuture<'static, ()> + Sync + Send + 'static>,
        max_step: Option<usize>,
    ) -> Self {
        let io = [
            (
                "print".to_string(),
                Value::fn_native(move |args, _| {
                    let out = expect_any(args.into_iter().next())
                        .map(|v| out.as_ref().map(|out| out.clone()(v)));
                    async {
                        if let Some(out) = out? {
                            out.await;
                        }
                        Ok(Value::null())
                    }
                    .boxed()
                }),
            ),
            (
                "readline".to_string(),
                Value::fn_native(move |args, _| {
                    let in_ = String::try_from(args.into_iter().next().unwrap_or_default())
                        .map(|q| in_.as_ref().map(|in_| in_.clone()(q)));
                    async {
                        Ok(if let Some(in_) = in_? {
                            let a = in_.await;
                            Value::str(a)
                        } else {
                            Value::null()
                        })
                    }
                    .boxed()
                }),
            ),
        ];
        let mut states = Vec::from_iter(consts);
        states.extend(std());
        states.extend(io);
        let states = states
            .into_iter()
            .map(|(k, v)| (k, Variable::Const(v)))
            .collect();
        Interpreter {
            step_count: Arc::new(AtomicUsize::new(0)),
            stop: Arc::new(AtomicBool::new(false)),
            scope: Scope::new(states),
            abort_handlers: Arc::new(Mutex::new(tokio::task::JoinSet::new())),
            err: match err {
                Some(err) => Some(Arc::new(err)),
                None => None,
            },
            max_step,
        }
    }

    pub fn builder() -> InterpreterBuilder {
        InterpreterBuilder::default()
    }

    pub async fn exec(&self, script: Vec<ast::Node>) -> Result<Option<Value>, AiScriptError> {
        self.stop.store(false, Ordering::Release);
        let script = self.collect_ns(script, &self.scope).await?;
        let result = self.run(script, &self.scope).await;
        self.handle_error(result).await
    }

    /// Executes AiScript Function.
    ///
    /// When it fails,
    /// 1. If error callback is registered via constructor, [`Self::abort`] is called and the callback executed, then returns ERROR('func_failed').
    /// 2. Otherwise, just returns an error.
    pub async fn exec_fn(
        &self,
        fn_: VFn,
        args: impl IntoIterator<Item = Value>,
    ) -> Result<Value, AiScriptError> {
        let result = self.fn_(fn_, args).await;
        let result = self.handle_error(result).await?;
        Ok(result.unwrap_or_else(|| Value::error("func_failed", None)))
    }

    /// Executes AiScript Function.
    ///
    /// Almost same as [`Self::exec_fn`] but when error occurs this always returns it and never calls callback.
    pub async fn exec_fn_simple(
        &self,
        fn_: VFn,
        args: impl IntoIterator<Item = Value>,
    ) -> Result<Value, AiScriptError> {
        self.fn_(fn_, args).await
    }

    pub fn collect_metadata(script: Vec<ast::Node>) -> IndexMap<Option<String>, Option<Value>> {
        let mut meta = IndexMap::new();

        for node in script {
            if let ast::Node::Meta(m) = node {
                meta.insert(m.name, node_to_value(m.value));
            }
        }

        meta
    }

    async fn handle_error(
        &self,
        result: Result<Value, AiScriptError>,
    ) -> Result<Option<Value>, AiScriptError> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                if let Some(err) = &self.err
                    && !self.stop.load(Ordering::SeqCst)
                {
                    self.abort().await;
                    err(e).await;
                    return Ok(None);
                }
                Err(e)
            }
        }
    }

    async fn collect_ns(
        &self,
        script: Vec<ast::Node>,
        scope: &Scope,
    ) -> Result<Vec<ast::StatementOrExpression>, AiScriptError> {
        let mut nodes = Vec::with_capacity(script.len());
        for node in script {
            match node {
                ast::Node::Namespace(namespace) => {
                    let loc = namespace.loc.clone();
                    self.collect_ns_member(*namespace, scope.clone()).await?;
                    nodes.push(ast::StatementOrExpression::Expression(
                        ast::Expression::Null(ast::Null { loc }.into()),
                    ))
                }
                ast::Node::Statement(statement) => {
                    nodes.push(ast::StatementOrExpression::Statement(statement))
                }
                ast::Node::Expression(expression) => {
                    nodes.push(ast::StatementOrExpression::Expression(expression))
                }
                ast::Node::Meta(meta) => nodes.push(ast::StatementOrExpression::Expression(
                    ast::Expression::Null(ast::Null { loc: meta.loc }.into()),
                )),
            }
        }
        Ok(nodes)
    }

    fn collect_ns_member(
        &self,
        ns: ast::Namespace,
        scope: Scope,
    ) -> BoxFuture<'_, Result<(), AiScriptError>> {
        async move {
            let ns_scope = scope.create_child_namespace_scope(ns.name, HashMap::new());
            for node in &ns.members {
                if let ast::DefinitionOrNamespace::Namespace(ns) = node {
                    self.collect_ns_member(*ns.clone(), ns_scope.clone())
                        .await?;
                }
            }
            for node in ns.members {
                if let ast::DefinitionOrNamespace::Definition(definition) = node {
                    if let ast::Expression::Identifier(identifier) = definition.dest {
                        if definition.mut_ {
                            Err(AiScriptNamespaceError {
                                kind: AiScriptNamespaceErrorKind::Mutable(identifier.name),
                                pos: definition.loc.start,
                            })?;
                        } else {
                            let mut value = self.run(vec![definition.expr], &ns_scope).await?;

                            value.attr = definition.attr.map(|attr| {
                                attr.into_iter()
                                    .filter_map(|attr| {
                                        node_to_value(attr.value).map(|value| Attr {
                                            name: attr.name,
                                            value,
                                        })
                                    })
                                    .collect()
                            });

                            ns_scope
                                .add(identifier.name, Variable::Const(value))
                                .await?;
                        }
                    } else {
                        Err(AiScriptNamespaceError {
                            kind: AiScriptNamespaceErrorKind::DestructuringAssignment,
                            pos: definition.dest.into_loc().start,
                        })?
                    }
                }
            }
            Ok(())
        }
        .boxed()
    }

    fn fn_(
        &self,
        fn_: VFn,
        args: impl IntoIterator<Item = Value>,
    ) -> BoxFuture<'_, Result<Value, AiScriptError>> {
        match fn_ {
            VFn::Fn {
                params,
                statements,
                scope,
            } => {
                let mut program = Vec::with_capacity(params.len() + statements.len());
                for (param, arg) in zip(params, args.into_iter().map(Some).chain(repeat(None))) {
                    let arg = expect_any(arg.or(param.default));
                    match arg {
                        Ok(arg) => program.push(Frame::Definition2 {
                            dest: param.dest,
                            value: arg,
                            mut_: true,
                        }),
                        Err(e) => return async { Err(e) }.boxed(),
                    }
                }
                for node in statements.into_iter() {
                    program.push(node.into());
                }
                async move {
                    self.run(program, &scope.create_child_scope(HashMap::new()))
                        .await
                        .map(unwrap_ret)
                }
                .boxed()
            }
            VFn::FnNative(fn_) => fn_(args.into_iter().collect(), self),
            VFn::FnNativeSync(fn_) => {
                let result = fn_(args.into_iter().collect());
                async { result }.boxed()
            }
        }
    }

    pub async fn run(
        &self,
        program: Vec<impl Into<Frame>>,
        scope: &Scope,
    ) -> Result<Value, AiScriptError> {
        let mut stack = Vec::new();
        let mut value_stack = Vec::new();
        let mut scope = scope.clone();

        stack.run(program);

        while let Some(frame) = stack.pop() {
            match frame {
                Frame::Statement(statement) => match statement {
                    ast::Statement::Definition(definition) => {
                        let ast::Definition {
                            dest,
                            expr,
                            mut_,
                            attr,
                            ..
                        } = *definition;
                        stack.push(Frame::Definition1 { dest, mut_, attr });
                        stack.eval(expr);
                    }
                    ast::Statement::Return(return_) => {
                        stack.push(Frame::Return);
                        stack.eval(return_.expr);
                    }
                    ast::Statement::Each(each) => {
                        let ast::Each {
                            label,
                            var,
                            items,
                            for_,
                            ..
                        } = *each;
                        stack.push(Frame::Each1 {
                            label,
                            var,
                            for_: *for_,
                        });
                        stack.eval(items);
                    }
                    ast::Statement::For(for_) => {
                        let ast::For {
                            label, times, for_, ..
                        } = *for_;
                        stack.push(Frame::For1 { label, for_: *for_ });
                        stack.eval(times);
                    }
                    ast::Statement::ForLet(for_let) => {
                        let ast::ForLet {
                            label,
                            var,
                            from,
                            to,
                            for_,
                            ..
                        } = *for_let;
                        stack.push(Frame::ForLet1 {
                            label,
                            var,
                            to,
                            for_: *for_,
                        });
                        stack.eval(from);
                    }
                    ast::Statement::Loop(loop_) => {
                        let ast::Loop {
                            label, statements, ..
                        } = *loop_;
                        stack.push(Frame::Loop1 { label, statements })
                    }
                    ast::Statement::Break(break_) => {
                        let ast::Break { label, expr, .. } = *break_;
                        if let Some(expr) = expr {
                            stack.push(Frame::Break { label });
                            stack.eval(expr);
                        } else {
                            value_stack.push(Value::break_(label, None));
                        }
                    }
                    ast::Statement::Continue(continue_) => {
                        value_stack.push(Value::continue_(continue_.label))
                    }
                    ast::Statement::Assign(assign) => {
                        let ast::Assign { dest, expr, .. } = *assign;
                        stack.push(Frame::Assign {
                            dest,
                            expr: Some(expr),
                            op: None,
                        });
                        value_stack.push(Value::null());
                    }
                    ast::Statement::AddAssign(add_assign) => {
                        let ast::AddAssign { dest, expr, .. } = *add_assign;
                        stack.push(Frame::Assign {
                            dest,
                            expr: Some(expr),
                            op: Some(AssignmentOperator::Add),
                        });
                        value_stack.push(Value::null());
                    }
                    ast::Statement::SubAssign(sub_assign) => {
                        let ast::SubAssign { dest, expr, .. } = *sub_assign;
                        stack.push(Frame::Assign {
                            dest,
                            expr: Some(expr),
                            op: Some(AssignmentOperator::Sub),
                        });
                        value_stack.push(Value::null());
                    }
                },
                Frame::Expression(expression) => match expression {
                    ast::Expression::If(if_) => {
                        let ast::If {
                            label,
                            cond,
                            then,
                            mut elseif,
                            else_,
                            ..
                        } = *if_;
                        elseif.reverse();
                        stack.push(Frame::If1 {
                            label,
                            then: *then,
                            elseif,
                            else_,
                        });
                        stack.eval(*cond);
                    }
                    ast::Expression::Fn(fn_) => {
                        let ast::Fn {
                            params, children, ..
                        } = *fn_;
                        let params = try_join_all(params.into_iter().map(async |param| {
                            Ok::<VFnParam, AiScriptError>(VFnParam {
                                dest: param.dest,
                                default: if let Some(default) = param.default {
                                    Some(self.run(vec![default], &scope).await?)
                                } else if param.optional {
                                    Some(Value::null())
                                } else {
                                    None
                                },
                            })
                        }))
                        .await?;
                        value_stack.push(
                            if let Some(control) = params.iter().find_map(|param| {
                                param.default.as_ref().and_then(|value| {
                                    if value.is_control() {
                                        Some(value)
                                    } else {
                                        None
                                    }
                                })
                            }) {
                                control.clone()
                            } else {
                                Value::fn_(
                                    params,
                                    children,
                                    scope.create_child_scope(HashMap::new()),
                                )
                            },
                        );
                    }
                    ast::Expression::Match(match_) => {
                        let ast::Match {
                            label,
                            about,
                            qs,
                            default,
                            ..
                        } = *match_;
                        stack.push(Frame::Match1 { label, qs, default });
                        stack.eval(*about);
                    }
                    ast::Expression::Block(block) => {
                        let ast::Block {
                            label, statements, ..
                        } = *block;
                        scope = scope.create_child_scope(HashMap::new());
                        stack.push(Frame::Block { label });
                        stack.run(statements);
                    }
                    ast::Expression::Exists(exists) => {
                        let exists = scope.exists(&exists.identifier.name).await;
                        value_stack.push(Value::bool(exists));
                    }
                    ast::Expression::Tmpl(tmpl) => {
                        let mut tmpl = tmpl.tmpl;
                        tmpl.reverse();
                        stack.push(Frame::Tmpl1 {
                            tmpl,
                            str: String::new(),
                        });
                    }
                    ast::Expression::Str(str) => value_stack.push(Value::str(str.value)),
                    ast::Expression::Num(num) => value_stack.push(Value::num(num.value)),
                    ast::Expression::Bool(bool) => value_stack.push(Value::bool(bool.value)),
                    ast::Expression::Null(_) => value_stack.push(Value::null()),
                    ast::Expression::Obj(obj) => {
                        let mut obj = obj.value;
                        let len = obj.len();
                        obj.reverse();
                        stack.push(Frame::Obj1 {
                            obj: obj.into(),
                            value: IndexMap::with_capacity(len).into(),
                        });
                    }
                    ast::Expression::Arr(arr) => {
                        let mut arr = arr.value;
                        let len = arr.len();
                        arr.reverse();
                        stack.push(Frame::Arr1 {
                            arr,
                            value: Vec::with_capacity(len),
                        })
                    }
                    ast::Expression::Plus(plus) => {
                        stack.push(Frame::Plus);
                        stack.eval(*plus.expr);
                    }
                    ast::Expression::Minus(minus) => {
                        stack.push(Frame::Minus);
                        stack.eval(*minus.expr);
                    }
                    ast::Expression::Not(not) => {
                        stack.push(Frame::Not);
                        stack.eval(*not.expr);
                    }
                    ast::Expression::Pow(pow) => {
                        let callee = scope.get("Core:pow").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Pow { left, right, .. } = *pow;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Mul(mul) => {
                        let callee = scope.get("Core:mul").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Mul { left, right, .. } = *mul;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Div(div) => {
                        let callee = scope.get("Core:div").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Div { left, right, .. } = *div;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Rem(rem) => {
                        let callee = scope.get("Core:mod").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Rem { left, right, .. } = *rem;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Add(add) => {
                        let callee = scope.get("Core:add").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Add { left, right, .. } = *add;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Sub(sub) => {
                        let callee = scope.get("Core:sub").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Sub { left, right, .. } = *sub;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Lt(lt) => {
                        let callee = scope.get("Core:lt").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Lt { left, right, .. } = *lt;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Lteq(lteq) => {
                        let callee = scope.get("Core:lteq").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Lteq { left, right, .. } = *lteq;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Gt(gt) => {
                        let callee = scope.get("Core:gt").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Gt { left, right, .. } = *gt;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Gteq(gteq) => {
                        let callee = scope.get("Core:gteq").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Gteq { left, right, .. } = *gteq;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Eq(eq) => {
                        let callee = scope.get("Core:eq").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Eq { left, right, .. } = *eq;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::Neq(neq) => {
                        let callee = scope.get("Core:neq").await?;
                        let callee = <VFn>::try_from(callee)?;
                        let ast::Neq { left, right, .. } = *neq;
                        stack.push(Frame::BinOp1 {
                            callee,
                            right: *right,
                        });
                        stack.eval(*left);
                    }
                    ast::Expression::And(and) => {
                        let ast::And { left, right, .. } = *and;
                        stack.push(Frame::And1 { right: *right });
                        stack.eval(*left);
                    }
                    ast::Expression::Or(or) => {
                        let ast::Or { left, right, .. } = *or;
                        stack.push(Frame::Or1 { right: *right });
                        stack.eval(*left);
                    }
                    ast::Expression::Identifier(identifier) => {
                        let value = scope.get(&identifier.name).await?;
                        value_stack.push(value);
                    }
                    ast::Expression::Call(call) => {
                        let ast::Call { target, args, .. } = *call;
                        stack.push(Frame::Call1 { args });
                        stack.eval(*target);
                    }
                    ast::Expression::Index(index) => {
                        let ast::Index { target, index, .. } = *index;
                        stack.push(Frame::Index1 { index: *index });
                        stack.eval(*target);
                    }
                    ast::Expression::Prop(prop) => {
                        let ast::Prop { target, name, .. } = *prop;
                        stack.push(Frame::Prop { name });
                        stack.eval(*target);
                    }
                },
                Frame::Definition1 { dest, mut_, attr } => {
                    let mut value = value_stack.pop_value()?;
                    if value.is_control() {
                        value_stack.push(value);
                    } else {
                        value.attr = attr.map(|attr| {
                            attr.into_iter()
                                .filter_map(|ast::Attribute { name, value, .. }| {
                                    node_to_value(value).map(|value| Attr { name, value })
                                })
                                .collect()
                        });
                        stack.push(Frame::Definition2 { dest, mut_, value });
                        value_stack.push(Value::null());
                    }
                }
                Frame::Definition2 { dest, value, mut_ } => match dest {
                    ast::Expression::Identifier(identifier) => {
                        scope
                            .add(
                                identifier.name,
                                if mut_ {
                                    Variable::Mut(value)
                                } else {
                                    Variable::Const(value)
                                },
                            )
                            .await?;
                    }
                    ast::Expression::Arr(arr) => {
                        let value = <Vec<Value>>::try_from(value)?;
                        for (index, item) in arr.value.into_iter().enumerate().rev() {
                            stack.push(Frame::Definition2 {
                                dest: item,
                                value: value.get(index).cloned().unwrap_or_default(),
                                mut_,
                            })
                        }
                    }
                    ast::Expression::Obj(obj) => {
                        let value = <IndexMap<String, Value>>::try_from(value)?;
                        for (key, item) in obj.value.into_iter().rev() {
                            stack.push(Frame::Definition2 {
                                dest: item,
                                value: value.get(&key).cloned().unwrap_or_default(),
                                mut_,
                            });
                        }
                    }
                    _ => Err(AiScriptRuntimeError::InvalidDefinition)?,
                },
                Frame::Return => {
                    let val = value_stack.pop_value()?;
                    value_stack.push(if val.is_control() {
                        val
                    } else {
                        Value::return_(val)
                    });
                }
                Frame::Each1 { label, var, for_ } => {
                    let items = value_stack.pop_value()?;
                    if items.is_control() {
                        value_stack.push(items);
                    } else {
                        let mut items = <Vec<Value>>::try_from(items)?;
                        items.reverse();
                        stack.push(Frame::Each2 {
                            label,
                            var,
                            items,
                            for_,
                        });
                    }
                }
                Frame::Each2 {
                    label,
                    var,
                    mut items,
                    for_,
                } => {
                    if let Some(item) = items.pop() {
                        stack.push(Frame::Each3 {
                            label,
                            var: var.clone(),
                            items,
                            for_: for_.clone(),
                        });
                        scope = scope.create_child_scope(HashMap::new());
                        stack.eval(for_);
                        stack.push(Frame::Definition2 {
                            dest: var,
                            value: item,
                            mut_: false,
                        });
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::Each3 {
                    label,
                    var,
                    items,
                    for_,
                } => {
                    scope = scope.get_parent()?;
                    let v = value_stack.pop_value()?;
                    match &*v.value {
                        V::Break { label: l, .. } => {
                            value_stack.push(if l.is_some() && *l != label {
                                v
                            } else {
                                Value::null()
                            });
                        }
                        V::Continue { label: l, .. } if l.is_some() && *l != label => {
                            value_stack.push(v);
                        }
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::Each2 {
                            label,
                            var,
                            items,
                            for_,
                        }),
                    }
                }
                Frame::For1 { label, for_ } => {
                    let times = value_stack.pop_value()?;
                    if times.is_control() {
                        value_stack.push(times);
                    } else {
                        let times = f64::try_from(times)?;
                        stack.push(Frame::For2 {
                            label,
                            i: 0.0,
                            times,
                            for_,
                        });
                    }
                }
                Frame::For2 {
                    label,
                    i,
                    times,
                    for_,
                } => {
                    if i < times {
                        if let ast::StatementOrExpression::Statement(_) = for_ {
                            scope = scope.create_child_scope(HashMap::new());
                        }
                        stack.push(Frame::For3 {
                            label,
                            i,
                            times,
                            for_: for_.clone(),
                        });
                        stack.eval(for_);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::For3 {
                    label,
                    i,
                    times,
                    for_,
                } => {
                    if let ast::StatementOrExpression::Statement(_) = for_ {
                        scope = scope.get_parent()?;
                    }
                    let v = value_stack.pop_value()?;
                    match &*v.value {
                        V::Break { label: l, .. } => {
                            value_stack.push(if l.is_some() && *l != label {
                                v
                            } else {
                                Value::null()
                            });
                        }
                        V::Continue { label: l, .. } if l.is_some() && *l != label => {
                            value_stack.push(v);
                        }
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::For2 {
                            label,
                            i: i + 1.0,
                            times,
                            for_,
                        }),
                    }
                }
                Frame::ForLet1 {
                    label,
                    var,
                    to,
                    for_,
                } => {
                    let from = value_stack.pop_value()?;
                    if from.is_control() {
                        value_stack.push(from);
                    } else {
                        let from = f64::try_from(from)?;
                        stack.push(Frame::ForLet2 {
                            label,
                            var,
                            from,
                            for_,
                        });
                        stack.eval(to);
                    }
                }
                Frame::ForLet2 {
                    label,
                    var,
                    from,
                    for_,
                } => {
                    let to = value_stack.pop_value()?;
                    if to.is_control() {
                        value_stack.push(to);
                    } else {
                        let to = f64::try_from(to)?;
                        stack.push(Frame::ForLet3 {
                            label,
                            var,
                            i: from,
                            until: from + to,
                            for_,
                        });
                    }
                }
                Frame::ForLet3 {
                    label,
                    var,
                    i,
                    until,
                    for_,
                } => {
                    if i < until {
                        scope = scope.create_child_scope(HashMap::from_iter([(
                            var.clone(),
                            Variable::Const(Value::num(i)),
                        )]));
                        stack.push(Frame::ForLet4 {
                            label,
                            var,
                            i,
                            until,
                            for_: for_.clone(),
                        });
                        stack.eval(for_);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::ForLet4 {
                    label,
                    var,
                    i,
                    until,
                    for_,
                } => {
                    scope = scope.get_parent()?;
                    let v = value_stack.pop_value()?;
                    match &*v.value {
                        V::Break { label: l, .. } => {
                            value_stack.push(if l.is_some() && *l != label {
                                v
                            } else {
                                Value::null()
                            });
                        }
                        V::Continue { label: l, .. } if l.is_some() && *l != label => {
                            value_stack.push(v);
                        }
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::ForLet3 {
                            label,
                            var,
                            i: i + 1.0,
                            until,
                            for_,
                        }),
                    }
                }
                Frame::Loop1 { label, statements } => {
                    stack.push(Frame::Loop2 {
                        label,
                        statements: statements.clone(),
                    });
                    scope = scope.create_child_scope(HashMap::new());
                    stack.run(statements);
                }
                Frame::Loop2 { label, statements } => {
                    scope = scope.get_parent()?;
                    let v = value_stack.pop_value()?;
                    match &*v.value {
                        V::Break { label: l, .. } => {
                            value_stack.push(if l.is_some() && *l != label {
                                v
                            } else {
                                Value::null()
                            });
                        }
                        V::Continue { label: l, .. } if l.is_some() && *l != label => {
                            value_stack.push(v);
                        }
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::Loop1 { label, statements }),
                    }
                }
                Frame::Break { label } => {
                    let value = value_stack.pop_value()?;
                    value_stack.push(if value.is_control() {
                        value
                    } else {
                        Value::break_(label, Some(value))
                    })
                }
                Frame::Assign { dest, expr, op } => {
                    match dest {
                        ast::Expression::Identifier(identifier) => {
                            stack.push(Frame::AssignIdentifier {
                                name: identifier.name,
                                op,
                            });
                            if let Some(expr) = expr {
                                stack.eval(expr);
                            }
                        }
                        ast::Expression::Index(index) => {
                            let ast::Index { target, index, .. } = *index;
                            stack.push(Frame::AssignIndex1 {
                                index: *index,
                                expr,
                                op,
                            });
                            stack.eval(*target);
                        }
                        ast::Expression::Prop(prop) => {
                            let ast::Prop { target, name, .. } = *prop;
                            stack.push(Frame::AssignProp1 { name, expr, op });
                            stack.eval(*target);
                        }
                        ast::Expression::Arr(arr) => {
                            let arr = arr.value;
                            let len = arr.len();
                            for item in arr.into_iter().rev() {
                                stack.push(Frame::Assign {
                                    dest: item,
                                    expr: None,
                                    op: None,
                                });
                            }
                            stack.push(Frame::AssignArr { len, op });
                            if let Some(expr) = expr {
                                stack.eval(expr);
                            }
                        }
                        ast::Expression::Obj(obj) => {
                            let (keys, values): (Vec<String>, Vec<ast::Expression>) =
                                obj.value.into_iter().rev().unzip();
                            for item in values {
                                stack.push(Frame::Assign {
                                    dest: item,
                                    expr: None,
                                    op: None,
                                });
                            }
                            stack.push(Frame::AssignObj { keys, op });
                            if let Some(expr) = expr {
                                stack.eval(expr);
                            }
                        }
                        _ => Err(AiScriptRuntimeError::InvalidAssignment)?,
                    };
                }
                Frame::AssignIdentifier { name, op } => {
                    let value = value_stack.pop_value()?;
                    if value.is_control() {
                        value_stack.push(value);
                    } else if let Some(op) = op {
                        let v = f64::try_from(value)?;
                        let target = scope.get(&name).await?;
                        let target = f64::try_from(target)?;
                        scope
                            .assign(
                                name,
                                Value::num(match op {
                                    AssignmentOperator::Add => target + v,
                                    AssignmentOperator::Sub => target - v,
                                }),
                            )
                            .await?;
                    } else {
                        scope.assign(name, value).await?;
                    }
                }
                Frame::AssignIndex1 { index, expr, op } => {
                    let assignee = value_stack.pop_value()?;
                    if assignee.is_control() {
                        value_stack.push(assignee);
                    } else {
                        stack.push(Frame::AssignIndex2 { assignee, expr, op });
                        stack.eval(index);
                    }
                }
                Frame::AssignIndex2 { assignee, expr, op } => {
                    let i = value_stack.pop_value()?;
                    if i.is_control() {
                        value_stack.push(i);
                    } else {
                        match *assignee.value {
                            V::Arr(arr) => {
                                let i = f64::try_from(i)?;
                                let index = i as usize;
                                let len = arr.read().map_err(AiScriptError::internal)?.len();
                                if index as f64 == i && index < len {
                                    stack.push(Frame::AssignIndexArr {
                                        assignee: arr,
                                        index,
                                        op,
                                    });
                                    if let Some(expr) = expr {
                                        stack.eval(expr);
                                    }
                                } else {
                                    Err(AiScriptRuntimeError::IndexOutOfRange {
                                        index: i,
                                        max: len as isize - 1,
                                    })?
                                }
                            }
                            V::Obj(obj) => {
                                let i = String::try_from(i)?;
                                stack.push(Frame::AssignProp2 {
                                    assignee: obj,
                                    name: i,
                                    op,
                                });
                                if let Some(expr) = expr {
                                    stack.eval(expr);
                                }
                            }
                            _ => Err(AiScriptRuntimeError::InvalidProperty {
                                name: i.repr_value().to_string(),
                                target_type: assignee.display_type().to_string(),
                            })?,
                        }
                    }
                }
                Frame::AssignIndexArr {
                    assignee,
                    index,
                    op,
                } => {
                    let v = value_stack.pop_value()?;
                    if v.is_control() {
                        value_stack.push(v);
                    } else {
                        let mut arr = assignee.write().map_err(AiScriptError::internal)?;
                        if let Some(target) = arr.get_mut(index) {
                            if let Some(op) = op {
                                let v = f64::try_from(v)?;
                                let target_value = f64::try_from(target.clone())?;
                                *target = Value::num(match op {
                                    AssignmentOperator::Add => target_value + v,
                                    AssignmentOperator::Sub => target_value - v,
                                });
                            } else {
                                *target = v;
                            }
                        } else {
                            Err(AiScriptRuntimeError::IndexOutOfRange {
                                index: index as f64,
                                max: arr.len() as isize - 1,
                            })?
                        }
                    }
                }
                Frame::AssignProp1 { name, expr, op } => {
                    let assignee = value_stack.pop_value()?;
                    if assignee.is_control() {
                        value_stack.push(assignee);
                    } else {
                        let assignee = VObj::try_from(assignee)?;
                        stack.push(Frame::AssignProp2 { assignee, name, op });
                        if let Some(expr) = expr {
                            stack.eval(expr);
                        }
                    }
                }
                Frame::AssignProp2 { assignee, name, op } => {
                    let value = value_stack.pop_value()?;
                    if value.is_control() {
                        value_stack.push(value);
                    } else if let Some(op) = op {
                        let v = f64::try_from(value)?;
                        let target = assignee
                            .read()
                            .map_err(AiScriptError::internal)?
                            .get(&name)
                            .cloned();
                        let target_value = f64::try_from(target.unwrap_or_default())?;
                        assignee.write().map_err(AiScriptError::internal)?.insert(
                            name,
                            Value::num(match op {
                                AssignmentOperator::Add => target_value + v,
                                AssignmentOperator::Sub => target_value - v,
                            }),
                        );
                    } else {
                        assignee
                            .write()
                            .map_err(AiScriptError::internal)?
                            .insert(name, value);
                    }
                }
                Frame::AssignArr { len, op } => {
                    let value = value_stack.pop_value()?;
                    if value.is_control() {
                        value_stack.push(value);
                    } else {
                        let value = <Vec<Value>>::try_from(value)?;
                        if op.is_some() {
                            Err(AiScriptRuntimeError::TypeMismatch {
                                expected: "number".to_string(),
                                actual: "arr".to_string(),
                            })?
                        }
                        let value_len = value.len();
                        value_stack.extend(value.into_iter().take(len).rev());
                        if value_len < len {
                            value_stack.extend(repeat_with(Value::null).take(len - value_len));
                        }
                    }
                }
                Frame::AssignObj { keys, op } => {
                    let value = value_stack.pop_value()?;
                    if value.is_control() {
                        value_stack.push(value);
                    } else {
                        let value = <IndexMap<String, Value>>::try_from(value)?;
                        if op.is_some() {
                            Err(AiScriptRuntimeError::TypeMismatch {
                                expected: "number".to_string(),
                                actual: "obj".to_string(),
                            })?
                        }
                        for key in keys {
                            value_stack.push(value.get(&key).cloned().unwrap_or_default());
                        }
                    }
                }
                Frame::If1 {
                    label,
                    then,
                    mut elseif,
                    else_,
                } => {
                    let cond = value_stack.pop_value()?;
                    if cond.is_control() {
                        value_stack.push(cond);
                    } else {
                        let cond = bool::try_from(cond)?;
                        if cond {
                            let is_statement =
                                matches!(then, ast::StatementOrExpression::Statement(_));
                            if is_statement {
                                scope = scope.create_child_scope(HashMap::new());
                            }
                            stack.push(Frame::If2 {
                                label,
                                is_statement,
                            });
                            stack.eval(then);
                        } else if let Some(ast::Elseif { cond, then }) = elseif.pop() {
                            stack.push(Frame::If1 {
                                label,
                                then,
                                elseif,
                                else_,
                            });
                            stack.eval(cond);
                        } else if let Some(else_) = else_ {
                            let else_ = *else_;
                            let is_statement =
                                matches!(else_, ast::StatementOrExpression::Statement(_));
                            if is_statement {
                                scope = scope.create_child_scope(HashMap::new());
                            }
                            stack.push(Frame::If2 {
                                label,
                                is_statement,
                            });
                            stack.eval(else_);
                        } else {
                            value_stack.push(Value::null());
                        }
                    }
                }
                Frame::If2 {
                    label,
                    is_statement,
                } => {
                    if is_statement {
                        scope = scope.get_parent()?;
                    }
                    let value = value_stack.pop_value()?;
                    value_stack.push(unwrap_labeled_break(value, label));
                }
                Frame::Match1 {
                    label,
                    mut qs,
                    default,
                } => {
                    let about = value_stack.pop_value()?;
                    if about.is_control() {
                        value_stack.push(about);
                    } else {
                        qs.reverse();
                        stack.push(Frame::Match2 {
                            label,
                            about,
                            qs,
                            default,
                        });
                    }
                }
                Frame::Match2 {
                    label,
                    about,
                    mut qs,
                    default,
                } => {
                    if let Some(ast::QA { q, a }) = qs.pop() {
                        stack.push(Frame::Match3 {
                            label,
                            about,
                            a: a.into(),
                            qs,
                            default,
                        });
                        stack.eval(q);
                    } else if let Some(default) = default {
                        let default = *default;
                        let is_statement =
                            matches!(default, ast::StatementOrExpression::Statement(_));
                        if is_statement {
                            scope = scope.create_child_scope(HashMap::new());
                        }
                        stack.push(Frame::Match4 {
                            label,
                            is_statement,
                        });
                        stack.eval(default);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::Match3 {
                    label,
                    about,
                    a,
                    qs,
                    default,
                } => {
                    let q = value_stack.pop_value()?;
                    if q.is_control() {
                        value_stack.push(q);
                    } else if about == q {
                        let a = *a;
                        let is_statement = matches!(a, ast::StatementOrExpression::Statement(_));
                        if is_statement {
                            scope = scope.create_child_scope(HashMap::new());
                        }
                        stack.push(Frame::Match4 {
                            label,
                            is_statement,
                        });
                        stack.eval(a);
                    } else {
                        stack.push(Frame::Match2 {
                            label,
                            about,
                            qs,
                            default,
                        });
                    }
                }
                Frame::Match4 {
                    label,
                    is_statement,
                } => {
                    if is_statement {
                        scope = scope.get_parent()?;
                    }
                    let value = value_stack.pop_value()?;
                    value_stack.push(unwrap_labeled_break(value, label));
                }
                Frame::Block { label } => {
                    scope = scope.get_parent()?;
                    let value = value_stack.pop_value()?;
                    value_stack.push(unwrap_labeled_break(value, label));
                }
                Frame::Tmpl1 { mut tmpl, str } => {
                    if let Some(x) = tmpl.pop() {
                        stack.push(Frame::Tmpl2 { tmpl, str });
                        stack.eval(x);
                    } else {
                        value_stack.push(Value::str(str));
                    }
                }
                Frame::Tmpl2 { tmpl, mut str } => {
                    let v = value_stack.pop_value()?;
                    if v.is_control() {
                        value_stack.push(v);
                    } else {
                        str += &v.repr_value().to_string();
                        stack.push(Frame::Tmpl1 { tmpl, str });
                    }
                }
                Frame::Obj1 { mut obj, value } => {
                    if let Some((k, v)) = obj.pop() {
                        stack.push(Frame::Obj2 { obj, value, k });
                        stack.eval(v);
                    } else {
                        value_stack.push(Value::obj(*value));
                    }
                }
                Frame::Obj2 { obj, mut value, k } => {
                    let v = value_stack.pop_value()?;
                    if v.is_control() {
                        value_stack.push(v);
                    } else {
                        value.insert(k, v);
                        stack.push(Frame::Obj1 { obj, value });
                    }
                }
                Frame::Arr1 { mut arr, value } => {
                    if let Some(v) = arr.pop() {
                        stack.push(Frame::Arr2 { arr, value });
                        stack.eval(v);
                    } else {
                        value_stack.push(Value::arr(value));
                    }
                }
                Frame::Arr2 { arr, mut value } => {
                    let v = value_stack.pop_value()?;
                    if v.is_control() {
                        value_stack.push(v);
                    } else {
                        value.push(v);
                        stack.push(Frame::Arr1 { arr, value });
                    }
                }
                Frame::Plus => {
                    let v = value_stack.pop_value()?;
                    if v.is_control() {
                        value_stack.push(v);
                    } else {
                        let v = f64::try_from(v)?;
                        value_stack.push(Value::num(v));
                    }
                }
                Frame::Minus => {
                    let v = value_stack.pop_value()?;
                    if v.is_control() {
                        value_stack.push(v);
                    } else {
                        let v = f64::try_from(v)?;
                        value_stack.push(Value::num(-v));
                    }
                }
                Frame::Not => {
                    let v = value_stack.pop_value()?;
                    if v.is_control() {
                        value_stack.push(v);
                    } else {
                        let v = bool::try_from(v)?;
                        value_stack.push(Value::bool(!v));
                    }
                }
                Frame::BinOp1 { callee, right } => {
                    let left = value_stack.pop_value()?;
                    if left.is_control() {
                        value_stack.push(left);
                    } else {
                        stack.push(Frame::BinOp2 { callee, left });
                        stack.eval(right);
                    }
                }
                Frame::BinOp2 { callee, left } => {
                    let right = value_stack.pop_value()?;
                    if right.is_control() {
                        value_stack.push(right);
                    } else {
                        stack.push(Frame::Call4 {
                            callee,
                            args: vec![left, right],
                        })
                    }
                }
                Frame::And1 { right } => {
                    let left_value = value_stack.pop_value()?;
                    if left_value.is_control() {
                        value_stack.push(left_value);
                    } else {
                        let left_value = bool::try_from(left_value)?;
                        if !left_value {
                            value_stack.push(Value::bool(left_value))
                        } else {
                            stack.push(Frame::And2);
                            stack.eval(right);
                        }
                    }
                }
                Frame::And2 => {
                    let right_value = value_stack.pop_value()?;
                    if right_value.is_control() {
                        value_stack.push(right_value);
                    } else {
                        let right_value = bool::try_from(right_value)?;
                        value_stack.push(Value::bool(right_value))
                    }
                }
                Frame::Or1 { right } => {
                    let left_value = value_stack.pop_value()?;
                    if left_value.is_control() {
                        value_stack.push(left_value);
                    } else {
                        let left_value = bool::try_from(left_value)?;
                        if left_value {
                            value_stack.push(Value::bool(left_value))
                        } else {
                            stack.push(Frame::Or2);
                            stack.eval(right);
                        }
                    }
                }
                Frame::Or2 => {
                    let right_value = value_stack.pop_value()?;
                    if right_value.is_control() {
                        value_stack.push(right_value);
                    } else {
                        let right_value = bool::try_from(right_value)?;
                        value_stack.push(Value::bool(right_value))
                    }
                }
                Frame::Call1 { mut args } => {
                    let callee = value_stack.pop_value()?;
                    if callee.is_control() {
                        value_stack.push(callee);
                    } else {
                        let callee = VFn::try_from(callee)?;
                        let len = args.len();
                        args.reverse();
                        stack.push(Frame::Call2 {
                            callee,
                            args,
                            value: Vec::with_capacity(len),
                        });
                    }
                }
                Frame::Call2 {
                    callee,
                    mut args,
                    value,
                } => {
                    if let Some(arg) = args.pop() {
                        stack.push(Frame::Call3 {
                            callee,
                            args,
                            value,
                        });
                        stack.eval(arg);
                    } else {
                        stack.push(Frame::Call4 {
                            callee,
                            args: value,
                        });
                    }
                }
                Frame::Call3 {
                    callee,
                    args,
                    mut value,
                } => {
                    let arg = value_stack.pop_value()?;
                    if arg.is_control() {
                        value_stack.push(arg);
                    } else {
                        value.push(arg);
                        stack.push(Frame::Call2 {
                            callee,
                            args,
                            value,
                        });
                    }
                }
                Frame::Call4 { callee, args } => match callee {
                    VFn::Fn {
                        params,
                        statements,
                        scope: fn_scope,
                    } => {
                        stack.push(Frame::Call5 { scope });
                        scope = fn_scope.create_child_scope(HashMap::new());
                        let mut definitions = Vec::with_capacity(params.len());
                        for (param, arg) in
                            zip(params, args.into_iter().map(Some).chain(repeat(None)))
                        {
                            let arg = expect_any(arg.or(param.default))?;
                            definitions.push((param.dest, arg));
                        }
                        stack.run(statements);
                        while let Some((dest, arg)) = definitions.pop() {
                            stack.push(Frame::Definition2 {
                                dest,
                                value: arg,
                                mut_: true,
                            })
                        }
                    }
                    VFn::FnNative(fn_) => {
                        value_stack.push(fn_(args.into_iter().collect(), self).await?);
                    }
                    VFn::FnNativeSync(fn_) => {
                        value_stack.push(fn_(args.into_iter().collect())?);
                    }
                },
                Frame::Call5 {
                    scope: previous_scope,
                } => {
                    scope = previous_scope;
                    let r = value_stack.pop_value()?;
                    value_stack.push(unwrap_ret(r));
                }
                Frame::Index1 { index } => {
                    let target = value_stack.pop_value()?;
                    if target.is_control() {
                        value_stack.push(target);
                    } else {
                        stack.push(Frame::Index2 { target });
                        stack.eval(index);
                    }
                }
                Frame::Index2 { target } => {
                    let i = value_stack.pop_value()?;
                    if i.is_control() {
                        value_stack.push(i);
                    } else {
                        match *target.value {
                            V::Arr(arr) => {
                                let i = f64::try_from(i)?;
                                let item = if i.trunc() == i {
                                    arr.read()
                                        .map_err(AiScriptError::internal)?
                                        .get(i as usize)
                                        .cloned()
                                } else {
                                    None
                                };
                                value_stack.push(if let Some(item) = item {
                                    item
                                } else {
                                    Err(AiScriptRuntimeError::IndexOutOfRange {
                                        index: i,
                                        max: arr.read().map_err(AiScriptError::internal)?.len()
                                            as isize
                                            - 1,
                                    })?
                                });
                            }
                            V::Obj(obj) => {
                                let i = String::try_from(i)?;
                                value_stack.push(
                                    obj.read()
                                        .map_err(AiScriptError::internal)?
                                        .get(&i)
                                        .cloned()
                                        .unwrap_or_default(),
                                );
                            }
                            target => Err(AiScriptRuntimeError::InvalidProperty {
                                name: i.repr_value().to_string(),
                                target_type: target.display_type().to_string(),
                            })?,
                        }
                    }
                }
                Frame::Prop { name } => {
                    let target = value_stack.pop_value()?;

                    value_stack.push(if target.is_control() {
                        target
                    } else if let V::Obj(value) = *target.value {
                        value
                            .read()
                            .map_err(AiScriptError::internal)?
                            .get(&name)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        get_prim_prop(target, &name)?
                    });
                }
                Frame::Run => {
                    if value_stack.is_empty() {
                        value_stack.push(Value::null());
                    }
                }
                Frame::Unwind => {
                    if let Some(v) = value_stack.last()
                        && let V::Return(_) | V::Break { .. } | V::Continue { .. } = *v.value
                    {
                        while let Some(frame) = stack.pop() {
                            if let Frame::Run = frame {
                                break;
                            }
                        }
                    }
                }
                Frame::Eval => {
                    if self.stop.load(Ordering::Acquire) {
                        return Ok(Value::null());
                    }
                    let step_count = self.step_count.fetch_add(1, Ordering::Relaxed);
                    if let Some(max_step) = self.max_step
                        && step_count > max_step
                    {
                        Err(AiScriptRuntimeError::MaxStepExceeded)?
                    }
                }
            }
        }

        value_stack.pop_value()
    }

    pub async fn register_abort_handler(
        &self,
        task: impl Future<Output = Result<(), AiScriptError>> + Send + 'static,
    ) -> tokio::task::AbortHandle {
        self.abort_handlers.lock().await.spawn(task)
    }

    pub async fn abort(&self) {
        self.stop.store(true, Ordering::Release);
        self.abort_handlers.lock().await.abort_all();
    }
}
