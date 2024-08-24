use super::value::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum Variable {
    Mut(Value),
    Const(Value),
}
