use std::sync::{Arc, RwLock};

use futures::future::BoxFuture;
use indexmap::IndexMap;

use crate::{error::AiScriptError, node::StatementOrExpression};

use super::{scope::Scope, Interpreter};

#[derive(Clone, Debug, Default)]
pub enum V {
    #[default]
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(VArr),
    Obj(VObj),
    Fn(VFn),
    Return(Box<Value>),
    Break,
    Continue,
    Error {
        value: String,
        info: Option<Box<Value>>,
    },
}

pub type VArr = Arc<RwLock<Vec<Value>>>;

pub type VObj = Arc<RwLock<IndexMap<String, Value>>>;

impl PartialEq for V {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Num(l0), Self::Num(r0)) => l0 == r0,
            (Self::Str(l0), Self::Str(r0)) => l0 == r0,
            (Self::Arr(l0), Self::Arr(r0)) => {
                l0.read().unwrap().clone() == r0.read().unwrap().clone()
            }
            (Self::Obj(l0), Self::Obj(r0)) => {
                l0.read().unwrap().clone() == r0.read().unwrap().clone()
            }
            (Self::Fn(_), Self::Fn(_)) => false,
            (Self::Return(l0), Self::Return(r0)) => l0 == r0,
            (
                Self::Error {
                    value: l_value,
                    info: l_info,
                },
                Self::Error {
                    value: r_value,
                    info: r_info,
                },
            ) => l_value == r_value && l_info == r_info,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(Clone)]
pub enum VFn {
    Fn {
        args: Vec<String>,
        statements: Vec<StatementOrExpression>,
        scope: Scope,
    },
    FnNative(VFnNative),
}

pub type VFnNative = Arc<
    dyn Fn(Vec<Value>, &Interpreter) -> BoxFuture<'static, Result<Value, AiScriptError>>
        + Sync
        + Send,
>;

impl std::fmt::Debug for VFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fn {
                args,
                statements,
                scope,
            } => f
                .debug_struct("Fn")
                .field("args", args)
                .field("statements", statements)
                .field("scope", scope)
                .finish(),
            Self::FnNative(_) => f.debug_tuple("FnNative").finish(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attr {
    pub name: String,
    pub value: Value,
}

#[derive(Clone, Debug, Default)]
pub struct Value {
    pub value: V,
    pub attr: Option<Vec<Attr>>,
}

impl Value {
    pub const fn new(value: V) -> Self {
        Value { value, attr: None }
    }

    pub const fn null() -> Self {
        Value::new(V::Null)
    }

    pub const fn bool(value: bool) -> Self {
        Value::new(V::Bool(value))
    }

    pub fn num(value: impl Into<f64>) -> Self {
        Value::new(V::Num(value.into()))
    }

    pub fn str(value: impl Into<String>) -> Self {
        Value::new(V::Str(value.into()))
    }

    pub fn arr(value: impl IntoIterator<Item = Value>) -> Self {
        Value::new(V::Arr(Arc::new(RwLock::new(value.into_iter().collect()))))
    }

    pub fn obj(value: impl IntoIterator<Item = (impl Into<String>, Value)>) -> Self {
        Value::new(V::Obj(Arc::new(RwLock::new(
            value
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        ))))
    }

    pub fn fn_(
        args: impl IntoIterator<Item = impl Into<String>>,
        statements: impl IntoIterator<Item = StatementOrExpression>,
        scope: Scope,
    ) -> Self {
        Value::new(V::Fn(VFn::Fn {
            args: args.into_iter().map(Into::into).collect(),
            statements: statements.into_iter().collect(),
            scope,
        }))
    }

    pub fn fn_native(
        value: impl Fn(Vec<Value>, &Interpreter) -> BoxFuture<'static, Result<Value, AiScriptError>>
            + Sync
            + Send
            + 'static,
    ) -> Self {
        Value::new(V::Fn(VFn::FnNative(Arc::new(value))))
    }

    pub fn return_(value: Value) -> Self {
        Value::new(V::Return(Box::new(value)))
    }

    pub fn break_() -> Self {
        Value::new(V::Break)
    }

    pub fn continue_() -> Self {
        Value::new(V::Continue)
    }

    pub fn error(value: impl Into<String>, info: Option<Value>) -> Self {
        Value::new(V::Error {
            value: value.into(),
            info: info.map(Box::new),
        })
    }
}

pub fn unwrap_ret(v: Value) -> Value {
    if let V::Return(value) = v.value {
        *value
    } else {
        v
    }
}
