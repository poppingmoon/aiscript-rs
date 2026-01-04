use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize},
};

use futures::{FutureExt, future::BoxFuture};
use tokio::sync::Mutex;

use crate::error::AiScriptError;

use super::{
    Interpreter, lib::std::std, scope::Scope, util::expect_any, value::Value, variable::Variable,
};

#[derive(Default)]
pub struct InterpreterBuilder {
    consts: Vec<(String, Value)>,
    print: Option<Value>,
    readline: Option<Value>,
    err: Option<Arc<dyn (Fn(AiScriptError) -> BoxFuture<'static, ()>) + Sync + Send + 'static>>,
    max_step: Option<usize>,
}

impl InterpreterBuilder {
    pub fn consts(mut self, consts: impl IntoIterator<Item = (String, Value)>) -> Self {
        self.consts = consts.into_iter().collect();
        self
    }

    pub fn in_<F>(mut self, in_: impl Fn(String) -> F + Sync + Send + 'static) -> Self
    where
        F: Future<Output = String> + Send + 'static,
    {
        self.readline = Some(Value::fn_native(move |args, _| {
            let in_ = String::try_from(args.into_iter().next().unwrap_or_default()).map(&in_);
            async {
                let a = in_?.await;
                Ok(Value::str(a))
            }
            .boxed()
        }));
        self
    }

    pub fn in_sync(mut self, in_: impl Fn(String) -> String + Sync + Send + 'static) -> Self {
        self.readline = Some(Value::fn_native_sync(move |args| {
            String::try_from(args.into_iter().next().unwrap_or_default())
                .map(|q| Value::str(in_(q)))
        }));
        self
    }

    pub fn out<F>(mut self, out: impl Fn(Value) -> F + Sync + Send + 'static) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.print = Some(Value::fn_native(move |args, _| {
            let out = expect_any(args.into_iter().next()).map(&out);
            async {
                out?.await;
                Ok(Value::null())
            }
            .boxed()
        }));
        self
    }

    pub fn out_sync(mut self, out: impl Fn(Value) + Sync + Send + 'static) -> Self {
        self.print = Some(Value::fn_native_sync(move |args| {
            expect_any(args.into_iter().next()).map(|v| {
                out(v);
                Value::null()
            })
        }));
        self
    }

    pub fn err<F>(
        mut self,
        err: impl Fn(AiScriptError) -> F + Sync + Send + Clone + 'static,
    ) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.err = Some(Arc::new(move |e| {
            let err = err.clone();
            async move { err(e).await }.boxed()
        }));
        self
    }

    pub fn err_sync(mut self, err: impl Fn(AiScriptError) + Sync + Send + 'static) -> Self {
        self.err = Some(Arc::new(move |e| {
            err(e);
            async {}.boxed()
        }));
        self
    }

    pub fn max_step(mut self, max_step: usize) -> Self {
        self.max_step = Some(max_step);
        self
    }

    pub fn build(self) -> Interpreter {
        let mut states = Vec::from_iter(self.consts);
        states.extend(std());
        states.push((
            "print".to_string(),
            self.print.unwrap_or_else(|| {
                Value::fn_native_sync(|args| {
                    expect_any(args.into_iter().next()).map(|_| Value::null())
                })
            }),
        ));
        states.push((
            "readline".to_string(),
            self.readline.unwrap_or_else(|| {
                Value::fn_native_sync(|args| {
                    String::try_from(args.into_iter().next().unwrap_or_default())
                        .map(|_| Value::null())
                })
            }),
        ));
        let states = states
            .into_iter()
            .map(|(k, v)| (k, Variable::Const(v)))
            .collect();
        Interpreter {
            step_count: Arc::new(AtomicUsize::new(0)),
            stop: Arc::new(AtomicBool::new(false)),
            scope: Scope::new(states),
            abort_handlers: Arc::new(Mutex::new(tokio::task::JoinSet::new())),
            err: self.err,
            max_step: self.max_step,
        }
    }
}
