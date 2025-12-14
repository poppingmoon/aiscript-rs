use futures::{
    FutureExt,
    future::{BoxFuture, try_join_all},
};
use tokio::try_join;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    Interpreter,
    error::{AiScriptError, AiScriptRuntimeError},
};

use super::{
    util::expect_any,
    value::{V, VFn, Value},
};

pub fn get_prim_prop(target: Value, name: &str) -> Result<Value, AiScriptError> {
    Ok(match *target.value {
        V::Num(target) => match name {
            "to_str" => Value::fn_native(move |_, _| {
                let result = Ok(Value::str(target.to_string()));
                async { result }.boxed()
            }),
            _ => Err(AiScriptRuntimeError::runtime(format!(
                "No such prop ({name}) in num."
            )))?,
        },
        V::Str(target) => match name {
            "to_num" => Value::fn_native(move |_, _| {
                let parsed = target.parse::<f64>();
                let result = Ok(Value::new(parsed.map_or_else(
                    |_| V::Null,
                    |parsed| {
                        if parsed.is_nan() {
                            V::Null
                        } else {
                            V::Num(parsed)
                        }
                    },
                )));
                async { result }.boxed()
            }),
            "to_arr" => Value::fn_native(move |_, _| {
                let arr = target.graphemes(true).map(Value::str);
                let result = Ok(Value::arr(arr));
                async { result }.boxed()
            }),
            "to_unicode_arr" => Value::fn_native(move |_, _| {
                let arr = target.chars().map(Value::str);
                let result = Ok(Value::arr(arr));
                async { result }.boxed()
            }),
            "to_unicode_codepoint_arr" => Value::fn_native(move |_, _| {
                let arr = target.chars().map(|c| Value::num(c as u32));
                let result = Ok(Value::arr(arr));
                async { result }.boxed()
            }),
            "to_char_arr" => Value::fn_native(move |_, _| {
                let arr = target
                    .encode_utf16()
                    .map(|u| Value::str(String::from_utf16_lossy(&[u])));
                let result = Ok(Value::arr(arr));
                async { result }.boxed()
            }),
            "to_charcode_arr" => Value::fn_native(move |_, _| {
                let arr = target.encode_utf16().map(Value::num);
                let result = Ok(Value::arr(arr));
                async { result }.boxed()
            }),
            "to_utf8_byte_arr" => Value::fn_native(move |_, _| {
                let arr = target.as_bytes().iter().map(|&u| Value::num(u));
                let result = Ok(Value::arr(arr));
                async { result }.boxed()
            }),
            "len" => Value::num(target.graphemes(true).count() as f64),
            "replace" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = String::try_from(args.next().unwrap_or_default()).and_then(|a| {
                    let b = String::try_from(args.next().unwrap_or_default())?;
                    Ok(Value::str(target.replace(&a, &b)))
                });
                async { result }.boxed()
            }),
            "index_of" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = String::try_from(args.next().unwrap_or_default()).and_then(|search| {
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
                });
                async { result }.boxed()
            }),
            "incl" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let search = String::try_from(args.next().unwrap_or_default());
                let result = search.map(|search| Value::bool(target.contains(&search)));
                async { result }.boxed()
            }),
            "trim" => Value::fn_native(move |_, _| {
                let s = target.trim();
                let result = Ok(Value::str(s));
                async { result }.boxed()
            }),
            "upper" => Value::fn_native(move |_, _| {
                let s = target.to_uppercase();
                let result = Ok(Value::str(s));
                async { result }.boxed()
            }),
            "lower" => Value::fn_native(move |_, _| {
                let s = target.to_lowercase();
                let result = Ok(Value::str(s));
                async { result }.boxed()
            }),
            "split" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let splitter = args
                    .next()
                    .map(String::try_from)
                    .map_or(Ok(None), |r| r.map(Some));
                let result = splitter.map(|splitter| {
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
                });
                async { result }.boxed()
            }),
            "slice" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = f64::try_from(args.next().unwrap_or_default()).and_then(|begin| {
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
                });
                async { result }.boxed()
            }),
            "pick" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let i = f64::try_from(args.next().unwrap_or_default());
                let result = i.map(|i| {
                    target
                        .graphemes(true)
                        .nth(i as usize)
                        .map_or_else(Value::null, Value::str)
                });
                async { result }.boxed()
            }),
            "charcode_at" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let i = f64::try_from(args.next().unwrap_or_default());
                let result = i.map(|i| {
                    target
                        .encode_utf16()
                        .map(Value::num)
                        .nth(i as usize)
                        .unwrap_or_default()
                });
                async { result }.boxed()
            }),
            "codepoint_at" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let i = f64::try_from(args.next().unwrap_or_default());
                let result = i.map(|i| {
                    let c = char::decode_utf16(target.encode_utf16().skip(i as usize))
                        .map(|r| {
                            r.map_or_else(|e| e.unpaired_surrogate() as f64, |c| c as u32 as f64)
                        })
                        .next();
                    c.map_or_else(Value::null, Value::num)
                });
                async { result }.boxed()
            }),
            "starts_with" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = String::try_from(args.next().unwrap_or_default()).and_then(|prefix| {
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
                });
                async { result }.boxed()
            }),
            "ends_with" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = String::try_from(args.next().unwrap_or_default()).and_then(|suffix| {
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
                });
                async { result }.boxed()
            }),
            "pad_start" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = f64::try_from(args.next().unwrap_or_default()).and_then(|width| {
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
                });
                async { result }.boxed()
            }),
            "pad_end" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = f64::try_from(args.next().unwrap_or_default()).and_then(|width| {
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
                });
                async { result }.boxed()
            }),
            _ => Err(AiScriptRuntimeError::runtime(format!(
                "No such prop ({name}) in str."
            )))?,
        },
        V::Arr(target) => match name {
            "len" => Value::num(target.read().map_err(AiScriptError::internal)?.len() as f64),
            "push" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = expect_any(args.next()).and_then(|val| {
                    target.write().map_err(AiScriptError::internal)?.push(val);
                    Ok(Value::new(V::Arr(target.clone())))
                });
                async { result }.boxed()
            }),
            "unshift" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = expect_any(args.next()).and_then(|val| {
                    target
                        .write()
                        .map_err(AiScriptError::internal)?
                        .insert(0, val);
                    Ok(Value::new(V::Arr(target.clone())))
                });
                async { result }.boxed()
            }),
            "pop" => Value::fn_native(move |_, _| {
                let result = target
                    .write()
                    .map_err(AiScriptError::internal)
                    .map(|mut target| target.pop().unwrap_or_default());
                async { result }.boxed()
            }),
            "shift" => Value::fn_native(move |_, _| {
                let result = target
                    .read()
                    .map_err(AiScriptError::internal)
                    .map(|target| target.is_empty())
                    .and_then(|is_empty| {
                        Ok(if is_empty {
                            Value::null()
                        } else {
                            target.write().map_err(AiScriptError::internal)?.remove(0)
                        })
                    });
                async { result }.boxed()
            }),
            "concat" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result =
                    <Vec<Value>>::try_from(args.next().unwrap_or_default()).and_then(|x| {
                        let mut target = target.read().map_err(AiScriptError::internal)?.clone();
                        target.extend(x);
                        Ok(Value::arr(target))
                    });
                async { result }.boxed()
            }),
            "slice" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = target
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
                    });
                async { result }.boxed()
            }),
            "join" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = args
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
                    });
                async { result }.boxed()
            }),
            "map" => Value::fn_native(move |args, interpreter| {
                let mut args = args.into_iter();
                let fn_ = match VFn::try_from(args.next().unwrap_or_default()) {
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
                let mut args = args.into_iter();
                let fn_ = match VFn::try_from(args.next().unwrap_or_default()) {
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
                        return async {
                            Err(AiScriptRuntimeError::runtime(
                                "Reduce of empty array without initial value",
                            )
                            .into())
                        }
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
                let mut args = args.into_iter();
                let fn_ = match VFn::try_from(args.next().unwrap_or_default()) {
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
            "incl" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = expect_any(args.next()).and_then(|val| {
                    Ok(Value::bool(
                        target
                            .read()
                            .map_err(AiScriptError::internal)?
                            .contains(&val),
                    ))
                });
                async { result }.boxed()
            }),
            "index_of" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = expect_any(args.next()).and_then(|val| {
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
                });
                async { result }.boxed()
            }),
            "reverse" => Value::fn_native(move |_, _| {
                let result = target
                    .write()
                    .map_err(AiScriptError::internal)
                    .map(|mut target| {
                        target.reverse();
                        Value::null()
                    });
                async { result }.boxed()
            }),
            "copy" => Value::fn_native(move |_, _| {
                let result = target
                    .read()
                    .map_err(AiScriptError::internal)
                    .map(|target| Value::arr(target.clone()));
                async { result }.boxed()
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
                        if comp_value < 0.0 {
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
                    let mut args = args.into_iter();
                    let comp = match VFn::try_from(args.next().unwrap_or_default()) {
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
            "fill" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let val = args.next().unwrap_or_default();
                let result = target
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
                    });
                async { result }.boxed()
            }),
            "repeat" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = f64::try_from(args.next().unwrap_or_default()).and_then(|times| {
                    if times < 0.0 {
                        Err(AiScriptRuntimeError::runtime(
                            "arr.repeat expected non-negative number, got negative",
                        ))?
                    } else if times.trunc() != times {
                        Err(AiScriptRuntimeError::runtime(
                            "arr.repeat expected integer, got non-integer",
                        ))?
                    } else {
                        let mut value = Vec::new();
                        let target = &target.read().map_err(AiScriptError::internal)?[..];
                        for _ in 0..times as usize {
                            value.extend_from_slice(target);
                        }
                        Ok(Value::arr(value))
                    }
                });
                async { result }.boxed()
            }),
            "splice" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = f64::try_from(args.next().unwrap_or_default()).and_then(|idx| {
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
                });
                async { result }.boxed()
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

                Value::fn_native(move |args, _| {
                    let mut args = args.into_iter();
                    let result = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))
                        .and_then(|depth| {
                            let depth = depth.unwrap_or(1.0);
                            if depth < 0.0 {
                                Err(AiScriptRuntimeError::runtime(
                                    "arr.flat expected non-negative number, got negative",
                                ))?
                            } else if depth.trunc() != depth {
                                Err(AiScriptRuntimeError::runtime(
                                    "arr.flat expected integer, got non-integer",
                                ))?
                            } else {
                                let target = target.read().map_err(AiScriptError::internal)?;
                                let mut result = Vec::new();
                                flat(&target[..], depth as usize, &mut result)?;
                                Ok(Value::arr(result))
                            }
                        });
                    async { result }.boxed()
                })
            }
            "flat_map" => Value::fn_native(move |args, interpreter| {
                let mut args = args.into_iter();
                let fn_ = match VFn::try_from(args.next().unwrap_or_default()) {
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
                let mut args = args.into_iter();
                let fn_ = match VFn::try_from(args.next().unwrap_or_default()) {
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
                let mut args = args.into_iter();
                let fn_ = match VFn::try_from(args.next().unwrap_or_default()) {
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
            "insert" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = f64::try_from(args.next().unwrap_or_default()).and_then(|idx| {
                    let target_len = target.read().map_err(AiScriptError::internal)?.len() as f64;
                    let index = if idx < 0.0 { target_len + idx } else { idx }
                        .clamp(0.0, target_len) as usize;
                    let item = expect_any(args.next())?;
                    target
                        .write()
                        .map_err(AiScriptError::internal)?
                        .insert(index, item);
                    Ok(Value::null())
                });
                async { result }.boxed()
            }),
            "remove" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = f64::try_from(args.next().unwrap_or_default()).and_then(|idx| {
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
                });
                async { result }.boxed()
            }),
            "at" => Value::fn_native(move |args, _| {
                let mut args = args.into_iter();
                let result = f64::try_from(args.next().unwrap_or_default()).and_then(|idx| {
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
                });
                async { result }.boxed()
            }),
            _ => Err(AiScriptRuntimeError::runtime(format!(
                "No such prop ({name}) in arr."
            )))?,
        },
        V::Error { value, info } => match name {
            "name" => Value::str(value),
            "info" => info.map_or_else(Value::null, |info| *info),
            _ => Err(AiScriptRuntimeError::runtime(format!(
                "No such prop ({name}) in error."
            )))?,
        },
        value => Err(AiScriptRuntimeError::runtime(format!(
            "Cannot read prop of {}. (reading {name})",
            value.display_type()
        )))?,
    })
}
