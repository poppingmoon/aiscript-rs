use futures::{
    FutureExt,
    future::{BoxFuture, try_join_all},
};
use tokio::try_join;
use unicode_segmentation::UnicodeSegmentation;

use crate::error::{AiScriptError, AiScriptRuntimeError};

use super::{
    Interpreter,
    util::expect_any,
    value::{V, VFn, Value},
};

pub fn get_prim_prop(target: Value, name: &str) -> Result<Value, AiScriptError> {
    Ok(match *target.value {
        V::Num(target) => match name {
            "to_str" => Value::fn_native_sync(move |_| Ok(Value::str(target.to_string()))),
            "to_hex" => Value::fn_native_sync(move |_| {
                if target == 0.0 {
                    return Ok(Value::str("0"));
                }
                let mut target = target;
                let is_negative = target.is_sign_negative();
                let mut result = if is_negative {
                    target = -target;
                    format!("-{:x}", target as u64)
                } else {
                    format!("{:x}", target as u64)
                };
                let fraction = target - target.trunc();
                if fraction > 0.0 {
                    const MAX_DIGITS: isize = 14;
                    let mut digits = target.log(16.0).floor() as isize + 1;
                    target = fraction;
                    if digits < MAX_DIGITS {
                        result.push('.');
                        target *= 16.0;
                        while digits < MAX_DIGITS - 1 && target > 0.0 {
                            let digit = target.trunc();
                            if let Some(c) = char::from_digit(digit as u32, 16) {
                                result.push(c);
                            }
                            digits += 1;
                            target -= digit;
                            target *= 16.0;
                        }
                        if target > 0.0 {
                            let digit = target.round();
                            if let Some(c) = char::from_digit(digit as u32, 16) {
                                result.push(c);
                            }
                        }
                    }
                }
                Ok(Value::str(result))
            }),
            _ => Err(AiScriptRuntimeError::NoSuchProperty {
                name: name.to_string(),
                target_type: "num".to_string(),
            })?,
        },
        V::Str(target) => match name {
            "to_num" => Value::fn_native_sync(move |_| {
                let parsed = target.parse::<f64>();
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
            }),
            "to_arr" => Value::fn_native_sync(move |_| {
                let arr = target.graphemes(true).map(Value::str);
                Ok(Value::arr(arr))
            }),
            "to_unicode_arr" => Value::fn_native_sync(move |_| {
                let arr = target.chars().map(Value::str);
                Ok(Value::arr(arr))
            }),
            "to_unicode_codepoint_arr" => Value::fn_native_sync(move |_| {
                let arr = target.chars().map(|c| Value::num(c as u32));
                Ok(Value::arr(arr))
            }),
            "to_char_arr" => Value::fn_native_sync(move |_| {
                let arr = target
                    .encode_utf16()
                    .map(|u| Value::str(String::from_utf16_lossy(&[u])));
                Ok(Value::arr(arr))
            }),
            "to_charcode_arr" => Value::fn_native_sync(move |_| {
                let arr = target.encode_utf16().map(Value::num);
                Ok(Value::arr(arr))
            }),
            "to_utf8_byte_arr" => Value::fn_native_sync(move |_| {
                let arr = target.as_bytes().iter().map(|&u| Value::num(u));
                Ok(Value::arr(arr))
            }),
            "len" => Value::num(target.graphemes(true).count() as f64),
            "replace" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                String::try_from(args.next().unwrap_or_default()).and_then(|a| {
                    let b = String::try_from(args.next().unwrap_or_default())?;
                    Ok(Value::str(target.replace(&a, &b)))
                })
            }),
            "index_of" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                String::try_from(args.next().unwrap_or_default()).and_then(|search| {
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
                })
            }),
            "incl" => Value::fn_native_sync(move |args| {
                let search = String::try_from(args.into_iter().next().unwrap_or_default());
                search.map(|search| Value::bool(target.contains(&search)))
            }),
            "trim" => Value::fn_native_sync(move |_| {
                let s = target.trim();
                Ok(Value::str(s))
            }),
            "upper" => Value::fn_native_sync(move |_| {
                let s = target.to_uppercase();
                Ok(Value::str(s))
            }),
            "lower" => Value::fn_native_sync(move |_| {
                let s = target.to_lowercase();
                Ok(Value::str(s))
            }),
            "split" => Value::fn_native_sync(move |args| {
                let splitter = args
                    .into_iter()
                    .next()
                    .map(String::try_from)
                    .map_or(Ok(None), |r| r.map(Some));
                splitter.map(|splitter| {
                    Value::arr(match splitter {
                        Some(splitter) if !splitter.is_empty() => target
                            .split(&splitter)
                            .map(Value::str)
                            .collect::<Vec<Value>>(),
                        _ => target
                            .graphemes(true)
                            .map(Value::str)
                            .collect::<Vec<Value>>(),
                    })
                })
            }),
            "slice" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                f64::try_from(args.next().unwrap_or_default()).and_then(|begin| {
                    let target_len = target.len();
                    let begin = target
                        .grapheme_indices(true)
                        .nth(begin as usize)
                        .map_or(begin as usize, |(i, _)| i)
                        .clamp(0, target_len);
                    let end = f64::try_from(args.next().unwrap_or_default())?;
                    let end = target
                        .grapheme_indices(true)
                        .nth(end as usize)
                        .map_or_else(|| target_len, |(i, _)| i)
                        .clamp(begin, target_len);
                    Ok(Value::str(&target[begin..end]))
                })
            }),
            "pick" => Value::fn_native_sync(move |args| {
                let i = f64::try_from(args.into_iter().next().unwrap_or_default());
                i.map(|i| {
                    target
                        .graphemes(true)
                        .nth(i as usize)
                        .map_or_else(Value::null, Value::str)
                })
            }),
            "charcode_at" => Value::fn_native_sync(move |args| {
                let i = f64::try_from(args.into_iter().next().unwrap_or_default());
                i.map(|i| {
                    target
                        .encode_utf16()
                        .map(Value::num)
                        .nth(i as usize)
                        .unwrap_or_default()
                })
            }),
            "codepoint_at" => Value::fn_native_sync(move |args| {
                let i = f64::try_from(args.into_iter().next().unwrap_or_default());
                i.map(|i| {
                    let c = char::decode_utf16(target.encode_utf16().skip(i as usize))
                        .map(|r| {
                            r.map_or_else(|e| e.unpaired_surrogate() as f64, |c| c as u32 as f64)
                        })
                        .next();
                    c.map_or_else(Value::null, Value::num)
                })
            }),
            "starts_with" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                String::try_from(args.next().unwrap_or_default()).and_then(|prefix| {
                    if prefix.is_empty() {
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
                            .starts_with(&prefix),
                    ))
                })
            }),
            "ends_with" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                String::try_from(args.next().unwrap_or_default()).and_then(|suffix| {
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
                            .map_or_else(|| target.len(), |(i, _)| i)]
                            .ends_with(&suffix),
                    ))
                })
            }),
            "pad_start" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                f64::try_from(args.next().unwrap_or_default()).and_then(|width| {
                    let width = width as usize;
                    let pad = args
                        .next()
                        .map(String::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .unwrap_or_else(|| " ".to_string());
                    let target_len = target.graphemes(true).count();
                    let pad_len = pad.graphemes(true).count();
                    Ok(Value::str(if width <= target_len {
                        target.clone()
                    } else {
                        let width = width - target_len;
                        let mut s = pad.repeat(width / pad_len);
                        s += &pad[..pad.grapheme_indices(true).nth(width % pad_len).unwrap().0];
                        s += &target;
                        s
                    }))
                })
            }),
            "pad_end" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                f64::try_from(args.next().unwrap_or_default()).and_then(|width| {
                    let pad = args
                        .next()
                        .map(String::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .unwrap_or_else(|| " ".to_string());
                    let target_len = target.graphemes(true).count();
                    let pad_len = pad.graphemes(true).count();
                    Ok(Value::str(if width as usize <= target_len {
                        target.clone()
                    } else {
                        let width = width as usize - target_len;
                        let mut s = target.clone();
                        s += &pad.repeat(width / pad_len);
                        s += &pad[..pad.grapheme_indices(true).nth(width % pad_len).unwrap().0];
                        s
                    }))
                })
            }),
            _ => Err(AiScriptRuntimeError::NoSuchProperty {
                name: name.to_string(),
                target_type: "str".to_string(),
            })?,
        },
        V::Arr(target) => match name {
            "len" => Value::num(target.read().map_err(AiScriptError::internal)?.len() as f64),
            "push" => Value::fn_native_sync(move |args| {
                expect_any(args.into_iter().next()).and_then(|val| {
                    target.write().map_err(AiScriptError::internal)?.push(val);
                    Ok(Value::new(V::Arr(target.clone())))
                })
            }),
            "unshift" => Value::fn_native_sync(move |args| {
                expect_any(args.into_iter().next()).and_then(|val| {
                    target
                        .write()
                        .map_err(AiScriptError::internal)?
                        .insert(0, val);
                    Ok(Value::new(V::Arr(target.clone())))
                })
            }),
            "pop" => Value::fn_native_sync(move |_| {
                target
                    .write()
                    .map_err(AiScriptError::internal)
                    .map(|mut target| target.pop().unwrap_or_default())
            }),
            "shift" => Value::fn_native_sync(move |_| {
                target
                    .read()
                    .map_err(AiScriptError::internal)
                    .map(|target| target.is_empty())
                    .and_then(|is_empty| {
                        Ok(if is_empty {
                            Value::null()
                        } else {
                            target.write().map_err(AiScriptError::internal)?.remove(0)
                        })
                    })
            }),
            "concat" => Value::fn_native_sync(move |args| {
                <Vec<Value>>::try_from(args.into_iter().next().unwrap_or_default()).and_then(|x| {
                    let mut target = target.read().map_err(AiScriptError::internal)?.clone();
                    target.extend(x);
                    Ok(Value::arr(target))
                })
            }),
            "slice" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                target
                    .read()
                    .map_err(AiScriptError::internal)
                    .map(|target| target.len())
                    .and_then(|target_len| {
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
                        Ok(Value::arr(
                            target.read().map_err(AiScriptError::internal)?[begin..end].to_vec(),
                        ))
                    })
            }),
            "join" => Value::fn_native_sync(move |args| {
                args.into_iter()
                    .next()
                    .map(String::try_from)
                    .map_or(Ok(None), |r| r.map(Some))
                    .and_then(|joiner| {
                        let joiner = joiner.unwrap_or_default();
                        Ok(Value::str(
                            target
                                .read()
                                .map_err(AiScriptError::internal)?
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
                    })
            }),
            "map" => Value::fn_native(move |args, interpreter| {
                let fn_ = match VFn::try_from(args.into_iter().next().unwrap_or_default()) {
                    Ok(fn_) => fn_,
                    Err(e) => return async { Err(e) }.boxed(),
                };
                let target = match target.read() {
                    Ok(target) => target.clone(),
                    Err(e) => {
                        let e = AiScriptError::internal(e);
                        return async { Err(e) }.boxed();
                    }
                };
                let interpreter = interpreter.clone();
                async move {
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
                let fn_ = match VFn::try_from(args.into_iter().next().unwrap_or_default()) {
                    Ok(fn_) => fn_,
                    Err(e) => return async { Err(e) }.boxed(),
                };
                let target = match target.read().map_err(AiScriptError::internal) {
                    Ok(target) => target.clone(),
                    Err(e) => return async { Err(e) }.boxed(),
                };
                let interpreter = interpreter.clone();
                let mut vals = Vec::new();
                async move {
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
                let mut args = args.into_iter();
                let fn_ = match VFn::try_from(args.next().unwrap_or_default()) {
                    Ok(fn_) => fn_,
                    Err(e) => return async { Err(e) }.boxed(),
                };
                let initial_value = args.next();
                let with_initial_value = initial_value.is_some();
                let mut target = match target.read() {
                    Ok(target) if !with_initial_value && target.is_empty() => {
                        return async { Err(AiScriptRuntimeError::ReduceWithoutInitialValue)? }
                            .boxed();
                    }
                    Ok(target) => target.clone().into_iter(),
                    Err(e) => {
                        let e = AiScriptError::internal(e);
                        return async { Err(e) }.boxed();
                    }
                };
                let interpreter = interpreter.clone();
                let mut accumlator = initial_value.unwrap_or_else(|| target.next().unwrap());
                async move {
                    for (i, item) in target.enumerate() {
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
                let fn_ = match VFn::try_from(args.into_iter().next().unwrap_or_default()) {
                    Ok(fn_) => fn_,
                    Err(e) => return async { Err(e) }.boxed(),
                };
                let target = match target.read() {
                    Ok(target) => target.clone(),
                    Err(e) => {
                        let e = AiScriptError::internal(e);
                        return async { Err(e) }.boxed();
                    }
                };
                let interpreter = interpreter.clone();
                async move {
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
            "incl" => Value::fn_native_sync(move |args| {
                expect_any(args.into_iter().next()).and_then(|val| {
                    Ok(Value::bool(
                        target
                            .read()
                            .map_err(AiScriptError::internal)?
                            .contains(&val),
                    ))
                })
            }),
            "index_of" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                expect_any(args.next()).and_then(|val| {
                    let target_len = target.read().map_err(AiScriptError::internal)?.len() as f64;
                    let from_i = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .map_or(0.0, |i| if i < 0.0 { target_len + i } else { i })
                        .clamp(0.0, target_len) as usize;
                    Ok(Value::num(
                        target.read().map_err(AiScriptError::internal)?[from_i..]
                            .iter()
                            .position(|item| item == &val)
                            .map_or(-1.0, |result| (result + from_i) as f64),
                    ))
                })
            }),
            "reverse" => Value::fn_native_sync(move |_| {
                target
                    .write()
                    .map_err(AiScriptError::internal)
                    .map(|mut target| {
                        target.reverse();
                        Value::null()
                    })
            }),
            "copy" => Value::fn_native_sync(move |_| {
                target
                    .read()
                    .map_err(AiScriptError::internal)
                    .map(|target| Value::arr(target.clone()))
            }),
            "sort" => {
                fn merge_sort(
                    arr: Vec<Value>,
                    comp: VFn,
                    interpreter: &Interpreter,
                ) -> BoxFuture<'static, Result<Vec<Value>, AiScriptError>> {
                    let len = arr.len();
                    if len <= 1 {
                        return async { Ok(arr) }.boxed();
                    }
                    let mid = len / 2;
                    let mut left = arr;
                    let right = left.split_off(mid);
                    let interpreter = interpreter.clone();
                    async move {
                        let (left, right) = try_join!(
                            merge_sort(left, comp.clone(), &interpreter),
                            merge_sort(right, comp.clone(), &interpreter),
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
                    let mut left = left.into_iter();
                    let mut l = left.next().unwrap();
                    let mut right = right.into_iter();
                    let mut r = right.next().unwrap();
                    loop {
                        let comp_value = interpreter
                            .exec_fn_simple(comp.clone(), vec![l.clone(), r.clone()])
                            .await?;
                        let comp_value = f64::try_from(comp_value)?;
                        if comp_value <= 0.0 {
                            result.push(l);
                            l = if let Some(l) = left.next() {
                                l
                            } else {
                                result.push(r);
                                result.extend(right);
                                break;
                            };
                        } else {
                            result.push(r);
                            r = if let Some(r) = right.next() {
                                r
                            } else {
                                result.push(l);
                                result.extend(left);
                                break;
                            };
                        }
                    }
                    Ok(result)
                }

                Value::fn_native(move |args, interpreter| {
                    let comp = match VFn::try_from(args.into_iter().next().unwrap_or_default()) {
                        Ok(comp) => comp,
                        Err(e) => return async { Err(e) }.boxed(),
                    };
                    let arr = match target.read() {
                        Ok(target) => target.clone(),
                        Err(e) => {
                            let e = AiScriptError::internal(e);
                            return async { Err(e) }.boxed();
                        }
                    };
                    let target = target.clone();
                    let sort = merge_sort(arr.to_vec(), comp, interpreter);
                    async {
                        let sorted = sort.await?;
                        target
                            .write()
                            .map_err(AiScriptError::internal)?
                            .splice(.., sorted);
                        Ok(Value::new(V::Arr(target)))
                    }
                    .boxed()
                })
            }
            "fill" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                let val = args.next().unwrap_or_default();
                target
                    .read()
                    .map_err(AiScriptError::internal)
                    .map(|target| target.len())
                    .and_then(|target_len| {
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
                        target
                            .write()
                            .map_err(AiScriptError::internal)?
                            .splice(start..end, std::iter::repeat_n(val, end - start));
                        Ok(Value::new(V::Arr(target.clone())))
                    })
            }),
            "repeat" => Value::fn_native_sync(move |args| {
                f64::try_from(args.into_iter().next().unwrap_or_default()).and_then(|times| {
                    if times < 0.0 {
                        Err(AiScriptRuntimeError::UnexpectedNegative(
                            "arr.repeat".to_string(),
                        ))?
                    } else if times.trunc() != times {
                        Err(AiScriptRuntimeError::UnexpectedNonInteger(
                            "arr.repeat".to_string(),
                        ))?
                    } else {
                        let mut value = Vec::new();
                        let target = &target.read().map_err(AiScriptError::internal)?[..];
                        for _ in 0..times as usize {
                            value.extend_from_slice(target);
                        }
                        Ok(Value::arr(value))
                    }
                })
            }),
            "splice" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                f64::try_from(args.next().unwrap_or_default()).and_then(|idx| {
                    let target_len = target.read().map_err(AiScriptError::internal)?.len();
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
                    Ok(Value::arr(
                        target
                            .write()
                            .map_err(AiScriptError::internal)?
                            .splice(index..index + remove_count, items),
                    ))
                })
            }),
            "flat" => {
                fn flat(
                    arr: &[Value],
                    depth: usize,
                    result: &mut Vec<Value>,
                ) -> Result<(), AiScriptError> {
                    if depth == 0 {
                        result.extend_from_slice(arr);
                    } else {
                        for v in arr {
                            if let V::Arr(value) = v.value.as_ref() {
                                flat(
                                    &value.read().map_err(AiScriptError::internal)?[..],
                                    depth - 1,
                                    result,
                                )?;
                            } else {
                                result.push(v.clone());
                            }
                        }
                    }
                    Ok(())
                }

                Value::fn_native_sync(move |args| {
                    args.into_iter()
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))
                        .and_then(|depth| {
                            let depth = depth.unwrap_or(1.0);
                            if depth < 0.0 {
                                Err(AiScriptRuntimeError::UnexpectedNegative(
                                    "arr.flat".to_string(),
                                ))?
                            } else if depth.trunc() != depth {
                                Err(AiScriptRuntimeError::UnexpectedNonInteger(
                                    "arr.flat".to_string(),
                                ))?
                            } else {
                                let target = target.read().map_err(AiScriptError::internal)?;
                                let mut result = Vec::new();
                                flat(&target[..], depth as usize, &mut result)?;
                                Ok(Value::arr(result))
                            }
                        })
                })
            }
            "flat_map" => Value::fn_native(move |args, interpreter| {
                let fn_ = match VFn::try_from(args.into_iter().next().unwrap_or_default()) {
                    Ok(fn_) => fn_,
                    Err(e) => return async { Err(e) }.boxed(),
                };
                let target = match target.read() {
                    Ok(target) => target.clone(),
                    Err(e) => {
                        let e = AiScriptError::internal(e);
                        return async { Err(e) }.boxed();
                    }
                };
                let interpreter = interpreter.clone();
                async move {
                    let mapped_vals =
                        try_join_all(target.into_iter().enumerate().map(|(i, item)| {
                            interpreter
                                .exec_fn_simple(fn_.clone(), vec![item, Value::num(i as f64)])
                        }))
                        .await?;
                    let mut result = Vec::new();
                    for value in mapped_vals {
                        if let V::Arr(value) = *value.value {
                            result.extend(value.read().map_err(AiScriptError::internal)?.clone())
                        } else {
                            result.push(value)
                        }
                    }
                    Ok(Value::arr(result))
                }
                .boxed()
            }),
            "every" => Value::fn_native(move |args, interpreter| {
                let fn_ = match VFn::try_from(args.into_iter().next().unwrap_or_default()) {
                    Ok(fn_) => fn_,
                    Err(e) => return async { Err(e) }.boxed(),
                };
                let target = match target.read() {
                    Ok(target) => target.clone(),
                    Err(e) => {
                        let e = AiScriptError::internal(e);
                        return async { Err(e) }.boxed();
                    }
                };
                let interpreter = interpreter.clone();
                async move {
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
                let fn_ = match VFn::try_from(args.into_iter().next().unwrap_or_default()) {
                    Ok(fn_) => fn_,
                    Err(e) => return async { Err(e) }.boxed(),
                };
                let target = match target.read() {
                    Ok(target) => target.clone(),
                    Err(e) => {
                        let e = AiScriptError::internal(e);
                        return async { Err(e) }.boxed();
                    }
                };
                let interpreter = interpreter.clone();
                async move {
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
            "insert" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                f64::try_from(args.next().unwrap_or_default()).and_then(|idx| {
                    let target_len = target.read().map_err(AiScriptError::internal)?.len() as f64;
                    let index = if idx < 0.0 { target_len + idx } else { idx }
                        .clamp(0.0, target_len) as usize;
                    let item = expect_any(args.next())?;
                    target
                        .write()
                        .map_err(AiScriptError::internal)?
                        .insert(index, item);
                    Ok(Value::null())
                })
            }),
            "remove" => Value::fn_native_sync(move |args| {
                f64::try_from(args.into_iter().next().unwrap_or_default()).and_then(|idx| {
                    let target_len = target.read().map_err(AiScriptError::internal)?.len();
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
                            target
                                .write()
                                .map_err(AiScriptError::internal)?
                                .remove(index)
                        }
                    })
                })
            }),
            "at" => Value::fn_native_sync(move |args| {
                let mut args = args.into_iter();
                f64::try_from(args.next().unwrap_or_default()).and_then(|idx| {
                    let idx = idx as isize;
                    let index = if idx < 0 {
                        let target_len = target.read().map_err(AiScriptError::internal)?.len();
                        target_len as isize + idx
                    } else {
                        idx
                    };
                    Ok(if index < 0 {
                        None
                    } else {
                        target
                            .read()
                            .map_err(AiScriptError::internal)?
                            .get(index as usize)
                            .cloned()
                    }
                    .unwrap_or_else(|| args.next().unwrap_or_default()))
                })
            }),
            _ => Err(AiScriptRuntimeError::NoSuchProperty {
                name: name.to_string(),
                target_type: "arr".to_string(),
            })?,
        },
        V::Error { value, info } => match name {
            "name" => Value::str(value),
            "info" => info.map_or_else(Value::null, |info| *info),
            _ => Err(AiScriptRuntimeError::NoSuchProperty {
                name: name.to_string(),
                target_type: "error".to_string(),
            })?,
        },
        value => Err(AiScriptRuntimeError::InvalidPrimitiveProperty {
            name: name.to_string(),
            target_type: value.display_type().to_string(),
        })?,
    })
}
