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
    frame::Frame,
    lib::std::std,
    primitive_props::get_prim_prop,
    scope::Scope,
    util::expect_any,
    value::{V, VFn, Value, unwrap_ret},
    variable::Variable,
};

mod frame;
mod lib;
mod primitive_props;
pub mod scope;
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
                    let out = out.clone();
                    async move {
                        let mut args = args.into_iter();
                        let v = expect_any(args.next())?;
                        if let Some(out) = out {
                            out(v).await;
                        }
                        Ok(Value::null())
                    }
                    .boxed()
                }),
            ),
            (
                "readline".to_string(),
                Value::fn_native(move |args, _| {
                    let in_ = in_.clone();
                    async move {
                        let mut args = args.into_iter();
                        let q = String::try_from(args.next().unwrap_or_default())?;
                        if let Some(in_) = in_ {
                            let a = in_(q).await;
                            Ok(Value::str(a))
                        } else {
                            Ok(Value::null())
                        }
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
                if let Some(err) = &self.err {
                    if !self.stop.load(Ordering::SeqCst) {
                        self.abort().await;
                        err(e).await;
                        return Ok(None);
                    }
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
                        Err(AiScriptError::Internal(
                            "Namespaces cannot include mutable variable: {name}".to_string(),
                        ))?;
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
        let mut scope = scope.clone();
        let mut value_stack = Vec::new();

        fn eval(node: impl Into<Frame>, stack: &mut Vec<Frame>) {
            stack.push(node.into());
            stack.push(Frame::Eval);
        }

        fn run(program: Vec<impl Into<Frame>>, stack: &mut Vec<Frame>) {
            stack.push(Frame::Run);
            for node in program.into_iter().rev() {
                stack.push(Frame::Unwind);
                eval(node, stack);
            }
        }

        run(program, &mut stack);

        while let Some(frame) = stack.pop() {
            match frame {
                Frame::Statement(statement) => match statement {
                    ast::Statement::Definition(definition) => {
                        stack.push(Frame::Definition {
                            name: definition.name,
                            mut_: definition.mut_,
                        });
                        eval(definition.expr, &mut stack);
                    }
                    ast::Statement::Return(return_) => {
                        stack.push(Frame::Return);
                        eval(return_.expr, &mut stack);
                    }
                    ast::Statement::Each(each) => {
                        stack.push(Frame::Each1 {
                            var: each.var,
                            for_: *each.for_,
                        });
                        eval(each.items, &mut stack);
                    }
                    ast::Statement::For(for_) => {
                        if let Some(times) = for_.times {
                            stack.push(Frame::For1 { for_: *for_.for_ });
                            eval(times, &mut stack);
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
                            eval(from, &mut stack);
                        }
                    }
                    ast::Statement::Loop(loop_) => stack.push(Frame::Loop1 {
                        statements: loop_.statements,
                    }),
                    ast::Statement::Break(_) => value_stack.push(Value::break_()),
                    ast::Statement::Continue(_) => value_stack.push(Value::continue_()),
                    ast::Statement::Assign(assign) => {
                        stack.push(Frame::Assign1 { dest: assign.dest });
                        eval(assign.expr, &mut stack);
                    }
                    ast::Statement::AddAssign(add_assign) => {
                        stack.push(Frame::AddAssign1 {
                            dest: add_assign.dest.clone(),
                            expr: add_assign.expr,
                        });
                        eval(add_assign.dest, &mut stack);
                    }
                    ast::Statement::SubAssign(sub_assign) => {
                        stack.push(Frame::SubAssign1 {
                            dest: sub_assign.dest.clone(),
                            expr: sub_assign.expr,
                        });
                        eval(sub_assign.dest, &mut stack);
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
                        eval(*if_.cond, &mut stack);
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
                        eval(*match_.about, &mut stack);
                    }
                    ast::Expression::Block(block) => {
                        scope = scope.create_child_scope(HashMap::new());
                        stack.push(Frame::Block);
                        run(block.statements, &mut stack);
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
                        eval(*not.expr, &mut stack);
                    }
                    ast::Expression::And(and) => {
                        stack.push(Frame::And1 { right: *and.right });
                        eval(*and.left, &mut stack);
                    }
                    ast::Expression::Or(or) => {
                        stack.push(Frame::Or1 { right: *or.right });
                        eval(*or.left, &mut stack);
                    }
                    ast::Expression::Identifier(identifier) => {
                        let value = scope.get(&identifier.name).await?;
                        value_stack.push(value);
                    }
                    ast::Expression::Call(call) => {
                        stack.push(Frame::Call1 { args: call.args });
                        eval(*call.target, &mut stack);
                    }
                    ast::Expression::Index(index) => {
                        stack.push(Frame::Index);
                        eval(*index.index, &mut stack);
                        eval(*index.target, &mut stack);
                    }
                    ast::Expression::Prop(prop) => {
                        stack.push(Frame::Prop { name: prop.name });
                        eval(*prop.target, &mut stack);
                    }
                },
                Frame::Definition { name, mut_ } => {
                    let value = value_stack.pop().unwrap();
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
                    let val = value_stack.pop().unwrap();
                    value_stack.push(Value::return_(val));
                }
                Frame::Each1 { var, for_ } => {
                    let items = value_stack.pop().unwrap();
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
                        eval(for_, &mut stack);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::Each3 { var, items, for_ } => {
                    scope = *scope.parent.unwrap();
                    let v = value_stack.pop().unwrap();
                    match *v.value {
                        V::Break => value_stack.push(Value::null()),
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::Each2 { var, items, for_ }),
                    }
                }
                Frame::For1 { for_ } => {
                    let times = value_stack.pop().unwrap();
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
                        eval(for_, &mut stack);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::For3 { i, times, for_ } => {
                    let v = value_stack.pop().unwrap();
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
                    let from = value_stack.pop().unwrap();
                    let from = f64::try_from(from)?;
                    stack.push(Frame::ForLet2 { var, from, for_ });
                    eval(to, &mut stack);
                }
                Frame::ForLet2 { var, from, for_ } => {
                    let to = value_stack.pop().unwrap();
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
                        eval(for_, &mut stack);
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
                    scope = *scope.parent.unwrap();
                    let v = value_stack.pop().unwrap();
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
                    run(statements, &mut stack);
                }
                Frame::Loop2 { statements } => {
                    scope = *scope.parent.unwrap();
                    let v = value_stack.pop().unwrap();
                    match *v.value {
                        V::Break => value_stack.push(Value::null()),
                        V::Return(_) => value_stack.push(v),
                        _ => stack.push(Frame::Loop1 { statements }),
                    }
                }
                Frame::Assign1 { dest } => {
                    let value = value_stack.pop().unwrap();
                    stack.push(Frame::Assign2 { dest, value });
                }
                Frame::Assign2 { dest, value } => match dest {
                    ast::Expression::Identifier(identifier) => {
                        scope.assign(&identifier.name, value).await?;
                        value_stack.push(Value::null());
                    }
                    ast::Expression::Index(index) => {
                        stack.push(Frame::AssignIndex { value });
                        eval(*index.index, &mut stack);
                        eval(*index.target, &mut stack);
                    }
                    ast::Expression::Prop(prop) => {
                        stack.push(Frame::AssignProp {
                            name: prop.name,
                            value,
                        });
                        eval(*prop.target, &mut stack);
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
                    _ => Err(AiScriptRuntimeError::Runtime(
                        "The left-hand side of an assignment expression must be a variable or a \
                            property/index access."
                            .to_string(),
                    ))?,
                },
                Frame::AssignIndex { value } => {
                    let i = value_stack.pop().unwrap();
                    let assignee = value_stack.pop().unwrap();
                    match *assignee.value {
                        V::Arr(arr) => {
                            let i = f64::try_from(i)?;
                            if i.trunc() == i && arr.read().unwrap().get(i as usize).is_some() {
                                arr.write().unwrap()[i as usize] = value;
                                value_stack.push(Value::null());
                            } else {
                                Err(AiScriptRuntimeError::IndexOutOfRange(
                                    i,
                                    arr.read().unwrap().len() as isize - 1,
                                ))?
                            }
                        }
                        V::Obj(obj) => {
                            let i = String::try_from(i)?;
                            obj.write().unwrap().insert(i, value);
                            value_stack.push(Value::null());
                        }
                        _ => Err(AiScriptRuntimeError::Runtime(format!(
                            "Cannot read prop ({}) of {}.",
                            i.value.repr_value(),
                            assignee.value.display_type()
                        )))?,
                    }
                }
                Frame::AssignProp { name, value } => {
                    let assignee = value_stack.pop().unwrap();
                    let assignee = VObj::try_from(assignee)?;
                    assignee.write().unwrap().insert(name, value);
                }
                Frame::AddAssign1 { dest, expr } => {
                    let target = value_stack.pop().unwrap();
                    let target = f64::try_from(target)?;
                    stack.push(Frame::AddAssign2 { dest, target });
                    eval(expr, &mut stack);
                }
                Frame::AddAssign2 { dest, target } => {
                    let v = value_stack.pop().unwrap();
                    let v = f64::try_from(v)?;
                    stack.push(Frame::Assign2 {
                        dest,
                        value: Value::num(target + v),
                    });
                }
                Frame::SubAssign1 { dest, expr } => {
                    let target = value_stack.pop().unwrap();
                    let target = f64::try_from(target)?;
                    stack.push(Frame::SubAssign2 { dest, target });
                    eval(expr, &mut stack);
                }
                Frame::SubAssign2 { dest, target } => {
                    let v = value_stack.pop().unwrap();
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
                    let cond = value_stack.pop().unwrap();
                    let cond = bool::try_from(cond)?;
                    if cond {
                        eval(then, &mut stack);
                    } else if let Some(ast::Elseif { cond, then }) = elseif.pop() {
                        stack.push(Frame::If {
                            then,
                            elseif,
                            else_,
                        });
                        eval(cond, &mut stack);
                    } else if let Some(else_) = else_ {
                        eval(*else_, &mut stack);
                    } else {
                        value_stack.push(Value::null());
                    }
                }
                Frame::Match1 { mut qs, default } => {
                    let about = value_stack.pop().unwrap();
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
                        eval(q, &mut stack);
                    } else if let Some(default) = default {
                        eval(*default, &mut stack);
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
                    let q = value_stack.pop().unwrap();
                    if about == q {
                        eval(*a, &mut stack);
                    } else {
                        stack.push(Frame::Match2 { about, qs, default });
                    }
                }
                Frame::Block => scope = *scope.parent.unwrap(),
                Frame::Tmpl1 { mut tmpl, mut str } => {
                    if let Some(x) = tmpl.pop() {
                        match x {
                            ast::StringOrExpression::String(x) => {
                                str += &x;
                                stack.push(Frame::Tmpl1 { tmpl, str });
                            }
                            ast::StringOrExpression::Expression(x) => {
                                stack.push(Frame::Tmpl2 { tmpl, str });
                                eval(x, &mut stack);
                            }
                        }
                    } else {
                        value_stack.push(Value::str(str));
                    }
                }
                Frame::Tmpl2 { tmpl, mut str } => {
                    let v = value_stack.pop().unwrap();
                    str += &v.repr_value().to_string();
                    stack.push(Frame::Tmpl1 { tmpl, str });
                }
                Frame::Obj1 { mut obj, map } => {
                    if let Some((k, v)) = obj.pop() {
                        stack.push(Frame::Obj2 { obj, map, k });
                        eval(v, &mut stack);
                    } else {
                        value_stack.push(Value::obj(*map));
                    }
                }
                Frame::Obj2 { obj, mut map, k } => {
                    let v = value_stack.pop().unwrap();
                    map.insert(k, v);
                    stack.push(Frame::Obj1 { obj, map });
                }
                Frame::Not => {
                    let v = value_stack.pop().unwrap();
                    let v = bool::try_from(v)?;
                    value_stack.push(Value::bool(!v));
                }
                Frame::And1 { right } => {
                    let left_value = value_stack.pop().unwrap();
                    let left_value = bool::try_from(left_value)?;
                    if !left_value {
                        value_stack.push(Value::bool(left_value))
                    } else {
                        stack.push(Frame::And2);
                        eval(right, &mut stack);
                    }
                }
                Frame::And2 => {
                    let right_value = value_stack.pop().unwrap();
                    let right_value = bool::try_from(right_value)?;
                    value_stack.push(Value::bool(right_value))
                }
                Frame::Or1 { right } => {
                    let left_value = value_stack.pop().unwrap();
                    let left_value = bool::try_from(left_value)?;
                    if left_value {
                        value_stack.push(Value::bool(left_value))
                    } else {
                        stack.push(Frame::Or2);
                        eval(right, &mut stack);
                    }
                }
                Frame::Or2 => {
                    let right_value = value_stack.pop().unwrap();
                    let right_value = bool::try_from(right_value)?;
                    value_stack.push(Value::bool(right_value))
                }
                Frame::Call1 { args } => {
                    let callee = value_stack.pop().unwrap();
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
                        run(statements, &mut stack);
                    }
                    VFn::FnNative(fn_) => {
                        value_stack.push(fn_(args.into_iter().collect(), self).await?);
                    }
                },
                Frame::Call3 {
                    scope: previous_scope,
                } => {
                    scope = previous_scope;
                    let r = value_stack.pop().unwrap();
                    value_stack.push(unwrap_ret(r));
                }
                Frame::Index => {
                    let i = value_stack.pop().unwrap();
                    let target = value_stack.pop().unwrap();
                    match *target.value {
                        V::Arr(arr) => {
                            let i = f64::try_from(i)?;
                            let item = if i.trunc() == i {
                                arr.read().unwrap().get(i as usize).cloned()
                            } else {
                                None
                            };
                            value_stack.push(if let Some(item) = item {
                                item
                            } else {
                                Err(AiScriptRuntimeError::IndexOutOfRange(
                                    i,
                                    arr.read().unwrap().len() as isize - 1,
                                ))?
                            });
                        }
                        V::Obj(obj) => {
                            let i = String::try_from(i)?;
                            value_stack.push(if let Some(item) = obj.read().unwrap().get(&i) {
                                item.clone()
                            } else {
                                Value::null()
                            });
                        }
                        target => Err(AiScriptRuntimeError::Runtime(format!(
                            "Cannot read prop ({}) of {}.",
                            i.value.repr_value(),
                            target.display_type(),
                        )))?,
                    }
                }
                Frame::Prop { name } => {
                    let value = value_stack.pop().unwrap();
                    value_stack.push(if let V::Obj(value) = *value.value {
                        if let Some(value) = value.read().unwrap().get(&name) {
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
                    if let Some(v) = value_stack.last() {
                        if let V::Return(_) | V::Break | V::Continue = *v.value {
                            while let Some(frame) = stack.pop() {
                                if let Frame::Run = frame {
                                    break;
                                }
                            }
                        }
                    }
                }
                Frame::Eval => {
                    if self.stop.load(Ordering::Acquire) {
                        return Ok(Value::null());
                    }
                    let step_count = self.step_count.fetch_add(1, Ordering::Relaxed);
                    if let Some(max_step) = self.max_step {
                        if step_count > max_step {
                            Err(AiScriptRuntimeError::Runtime(
                                "max step exceeded".to_string(),
                            ))?
                        }
                    }
                }
            }
        }

        Ok(value_stack.pop().unwrap())
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
