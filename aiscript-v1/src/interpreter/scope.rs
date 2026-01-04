use std::{collections::HashMap, sync::Arc};

use futures::{FutureExt, future::BoxFuture};
use tokio::sync::RwLock;

use crate::error::{AiScriptError, AiScriptRuntimeError};

use super::{value::Value, variable::Variable};

#[derive(Clone, Debug, Default)]
pub struct Scope {
    parent: Option<Box<Scope>>,
    states: Arc<RwLock<HashMap<String, Variable>>>,
    ns_name: Option<String>,
}

impl Scope {
    pub fn new(states: HashMap<String, Variable>) -> Self {
        Scope {
            parent: None,
            states: Arc::new(RwLock::new(states)),
            ns_name: None,
        }
    }

    fn name(&self) -> &'static str {
        if self.parent.is_none() {
            "<root>"
        } else {
            "<anonymous>"
        }
    }

    pub fn create_child_scope(&self, states: HashMap<String, Variable>) -> Self {
        Scope {
            parent: Some(self.clone().into()),
            states: Arc::new(RwLock::new(states)),
            ns_name: None,
        }
    }

    pub fn create_child_namespace_scope(
        &self,
        ns_name: String,
        states: HashMap<String, Variable>,
    ) -> Self {
        Scope {
            parent: Some(self.clone().into()),
            states: Arc::new(RwLock::new(states)),
            ns_name: Some(ns_name),
        }
    }

    pub async fn get(&self, name: &str) -> Result<Value, AiScriptError> {
        self.get_(name, self.name()).await
    }

    fn get_<'a>(
        &'a self,
        name: &'a str,
        scope_name: &'a str,
    ) -> BoxFuture<'a, Result<Value, AiScriptError>> {
        async move {
            if let Some(Variable::Mut(state) | Variable::Const(state)) =
                self.states.read().await.get(name)
            {
                Ok(state.clone())
            } else if let Some(parent) = &self.parent {
                parent.get_(name, scope_name).await
            } else {
                Err(AiScriptRuntimeError::NoSuchVariable {
                    name: name.to_string(),
                    scope_name: scope_name.to_string(),
                })?
            }
        }
        .boxed()
    }

    pub fn exists<'a>(&'a self, name: &'a str) -> BoxFuture<'a, bool> {
        async move {
            if self.states.read().await.contains_key(name) {
                true
            } else if let Some(parent) = &self.parent {
                parent.exists(name).await
            } else {
                false
            }
        }
        .boxed()
    }

    pub fn get_all(&self) -> BoxFuture<'_, HashMap<String, Variable>> {
        async move {
            if let Some(parent) = &self.parent {
                let mut states = parent.get_all().await;
                states.extend(self.states.clone().read().await.clone());
                states
            } else {
                self.states.clone().read().await.clone()
            }
        }
        .boxed()
    }

    pub fn add(
        &self,
        name: String,
        variable: Variable,
    ) -> BoxFuture<'_, Result<(), AiScriptError>> {
        async move {
            if self.states.read().await.contains_key(&name) {
                Err(AiScriptRuntimeError::VariableAlreadyExists {
                    name,
                    scope_name: self.name().to_string(),
                })?
            } else {
                if let Some(parent) = &self.parent
                    && let Some(ns_name) = &self.ns_name
                {
                    parent
                        .add(format!("{ns_name}:{name}"), variable.clone())
                        .await?;
                }
                self.states.write().await.insert(name, variable);
                Ok(())
            }
        }
        .boxed()
    }

    pub async fn assign(&self, name: String, val: Value) -> Result<(), AiScriptError> {
        self.assign_(name, val, self.name()).await
    }

    fn assign_<'a>(
        &'a self,
        name: String,
        val: Value,
        scope_name: &'a str,
    ) -> BoxFuture<'a, Result<(), AiScriptError>> {
        async move {
            let is_mut = self
                .states
                .read()
                .await
                .get(&name)
                .map(|variable| matches!(variable, Variable::Mut(_)));
            match is_mut {
                Some(true) => {
                    self.states.write().await.insert(name, Variable::Mut(val));
                    Ok(())
                }
                Some(false) => Err(AiScriptRuntimeError::AssignmentToImmutable(name))?,
                None => {
                    if let Some(parent) = &self.parent {
                        parent.assign_(name, val, scope_name).await
                    } else {
                        Err(AiScriptRuntimeError::NoSuchVariable {
                            name,
                            scope_name: scope_name.to_string(),
                        })?
                    }
                }
            }
        }
        .boxed()
    }

    pub fn get_parent(self) -> Result<Scope, AiScriptError> {
        self.parent.map_or_else(
            || Err(AiScriptError::internal("scope has no parent")),
            |parent| Ok(*parent),
        )
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent
            && Arc::ptr_eq(&self.states, &other.states)
            && self.ns_name == other.ns_name
    }
}
