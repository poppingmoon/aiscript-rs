mod testutils;

use aiscript_v1::{
    errors::{AiScriptError, AiScriptRuntimeError},
    values::Value,
};
use testutils::*;

mod core {
    use super::*;

    #[tokio::test]
    async fn range() {
        test("<: Core:range(1, 10)", |res| {
            assert_eq!(
                res,
                arr([
                    num(1),
                    num(2),
                    num(3),
                    num(4),
                    num(5),
                    num(6),
                    num(7),
                    num(8),
                    num(9),
                    num(10),
                ]),
            )
        })
        .await
        .unwrap();

        test("<: Core:range(1, 1)", |res| assert_eq!(res, arr([num(1)])))
            .await
            .unwrap();

        test("<: Core:range(9, 7)", |res| {
            assert_eq!(res, arr([num(9), num(8), num(7)]))
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_str() {
        test(r#"<: Core:to_str("abc")"#, |res| {
            assert_eq!(res, str("abc"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str(123)"#, |res| assert_eq!(res, str("123")))
            .await
            .unwrap();

        test(r#"<: Core:to_str(true)"#, |res| {
            assert_eq!(res, str("true"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str(false)"#, |res| {
            assert_eq!(res, str("false"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str(null)"#, |res| {
            assert_eq!(res, str("null"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str({ a: "abc", b: 1234 })"#, |res| {
            assert_eq!(res, str(r#"{ a: "abc", b: 1234 }"#))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str([ true, 123, null ])"#, |res| {
            assert_eq!(res, str("[ true, 123, null ]"))
        })
        .await
        .unwrap();

        test(r#"<: Core:to_str(@( a, b, c ) {})"#, |res| {
            assert_eq!(res, str("@( a, b, c ) { ... }"))
        })
        .await
        .unwrap();

        test(
            r#"
            let arr = []
            arr.push(arr)
            <: Core:to_str(arr)
            "#,
            |res| assert_eq!(res, str("[ ... ]")),
        )
        .await
        .unwrap();

        test(
            r#"
            let arr = []
            arr.push({ value: arr })
            <: Core:to_str(arr)
            "#,
            |res| assert_eq!(res, str("[ { value: ... } ]")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn abort() {
        let err = test(r#"Core:abort("hoge")"#, |_| {}).await.unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::User(message)) = err {
            assert_eq!(message, "hoge");
        } else {
            panic!("{err}");
        }
    }
}

mod arr {
    use super::*;

    #[tokio::test]
    async fn create() {
        test("<: Arr:create(0)", |res| assert_eq!(res, arr([])))
            .await
            .unwrap();

        test("<: Arr:create(3)", |res| {
            assert_eq!(res, arr([null(), null(), null()]))
        })
        .await
        .unwrap();

        test("<: Arr:create(3, 1)", |res| {
            assert_eq!(res, arr([num(1), num(1), num(1)]))
        })
        .await
        .unwrap();
    }
}

mod math {
    use super::*;

    #[tokio::test]
    async fn trig() {
        test("<: Math:sin(Math:PI / 2)", |res| assert_eq!(res, num(1)))
            .await
            .unwrap();

        test("<: Math:sin(0 - (Math:PI / 2))", |res| {
            assert_eq!(res, num(-1))
        })
        .await
        .unwrap();

        test("<: Math:sin(Math:PI / 4) * Math:cos(Math:PI / 4)", |res| {
            assert!((f64::try_from(res).unwrap() - 0.5).abs() <= f64::EPSILON)
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn abs() {
        test("<: Math:abs(1 - 6)", |res| assert_eq!(res, num(5)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn pow_and_sqrt() {
        test("<: Math:sqrt(3^2 + 4^2)", |res| assert_eq!(res, num(5)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn round() {
        test("<: Math:round(3.14)", |res| assert_eq!(res, num(3)))
            .await
            .unwrap();

        test("<: Math:round(-1.414213)", |res| assert_eq!(res, num(-1)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn ceil() {
        test("<: Math:ceil(2.71828)", |res| assert_eq!(res, num(3)))
            .await
            .unwrap();

        test("<: Math:ceil(0 - Math:PI)", |res| assert_eq!(res, num(-3)))
            .await
            .unwrap();

        test("<: Math:ceil(1 / Math:Infinity)", |res| {
            assert_eq!(res, num(0))
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn floor() {
        test("<: Math:floor(23.14069)", |res| assert_eq!(res, num(23)))
            .await
            .unwrap();

        test("<: Math:floor(Math:Infinity / 0)", |res| {
            assert_eq!(res, num(f64::INFINITY))
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn min() {
        test("<: Math:min(2, 3)", |res| assert_eq!(res, num(2)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn max() {
        test("<: Math:max(-2, -3)", |res| assert_eq!(res, num(-2)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn rnd_with_arg() {
        test("<: Math:rnd(1, 1.5)", |res| assert_eq!(res, num(1)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn gen_rng() {
        // 2つのシード値から1~maxの乱数をn回生成して一致率を見る
        test(
            r#"
            @test(seed1, seed2) {
                let n = 100
                let max = 100000
                let threshold = 0.05
                let random1 = Math:gen_rng(seed1)
                let random2 = Math:gen_rng(seed2)
                var same = 0
                for n {
                    if random1(1, max) == random2(1, max) {
                        same += 1
                    }
                }
                let rate = same / n
                if seed1 == seed2 { rate == 1 }
                else { rate < threshold }
            }
            let seed1 = `{Util:uuid()}`
            let seed2 = `{Date:year()}`
            <: [
                test(seed1, seed1)
                test(seed1, seed2)
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), bool(true)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn gen_rng_should_reject_when_null_is_provided_as_a_seed() {
        let err = test("Math:gen_rng(null)", |_| {}).await.unwrap_err();
        let AiScriptError::Runtime(AiScriptRuntimeError::InvalidSeed) = err else {
            panic!("{err}");
        };
    }
}

mod obj {
    use super::*;

    #[tokio::test]
    async fn keys() {
        test(
            r#"
            let o = { a: 1, b: 2, c: 3, }

            <: Obj:keys(o)
            "#,
            |res| assert_eq!(res, arr([str("a"), str("b"), str("c")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn vals() {
        test(
            r#"
            let o = { _nul: null, _num: 24, _str: 'hoge', _arr: [], _obj: {}, }

            <: Obj:vals(o)
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        null(),
                        num(24),
                        str("hoge"),
                        arr([]),
                        obj([] as [(String, Value); 0]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn kvs() {
        test(
            r#"
            let o = { a: 1, b: 2, c: 3, }

            <: Obj:kvs(o)
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([str("a"), num(1)]),
                        arr([str("b"), num(2)]),
                        arr([str("c"), num(3)]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn merge() {
        test(
            r#"
            let o1 = { a: 1, b: 2 }
            let o2 = { b: 3, c: 4 }

            <: Obj:merge(o1, o2)
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(3)), ("c", num(4))])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn pick() {
        test(
            r#"
            let o = { a: 1, b: 2, c: 3 }

            <: Obj:pick(o, ['b', 'd'])
            "#,
            |res| assert_eq!(res, obj([("b", num(2)), ("d", null())])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn from_kvs() {
        test(
            r#"
            let kvs = [['a', 1], ['b', 2], ['c', 3]]

            <: Obj:from_kvs(kvs)
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(2)), ("c", num(3))])),
        )
        .await
        .unwrap();
    }
}

mod str {
    use super::*;

    #[tokio::test]
    async fn lf() {
        test(
            r#"
            <: Str:lf
            "#,
            |res| assert_eq!(res, str("\n")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn from_codepoint() {
        test(
            r#"
            <: Str:from_codepoint(65)
            "#,
            |res| assert_eq!(res, str("A")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn from_unicode_codepoints() {
        test(
            r#"
            <: Str:from_unicode_codepoints([171581, 128073, 127999, 128104, 8205, 128102])
            "#,
            |res| assert_eq!(res, str("𩸽👉🏿👨‍👦")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn from_utf8_bytes() {
        test(
            r#"
            <: Str:from_utf8_bytes([240, 169, 184, 189, 240, 159, 145, 137, 240, 159, 143, 191, 240, 159, 145, 168, 226, 128, 141, 240, 159, 145, 166])
            "#,
            |res| assert_eq!(res, str("𩸽👉🏿👨‍👦")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn charcode_at() {
        test(
            r#"
            <: "aiscript".split().map(@(x, _) { x.charcode_at(0) })
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(97),
                        num(105),
                        num(115),
                        num(99),
                        num(114),
                        num(105),
                        num(112),
                        num(116),
                    ]),
                )
            },
        )
        .await
        .unwrap();

        test(
            r#"
            <: "".charcode_at(0)
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
    }
}

mod uri {
    use super::*;

    #[tokio::test]
    async fn encode_full() {
        test(
            r#"
            <: Uri:encode_full("https://example.com/?q=あいちゃん")
            "#,
            |res| {
                assert_eq!(
                    res,
                    str("https://example.com/?q=%E3%81%82%E3%81%84%E3%81%A1%E3%82%83%E3%82%93"),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn encode_component() {
        test(
            r#"
            <: Uri:encode_component("https://example.com/?q=あいちゃん")
            "#,
            |res| assert_eq!(res, str("https%3A%2F%2Fexample.com%2F%3Fq%3D%E3%81%82%E3%81%84%E3%81%A1%E3%82%83%E3%82%93")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn decode_full() {
        test(
            r#"
            <: Uri:decode_full("https%3A%2F%2Fexample.com%2F%3Fq%3D%E3%81%82%E3%81%84%E3%81%A1%E3%82%83%E3%82%93")
            "#,
            |res| assert_eq!(res, str("https%3A%2F%2Fexample.com%2F%3Fq%3Dあいちゃん")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn decode_component() {
        test(
            r#"
            <: Uri:decode_component("https%3A%2F%2Fexample.com%2F%3Fq%3D%E3%81%82%E3%81%84%E3%81%A1%E3%82%83%E3%82%93")
            "#,
            |res| assert_eq!(res, str("https://example.com/?q=あいちゃん")),
        )
        .await
        .unwrap();
    }
}

mod error {
    use super::*;

    #[tokio::test]
    async fn create() {
        test(
            r#"
            <: Error:create('ai', {chan: 'kawaii'})
            "#,
            |res| assert_eq!(res, error("ai", Some(obj([("chan", str("kawaii"))])))),
        )
        .await
        .unwrap();
    }
}

mod json {
    use super::*;

    #[tokio::test]
    async fn stringify_fn() {
        test(
            r#"
            <: Json:stringify(@(){})
            "#,
            |res| assert_eq!(res, str(r#""<function>""#)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn parsable() {
        test(
            r#"
            <: [
                Json:parsable('null')
                Json:stringify(Json:parse('null'))
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), str("null")])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('"hoge"')
                Json:stringify(Json:parse('"hoge"'))
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), str(r#""hoge""#)])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('[]')
                Json:stringify(Json:parse('[]'))
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), str("[]")])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('{}')
                Json:stringify(Json:parse('{}'))
            ]
            "#,
            |res| assert_eq!(res, arr([bool(true), str("{}")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn unparsable() {
        test(
            r#"
            <: [
                Json:parsable('')
                Json:parse('')
            ]
            "#,
            |res| assert_eq!(res, arr([bool(false), error("not_json", None)])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('hoge')
                Json:parse('hoge')
            ]
            "#,
            |res| assert_eq!(res, arr([bool(false), error("not_json", None)])),
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                Json:parsable('[')
                Json:parse('[')
            ]
            "#,
            |res| assert_eq!(res, arr([bool(false), error("not_json", None)])),
        )
        .await
        .unwrap();
    }
}

mod date {
    use chrono::{Datelike, Local, NaiveDate, TimeZone, Timelike};

    use super::*;

    #[tokio::test]
    async fn year() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:year(0), Date:year({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(Local.timestamp_millis_opt(0).unwrap().year()),
                        num(2024),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn month() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:month(0), Date:month({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().month()), num(1)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn day() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:day(0), Date:day({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().day()), num(2)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn hour() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:hour(0), Date:hour({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().hour()), num(3)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn minute() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:minute(0), Date:minute({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().minute()), num(4)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn second() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:second(0), Date:second({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([num(Local.timestamp_millis_opt(0).unwrap().second()), num(5)]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn millisecond() {
        let example_time = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_milli_opt(3, 4, 5, 6)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        test(
            &format!(
                "
                <: [Date:millisecond(0), Date:millisecond({example_time})]
                "
            ),
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(
                            (Local.timestamp_millis_opt(0).unwrap().timestamp_millis() % 1000)
                                as f64
                        ),
                        num(6),
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_iso_str() {
        test(
            r#"
            let d1 = Date:parse("2024-04-12T01:47:46.021+09:00")
            let s1 = Date:to_iso_str(d1)
            let d2 = Date:parse(s1)
            <: [d1, d2, s1]
            "#,
            |res| {
                let res = <Vec<Value>>::try_from(res).unwrap();
                assert_eq!(res[0], res[1]);
                let s1 = String::try_from(res[2].clone()).unwrap();
                regex::Regex::new(
                    r"(?x)
                    ^[0-9]{4,4}-[0-9]{2,2}-[0-9]{2,2}T
                    [0-9]{2,2}:[0-9]{2,2}:[0-9]{2,2}\.[0-9]{3,3}
                    (Z|[-+][0-9]{2,2}:[0-9]{2,2})$
                    ",
                )
                .unwrap()
                .captures(&s1)
                .unwrap();
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_iso_str_utc() {
        test(
            r#"
            let d1 = Date:parse("2024-04-12T01:47:46.021+09:00")
            let s1 = Date:to_iso_str(d1, 0)
            let d2 = Date:parse(s1)
            <: [d1, d2, s1]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(1712854066021.0),
                        num(1712854066021.0),
                        str("2024-04-11T16:47:46.021Z")
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_iso_str_09_00() {
        test(
            r#"
            let d1 = Date:parse("2024-04-12T01:47:46.021+09:00")
            let s1 = Date:to_iso_str(d1, 9*60)
            let d2 = Date:parse(s1)
            <: [d1, d2, s1]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(1712854066021.0),
                        num(1712854066021.0),
                        str("2024-04-12T01:47:46.021+09:00")
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_iso_str_05_18() {
        test(
            r#"
            let d1 = Date:parse("2024-04-12T01:47:46.021+09:00")
            let s1 = Date:to_iso_str(d1, -5*60-18)
            let d2 = Date:parse(s1)
            <: [d1, d2, s1]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(1712854066021.0),
                        num(1712854066021.0),
                        str("2024-04-11T11:29:46.021-05:18"),
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn parse() {
        test(
            r#"
            <: [
				'01 Jan 1970 00:00:00 GMT'
				'1970-01-01'
				'1970-01-01T00:00:00.000Z'
				'1970-01-01T00:00:00.000+00:00'
				'hoge'
			].map(Date:parse)
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([num(0), num(0), num(0), num(0), error("not_date", None)]),
                )
            },
        )
        .await
        .unwrap();
    }
}
