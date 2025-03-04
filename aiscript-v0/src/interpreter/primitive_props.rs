use futures::{
    future::{try_join_all, BoxFuture},
    try_join, FutureExt,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    error::{AiScriptError, AiScriptRuntimeError},
    Interpreter,
};

use super::{
    util::expect_any,
    value::{VFn, Value, V},
};

pub fn get_prim_prop(target: Value, name: &str) -> Result<Value, AiScriptError> {
    Ok(match *target.value {
        V::Num(target) => match name {
            "to_str" => Value::fn_native(move |_, _| {
                async move { Ok(Value::str(target.to_string())) }.boxed()
            }),
            _ => Err(AiScriptRuntimeError::Runtime(format!(
                "No such prop ({name}) in number."
            )))?,
        },
        V::Str(target) => match name {
            "to_num" => Value::fn_native(move |_, _| {
                let parsed = target.parse::<f64>();
                async move {
                    Ok(Value::new(parsed.map_or_else(
                        |_| V::Null,
                        |parsed| {
                            if parsed.is_nan() {
                                V::Null
                            } else {
                                V::Num(parsed)
                            }
                        },
                    )))
                }
                .boxed()
            }),
            "to_arr" => Value::fn_native(move |_, _| {
                let arr = target
                    .graphemes(true)
                    .map(Value::str)
                    .collect::<Vec<Value>>();
                async move { Ok(Value::arr(arr)) }.boxed()
            }),
            "to_unicode_arr" => Value::fn_native(move |_, _| {
                let arr = target.chars().map(Value::str).collect::<Vec<Value>>();
                async move { Ok(Value::arr(arr)) }.boxed()
            }),
            "to_unicode_codepoint_arr" => Value::fn_native(move |_, _| {
                let arr = target
                    .chars()
                    .map(|c| Value::num(c as u32))
                    .collect::<Vec<Value>>();
                async move { Ok(Value::arr(arr)) }.boxed()
            }),
            "to_char_arr" => Value::fn_native(move |_, _| {
                let arr = target
                    .encode_utf16()
                    .map(|u| Value::str(String::from_utf16_lossy(&[u])))
                    .collect::<Vec<Value>>();
                async move { Ok(Value::arr(arr)) }.boxed()
            }),
            "to_charcode_arr" => Value::fn_native(move |_, _| {
                let arr = target
                    .encode_utf16()
                    .map(Value::num)
                    .collect::<Vec<Value>>();
                async move { Ok(Value::arr(arr)) }.boxed()
            }),
            "to_utf8_byte_arr" => Value::fn_native(move |_, _| {
                let arr = target
                    .as_bytes()
                    .iter()
                    .map(|&u| Value::num(u))
                    .collect::<Vec<Value>>();
                async move { Ok(Value::arr(arr)) }.boxed()
            }),
            "len" => Value::num(target.graphemes(true).count() as f64),
            "replace" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let a = String::try_from(args.next().unwrap_or_default())?;
                    let b = String::try_from(args.next().unwrap_or_default())?;
                    Ok(Value::str(target.replace(&a, &b)))
                }
                .boxed()
            }),
            "index_of" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let search = String::try_from(args.next().unwrap_or_default())?;
                    let pos = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .map(|i| if i < 0.0 { target.len() as f64 + i } else { i });
                    Ok(Value::num(if let Some(pos) = pos {
                        let pos = pos as usize;
                        let pos = target.grapheme_indices(true).nth(pos).map(|(pos, _)| pos);
                        if let Some(pos) = pos {
                            target[pos..].find(&search).map_or(-1.0, |index| {
                                target
                                    .grapheme_indices(true)
                                    .enumerate()
                                    .find(|(_, (i, _))| index <= *i)
                                    .map_or(-1.0, |(i, _)| (i + pos) as f64)
                            })
                        } else {
                            -1.0
                        }
                    } else {
                        target.find(&search).map_or(-1.0, |index| {
                            target
                                .grapheme_indices(true)
                                .enumerate()
                                .find(|(_, (i, _))| index <= *i)
                                .map_or(-1.0, |(i, _)| i as f64)
                        })
                    }))
                }
                .boxed()
            }),
            "incl" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let search = String::try_from(args.next().unwrap_or_default())?;
                    Ok(Value::bool(target.contains(&search)))
                }
                .boxed()
            }),
            "trim" => Value::fn_native(move |_, _| {
                let s = target.trim().to_string();
                async move { Ok(Value::str(s)) }.boxed()
            }),
            "upper" => Value::fn_native(move |_, _| {
                let s = target.to_uppercase();
                async move { Ok(Value::str(s)) }.boxed()
            }),
            "lower" => Value::fn_native(move |_, _| {
                let s = target.to_lowercase();
                async move { Ok(Value::str(s)) }.boxed()
            }),
            "split" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let splitter = args
                        .next()
                        .map(String::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?;
                    Ok(Value::arr(match splitter {
                        Some(splitter) if !splitter.is_empty() => target
                            .split(&splitter)
                            .map(Value::str)
                            .collect::<Vec<Value>>(),
                        _ => target
                            .graphemes(true)
                            .map(Value::str)
                            .collect::<Vec<Value>>(),
                    }))
                }
                .boxed()
            }),
            "slice" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let begin = f64::try_from(args.next().unwrap_or_default())?;
                    let begin = target
                        .grapheme_indices(true)
                        .nth(begin as usize)
                        .map_or(begin as usize, |(i, _)| i)
                        .clamp(0, target.len());
                    let end = f64::try_from(args.next().unwrap_or_default())?;
                    let end = target
                        .grapheme_indices(true)
                        .nth(end as usize)
                        .map_or_else(|| target.len(), |(i, _)| i)
                        .clamp(begin, target.len());
                    Ok(Value::str(&target[begin..end]))
                }
                .boxed()
            }),
            "pick" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let i = f64::try_from(args.next().unwrap_or_default())?;
                    Ok(target
                        .graphemes(true)
                        .nth(i as usize)
                        .map_or_else(Value::null, Value::str))
                }
                .boxed()
            }),
            "charcode_at" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let i = f64::try_from(args.next().unwrap_or_default())?;
                    Ok(target
                        .encode_utf16()
                        .map(Value::num)
                        .nth(i as usize)
                        .unwrap_or_default())
                }
                .boxed()
            }),
            "codepoint_at" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let i = f64::try_from(args.next().unwrap_or_default())?;
                    let c = char::decode_utf16(target.encode_utf16().skip(i as usize))
                        .map(|r| {
                            r.map_or_else(|e| e.unpaired_surrogate() as f64, |c| c as u32 as f64)
                        })
                        .next();
                    Ok(c.map_or_else(Value::null, Value::num))
                }
                .boxed()
            }),
            "starts_with" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let suffix = String::try_from(args.next().unwrap_or_default())?;
                    if suffix.is_empty() {
                        return Ok(Value::bool(true));
                    }

                    let target_len = target.graphemes(true).count() as isize;
                    let raw_index = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(|i| Some(i as isize)))?
                        .unwrap_or(target_len);
                    if raw_index < -target_len || target_len < raw_index {
                        return Ok(Value::bool(false));
                    }
                    let index = if raw_index >= 0 {
                        raw_index
                    } else {
                        target_len + raw_index
                    } as usize;

                    Ok(Value::bool(
                        target[target
                            .grapheme_indices(true)
                            .nth(index)
                            .map_or(0, |(i, _)| i)..]
                            .starts_with(&suffix),
                    ))
                }
                .boxed()
            }),
            "ends_with" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let suffix = String::try_from(args.next().unwrap_or_default())?;
                    if suffix.is_empty() {
                        return Ok(Value::bool(true));
                    }

                    let target_len = target.graphemes(true).count() as isize;
                    let raw_index = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(|i| Some(i as isize)))?
                        .unwrap_or(target_len);
                    if raw_index < -target_len || target_len < raw_index {
                        return Ok(Value::bool(false));
                    }
                    let index = if raw_index >= 0 {
                        raw_index
                    } else {
                        target_len + raw_index
                    } as usize;

                    Ok(Value::bool(
                        target[..target
                            .grapheme_indices(true)
                            .nth(index)
                            .map_or_else(|| target.len(), |(index, _)| index)]
                            .ends_with(&suffix),
                    ))
                }
                .boxed()
            }),
            "pad_start" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let width = f64::try_from(args.next().unwrap_or_default())?;
                    let pad = args
                        .next()
                        .map(String::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .unwrap_or_else(|| " ".to_string());
                    let target_len = target.graphemes(true).count();
                    let pad_len = pad.graphemes(true).count();
                    Ok(Value::str(if width as usize <= target_len {
                        target
                    } else {
                        let width = width as usize - target_len;
                        let mut s = pad.repeat(width / pad_len);
                        s += &pad[..pad.grapheme_indices(true).nth(width % pad_len).unwrap().0];
                        s += &target;
                        s
                    }))
                }
                .boxed()
            }),
            "pad_end" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let width = f64::try_from(args.next().unwrap_or_default())?;
                    let pad = args
                        .next()
                        .map(String::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .unwrap_or_else(|| " ".to_string());
                    let target_len = target.graphemes(true).count();
                    let pad_len = pad.graphemes(true).count();
                    Ok(Value::str(if width as usize <= target_len {
                        target
                    } else {
                        let width = width as usize - target_len;
                        let mut s = target;
                        s += &pad.repeat(width / pad_len);
                        s += &pad[..pad.grapheme_indices(true).nth(width % pad_len).unwrap().0];
                        s
                    }))
                }
                .boxed()
            }),
            _ => Err(AiScriptRuntimeError::Runtime(format!(
                "No such prop ({name}) in string."
            )))?,
        },
        V::Arr(target) => match name {
            "len" => Value::num(target.read().unwrap().len() as f64),
            "push" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let val = expect_any(args.next())?;
                    target.write().unwrap().push(val);
                    Ok(Value::new(V::Arr(target)))
                }
                .boxed()
            }),
            "unshift" => Value::fn_native(move |args, _| {
                let target = target.clone();
                async move {
                    let mut args = args.into_iter();
                    let val = expect_any(args.next())?;
                    target.write().unwrap().insert(0, val);
                    Ok(Value::new(V::Arr(target)))
                }
                .boxed()
            }),
            "pop" => Value::fn_native(move |_, _| {
                let target = target.clone();
                async move {
                    let val = target.write().unwrap().pop();
                    Ok(if let Some(val) = val {
                        val
                    } else {
                        Value::null()
                    })
                }
                .boxed()
            }),
            "shift" => Value::fn_native(move |_, _| {
                let target = target.clone();
                async move {
                    Ok(if target.read().unwrap().is_empty() {
                        Value::null()
                    } else {
                        target.write().unwrap().remove(0)
                    })
                }
                .boxed()
            }),
            "concat" => Value::fn_native(move |args, _| {
                let mut target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let x = <Vec<Value>>::try_from(args.next().unwrap_or_default())?;
                    target.extend(x);
                    Ok(Value::arr(target))
                }
                .boxed()
            }),
            "slice" => Value::fn_native(move |args, _| {
                let target = target.read().unwrap().clone();
                let target_len = target.len();
                async move {
                    let mut args = args.into_iter();
                    let begin = f64::try_from(args.next().unwrap_or_default())?;
                    let begin = if begin < 0.0 {
                        (target_len as f64 + begin) as usize
                    } else {
                        begin as usize
                    }
                    .clamp(0, target_len);
                    let end = f64::try_from(args.next().unwrap_or_default())?;
                    let end = if end < 0.0 {
                        (target_len as f64 + end) as usize
                    } else {
                        end as usize
                    }
                    .clamp(begin, target_len);
                    Ok(Value::arr(target[begin..end].iter().cloned()))
                }
                .boxed()
            }),
            "join" => Value::fn_native(move |args, _| {
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let joiner = args
                        .next()
                        .map(String::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .unwrap_or_else(String::new);
                    Ok(Value::str(
                        target
                            .iter()
                            .map(|i| {
                                if let V::Str(value) = &*i.value {
                                    value
                                } else {
                                    ""
                                }
                            })
                            .collect::<Vec<&str>>()
                            .join(&joiner),
                    ))
                }
                .boxed()
            }),
            "map" => Value::fn_native(move |args, interpreter| {
                let interpreter = interpreter.clone();
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let fn_ = VFn::try_from(args.next().unwrap_or_default())?;
                    Ok(Value::arr(
                        try_join_all(target.into_iter().enumerate().map(|(i, item)| {
                            interpreter
                                .exec_fn_simple(fn_.clone(), vec![item, Value::num(i as f64)])
                        }))
                        .await?,
                    ))
                }
                .boxed()
            }),
            "filter" => Value::fn_native(move |args, interpreter| {
                let interpreter = interpreter.clone();
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let fn_ = VFn::try_from(args.next().unwrap_or_default())?;
                    let mut vals = Vec::new();
                    for (i, item) in target.into_iter().enumerate() {
                        let res = interpreter
                            .exec_fn_simple(fn_.clone(), vec![item.clone(), Value::num(i as f64)])
                            .await?;
                        let res = bool::try_from(res)?;
                        if res {
                            vals.push(item);
                        }
                    }
                    Ok(Value::arr(vals))
                }
                .boxed()
            }),
            "reduce" => Value::fn_native(move |args, interpreter| {
                let interpreter = interpreter.clone();
                let mut target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let fn_ = VFn::try_from(args.next().unwrap_or_default())?;
                    let initial_value = args.next();
                    let with_initial_value = initial_value.is_some();
                    if !with_initial_value && target.is_empty() {
                        Err(AiScriptRuntimeError::Runtime(
                            "Reduce of empty array without initial value".to_string(),
                        ))?;
                    }
                    let mut accumlator = initial_value.unwrap_or_else(|| target.remove(0));
                    for (i, item) in target.into_iter().enumerate() {
                        accumlator = interpreter
                            .exec_fn_simple(
                                fn_.clone(),
                                vec![
                                    accumlator,
                                    item,
                                    Value::num(if with_initial_value { i } else { i + 1 } as f64),
                                ],
                            )
                            .await?;
                    }
                    Ok(accumlator)
                }
                .boxed()
            }),
            "find" => Value::fn_native(move |args, interpreter| {
                let interpreter = interpreter.clone();
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let fn_ = VFn::try_from(args.next().unwrap_or_default())?;
                    for (i, item) in target.into_iter().enumerate() {
                        let res = interpreter
                            .exec_fn_simple(fn_.clone(), vec![item.clone(), Value::num(i as f64)])
                            .await?;
                        let res = bool::try_from(res)?;
                        if res {
                            return Ok(item);
                        }
                    }
                    Ok(Value::null())
                }
                .boxed()
            }),
            "incl" => Value::fn_native(move |args, _| {
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let val = expect_any(args.next())?;
                    Ok(Value::bool(target.into_iter().any(|item| val == item)))
                }
                .boxed()
            }),
            "index_of" => Value::fn_native(move |args, _| {
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let val = expect_any(args.next())?;
                    let from_i = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .map_or(0.0, |i| if i < 0.0 { target.len() as f64 + i } else { i })
                        .clamp(0.0, target.len() as f64) as usize;
                    Ok(Value::num(
                        target[from_i..]
                            .iter()
                            .position(|item| item == &val)
                            .map_or(-1.0, |result| (result + from_i) as f64),
                    ))
                }
                .boxed()
            }),
            "reverse" => Value::fn_native(move |_, _| {
                target.write().unwrap().reverse();
                async move { Ok(Value::null()) }.boxed()
            }),
            "copy" => Value::fn_native(move |_, _| {
                let target = target.read().unwrap().clone();
                async move { Ok(Value::arr(target)) }.boxed()
            }),
            "sort" => Value::fn_native({
                fn merge_sort(
                    arr: Vec<Value>,
                    comp: VFn,
                    interpreter: &Interpreter,
                ) -> BoxFuture<'static, Result<Vec<Value>, AiScriptError>> {
                    let len = arr.len();
                    if len <= 1 {
                        return async move { Ok(arr) }.boxed();
                    }
                    let mid = len / 2;
                    let mut left = arr;
                    let right = left.split_off(mid);
                    let interpreter = interpreter.clone();
                    async move {
                        let (left, right) = try_join!(
                            merge_sort(left, comp.clone(), &interpreter),
                            merge_sort(right, comp.clone(), &interpreter)
                        )?;
                        merge(left, right, comp, &interpreter).await
                    }
                    .boxed()
                }
                async fn merge(
                    left: Vec<Value>,
                    right: Vec<Value>,
                    comp: VFn,
                    interpreter: &Interpreter,
                ) -> Result<Vec<Value>, AiScriptError> {
                    let mut result = Vec::new();
                    let mut left_index = 0;
                    let mut right_index = 0;
                    while left_index < left.len() && right_index < right.len() {
                        let l = (left[left_index]).clone();
                        let r = (right[right_index]).clone();
                        let comp_value = interpreter
                            .exec_fn_simple(comp.clone(), vec![l.clone(), r.clone()])
                            .await?;
                        let comp_value = f64::try_from(comp_value)?;
                        if comp_value < 0.0 {
                            result.push(l);
                            left_index += 1;
                        } else {
                            result.push(r);
                            right_index += 1;
                        }
                    }
                    result.extend_from_slice(&left[left_index..]);
                    result.extend_from_slice(&right[right_index..]);
                    Ok(result)
                }

                move |args, interpreter| {
                    let interpreter = interpreter.clone();
                    let target = target.clone();
                    async move {
                        let mut args = args.into_iter();
                        let comp = VFn::try_from(args.next().unwrap_or_default())?;
                        let arr = target.read().unwrap().clone();
                        let sorted = merge_sort(arr, comp, &interpreter).await?;
                        target.write().unwrap().splice(.., sorted);
                        Ok(Value::new(V::Arr(target)))
                    }
                    .boxed()
                }
            }),
            "fill" => Value::fn_native(move |args, _| {
                let target = target.clone();
                let target_len = target.read().unwrap().len();
                async move {
                    let mut args = args.into_iter();
                    let val = args.next().unwrap_or_default();
                    let start = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .map_or(0, |i| {
                            if i < 0.0 {
                                (target_len as f64 + i) as usize
                            } else {
                                i as usize
                            }
                        })
                        .clamp(0, target_len);
                    let end = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .map_or(target_len, |i| {
                            if i < 0.0 {
                                (target_len as f64 + i) as usize
                            } else {
                                i as usize
                            }
                        })
                        .clamp(start, target_len);
                    for i in start..end {
                        target.write().unwrap()[i] = val.clone();
                    }
                    Ok(Value::new(V::Arr(target)))
                }
                .boxed()
            }),
            "repeat" => Value::fn_native(move |args, _| {
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let times = f64::try_from(args.next().unwrap_or_default())?;
                    if times < 0.0 {
                        Err(AiScriptRuntimeError::Runtime(
                            "arr.repeat expected non-negative number, got negative".to_string(),
                        ))?
                    } else if times.trunc() != times {
                        Err(AiScriptRuntimeError::Runtime(
                            "arr.repeat expected integer, got non-integer".to_string(),
                        ))?
                    } else {
                        let mut value = Vec::new();
                        let target = &target[..];
                        for _ in 0..times as usize {
                            value.extend_from_slice(target)
                        }
                        Ok(Value::arr(value))
                    }
                }
                .boxed()
            }),
            "splice" => Value::fn_native(move |args, _| {
                let target = target.clone();
                let target_len = target.read().unwrap().len();
                async move {
                    let mut args = args.into_iter();
                    let idx = f64::try_from(args.next().unwrap_or_default())?;
                    let index = if idx < 0.0 {
                        target_len as f64 + idx
                    } else {
                        idx
                    }
                    .clamp(0.0, target_len as f64) as usize;
                    let remove_count = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .map_or_else(
                            || target_len - index,
                            |rc| rc.clamp(0.0, (target_len - index) as f64) as usize,
                        );
                    let items = args
                        .next()
                        .map(<Vec<Value>>::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .unwrap_or_default();
                    let result = target
                        .write()
                        .unwrap()
                        .splice(index..index + remove_count, items)
                        .collect::<Vec<Value>>();
                    Ok(Value::arr(result))
                }
                .boxed()
            }),
            "flat" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let target = target.read().unwrap().clone();
                async move {
                    let depth = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .unwrap_or(1.0);
                    if depth.trunc() != depth {
                        Err(AiScriptRuntimeError::Runtime(
                            "arr.flat expected integer, got non-integer".to_string(),
                        ))?
                    } else if depth < 0.0 {
                        Err(AiScriptRuntimeError::Runtime(
                            "arr.repeat expected non-negative number, got negative".to_string(),
                        ))?
                    } else {
                        fn flat(arr: Vec<Value>, depth: usize, result: &mut Vec<Value>) {
                            if depth == 0 {
                                result.extend(arr);
                                return;
                            }
                            for v in arr {
                                if let V::Arr(value) = *v.value {
                                    flat(value.read().unwrap().clone(), depth - 1, result);
                                } else {
                                    result.push(v);
                                }
                            }
                        }
                        let mut result = Vec::new();
                        flat(target, depth as usize, &mut result);
                        Ok(Value::arr(result))
                    }
                }
                .boxed()
            }),
            "flat_map" => Value::fn_native(move |args, interpreter| {
                let interpreter = interpreter.clone();
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let fn_ = VFn::try_from(args.next().unwrap_or_default())?;
                    let mapped_vals =
                        try_join_all(target.into_iter().enumerate().map(|(i, item)| {
                            interpreter
                                .exec_fn_simple(fn_.clone(), vec![item, Value::num(i as f64)])
                        }))
                        .await?;
                    let mut result = Vec::new();
                    for value in mapped_vals {
                        if let V::Arr(value) = *value.value {
                            result.extend(value.read().unwrap().clone())
                        } else {
                            result.push(value)
                        }
                    }
                    Ok(Value::arr(result))
                }
                .boxed()
            }),
            "every" => Value::fn_native(move |args, interpreter| {
                let interpreter = interpreter.clone();
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let fn_ = VFn::try_from(args.next().unwrap_or_default())?;
                    for (i, item) in target.into_iter().enumerate() {
                        let res = interpreter
                            .exec_fn_simple(fn_.clone(), vec![item, Value::num(i as f64)])
                            .await?;
                        let res = bool::try_from(res)?;
                        if !res {
                            return Ok(Value::bool(false));
                        }
                    }
                    Ok(Value::bool(true))
                }
                .boxed()
            }),
            "some" => Value::fn_native(move |args, interpreter| {
                let interpreter = interpreter.clone();
                let target = target.read().unwrap().clone();
                async move {
                    let mut args = args.into_iter();
                    let fn_ = VFn::try_from(args.next().unwrap_or_default())?;
                    for (i, item) in target.into_iter().enumerate() {
                        let res = interpreter
                            .exec_fn_simple(fn_.clone(), vec![item, Value::num(i as f64)])
                            .await?;
                        let res = bool::try_from(res)?;
                        if res {
                            return Ok(Value::bool(true));
                        }
                    }
                    Ok(Value::bool(false))
                }
                .boxed()
            }),
            "insert" => Value::fn_native(move |args, _| {
                let target = target.clone();
                let target_len = target.read().unwrap().len();
                async move {
                    let mut args = args.into_iter();
                    let idx = f64::try_from(args.next().unwrap_or_default())?;
                    let index = if idx < 0.0 {
                        target_len as f64 + idx
                    } else {
                        idx
                    }
                    .clamp(0.0, target_len as f64) as usize;
                    let item = expect_any(args.next())?;
                    target.write().unwrap().insert(index, item);
                    Ok(Value::null())
                }
                .boxed()
            }),
            "remove" => Value::fn_native(move |args, _| {
                let target = target.clone();
                let target_len = target.read().unwrap().len();
                async move {
                    let mut args = args.into_iter();
                    let idx = f64::try_from(args.next().unwrap_or_default())?;
                    Ok(if target_len == 0 {
                        Value::null()
                    } else {
                        let index = if idx < 0.0 {
                            target_len as f64 + idx
                        } else {
                            idx
                        }
                        .clamp(0.0, target_len as f64) as usize;
                        if index == target_len {
                            Value::null()
                        } else {
                            let removed = target.write().unwrap().remove(index);
                            removed
                        }
                    })
                }
                .boxed()
            }),
            "at" => Value::fn_native(move |args, _| {
                let target = target.clone();
                let target_len = target.read().unwrap().len();
                async move {
                    let mut args = args.into_iter();
                    let idx = f64::try_from(args.next().unwrap_or_default())? as isize;
                    let index = if idx < 0 {
                        target_len as isize + idx
                    } else {
                        idx
                    };
                    Ok(if index < 0 {
                        None
                    } else {
                        target.read().unwrap().get(index as usize).cloned()
                    }
                    .unwrap_or_else(|| args.next().unwrap_or_default()))
                }
                .boxed()
            }),
            _ => Err(AiScriptRuntimeError::Runtime(format!(
                "No such prop ({name}) in string."
            )))?,
        },
        V::Error { value, info } => match name {
            "name" => Value::str(value),
            "info" => info.map_or_else(Value::null, |info| *info),
            _ => Err(AiScriptRuntimeError::Runtime(format!(
                "No such prop ({name}) in number."
            )))?,
        },
        value => Err(AiScriptRuntimeError::Runtime(format!(
            "Cannot read prop of {}. (reading {name})",
            value.display_type()
        )))?,
    })
}
