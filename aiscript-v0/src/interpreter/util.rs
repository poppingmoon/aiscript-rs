use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;
use regex::Regex;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::Visitor,
    ser::{self, SerializeMap, SerializeSeq},
};

use crate::error::{AiScriptError, AiScriptRuntimeError};

use super::value::{V, VArr, VFn, VObj, Value};

pub fn expect_any(val: Option<Value>) -> Result<Value, AiScriptError> {
    Ok(val.ok_or_else(|| {
        AiScriptRuntimeError::Runtime("Expect anything, but got nothing.".to_string())
    })?)
}

impl TryFrom<V> for bool {
    type Error = AiScriptError;

    fn try_from(value: V) -> Result<Self, Self::Error> {
        if let V::Bool(value) = value {
            Ok(value)
        } else {
            Err(AiScriptRuntimeError::Runtime(format!(
                "Expect boolean, but got {}",
                value.display_type(),
            )))?
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = AiScriptError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (*value.value).try_into()
    }
}

impl TryFrom<V> for VFn {
    type Error = AiScriptError;

    fn try_from(value: V) -> Result<Self, Self::Error> {
        if let V::Fn(value) = value {
            Ok(value)
        } else {
            Err(AiScriptRuntimeError::Runtime(format!(
                "Expect function, but got {}",
                value.display_type(),
            )))?
        }
    }
}

impl TryFrom<Value> for VFn {
    type Error = AiScriptError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (*value.value).try_into()
    }
}

impl TryFrom<V> for String {
    type Error = AiScriptError;

    fn try_from(value: V) -> Result<Self, Self::Error> {
        if let V::Str(value) = value {
            Ok(value)
        } else {
            Err(AiScriptRuntimeError::Runtime(format!(
                "Expect string, but got {}",
                value.display_type(),
            )))?
        }
    }
}

impl TryFrom<Value> for String {
    type Error = AiScriptError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (*value.value).try_into()
    }
}

impl TryFrom<V> for f64 {
    type Error = AiScriptError;

    fn try_from(value: V) -> Result<Self, Self::Error> {
        if let V::Num(value) = value {
            Ok(value)
        } else {
            Err(AiScriptRuntimeError::Runtime(format!(
                "Expect number, but got {}",
                value.display_type(),
            )))?
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = AiScriptError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (*value.value).try_into()
    }
}

impl TryFrom<V> for VObj {
    type Error = AiScriptError;

    fn try_from(value: V) -> Result<Self, Self::Error> {
        if let V::Obj(value) = value {
            Ok(value)
        } else {
            Err(AiScriptRuntimeError::Runtime(format!(
                "Expect object, but got {}",
                value.display_type(),
            )))?
        }
    }
}

impl TryFrom<Value> for VObj {
    type Error = AiScriptError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (*value.value).try_into()
    }
}

impl TryFrom<V> for IndexMap<String, Value> {
    type Error = AiScriptError;

    fn try_from(value: V) -> Result<Self, Self::Error> {
        Ok(VObj::try_from(value)?.read().unwrap().clone())
    }
}

impl TryFrom<Value> for IndexMap<String, Value> {
    type Error = AiScriptError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (*value.value).try_into()
    }
}

impl TryFrom<V> for VArr {
    type Error = AiScriptError;

    fn try_from(value: V) -> Result<Self, Self::Error> {
        if let V::Arr(value) = value {
            Ok(value)
        } else {
            Err(AiScriptRuntimeError::Runtime(format!(
                "Expect array, but got {}",
                value.display_type(),
            )))?
        }
    }
}

impl TryFrom<Value> for VArr {
    type Error = AiScriptError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (*value.value).try_into()
    }
}

impl TryFrom<V> for Vec<Value> {
    type Error = AiScriptError;

    fn try_from(value: V) -> Result<Self, Self::Error> {
        Ok(VArr::try_from(value)?.read().unwrap().clone())
    }
}

