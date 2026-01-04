use std::sync::{Arc, RwLock};

use futures::future::BoxFuture;
use indexmap::IndexMap;

use crate::{
    error::AiScriptError,
    node::{Expression, StatementOrExpression},
};

use super::{Interpreter, scope::Scope};

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
    Break {
        label: Option<String>,
        value: Option<Box<Value>>,
    },
    Continue {
        label: Option<String>,
    },
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
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::Num(l), Self::Num(r)) => l == r,
            (Self::Str(l), Self::Str(r)) => l == r,
            (Self::Arr(l), Self::Arr(r)) => {
                if let Ok(l) = l.read() {
                    let l = l.clone();
                    if let Ok(r) = r.read() { l == *r } else { false }
                } else {
                    false
                }
            }
            (Self::Obj(l), Self::Obj(r)) => {
                if let (Ok(l), Ok(r)) = (l.read(), r.read()) {
                    *l == *r
                } else {
                    false
                }
            }
            (Self::Fn(l), Self::Fn(r)) => match (l, r) {
                (
                    VFn::Fn {
                        params: l_params,
                        statements: l_statements,
                        scope: l_scope,
                    },
                    VFn::Fn {
                        params: r_params,
                        statements: r_statements,
                        scope: r_scope,
                    },
                ) => l_params == r_params && l_statements == r_statements && l_scope == r_scope,
                (VFn::FnNative(l), VFn::FnNative(r)) => Arc::ptr_eq(l, r),
                (VFn::FnNativeSync(l), VFn::FnNativeSync(r)) => Arc::ptr_eq(l, r),
                _ => false,
            },
            (Self::Return(l), Self::Return(r)) => l == r,
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
        params: Vec<VFnParam>,
        statements: Vec<StatementOrExpression>,
        scope: Box<Scope>,
    },
    FnNative(VFnNative),
    FnNativeSync(VFnNativeSync),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VFnParam {
    pub dest: Expression,
    pub default: Option<Value>,
}

pub type VFnNative = Arc<
    dyn Fn(Vec<Value>, &Interpreter) -> BoxFuture<'static, Result<Value, AiScriptError>>
        + Sync
        + Send,
>;

pub type VFnNativeSync = Arc<dyn Fn(Vec<Value>) -> Result<Value, AiScriptError> + Sync + Send>;

impl std::fmt::Debug for VFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fn {
                params,
                statements,
                scope,
            } => f
                .debug_struct("Fn")
                .field("params", params)
                .field("statements", statements)
                .field("scope", scope)
                .finish(),
            Self::FnNative(_) => f.debug_tuple("FnNative").finish(),
            Self::FnNativeSync(_) => f.debug_tuple("FnNativeSync").finish(),
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
    pub value: Box<V>,
    pub attr: Option<Vec<Attr>>,
}

impl Value {
    pub fn new(value: V) -> Self {
        Value {
            value: value.into(),
            attr: None,
        }
    }

    pub fn null() -> Self {
        Value::new(V::Null)
    }

    pub fn bool(value: bool) -> Self {
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
        params: impl IntoIterator<Item = VFnParam>,
        statements: impl IntoIterator<Item = StatementOrExpression>,
        scope: Scope,
    ) -> Self {
        Value::new(V::Fn(VFn::Fn {
            params: params.into_iter().collect(),
            statements: statements.into_iter().collect(),
            scope: scope.into(),
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

    pub fn fn_native_sync(
        value: impl Fn(Vec<Value>) -> Result<Value, AiScriptError> + Sync + Send + 'static,
    ) -> Self {
        Value::new(V::Fn(VFn::FnNativeSync(Arc::new(value))))
    }

    pub fn return_(value: Value) -> Self {
        Value::new(V::Return(Box::new(value)))
    }

    pub fn break_(label: Option<impl Into<String>>, value: Option<Value>) -> Self {
        Value::new(V::Break {
            label: label.map(Into::into),
            value: value.map(Box::new),
        })
    }

    pub fn continue_(label: Option<impl Into<String>>) -> Self {
        Value::new(V::Continue {
            label: label.map(Into::into),
        })
    }

    pub fn error(value: impl Into<String>, info: Option<Value>) -> Self {
        Value::new(V::Error {
            value: value.into(),
            info: info.map(Box::new),
        })
    }

    pub fn is_control(&self) -> bool {
        match *self.value {
            V::Null
            | V::Bool(_)
            | V::Num(_)
            | V::Str(_)
            | V::Arr(_)
            | V::Obj(_)
            | V::Fn(_)
            | V::Error { .. } => false,
            V::Return(_) | V::Break { .. } | V::Continue { .. } => true,
        }
    }
}

pub fn unwrap_ret(v: Value) -> Value {
    if let V::Return(value) = *v.value {
        *value
    } else {
        v
    }
}

pub fn unwrap_labeled_break(v: Value, label: Option<String>) -> Value {
    match *v.value {
        V::Break {
            label: Some(l),
            value,
        } if label.is_some_and(|label| *l == label) => {
            if let Some(value) = value {
                *value
            } else {
                Value::null()
            }
        }
        _ => v,
    }
}
