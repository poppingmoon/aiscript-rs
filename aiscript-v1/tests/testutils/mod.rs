use aiscript_v1::{Interpreter, Parser, errors::AiScriptError, values::Value};
use futures::FutureExt;
use indexmap::IndexMap;

#[allow(dead_code)]
pub async fn test(program: &str, test: fn(Value)) -> Result<Value, AiScriptError> {
    let ast = Parser::default().parse(program)?;
    let aiscript = Interpreter::new(
        [],
        None::<fn(_) -> _>,
        Some(move |value| {
            test(value);
            async move {}.boxed()
        }),
        None::<fn(_) -> _>,
        Some(9999),
    );
    aiscript.exec(ast).await.map(|value| value.unwrap())
}

#[allow(dead_code)]
pub fn get_meta(program: &str) -> Result<IndexMap<Option<String>, Option<Value>>, AiScriptError> {
    let ast = Parser::default().parse(program)?;
    let metadata = Interpreter::collect_metadata(ast);
    Ok(metadata)
}

#[allow(dead_code)]
pub fn null() -> Value {
    Value::null()
}

#[allow(dead_code)]
pub fn bool(value: bool) -> Value {
    Value::bool(value)
}

#[allow(dead_code)]
pub fn num(value: impl Into<f64>) -> Value {
    Value::num(value.into())
}

#[allow(dead_code)]
pub fn str(value: impl Into<String>) -> Value {
    Value::str(value.into())
}

#[allow(dead_code)]
pub fn arr(value: impl IntoIterator<Item = Value>) -> Value {
    Value::arr(value)
}

#[allow(dead_code)]
pub fn obj(value: impl IntoIterator<Item = (impl Into<String>, Value)>) -> Value {
    Value::obj(value)
}

#[allow(dead_code)]
pub fn error(value: impl Into<String>, info: Option<Value>) -> Value {
    Value::error(value, info)
}