impl TryFrom<Value> for Vec<Value> {
    type Error = AiScriptError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        (*value.value).try_into()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl std::fmt::Display for V {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_type().fmt(f)?;
        match self {
            V::Num(value) => write!(f, "<{}>", value),
            V::Bool(value) => write!(f, "<{}>", value),
            V::Str(value) => write!(f, "<\"{}\">", value),
            V::Fn { .. } => write!(f, "<...>"),
            V::Obj(_) => write!(f, "<..>"),
            V::Null => write!(f, "<>"),
            _ => write!(f, "<null>"),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl V {
    pub fn display_type(&self) -> DisplayType<'_> {
        DisplayType(self)
    }

    pub fn display_simple(&self) -> DisplaySimple<'_> {
        DisplaySimple(self)
    }
}

impl Value {
    pub fn display_type(&self) -> DisplayType<'_> {
        self.value.display_type()
    }

    pub fn display_simple(&self) -> DisplaySimple<'_> {
        self.value.display_simple()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DisplayType<'a>(&'a V);

impl std::fmt::Display for DisplayType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                V::Null => "null",
                V::Bool(_) => "bool",
                V::Num(_) => "num",
                V::Str(_) => "str",
                V::Arr(_) => "arr",
                V::Obj(_) => "obj",
                V::Fn { .. } => "fn",
                V::Return(_) => "return",
                V::Break => "break",
                V::Continue => "continue",
                V::Error { .. } => "error",
            }
        )
    }
}

pub struct DisplaySimple<'a>(&'a V);

impl std::fmt::Display for DisplaySimple<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            V::Num(value) => write!(f, "{}", value),
            V::Bool(value) => write!(f, "{}", value),
            V::Str(value) => write!(f, "\"{}\"", value),
            V::Arr(value) => write!(
                f,
                "[{}]",
                value
                    .read()
                    .unwrap()
                    .iter()
                    .map(|value| value.display_simple().to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            V::Null => write!(f, "(null)"),
            v => v.fmt(f),
        }
    }
}

impl Serialize for V {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        VWithMemo::new(self.clone()).serialize(serializer)
    }
}

struct VWithMemo {
    pub value: V,
    pub processed_arrays: Rc<Vec<VArr>>,
    pub processed_objects: Rc<Vec<VObj>>,
}

impl VWithMemo {
    pub fn new(value: V) -> Self {
        VWithMemo {
            value,
            processed_arrays: Rc::new(Vec::new()),
            processed_objects: Rc::new(Vec::new()),
        }
    }
}

