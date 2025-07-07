//! AiScript interpreter

use std::{
    collections::HashMap,
    iter::{repeat, zip},
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
use value::VObj;

use crate::{
    error::{AiScriptError, AiScriptRuntimeError},
    node as ast,
};

use self::{
    builder::InterpreterBuilder,
    frame::Frame,
    lib::std::std,
    primitive_props::get_prim_prop,
    scope::Scope,
    stack::{StackExt, ValueStackExt},
    util::expect_any,
    value::{V, VFn, Value, unwrap_ret},
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
                    let mut args = args.into_iter();
                    let out =
                        expect_any(args.next()).map(|v| out.as_ref().map(|out| out.clone()(v)));
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
                    let mut args = args.into_iter();
                    let in_ = String::try_from(args.next().unwrap_or_default())
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
        fn node_to_value(node: ast::Expression) -> Option<Value> {
            match node {
                ast::Expression::Arr(arr) => Some(Value::arr({
                    let mut vec = Vec::new();
                    for node in arr.value {
                        if let Some(value) = node_to_value(node) {
                            vec.push(value);
                        }
                    }
                    vec
                })),
                ast::Expression::Bool(bool) => Some(Value::bool(bool.value)),
                ast::Expression::Null(_) => Some(Value::null()),
                ast::Expression::Num(num) => Some(Value::num(num.value)),
                ast::Expression::Obj(obj) => Some(Value::obj({
                    let mut map = IndexMap::new();
                    for (k, v) in obj.value.into_iter() {
                        if let Some(value) = node_to_value(v) {
                            map.insert(k, value);
                        }
                    }
                    map
                })),
                ast::Expression::Str(str) => Some(Value::str(str.value)),
                _ => None,
            }
        }

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
        script: impl IntoIterator<Item = impl Into<ast::Node>>,
        scope: &Scope,
    ) -> Result<Vec<ast::StatementOrExpression>, AiScriptError> {
        let mut nodes = Vec::new();
        for node in script {
            match node.into() {
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
                    if definition.mut_ {
                        Err(AiScriptError::internal(format!(
                            "Namespaces cannot include mutable variable: {}",
                            definition.name,
                        )))?;
                    } else {
                        let variable =
                            Variable::Const(self.run(vec![definition.expr], &ns_scope).await?);
                        ns_scope.add(&definition.name, variable).await?;
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
                args: fn_args,
                statements,
                scope,
            } => {
                let args = zip(
                    fn_args,
                    args.into_iter()
                        .chain(repeat(Value::null()))
                        .map(Variable::Mut),
                )
                .collect();
                async move {
                    self.run(statements, &scope.create_child_scope(args))
                        .await
                        .map(unwrap_ret)
                }
                .boxed()
            }
            VFn::FnNative(fn_) => fn_(args.into_iter().collect(), self),
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
                        stack.push(Frame::Definition {
                            name: definition.name,
                            mut_: definition.mut_,
                        });
                        stack.eval(definition.expr);
                    }
                    ast::Statement::Return(return_) => {
                        stack.push(Frame::Return);
                        stack.eval(return_.expr);
                    }
                    ast::Statement::Each(each) => {
                        stack.push(Frame::Each1 {
                            var: each.var,
                            for_: *each.for_,
                        });
                        stack.eval(each.items);
                    }
                    ast::Statement::For(for_) => {
                        if let Some(times) = for_.times {
                            stack.push(Frame::For1 { for_: *for_.for_ });
                            stack.eval(times);
                        } else if let ast::For {
                            var: Some(var),
                            from: Some(from),
                            to: Some(to),
                            for_,
                            ..
                        } = *for_
                        {
                            stack.push(Frame::ForLet1 {
                                var,
                                to,
                                for_: *for_,
                            });
                            stack.eval(from);
                        }
                    }
                    ast::Statement::Loop(loop_) => stack.push(Frame::Loop1 {
                        statements: loop_.statements,
                    }),
                    ast::Statement::Break(_) => value_stack.push(Value::break_()),
                    ast::Statement::Continue(_) => value_stack.push(Value::continue_()),
                    ast::Statement::Assign(assign) => {
                        stack.push(Frame::Assign1 { dest: assign.dest });
                        stack.eval(assign.expr);
                    }
                    ast::Statement::AddAssign(add_assign) => {
                        stack.push(Frame::AddAssign1 {
                            dest: add_assign.dest.clone(),
                            expr: add_assign.expr,
                        });
                        stack.eval(add_assign.dest);
                    }
                    ast::Statement::SubAssign(sub_assign) => {
                        stack.push(Frame::SubAssign1 {
                            dest: sub_assign.dest.clone(),
                            expr: sub_assign.expr,
                        });
                        stack.eval(sub_assign.dest);
                    }
                },
                Frame::Expression(expression) => match expression {
                    ast::Expression::If(if_) => {
                        let mut elseif = if_.elseif;
                        elseif.reverse();
                        stack.push(Frame::If {
                            then: *if_.then,
                            elseif,
                            else_: if_.else_,
                        });
                        stack.eval(*if_.cond);
                    }
                    ast::Expression::Fn(fn_) => {
                        let scope = scope.clone();
                        value_stack.push(Value::fn_(
                            fn_.args.into_iter().map(|arg| arg.name),
                            fn_.children,
                            scope.into(),
                        ));
                    }
                    ast::Expression::Match(match_) => {
                        stack.push(Frame::Match1 {
                            qs: match_.qs,
                            default: match_.default,
                        });
                        stack.eval(*match_.about);
                    }
                    ast::Expression::Block(block) => {
                        scope = scope.create_child_scope(HashMap::new());
                        stack.push(Frame::Block);
                        stack.run(block.statements);
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
                        obj.reverse();
                        stack.push(Frame::Obj1 {
                            obj: obj.into(),
                            map: IndexMap::new().into(),
                        });
                    }
                    ast::Expression::Arr(arr) => {
                        let arr = try_join_all(
                            arr.value
                                .into_iter()
                                .map(|expr| self.run(vec![expr], &scope)),
                        )
                        .await?;
                        value_stack.push(Value::arr(arr));
                    }
                    ast::Expression::Not(not) => {
                        stack.push(Frame::Not);
                        stack.eval(*not.expr);
                    }
                    ast::Expression::And(and) => {
                        stack.push(Frame::And1 { right: *and.right });
                        stack.eval(*and.left);
                    }
                    ast::Expression::Or(or) => {
                        stack.push(Frame::Or1 { right: *or.right });
                        stack.eval(*or.left);
                    }
                    ast::Expression::Identifier(identifier) => {
                        let value = scope.get(&identifier.name).await?;
                        value_stack.push(value);
                    }
                    ast::Expression::Call(call) => {
                        stack.push(Frame::Call1 { args: call.args });
                        stack.eval(*call.target);
                    }
                    ast::Expression::Index(index) => {
                        stack.push(Frame::Index);
                        stack.eval(*index.index);
                        stack.eval(*index.target);
                    }
                    ast::Expression::Prop(prop) => {
                        stack.push(Frame::Prop { name: prop.name });
                        stack.eval(*prop.target);
                    }
                },
                Frame::Definition { name, mut_ } => {
                    let value = value_stack.pop_value()?;
                    scope
                        .add(
                            &name,
                            if mut_ {
                                Variable::Mut(value)
                            } else {
                                Variable::Const(value)
                            },
                        )
                        .await?;
                    value_stack.push(Value::null());
                }
                Frame::Return => {
                    let val = value_stack.pop_value()?;
                    value_stack.push(Value::return_(val));
                }
                Frame::Each1 { var, for_ } => {
                    let items = value_stack.pop_value()?;
                    let mut items = <Vec<Value>>::try_from(items)?;
                    items.reverse();
                    stack.push(Frame::Each2 { var, items, for_ });
                }
                Frame::Each2 {
                    var,
                    mut items,
                    for_,
                } => {
                    if let Some(item) = items.pop() {
                        stack.push(Frame::Each3 {
                            var: var.clone(),
                            items,
                            for_: for_.clone(),
                        });
                        scope = scope
                            .create_child_scope(HashMap::from_iter([(var, Variable::Const(item))]));
                        stack.eval(for_);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::Each3 { var, items, for_ } => {
                    scope = scope.get_parent()?;
                    let v = value_stack.pop_value()?;
                    match *v.value {
                        V::Break => value_stack.push(Value::null()),
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::Each2 { var, items, for_ }),
                    }
                }
                Frame::For1 { for_ } => {
                    let times = value_stack.pop_value()?;
                    let times = f64::try_from(times)?;
                    stack.push(Frame::For2 {
                        i: 0.0,
                        times,
                        for_,
                    });
                }
                Frame::For2 { i, times, for_ } => {
                    if i < times {
                        stack.push(Frame::For3 {
                            i,
                            times,
                            for_: for_.clone(),
                        });
                        stack.eval(for_);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::For3 { i, times, for_ } => {
                    let v = value_stack.pop_value()?;
                    match *v.value {
                        V::Break => value_stack.push(Value::null()),
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::For2 {
                            i: i + 1.0,
                            times,
                            for_,
                        }),
                    }
                }
                Frame::ForLet1 { var, to, for_ } => {
                    let from = value_stack.pop_value()?;
                    let from = f64::try_from(from)?;
                    stack.push(Frame::ForLet2 { var, from, for_ });
                    stack.eval(to);
                }
                Frame::ForLet2 { var, from, for_ } => {
                    let to = value_stack.pop_value()?;
                    let to = f64::try_from(to)?;
                    stack.push(Frame::ForLet3 {
                        var,
                        i: from,
                        until: from + to,
                        for_,
                    });
                }
                Frame::ForLet3 {
                    var,
                    i,
                    until,
                    for_,
                } => {
                    if i < until {
                        stack.push(Frame::ForLet4 {
                            var: var.clone(),
                            i,
                            until,
                            for_: for_.clone(),
                        });
                        scope = scope.create_child_scope(HashMap::from_iter([(
                            var,
                            Variable::Const(Value::num(i)),
                        )]));
                        stack.eval(for_);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::ForLet4 {
                    var,
                    i,
                    until,
                    for_,
                } => {
                    scope = scope.get_parent()?;
                    let v = value_stack.pop_value()?;
                    match *v.value {
                        V::Break => value_stack.push(Value::null()),
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::ForLet3 {
                            var,
                            i: i + 1.0,
                            until,
                            for_,
                        }),
                    }
                }
                Frame::Loop1 { statements } => {
                    stack.push(Frame::Loop2 {
                        statements: statements.clone(),
                    });
                    scope = scope.create_child_scope(HashMap::new());
                    stack.run(statements);
                }
                Frame::Loop2 { statements } => {
                    scope = scope.get_parent()?;
                    let v = value_stack.pop_value()?;
                    match *v.value {
                        V::Break => value_stack.push(Value::null()),
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::Loop1 { statements }),
                    }
                }
                Frame::Assign1 { dest } => {
                    let value = value_stack.pop_value()?;
                    stack.push(Frame::Assign2 { dest, value });
                }
                Frame::Assign2 { dest, value } => match dest {
                    ast::Expression::Identifier(identifier) => {
                        scope.assign(&identifier.name, value).await?;
                        value_stack.push(Value::null());
                    }
                    ast::Expression::Index(index) => {
                        stack.push(Frame::AssignIndex { value });
                        stack.eval(*index.index);
                        stack.eval(*index.target);
                    }
                    ast::Expression::Prop(prop) => {
                        stack.push(Frame::AssignProp {
                            name: prop.name,
                            value,
                        });
                        stack.eval(*prop.target);
                    }
                    ast::Expression::Arr(arr) => {
                        let value = <Vec<Value>>::try_from(value)?;
                        try_join_all(arr.value.into_iter().enumerate().map(|(index, item)| {
                            self.run(
                                vec![Frame::Assign2 {
                                    dest: item,
                                    value: value.get(index).cloned().unwrap_or_default(),
                                }],
                                &scope,
                            )
                        }))
                        .await?;
                        value_stack.push(Value::null());
                    }
                    ast::Expression::Obj(obj) => {
                        let value = <IndexMap<String, Value>>::try_from(value)?;
                        try_join_all(obj.value.into_iter().map(|(key, item)| {
                            self.run(
                                vec![Frame::Assign2 {
                                    dest: item,
                                    value: value.get(&key).cloned().unwrap_or_default(),
                                }],
                                &scope,
                            )
                        }))
                        .await?;
                        value_stack.push(Value::null());
                    }
                    _ => Err(AiScriptRuntimeError::runtime(
                        "The left-hand side of an assignment expression must be a variable or a \
                            property/index access.",
                    ))?,
                },
                Frame::AssignIndex { value } => {
                    let i = value_stack.pop_value()?;
                    let assignee = value_stack.pop_value()?;
                    match *assignee.value {
                        V::Arr(arr) => {
                            let i = f64::try_from(i)?;
                            if i.trunc() == i
                                && arr
                                    .read()
                                    .map_err(AiScriptError::internal)?
                                    .get(i as usize)
                                    .is_some()
                            {
                                arr.write().map_err(AiScriptError::internal)?[i as usize] = value;
                                value_stack.push(Value::null());
                            } else {
                                Err(AiScriptRuntimeError::index_out_of_range(
                                    i,
                                    arr.read().map_err(AiScriptError::internal)?.len() as isize - 1,
                                ))?
                            }
                        }
                        V::Obj(obj) => {
                            let i = String::try_from(i)?;
                            obj.write()
                                .map_err(AiScriptError::internal)?
                                .insert(i, value);
                            value_stack.push(Value::null());
                        }
                        _ => Err(AiScriptRuntimeError::runtime(format!(
                            "Cannot read prop ({}) of {}.",
                            i.value.repr_value(),
                            assignee.value.display_type()
                        )))?,
                    }
                }
                Frame::AssignProp { name, value } => {
                    let assignee = value_stack.pop_value()?;
                    let assignee = VObj::try_from(assignee)?;
                    assignee
                        .write()
                        .map_err(AiScriptError::internal)?
                        .insert(name, value);
                }
                Frame::AddAssign1 { dest, expr } => {
                    let target = value_stack.pop_value()?;
                    let target = f64::try_from(target)?;
                    stack.push(Frame::AddAssign2 { dest, target });
                    stack.eval(expr);
                }
                Frame::AddAssign2 { dest, target } => {
                    let v = value_stack.pop_value()?;
                    let v = f64::try_from(v)?;
                    stack.push(Frame::Assign2 {
                        dest,
                        value: Value::num(target + v),
                    });
                }
                Frame::SubAssign1 { dest, expr } => {
                    let target = value_stack.pop_value()?;
                    let target = f64::try_from(target)?;
                    stack.push(Frame::SubAssign2 { dest, target });
                    stack.eval(expr);
                }
                Frame::SubAssign2 { dest, target } => {
                    let v = value_stack.pop_value()?;
                    let v = f64::try_from(v)?;
                    stack.push(Frame::Assign2 {
                        dest,
                        value: Value::num(target - v),
                    });
                }
                Frame::If {
                    then,
                    mut elseif,
                    else_,
                } => {
                    let cond = value_stack.pop_value()?;
                    let cond = bool::try_from(cond)?;
                    if cond {
                        stack.eval(then);
                    } else if let Some(ast::Elseif { cond, then }) = elseif.pop() {
                        stack.push(Frame::If {
                            then,
                            elseif,
                            else_,
                        });
                        stack.eval(cond);
                    } else if let Some(else_) = else_ {
                        stack.eval(*else_);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::Match1 { mut qs, default } => {
                    let about = value_stack.pop_value()?;
                    qs.reverse();
                    stack.push(Frame::Match2 { about, qs, default });
                }
                Frame::Match2 {
                    about,
                    mut qs,
                    default,
                } => {
                    if let Some(ast::QA { q, a }) = qs.pop() {
                        stack.push(Frame::Match3 {
                            about,
                            a: a.into(),
                            qs,
                            default,
                        });
                        stack.eval(q);
                    } else if let Some(default) = default {
                        stack.eval(*default);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::Match3 {
                    about,
                    a,
                    qs,
                    default,
                } => {
                    let q = value_stack.pop_value()?;
                    if about == q {
                        stack.eval(*a);
                    } else {
                        stack.push(Frame::Match2 { about, qs, default });
                    }
                }
                Frame::Block => scope = scope.get_parent()?,
                Frame::Tmpl1 { mut tmpl, mut str } => {
                    if let Some(x) = tmpl.pop() {
                        match x {
                            ast::StringOrExpression::String(x) => {
                                str += &x;
                                stack.push(Frame::Tmpl1 { tmpl, str });
                            }
                            ast::StringOrExpression::Expression(x) => {
                                stack.push(Frame::Tmpl2 { tmpl, str });
                                stack.eval(x);
                            }
                        }
                    } else {
                        value_stack.push(Value::str(str));
                    }
                }
                Frame::Tmpl2 { tmpl, mut str } => {
                    let v = value_stack.pop_value()?;
                    str += &v.repr_value().to_string();
                    stack.push(Frame::Tmpl1 { tmpl, str });
                }
                Frame::Obj1 { mut obj, map } => {
                    if let Some((k, v)) = obj.pop() {
                        stack.push(Frame::Obj2 { obj, map, k });
                        stack.eval(v);
                    } else {
                        value_stack.push(Value::obj(*map));
                    }
                }
                Frame::Obj2 { obj, mut map, k } => {
                    let v = value_stack.pop_value()?;
                    map.insert(k, v);
                    stack.push(Frame::Obj1 { obj, map });
                }
                Frame::Not => {
                    let v = value_stack.pop_value()?;
                    let v = bool::try_from(v)?;
                    value_stack.push(Value::bool(!v));
                }
                Frame::And1 { right } => {
                    let left_value = value_stack.pop_value()?;
                    let left_value = bool::try_from(left_value)?;
                    if !left_value {
                        value_stack.push(Value::bool(left_value))
                    } else {
                        stack.push(Frame::And2);
                        stack.eval(right);
                    }
                }
                Frame::And2 => {
                    let right_value = value_stack.pop_value()?;
                    let right_value = bool::try_from(right_value)?;
                    value_stack.push(Value::bool(right_value))
                }
                Frame::Or1 { right } => {
                    let left_value = value_stack.pop_value()?;
                    let left_value = bool::try_from(left_value)?;
                    if left_value {
                        value_stack.push(Value::bool(left_value))
                    } else {
                        stack.push(Frame::Or2);
                        stack.eval(right);
                    }
                }
                Frame::Or2 => {
                    let right_value = value_stack.pop_value()?;
                    let right_value = bool::try_from(right_value)?;
                    value_stack.push(Value::bool(right_value))
                }
                Frame::Call1 { args } => {
                    let callee = value_stack.pop_value()?;
                    let callee = VFn::try_from(callee)?;
                    let args =
                        try_join_all(args.into_iter().map(|arg| self.run(vec![arg], &scope)))
                            .await?;
                    stack.push(Frame::Call2 { callee, args });
                }
                Frame::Call2 { callee, args } => match callee {
                    VFn::Fn {
                        args: fn_args,
                        statements,
                        scope: fn_scope,
                    } => {
                        let args = zip(
                            fn_args,
                            args.into_iter()
                                .chain(repeat(Value::null()))
                                .map(Variable::Mut),
                        )
                        .collect();
                        stack.push(Frame::Call3 { scope });
                        scope = fn_scope.create_child_scope(args);
                        stack.run(statements);
                    }
                    VFn::FnNative(fn_) => {
                        value_stack.push(fn_(args.into_iter().collect(), self).await?);
                    }
                },
                Frame::Call3 {
                    scope: previous_scope,
                } => {
                    scope = previous_scope;
                    let r = value_stack.pop_value()?;
                    value_stack.push(unwrap_ret(r));
                }
                Frame::Index => {
                    let i = value_stack.pop_value()?;
                    let target = value_stack.pop_value()?;
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
                                Err(AiScriptRuntimeError::index_out_of_range(
                                    i,
                                    arr.read().map_err(AiScriptError::internal)?.len() as isize - 1,
                                ))?
                            });
                        }
                        V::Obj(obj) => {
                            let i = String::try_from(i)?;
                            value_stack.push(
                                if let Some(item) =
                                    obj.read().map_err(AiScriptError::internal)?.get(&i)
                                {
                                    item.clone()
                                } else {
                                    Value::null()
                                },
                            );
                        }
                        target => Err(AiScriptRuntimeError::runtime(format!(
                            "Cannot read prop ({}) of {}.",
                            i.value.repr_value(),
                            target.display_type(),
                        )))?,
                    }
                }
                Frame::Prop { name } => {
                    let value = value_stack.pop_value()?;
                    value_stack.push(if let V::Obj(value) = *value.value {
                        if let Some(value) =
                            value.read().map_err(AiScriptError::internal)?.get(&name)
                        {
                            value.clone()
                        } else {
                            Value::null()
                        }
                    } else {
                        get_prim_prop(value, &name)?
                    });
                }
                Frame::Run => {
                    if value_stack.is_empty() {
                        value_stack.push(Value::null());
                    }
                }
                Frame::Unwind => {
                    if let Some(v) = value_stack.last()
                        && let V::Return(_) | V::Break | V::Continue = *v.value
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
                        Err(AiScriptRuntimeError::runtime("max step exceeded"))?
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
