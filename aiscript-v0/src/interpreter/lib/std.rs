use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::{Datelike, TimeZone, Timelike};
use futures::FutureExt;
use indexmap::IndexMap;

use crate::{
    constants::AISCRIPT_VERSION,
    error::{AiScriptError, AiScriptRuntimeError},
    interpreter::{
        util::expect_any,
        value::{V, VFn, VObj, Value},
    },
};

use self::{
    seedrandom::seedrandom,
    uri_encoding::{decode_uri, decode_uri_component, encode_uri, encode_uri_component},
};

mod seedrandom;
mod uri_encoding;

pub fn std() -> HashMap<String, Value> {
    let mut std = HashMap::new();

    std.insert(
        "help".to_string(),
        Value::str("SEE: https://github.com/syuilo/aiscript/blob/master/docs/get-started.md"),
    );

    std.insert("Core:v".to_string(), Value::str(AISCRIPT_VERSION));

    std.insert("Core:ai".to_string(), Value::str("kawaii"));

    std.insert(
        "Core:not".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = bool::try_from(args.next().unwrap_or_default()).map(|a| Value::bool(!a));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:eq".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = expect_any(args.next()).and_then(|a| {
                let b = expect_any(args.next())?;
                Ok(Value::bool(a == b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:neq".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = expect_any(args.next()).and_then(|a| {
                let b = expect_any(args.next())?;
                Ok(Value::bool(a != b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:and".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = bool::try_from(args.next().unwrap_or_default()).and_then(|a| {
                Ok(Value::bool(if !a {
                    false
                } else {
                    bool::try_from(args.next().unwrap_or_default())?
                }))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:or".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = bool::try_from(args.next().unwrap_or_default()).and_then(|a| {
                Ok(Value::bool(if a {
                    true
                } else {
                    bool::try_from(args.next().unwrap_or_default())?
                }))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:add".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(a + b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:sub".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(a - b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:mul".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(a * b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:pow".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                let res = a.powf(b);
                if res.is_nan() {
                    // ex. √-1)
                    Err(AiScriptRuntimeError::runtime("Invalid operation."))?
                } else {
                    Ok(Value::num(res))
                }
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:div".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                let res = a / b;
                if res.is_nan() {
                    Err(AiScriptRuntimeError::runtime("Invalid operation."))?
                } else {
                    Ok(Value::num(res))
                }
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:mod".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(a % b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:gt".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::bool(a > b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:lt".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::bool(a < b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:gteq".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::bool(a >= b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:lteq".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::bool(a <= b))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:type".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = expect_any(args.next()).map(|v| Value::str(v.display_type().to_string()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:to_str".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = expect_any(args.next()).map(|v| Value::str(v.repr_value().to_string()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:range".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::arr(if a < b {
                    let length = (b - a).floor() + 1.0;
                    let mut i = 0.0;
                    std::iter::from_fn(move || {
                        let v = if i < length { Value::num(a + i) } else { None? };
                        i += 1.0;
                        Some(v)
                    })
                    .collect()
                } else if a > b {
                    let length = (a - b).floor() + 1.0;
                    let mut i = 0.0;
                    std::iter::from_fn(move || {
                        let v = if i < length { Value::num(a - i) } else { None? };
                        i += 1.0;
                        Some(v)
                    })
                    .collect()
                } else {
                    vec![Value::num(a)]
                }))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Core:sleep".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let sleep = f64::try_from(args.next().unwrap_or_default())
                .map(|delay| tokio::time::sleep(Duration::from_millis(delay as u64)));
            async {
                sleep?.await;
                Ok(Value::null())
            }
            .boxed()
        }),
    );

    std.insert(
        "Core:abort".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default())
                .and_then(|message| Err(AiScriptRuntimeError::user(message))?);
            async { result }.boxed()
        }),
    );

    std.insert(
        "Util:uuid".to_string(),
        Value::fn_native(|_, _| {
            let result = Ok(Value::str(uuid::Uuid::new_v4()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Json:stringify".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = expect_any(args.next()).and_then(|v| {
                Ok(Value::str(
                    serde_json::to_string(&v.value).map_err(AiScriptError::internal)?,
                ))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Json:parse".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default()).map(|json| {
                serde_json::from_str(&json)
                    .map_or_else(|_| Value::error("not_json", None), Value::new)
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Json:parsable".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default())
                .map(|json| Value::bool(serde_json::from_str::<V>(&json).is_ok()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:now".to_string(),
        Value::fn_native(|_, _| {
            let result = Ok(Value::num(chrono::Local::now().timestamp_millis() as f64));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:year".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = args
                .next()
                .map(f64::try_from)
                .map_or(Ok(None), |r| r.map(Some))
                .and_then(|v| {
                    let date = if let Some(v) = v {
                        chrono::Local
                            .timestamp_millis_opt(v as i64)
                            .earliest()
                            .ok_or_else(|| {
                                AiScriptError::internal(format!("invalid timestamp: {v}"))
                            })?
                    } else {
                        chrono::Local::now()
                    };
                    Ok(Value::num(date.year()))
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:month".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = args
                .next()
                .map(f64::try_from)
                .map_or(Ok(None), |r| r.map(Some))
                .and_then(|v| {
                    let date = if let Some(v) = v {
                        chrono::Local
                            .timestamp_millis_opt(v as i64)
                            .earliest()
                            .ok_or_else(|| {
                                AiScriptError::internal(format!("invalid timestamp: {v}"))
                            })?
                    } else {
                        chrono::Local::now()
                    };
                    Ok(Value::num(date.month()))
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:day".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = args
                .next()
                .map(f64::try_from)
                .map_or(Ok(None), |r| r.map(Some))
                .and_then(|v| {
                    let date = if let Some(v) = v {
                        chrono::Local
                            .timestamp_millis_opt(v as i64)
                            .earliest()
                            .ok_or_else(|| {
                                AiScriptError::internal(format!("invalid timestamp: {v}"))
                            })?
                    } else {
                        chrono::Local::now()
                    };
                    Ok(Value::num(date.day()))
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:hour".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = args
                .next()
                .map(f64::try_from)
                .map_or(Ok(None), |r| r.map(Some))
                .and_then(|v| {
                    let date = if let Some(v) = v {
                        chrono::Local
                            .timestamp_millis_opt(v as i64)
                            .earliest()
                            .ok_or_else(|| {
                                AiScriptError::internal(format!("invalid timestamp: {v}"))
                            })?
                    } else {
                        chrono::Local::now()
                    };
                    Ok(Value::num(date.hour()))
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:minute".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = args
                .next()
                .map(f64::try_from)
                .map_or(Ok(None), |r| r.map(Some))
                .and_then(|v| {
                    let date = if let Some(v) = v {
                        chrono::Local
                            .timestamp_millis_opt(v as i64)
                            .earliest()
                            .ok_or_else(|| {
                                AiScriptError::internal(format!("invalid timestamp: {v}"))
                            })?
                    } else {
                        chrono::Local::now()
                    };
                    Ok(Value::num(date.minute()))
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:second".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = args
                .next()
                .map(f64::try_from)
                .map_or(Ok(None), |r| r.map(Some))
                .and_then(|v| {
                    let date = if let Some(v) = v {
                        chrono::Local
                            .timestamp_millis_opt(v as i64)
                            .earliest()
                            .ok_or_else(|| {
                                AiScriptError::internal(format!("invalid timestamp: {v}"))
                            })?
                    } else {
                        chrono::Local::now()
                    };
                    Ok(Value::num(date.second()))
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:millisecond".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = args
                .next()
                .map(f64::try_from)
                .map_or(Ok(None), |r| r.map(Some))
                .map(|v| {
                    let v = v.unwrap_or_else(|| chrono::Local::now().timestamp_millis() as f64);
                    Value::num(v % 1000.0)
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:parse".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default()).map(|v| {
                let v = v.trim();
                let date = v
                    .parse::<chrono::DateTime<chrono::FixedOffset>>()
                    .or_else(|_| chrono::DateTime::parse_from_rfc2822(v))
                    .ok()
                    .map(|date| date.timestamp_millis())
                    .or_else(|| {
                        Some(
                            v.parse::<chrono::NaiveDateTime>()
                                .ok()?
                                .and_local_timezone(chrono::Local)
                                .earliest()?
                                .timestamp_millis(),
                        )
                    })
                    .or_else(|| {
                        if v.is_empty() {
                            None?
                        }
                        let mut numbers: [u32; 9] = [0, 0, 0, 0, 0, 0, 1, 1, 0];
                        let mut index = 0;
                        let mut previous_byte = b' ';
                        let mut is_east = true;
                        for b in v.bytes() {
                            if index > 8 {
                                None?
                            }
                            match b {
                                b'0' => numbers[index] *= 10,
                                b'1' => numbers[index] = numbers[index] * 10 + 1,
                                b'2' => numbers[index] = numbers[index] * 10 + 2,
                                b'3' => numbers[index] = numbers[index] * 10 + 3,
                                b'4' => numbers[index] = numbers[index] * 10 + 4,
                                b'5' => numbers[index] = numbers[index] * 10 + 5,
                                b'6' => numbers[index] = numbers[index] * 10 + 6,
                                b'7' => numbers[index] = numbers[index] * 10 + 7,
                                b'8' => numbers[index] = numbers[index] * 10 + 8,
                                b'9' => numbers[index] = numbers[index] * 10 + 9,
                                b'-' | b'.' | b'/' if index < 2 && numbers[index] > 0 => index += 1,
                                b'-' | b'.' | b'/'
                                    if index <= 2 && previous_byte.is_ascii_whitespace() => {}
                                b'T' | b't' | b'_' if index == 2 && numbers[2] > 0 => index += 1,
                                b'T' | b't' | b'_'
                                    if index == 3 && previous_byte.is_ascii_whitespace() => {}
                                b if index < 3 && b.is_ascii_whitespace() => {
                                    if previous_byte.is_ascii_digit() {
                                        index += 1
                                    }
                                }
                                b':' if (index == 3 || index == 4 || index == 7)
                                    && (previous_byte.is_ascii_digit()
                                        || previous_byte.is_ascii_whitespace()) =>
                                {
                                    index += 1
                                }
                                b'.' if index == 5 && previous_byte.is_ascii_digit() => index += 1,
                                b'+' if (4..=6).contains(&index)
                                    && (previous_byte.is_ascii_digit()
                                        || previous_byte.is_ascii_whitespace()) =>
                                {
                                    index = 7
                                }
                                b'-' if (4..=6).contains(&index)
                                    && (previous_byte.is_ascii_digit()
                                        || previous_byte.is_ascii_whitespace()) =>
                                {
                                    is_east = false;
                                    index = 7
                                }
                                b'Z' | b'z'
                                    if (4..=6).contains(&index)
                                        && (previous_byte.is_ascii_digit()
                                            || previous_byte.is_ascii_whitespace()) =>
                                {
                                    index = 10
                                }
                                b if b.is_ascii_whitespace() => {}
                                _ => None?,
                            };
                            previous_byte = b;
                        }
                        let (year, month, day) = if index < 2 {
                            match numbers[0] {
                                100.. => (numbers[0], numbers[1].max(1), 1),
                                50.. => (numbers[0] + 1900, numbers[1].max(1), 1),
                                32.. => (numbers[0] + 2000, numbers[1].max(1), 1),
                                1..=12 => (2001, numbers[0].max(1), numbers[1].max(1)),
                                _ => None?,
                            }
                        } else if numbers[0] >= 100 {
                            (numbers[0], numbers[1].max(1), numbers[2].max(1))
                        } else {
                            (numbers[2], numbers[0].max(1), numbers[1].max(1))
                        };
                        let time = chrono::NaiveDate::from_ymd_opt(year as i32, month, day)?;
                        let hour = numbers[3];
                        let minute = numbers[4];
                        let second = numbers[5];
                        let millis_digits = numbers[6].ilog10();
                        let millisecond = numbers[6] - 10_u32.pow(millis_digits);
                        let millisecond = if millis_digits > 3 {
                            millisecond / 10_u32.pow(millis_digits - 3)
                        } else {
                            millisecond * 10_u32.pow(3 - millis_digits)
                        };
                        let time = time.and_hms_milli_opt(hour, minute, second, millisecond)?;
                        let tz = match index {
                            ..3 => chrono::FixedOffset::east_opt(0)?,
                            3..7 => *chrono::Local::now().offset(),
                            7 => {
                                let tz_digits = numbers[7].ilog10();
                                let tz = numbers[7] - 10_u32.pow(tz_digits);
                                let secs = if tz_digits >= 3 {
                                    let hour = tz / 100;
                                    let minutes = tz - hour * 100;
                                    hour * 3600 + minutes * 60
                                } else {
                                    tz * 3600
                                } as i32;
                                let secs = if is_east { secs } else { -secs };
                                chrono::FixedOffset::east_opt(secs)?
                            }
                            _ => {
                                let hour = numbers[7] - 10_u32.pow(numbers[7].ilog10());
                                let secs = (hour * 3600 + numbers[8] * 60) as i32;
                                let secs = if is_east { secs } else { -secs };
                                chrono::FixedOffset::east_opt(secs)?
                            }
                        };
                        let time = time.and_local_timezone(tz).earliest()?;
                        Some(time.timestamp_millis())
                    })
                    .map_or(f64::NAN, |date| date as f64);
                Value::num(date)
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Date:to_iso_str".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = args
                .next()
                .map(f64::try_from)
                .map_or(Ok(None), |r| r.map(Some))
                .and_then(|v| {
                    let date = if let Some(v) = v {
                        chrono::Local
                            .timestamp_millis_opt(v as i64)
                            .earliest()
                            .ok_or_else(|| {
                                AiScriptError::internal(format!("invalid timestamp: {v}"))
                            })?
                    } else {
                        chrono::Local::now()
                    };
                    let mut date = date.fixed_offset();
                    let offset = args
                        .next()
                        .map(f64::try_from)
                        .map_or(Ok(None), |r| r.map(Some))?
                        .and_then(|ofs| chrono::FixedOffset::east_opt((ofs * 60.0) as i32));
                    if let Some(offset) = offset {
                        date = date.with_timezone(&offset);
                    }
                    Ok(Value::str(format!(
                        "{}",
                        date.format(if date.offset().local_minus_utc() == 0 {
                            "%Y-%m-%dT%H:%M:%S%.3fZ"
                        } else {
                            "%Y-%m-%dT%H:%M:%S%.3f%:z"
                        }),
                    )))
                });
            async { result }.boxed()
        }),
    );

    std.insert("Math:Infinity".to_string(), Value::num(f64::INFINITY));

    std.insert("Math:E".to_string(), Value::num(std::f64::consts::E));

    std.insert("Math:LN2".to_string(), Value::num(std::f64::consts::LN_2));

    std.insert("Math:LN10".to_string(), Value::num(std::f64::consts::LN_10));

    std.insert(
        "Math:LOG2E".to_string(),
        Value::num(std::f64::consts::LOG2_E),
    );

    std.insert(
        "Math:LOG10E".to_string(),
        Value::num(std::f64::consts::LOG10_E),
    );

    std.insert("Math:PI".to_string(), Value::num(std::f64::consts::PI));

    std.insert(
        "Math:SQRT1_2".to_string(),
        Value::num(std::f64::consts::FRAC_1_SQRT_2),
    );

    std.insert(
        "Math:SQRT2".to_string(),
        Value::num(std::f64::consts::SQRT_2),
    );

    std.insert(
        "Math:abs".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.abs()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:acos".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.acos()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:acosh".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.acosh()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:asin".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.asin()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:asinh".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.asinh()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:atan".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.atan()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:atanh".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.atanh()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:atan2".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|y| {
                let x = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(y.atan2(x)))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:cbrt".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.cbrt()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:ceil".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.ceil()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:clz32".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default())
                .map(|v| Value::num((v as i32).leading_zeros()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:cos".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.cos()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:cosh".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.cosh()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:exp".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.exp()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:expm1".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.exp_m1()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:floor".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.floor()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:fround".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v as f32));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:hypot".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = <Vec<Value>>::try_from(args.next().unwrap_or_default()).and_then(|args| {
                Ok(Value::num(match args.len() {
                    0 => 0.0,
                    1 => f64::try_from(args.into_iter().next().unwrap_or_default())?.abs(),
                    2 => {
                        let mut args = args.into_iter();
                        let a = f64::try_from(args.next().unwrap_or_default())?;
                        let b = f64::try_from(args.next().unwrap_or_default())?;
                        a.hypot(b)
                    }
                    _ => {
                        let mut values = Vec::new();
                        for v in args {
                            let v = f64::try_from(v)?;
                            values.push(v);
                        }
                        values.iter().fold(0.0, |acc, v| acc + v * v).sqrt()
                    }
                }))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:imul".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num((a as i32) * (b as i32)))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:log".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.ln()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:log1p".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.ln_1p()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:log10".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.log10()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:log2".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.log2()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:max".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(a.max(b)))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:min".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(a.min(b)))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:pow".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = f64::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(a.powf(b)))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:round".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.round()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:sign".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).map(|v| {
                Value::num(if v < 0.0 {
                    -1.0
                } else if v == 0.0 {
                    0.0
                } else {
                    1.0
                })
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:sin".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.sin()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:sinh".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.sinh()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:sqrt".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.sqrt()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:tan".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.tan()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:tanh".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.tanh()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:trunc".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                f64::try_from(args.next().unwrap_or_default()).map(|v| Value::num(v.trunc()));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:rnd".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let min = args.next().and_then(|arg| f64::try_from(arg).ok());
            let max = args.next().and_then(|arg| f64::try_from(arg).ok());
            let result = Ok(Value::num(if let (Some(min), Some(max)) = (min, max) {
                let max = max.floor();
                let min = min.ceil();
                (rand::random::<f64>() * (max - min + 1.0)).floor() + min
            } else {
                rand::random()
            }));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Math:gen_rng".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = expect_any(args.next()).map(|seed| {
                match *seed.value {
                    V::Num(num) => Some(num.to_string()),
                    V::Str(str) => Some(str),
                    _ => None,
                }
                .map_or_else(Value::null, |seed| {
                    let rng = Arc::new(Mutex::new(seedrandom(seed)));
                    Value::fn_native(move |args, _| {
                        let result = rng
                            .lock()
                            .map_err(AiScriptError::internal)
                            .map(|mut rng| rng())
                            .map(|r| {
                                let mut args = args.into_iter();
                                let min = args.next().and_then(|arg| f64::try_from(arg).ok());
                                let max = args.next().and_then(|arg| f64::try_from(arg).ok());
                                Value::num(if let (Some(min), Some(max)) = (min, max) {
                                    let max = max.floor();
                                    let min = min.ceil();
                                    (r * (max - min + 1.0)).floor() + min
                                } else {
                                    r
                                })
                            });

                        async { result }.boxed()
                    })
                })
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Num:to_hex".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default())
                .map(|v| Value::str(format!("{:x}", v as i64)));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Num:from_hex".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default())
                .map(|v| Value::num(i64::from_str_radix(&v, 16).map_or(f64::NAN, |v| v as f64)));
            async { result }.boxed()
        }),
    );

    std.insert("Str:lf".to_string(), Value::str("\n"));

    std.insert(
        "Str:lt".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = String::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(match a.cmp(&b) {
                    std::cmp::Ordering::Less => -1.0,
                    std::cmp::Ordering::Equal => 0.0,
                    std::cmp::Ordering::Greater => 1.0,
                }))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Str:gt".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default()).and_then(|a| {
                let b = String::try_from(args.next().unwrap_or_default())?;
                Ok(Value::num(match a.cmp(&b) {
                    std::cmp::Ordering::Less => 1.0,
                    std::cmp::Ordering::Equal => 0.0,
                    std::cmp::Ordering::Greater => -1.0,
                }))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Str:from_codepoint".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|codepoint| {
                char::from_u32(codepoint as u32).map_or_else(
                    || {
                        Err(AiScriptError::internal(format!(
                            "{codepoint} is not a valid code point"
                        )))
                    },
                    |c| Ok(Value::str(c)),
                )
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Str:from_unicode_codepoints".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                <Vec<Value>>::try_from(args.next().unwrap_or_default()).and_then(|codepoints| {
                    let mut s = String::new();
                    for codepoint in codepoints {
                        let codepoint = f64::try_from(codepoint)?;
                        let c = char::from_u32(codepoint as u32).ok_or_else(|| {
                            AiScriptError::internal(format!(
                                "{codepoint} is not a valid code point"
                            ))
                        })?;
                        s += c.to_string().as_str();
                    }
                    Ok(Value::str(s))
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Str:from_utf8_bytes".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result =
                <Vec<Value>>::try_from(args.next().unwrap_or_default()).and_then(|bytes| {
                    let bytes = bytes
                        .into_iter()
                        .map(|a| f64::try_from(a).map(|a| a.trunc().rem_euclid(256.0) as u8))
                        .collect::<Result<Vec<u8>, AiScriptError>>()?;
                    Ok(Value::str(String::from_utf8(bytes).unwrap_or_default()))
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Uri:encode_full".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default())
                .map(|v| Value::str(encode_uri(&v)));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Uri:encode_component".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default())
                .map(|v| Value::str(encode_uri_component(&v)));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Uri:decode_full".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default())
                .and_then(|v| Ok(Value::str(decode_uri(&v).map_err(AiScriptError::internal)?)));
            async { result }.boxed()
        }),
    );

    std.insert(
        "Uri:decode_component".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default()).and_then(|v| {
                Ok(Value::str(
                    decode_uri_component(&v).map_err(AiScriptError::internal)?,
                ))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Arr:create".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = f64::try_from(args.next().unwrap_or_default()).and_then(|length| {
                if length < 0.0 {
                    Err(AiScriptRuntimeError::runtime(
                        "Arr:create expected non-negative number, got negative",
                    ))?
                } else if length.trunc() != length {
                    Err(AiScriptRuntimeError::runtime(
                        "Arr:create expected integer, got non-integer",
                    ))?
                } else {
                    let initial = args.next().unwrap_or_default();
                    Ok(Value::arr(std::iter::repeat_n(initial, length as usize)))
                }
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Obj:keys".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = VObj::try_from(args.next().unwrap_or_default()).and_then(|obj| {
                Ok(Value::arr(
                    obj.read()
                        .map_err(AiScriptError::internal)?
                        .keys()
                        .map(Value::str),
                ))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Obj:vals".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = VObj::try_from(args.next().unwrap_or_default()).and_then(|obj| {
                Ok(Value::arr(
                    obj.read()
                        .map_err(AiScriptError::internal)?
                        .values()
                        .cloned(),
                ))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Obj:kvs".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = VObj::try_from(args.next().unwrap_or_default()).and_then(|obj| {
                Ok(Value::arr(
                    obj.read()
                        .map_err(AiScriptError::internal)?
                        .iter()
                        .map(|(k, v)| Value::arr([Value::str(k), v.clone()])),
                ))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Obj:get".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = VObj::try_from(args.next().unwrap_or_default()).and_then(|obj| {
                let key = String::try_from(args.next().unwrap_or_default())?;
                let value = obj
                    .read()
                    .map_err(AiScriptError::internal)?
                    .get(&key)
                    .cloned()
                    .unwrap_or_default();
                Ok(value)
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Obj:set".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = VObj::try_from(args.next().unwrap_or_default()).and_then(|obj| {
                let key = String::try_from(args.next().unwrap_or_default())?;
                let value = expect_any(args.next())?;
                obj.write()
                    .map_err(AiScriptError::internal)?
                    .insert(key, value);
                Ok(Value::null())
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Obj:has".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = VObj::try_from(args.next().unwrap_or_default()).and_then(|obj| {
                let key = String::try_from(args.next().unwrap_or_default())?;
                let has = obj
                    .read()
                    .map_err(AiScriptError::internal)?
                    .contains_key(&key);
                Ok(Value::bool(has))
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Obj:copy".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = <IndexMap<String, Value>>::try_from(args.next().unwrap_or_default())
                .map(Value::obj);
            async { result }.boxed()
        }),
    );

    std.insert(
        "Obj:merge".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = <IndexMap<String, Value>>::try_from(args.next().unwrap_or_default())
                .and_then(|mut a| {
                    let b = <IndexMap<String, Value>>::try_from(args.next().unwrap_or_default())?;
                    a.extend(b);
                    Ok(Value::obj(a))
                });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Error:create".to_string(),
        Value::fn_native(|args, _| {
            let mut args = args.into_iter();
            let result = String::try_from(args.next().unwrap_or_default()).map(|name| {
                let info = args.next();
                Value::error(name, info)
            });
            async { result }.boxed()
        }),
    );

    std.insert(
        "Async:interval".to_string(),
        Value::fn_native(|args, interpreter| {
            let mut args = args.into_iter();
            let interval = match f64::try_from(args.next().unwrap_or_default()) {
                Ok(interval) => Duration::from_millis(interval as u64),
                Err(e) => return async { Err(e) }.boxed(),
            };
            let callback = match VFn::try_from(args.next().unwrap_or_default()) {
                Ok(callback) => callback,
                Err(e) => return async { Err(e) }.boxed(),
            };
            let immediate = match args
                .next()
                .map(bool::try_from)
                .map_or(Ok(None), |r| r.map(Some))
            {
                Ok(immediate) => immediate.unwrap_or(false),
                Err(e) => return async { Err(e) }.boxed(),
            };
            let interpreter = interpreter.clone();
            async move {
                let abort_handler = interpreter
                    .register_abort_handler({
                        let interpreter = interpreter.clone();
                        async move {
                            let mut interval = tokio::time::interval(interval);
                            if !immediate {
                                interval.tick().await;
                            }
                            loop {
                                interval.tick().await;
                                let interpreter = interpreter.clone();
                                let callback = callback.clone();
                                tokio::spawn(
                                    async move { interpreter.exec_fn(callback, []).await },
                                )
                                .await
                                .map_err(AiScriptError::internal)??;
                            }
                        }
                        .boxed()
                    })
                    .await;
                Ok(Value::fn_native(move |_, _| {
                    abort_handler.abort();
                    async { Ok(Value::null()) }.boxed()
                }))
            }
            .boxed()
        }),
    );

    std.insert(
        "Async:timeout".to_string(),
        Value::fn_native(|args, interpreter| {
            let mut args = args.into_iter();
            let delay = match f64::try_from(args.next().unwrap_or_default()) {
                Ok(delay) => Duration::from_millis(delay as u64),
                Err(e) => return async { Err(e) }.boxed(),
            };
            let callback = match VFn::try_from(args.next().unwrap_or_default()) {
                Ok(callback) => callback,
                Err(e) => return async { Err(e) }.boxed(),
            };
            let interpreter = interpreter.clone();
            async move {
                let abort_handler = interpreter
                    .register_abort_handler({
                        let interpreter = interpreter.clone();
                        let callback = callback.clone();
                        async move {
                            tokio::time::sleep(delay).await;
                            tokio::spawn(async move { interpreter.exec_fn(callback, []).await })
                                .await
                                .map_err(AiScriptError::internal)??;
                            Ok(())
                        }
                        .boxed()
                    })
                    .await;
                Ok(Value::fn_native(move |_, _| {
                    abort_handler.abort();
                    async { Ok(Value::null()) }.boxed()
                }))
            }
            .boxed()
        }),
    );

    std
}