impl Serialize for VWithMemo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.value {
            V::Null => serializer.serialize_unit(),
            V::Bool(value) => serializer.serialize_bool(*value),
            V::Num(value) => {
                if value.trunc() == *value {
                    serializer.serialize_i64(*value as i64)
                } else {
                    serializer.serialize_f64(*value)
                }
            }
            V::Str(value) => serializer.serialize_str(value),
            V::Arr(value) => {
                if self.processed_arrays.iter().any(|v| Arc::ptr_eq(v, value)) {
                    Err(ser::Error::custom("cyclic_reference"))?
                } else {
                    let mut processed_arrays = (*self.processed_arrays).clone();
                    processed_arrays.push(value.clone());
                    let processed_arrays = Rc::new(processed_arrays);
                    let value = value.read().unwrap();
                    let mut seq = serializer.serialize_seq(Some(value.len()))?;
                    for e in value.iter() {
                        seq.serialize_element(&VWithMemo {
                            value: *e.value.clone(),
                            processed_arrays: processed_arrays.clone(),
                            processed_objects: self.processed_objects.clone(),
                        })?;
                    }
                    seq.end()
                }
            }
            V::Obj(value) => {
                if self.processed_objects.iter().any(|v| Arc::ptr_eq(v, value)) {
                    Err(ser::Error::custom("cyclic_reference"))
                } else {
                    let mut processed_objects = (*self.processed_objects).clone();
                    processed_objects.push(value.clone());
                    let processed_objects = Rc::new(processed_objects);
                    let value = value.read().unwrap();
                    let mut map = serializer.serialize_map(Some(value.len()))?;
                    for (k, v) in value.iter() {
                        map.serialize_entry(
                            k,
                            &VWithMemo {
                                value: *v.value.clone(),
                                processed_arrays: self.processed_arrays.clone(),
                                processed_objects: processed_objects.clone(),
                            },
                        )?;
                    }
                    map.end()
                }
            }
            V::Fn(_) => serializer.serialize_str("<function>"),
            value => Err(ser::Error::custom(format!(
                "Unrecognized value type: {}",
                value.display_type(),
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for V {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(VVisitor)
    }
}

struct VVisitor;

impl<'de> Visitor<'de> for VVisitor {
    type Value = V;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "V")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(V::Null)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(V::Bool(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(V::Str(v.to_string()))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(V::Num(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut arr = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(value) = seq.next_element()? {
            arr.push(Value::new(value));
        }
        Ok(V::Arr(Arc::new(RwLock::new(arr))))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut obj = IndexMap::new();
        while let Some((key, value)) = map.next_entry()? {
            obj.insert(key, Value::new(value));
        }
        Ok(V::Obj(Arc::new(RwLock::new(obj))))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f64(v as f64)
    }
}

pub fn get_lang_version(input: &str) -> Option<String> {
    let re = Regex::new(r"^\s*///\s*@\s*([a-zA-Z0-9_.-]+)(?:[\r\n][\s\S]*)?$").unwrap();
    re.captures(input).map(|captures| captures[1].to_string())
}

impl V {
    pub fn repr_value(&self) -> ReprValue<'_> {
        ReprValue {
            value: self,
            literal_like: false,
            processed_arrays: Rc::new(Vec::new()),
            processed_objects: Rc::new(Vec::new()),
        }
    }

    pub fn literal_like(&self) -> ReprValue<'_> {
        ReprValue {
            value: self,
            literal_like: true,
            processed_arrays: Rc::new(Vec::new()),
            processed_objects: Rc::new(Vec::new()),
        }
    }
}

impl Value {
    pub fn repr_value(&self) -> ReprValue<'_> {
        self.value.repr_value()
    }

    pub fn literal_like(&self) -> ReprValue<'_> {
        self.value.literal_like()
    }
}

pub struct ReprValue<'a> {
    value: &'a V,
    literal_like: bool,
    processed_arrays: Rc<Vec<&'a VArr>>,
    processed_objects: Rc<Vec<&'a VObj>>,
}

impl std::fmt::Display for ReprValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            V::Str(value) => {
                if self.literal_like {
                    write!(
                        f,
                        "\"{}\"",
                        value
                            .replace('\\', "\\\\")
                            .replace('\r', "\\r")
                            .replace('\n', "\\n")
                    )
                } else {
                    write!(f, "{}", value)
                }
            }
            V::Num(value) => write!(f, "{}", value),
            V::Arr(value) => {
                if self.processed_arrays.iter().any(|v| Arc::ptr_eq(v, value)) {
                    write!(f, "...")
                } else {
                    let mut processed_arrays = (*self.processed_arrays).clone();
                    processed_arrays.push(value);
                    let processed_arrays = Rc::new(processed_arrays);
                    write!(
                        f,
                        "[ {} ]",
                        value
                            .read()
                            .unwrap()
                            .iter()
                            .map(|value| ReprValue {
                                value: &value.value,
                                literal_like: true,
                                processed_arrays: processed_arrays.clone(),
                                processed_objects: self.processed_objects.clone(),
                            }
                            .to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
            }
            V::Obj(value) => {
                if self.processed_objects.iter().any(|v| Arc::ptr_eq(v, value)) {
                    write!(f, "...")
                } else {
                    let mut processed_objects = (*self.processed_objects).clone();
                    processed_objects.push(value);
                    let processed_objects = Rc::new(processed_objects);
                    write!(
                        f,
                        "{{ {} }}",
                        value
                            .read()
                            .unwrap()
                            .iter()
                            .map(|(key, val)| format!(
                                "{key}: {}",
                                ReprValue {
                                    value: &val.value,
                                    literal_like: true,
                                    processed_arrays: self.processed_arrays.clone(),
                                    processed_objects: processed_objects.clone(),
                                }
                            ))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
            }
            V::Bool(value) => write!(f, "{}", value),
            V::Null => write!(f, "null"),
            V::Fn(value) => write!(
                f,
                "@( {} ) {{ ... }}",
                if let VFn::Fn { args, .. } = value {
                    args.join(", ")
                } else {
                    String::new()
                }
            ),
            _ => write!(f, "?"),
        }
    }
}
