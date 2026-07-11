use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use aiscript_v1::{Interpreter, Parser, errors::AiScriptError, values::Value};
use indexmap::IndexMap;

#[allow(dead_code)]
pub async fn test(program: &str, test: fn(Value)) -> Result<Value, AiScriptError> {
    let ast = Parser::default().parse(program)?;
    let test_count = Arc::new(AtomicUsize::new(0));
    let test_count_clone = test_count.clone();
    let aiscript = Interpreter::builder()
        .out_sync(move |value| {
            test(value);
            test_count_clone.fetch_add(1, Ordering::Relaxed);
        })
        .max_step(9999)
        .build();
    let result = aiscript.exec(ast).await.map(|value| value.unwrap())?;
    match test_count.load(Ordering::Relaxed) {
        0 => panic!("test has never been called"),
        1 => Ok(result),
        count => panic!("test has been called ${count} times"),
    }
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
