use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::error::{AiScriptError, AiScriptRuntimeError};

use super::{value::Value, variable::Variable};

#[derive(Debug, Clone)]
pub struct Scope {
    parent: Option<Box<Scope>>,
    states: Arc<RwLock<HashMap<String, Variable>>>,
    name: String,
    ns_name: Option<String>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            parent: Default::default(),
            states: Default::default(),
            name: "<root>".to_string(),
            ns_name: Default::default(),
        }
    }
}

impl Scope {
    pub fn new(states: HashMap<String, Variable>, name: Option<String>) -> Self {
        Scope {
            parent: None,
            states: Arc::new(RwLock::new(states)),
            name: name.unwrap_or_else(|| "<root>".to_string()),
            ns_name: None,
        }
    }

    pub fn create_child_scope(
        &self,
        states: HashMap<String, Variable>,
        name: Option<String>,
    ) -> Self {
        Scope {
            parent: Some(self.clone().into()),
            states: Arc::new(RwLock::new(states)),
            name: name.unwrap_or_else(|| "<anonymous>".to_string()),
            ns_name: None,
        }
    }

    pub fn create_child_namespace_scope(
        &self,
        ns_name: String,
        states: HashMap<String, Variable>,
        name: Option<String>,
    ) -> Self {
        Scope {
            parent: Some(self.clone().into()),
            states: Arc::new(RwLock::new(states)),
            name: name.unwrap_or_else(|| "<anonymous>".to_string()),
            ns_name: Some(ns_name),
        }
    }

    pub fn get(&self, name: &str) -> Result<Value, AiScriptError> {
        self.get_(name, &self.name)
    }

    fn get_(&self, name: &str, scope_name: &str) -> Result<Value, AiScriptError> {
        if let Some(Variable::Mut(state) | Variable::Const(state)) =
            self.states.read().unwrap().get(name)
        {
            Ok(state.clone())
        } else if let Some(parent) = &self.parent {
            parent.get_(name, scope_name)
        } else {
            Err(AiScriptRuntimeError::Runtime(format!(
                "No such variable '{name}' in scope '{scope_name}'",
            )))?
        }
    }

    pub fn exists(&self, name: &str) -> bool {
        if self.states.read().unwrap().contains_key(name) {
            true
        } else if let Some(parent) = &self.parent {
            parent.exists(name)
        } else {
            false
        }
    }

    pub fn get_all(&self) -> HashMap<String, Variable> {
        if let Some(parent) = &self.parent {
            let mut states = parent.get_all();
            states.extend(self.states.clone().read().unwrap().clone());
            states
        } else {
            self.states.clone().read().unwrap().clone()
        }
    }

    pub fn add(&self, name: String, variable: Variable) -> Result<(), AiScriptError> {
        if self.states.read().unwrap().contains_key(&name) {
            Err(AiScriptRuntimeError::Runtime(format!(
                "Variable '{name}' already exists in scope '{}'",
                self.name
            )))?
        } else {
            self.states
                .write()
                .unwrap()
                .insert(name.clone(), variable.clone());
            if let Some(parent) = &self.parent {
                if let Some(ns_name) = &self.ns_name {
                    parent.add(format!("{ns_name}:{name}"), variable)?;
                }
            }
            Ok(())
        }
    }

    pub fn assign(&self, name: String, val: Value) -> Result<(), AiScriptError> {
        self.assign_(name, val, &self.name)
    }

    fn assign_(&self, name: String, val: Value, scope_name: &str) -> Result<(), AiScriptError> {
        let is_mut = self
            .states
            .read()
            .unwrap()
            .get(&name)
            .map(|variable| matches!(variable, Variable::Mut(_)));
        match is_mut {
            Some(true) => {
                self.states
                    .write()
                    .unwrap()
                    .insert(name, Variable::Mut(val));
                Ok(())
            }
            Some(false) => Err(AiScriptRuntimeError::Runtime(format!(
                "Cannot assign to an immutable variable {name}."
            )))?,
            None => {
                if let Some(parent) = &self.parent {
                    parent.assign_(name, val, scope_name)
                } else {
                    Err(AiScriptRuntimeError::Runtime(format!(
                        "No such variable '{name}' in scope '{scope_name}"
                    )))?
                }
            }
        }
    }
}
