//! AiScript interpreter

use std::{
    collections::HashMap,
    iter::{repeat, zip},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use futures::{
    Future, FutureExt,
    future::{BoxFuture, try_join_all},
};
use indexmap::IndexMap;
use value::VObj;

use crate::{
    error::{AiScriptError, AiScriptRuntimeError},
    node as ast,
};

use self::{
    lib::std::std,
    primitive_props::get_prim_prop,
    scope::Scope,
    util::expect_any,
    value::{Attr, V, VFn, Value, unwrap_ret},
    variable::Variable,
};

mod lib;
mod primitive_props;
pub mod scope;
pub mod util;
pub mod value;
mod variable;

const IRQ_RATE: usize = 300;
const IRQ_AT: usize = IRQ_RATE - 1;

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
            scope: Scope::new(states, None),
            abort_handlers: Arc::new(Mutex::new(tokio::task::JoinSet::new())),
            err: match err {
                Some(err) => Some(Arc::new(err)),
                None => None,
            },
            max_step,
        }
    }

    pub async fn exec(&self, script: Vec<ast::Node>) -> Result<Option<Value>, AiScriptError> {
        self.stop.store(false, Ordering::SeqCst);
        let script = self.collect_ns(script, self.scope.clone()).await?;
        let result = self.run(&script, &self.scope).await;
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
                        self.abort();
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
        scope: Scope,
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
            let ns_scope = scope.create_child_namespace_scope(ns.name, HashMap::new(), None);
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
                        let variable = Variable::Const(
                            self.eval_expression(&definition.expr, &ns_scope).await?,
                        );
                        ns_scope.add(&definition.name, variable)?;
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
                    self.run(&statements, &scope.create_child_scope(args, None))
                        .map(|r| r.map(unwrap_ret))
                        .await
                }
                .boxed()
            }
            VFn::FnNative(fn_) => fn_(args.into_iter().collect(), self),
        }
    }

    fn eval<'a>(
        &'a self,
        statement_or_expression: &'a ast::StatementOrExpression,
        scope: &'a Scope,
    ) -> BoxFuture<'a, Result<Value, AiScriptError>> {
        if self.stop.load(Ordering::SeqCst) {
            return async move { Ok(Value::null()) }.boxed();
        }
        async move {
            let step_count = self.step_count.load(Ordering::SeqCst);
            if step_count % IRQ_RATE == IRQ_AT {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            let step_count = self.step_count.fetch_add(1, Ordering::SeqCst);
            if let Some(max_step) = self.max_step {
                if step_count > max_step {
                    Err(AiScriptRuntimeError::Runtime(
                        "max step exceeded".to_string(),
                    ))?
                }
            }
            Ok(match statement_or_expression {
                ast::StatementOrExpression::Statement(statement) => match statement {
                    ast::Statement::Definition(definition) => {
                        let value = self.eval_expression(&definition.expr, scope).await?;
                        let attr = match &definition.attr {
                            Some(attr) => {
                                let mut attrs = Vec::new();
                                for n_attr in attr {
                                    attrs.push(Attr {
                                        name: n_attr.name.to_string(),
                                        value: self.eval_expression(&n_attr.value, scope).await?,
                                    })
                                }
                                Some(attrs)
                            }
                            None => None,
                        };
                        scope.add(
                            &definition.name,
                            if definition.mut_ {
                                Variable::Mut(Value { attr, ..value })
                            } else {
                                Variable::Const(Value { attr, ..value })
                            },
                        )?;
                        Value::null()
                    }
                    ast::Statement::Return(return_) => {
                        let val = self.eval_expression(&return_.expr, scope).await?;
                        Value::return_(val)
                    }
                    ast::Statement::Each(each) => {
                        let items = self.eval_expression(&each.items, scope).await?;
                        let items = <Vec<Value>>::try_from(items)?;
                        for item in items {
                            let scope = scope.create_child_scope(
                                HashMap::from_iter([(each.var.clone(), Variable::Const(item))]),
                                None,
                            );
                            let v = self.eval(&each.for_, &scope).await?;
                            match *v.value {
                                V::Break => {
                                    break;
                                }
                                V::Return(_) => {
                                    return Ok(v);
                                }
                                _ => (),
                            }
                        }
                        Value::null()
                    }
                    ast::Statement::For(for_) => {
                        if let Some(times) = &for_.times {
                            let times = self.eval_expression(times, scope).await?;
                            let times = f64::try_from(times)?;
                            let mut i = 0.0;
                            while i < times {
                                let v = self.eval(&for_.for_, scope).await?;
                                match *v.value {
                                    V::Break => {
                                        break;
                                    }
                                    V::Return(_) => {
                                        return Ok(v);
                                    }
                                    _ => (),
                                }
                                i += 1.0;
                            }
                        } else if let (Some(from), Some(to), Some(var)) =
                            (&for_.from, &for_.to, &for_.var)
                        {
                            let from = self.eval_expression(from, scope).await?;
                            let to = self.eval_expression(to, scope).await?;
                            let from = f64::try_from(from)?;
                            let to = f64::try_from(to)?;
                            let mut i = from;
                            while i < from + to {
                                let scope = scope.create_child_scope(
                                    HashMap::from_iter([(
                                        var.clone(),
                                        Variable::Const(Value::num(i)),
                                    )]),
                                    None,
                                );
                                let v = self.eval(&for_.for_, &scope).await?;
                                match *v.value {
                                    V::Break => {
                                        break;
                                    }
                                    V::Return(_) => {
                                        return Ok(v);
                                    }
                                    _ => (),
                                }
                                i += 1.0;
                            }
                        }
                        Value::null()
                    }
                    ast::Statement::Loop(loop_) => loop {
                        let v = self
                            .run(
                                &loop_.statements,
                                &scope.create_child_scope(HashMap::new(), None),
                            )
                            .await?;
                        match *v.value {
                            V::Break => {
                                break Value::null();
                            }
                            V::Return(_) => {
                                break v;
                            }
                            _ => (),
                        }
                    },
                    ast::Statement::Break(_) => Value::break_(),
                    ast::Statement::Continue(_) => Value::continue_(),
                    ast::Statement::Assign(assign) => {
                        let v = self.eval_expression(&assign.expr, scope).await?;
                        self.assign(scope, &assign.dest, v).await?;
                        Value::null()
                    }
                    ast::Statement::AddAssign(add_assign) => {
                        let target = self.eval_expression(&add_assign.dest, scope).await?;
                        let target = f64::try_from(target)?;
                        let v = self.eval_expression(&add_assign.expr, scope).await?;
                        let v = f64::try_from(v)?;
                        self.assign(scope, &add_assign.dest, Value::num(target + v))
                            .await?;
                        Value::null()
                    }
                    ast::Statement::SubAssign(sub_assign) => {
                        let target = self.eval_expression(&sub_assign.dest, scope).await?;
                        let target = f64::try_from(target)?;
                        let v = self.eval_expression(&sub_assign.expr, scope).await?;
                        let v = f64::try_from(v)?;
                        self.assign(scope, &sub_assign.dest, Value::num(target - v))
                            .await?;
                        Value::null()
                    }
                },
                ast::StatementOrExpression::Expression(expression) => {
                    self.eval_expression(expression, scope).await?
                }
            })
        }
        .boxed()
    }

    fn eval_expression<'a>(
        &'a self,
        expression: &'a ast::Expression,
        scope: &'a Scope,
    ) -> BoxFuture<'a, Result<Value, AiScriptError>> {
        async move {
            Ok(match expression {
                ast::Expression::If(if_) => {
                    let cond = self.eval_expression(&if_.cond, scope).await?;
                    let cond = bool::try_from(cond)?;
                    if cond {
                        self.eval(&if_.then, scope).await?
                    } else {
                        for ast::Elseif { cond, then } in &if_.elseif {
                            let cond = self.eval_expression(cond, scope).await?;
                            let cond = bool::try_from(cond)?;
                            if cond {
                                return self.eval(then, scope).await;
                            }
                        }
                        if let Some(else_) = &if_.else_ {
                            self.eval(else_, scope).await?
                        } else {
                            Value::null()
                        }
                    }
                }
                ast::Expression::Fn(fn_) => Value::fn_(
                    fn_.args.iter().map(|arg| arg.name.to_string()),
                    fn_.children.clone(),
                    scope.clone().into(),
                ),
                ast::Expression::Match(match_) => {
                    let about = self.eval_expression(&match_.about, scope).await?;
                    for ast::QA { q, a } in &match_.qs {
                        let q = self.eval_expression(q, scope).await?;
                        if about == q {
                            return self.eval(a, scope).await;
                        }
                    }
                    if let Some(default) = &match_.default {
                        self.eval(default, scope).await?
                    } else {
                        Value::null()
                    }
                }
                ast::Expression::Block(block) => {
                    self.run(
                        &block.statements,
                        &scope.create_child_scope(HashMap::new(), None),
                    )
                    .await?
                }
                ast::Expression::Exists(exists) => {
                    Value::bool(scope.exists(&exists.identifier.name))
                }
                ast::Expression::Tmpl(tmpl) => {
                    let mut str = String::new();
                    for x in &tmpl.tmpl {
                        match x {
                            ast::StringOrExpression::String(x) => str += x,
                            ast::StringOrExpression::Expression(x) => {
                                let v = self.eval_expression(x, scope).await?;
                                str += &v.value.repr_value().to_string()
                            }
                        }
                    }
                    Value::str(str)
                }
                ast::Expression::Str(str) => Value::str(&str.value),
                ast::Expression::Num(num) => Value::num(num.value),
                ast::Expression::Bool(bool) => Value::bool(bool.value),
                ast::Expression::Null(_) => Value::null(),
                ast::Expression::Obj(obj) => {
                    let mut map = IndexMap::new();
                    for (k, v) in &obj.value {
                        map.insert(k, self.eval_expression(v, scope).await?);
                    }
                    Value::obj(map)
                }
                ast::Expression::Arr(arr) => Value::arr(
                    try_join_all(
                        arr.value
                            .iter()
                            .map(|expr| self.eval_expression(expr, scope)),
                    )
                    .await?,
                ),
                ast::Expression::Not(not) => {
                    let v = self.eval_expression(&not.expr, scope).await?;
                    let bool = bool::try_from(v)?;
                    Value::bool(!bool)
                }
                ast::Expression::And(and) => {
                    let Value {
                        value: left_value,
                        attr,
                    } = self.eval_expression(&and.left, scope).await?;
                    let left_value = bool::try_from(*left_value)?;
                    if !left_value {
                        Value {
                            value: Box::new(V::Bool(left_value)),
                            attr,
                        }
                    } else {
                        let Value {
                            value: right_value,
                            attr,
                        } = self.eval_expression(&and.right, scope).await?;
                        let right_value = bool::try_from(*right_value)?;
                        Value {
                            value: Box::new(V::Bool(right_value)),
                            attr,
                        }
                    }
                }
                ast::Expression::Or(or) => {
                    let Value {
                        value: left_value,
                        attr,
                    } = self.eval_expression(&or.left, scope).await?;
                    let left_value = bool::try_from(*left_value)?;
                    if left_value {
                        Value {
                            value: Box::new(V::Bool(left_value)),
                            attr,
                        }
                    } else {
                        let Value {
                            value: right_value,
                            attr,
                        } = self.eval_expression(&or.right, scope).await?;
                        let right_value = bool::try_from(*right_value)?;
                        Value {
                            value: Box::new(V::Bool(right_value)),
                            attr,
                        }
                    }
                }
                ast::Expression::Identifier(identifier) => scope.get(&identifier.name)?,
                ast::Expression::Call(call) => {
                    let callee = self.eval_expression(&call.target, scope).await?;
                    let callee = VFn::try_from(callee)?;
                    let args = try_join_all(
                        call.args
                            .iter()
                            .map(|expr| self.eval_expression(expr, scope)),
                    )
                    .await?;
                    self.fn_(callee, args).await?
                }
                ast::Expression::Index(index) => {
                    let target = self.eval_expression(&index.target, scope).await?;
                    let i = self.eval_expression(&index.index, scope).await?;
                    match *target.value {
                        V::Arr(arr) => {
                            let i = f64::try_from(i)?;
                            let item = if i.trunc() == i {
                                arr.read().unwrap().get(i as usize).cloned()
                            } else {
                                None
                            };
                            if let Some(item) = item {
                                item
                            } else {
                                Err(AiScriptRuntimeError::IndexOutOfRange(
                                    i,
                                    arr.read().unwrap().len() as isize - 1,
                                ))?
                            }
                        }
                        V::Obj(obj) => {
                            let i = String::try_from(i)?;
                            if let Some(item) = obj.read().unwrap().get(&i) {
                                item.clone()
                            } else {
                                Value::null()
                            }
                        }
                        target => Err(AiScriptRuntimeError::Runtime(format!(
                            "Cannot read prop ({}) of {}.",
                            i.value.repr_value(),
                            target.display_type(),
                        )))?,
                    }
                }
                ast::Expression::Prop(prop) => {
                    let value = self.eval_expression(&prop.target, scope).await?;
                    if let V::Obj(value) = *value.value {
                        if let Some(value) = value.read().unwrap().get(&prop.name) {
                            value.clone()
                        } else {
                            Value::null()
                        }
                    } else {
                        get_prim_prop(value, &prop.name)?
                    }
                }
            })
        }
        .boxed()
    }

    async fn run(
        &self,
        program: &[ast::StatementOrExpression],
        scope: &Scope,
    ) -> Result<Value, AiScriptError> {
        let mut v = Value::null();
        for node in program {
            v = self.eval(node, scope).await?;
            if let V::Return(_) | V::Break | V::Continue = *v.value {
                return Ok(v);
            }
        }
        Ok(v)
    }

    pub fn register_abort_handler(
        &self,
        task: impl Future<Output = Result<(), AiScriptError>> + Send + 'static,
    ) -> tokio::task::AbortHandle {
        self.abort_handlers.lock().unwrap().spawn(task)
    }

    pub fn abort(&self) {
        self.stop.store(true, Ordering::SeqCst);
        self.abort_handlers.lock().unwrap().abort_all();
    }

    fn assign<'a>(
        &'a self,
        scope: &'a Scope,
        dest: &'a ast::Expression,
        value: Value,
    ) -> BoxFuture<'a, Result<(), AiScriptError>> {
        async move {
            match dest {
                ast::Expression::Identifier(identifier) => scope.assign(&identifier.name, value)?,
                ast::Expression::Index(index) => {
                    let assignee = self.eval_expression(&index.target, scope).await?;
                    let i = self.eval_expression(&index.index, scope).await?;
                    match *assignee.value {
                        V::Arr(arr) => {
                            let i = f64::try_from(i)?;
                            if i.trunc() == i && arr.read().unwrap().get(i as usize).is_some() {
                                arr.write().unwrap()[i as usize] = value;
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
                        }
                        _ => Err(AiScriptRuntimeError::Runtime(format!(
                            "Cannot read prop ({}) of {}.",
                            i.value.repr_value(),
                            assignee.value.display_type()
                        )))?,
                    }
                }
                ast::Expression::Prop(prop) => {
                    let assignee = self.eval_expression(&prop.target, scope).await?;
                    let assignee = VObj::try_from(assignee)?;
                    assignee
                        .write()
                        .unwrap()
                        .insert(prop.name.to_string(), value);
                }
                ast::Expression::Arr(arr) => {
                    let value = <Vec<Value>>::try_from(value)?;
                    try_join_all(arr.value.iter().enumerate().map(|(index, item)| {
                        self.assign(scope, item, value.get(index).cloned().unwrap_or_default())
                    }))
                    .await?;
                }
                ast::Expression::Obj(obj) => {
                    let value = <IndexMap<String, Value>>::try_from(value)?;
                    try_join_all(obj.value.iter().map(|(key, item)| {
                        self.assign(scope, item, value.get(key).cloned().unwrap_or_default())
                    }))
                    .await?;
                }
                _ => Err(AiScriptRuntimeError::Runtime(
                    "The left-hand side of an assignment expression must be \
                    a variable or a property/index access."
                        .to_string(),
                ))?,
            }
            Ok(())
        }
        .boxed()
    }
}
