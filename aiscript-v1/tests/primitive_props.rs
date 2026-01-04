mod testutils;

use aiscript_v1::{
    errors::{AiScriptError, AiScriptRuntimeError},
    values::Value,
};
use testutils::*;

mod num {
    use super::*;

    #[tokio::test]
    async fn to_str() {
        test(
            r#"
            let num = 123
            <: num.to_str()
            "#,
            |res| assert_eq!(res, str("123")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_hex() {
        test(
            r#"
            <: [
                0, 10, 16,
                -10, -16,
                0.5,
            ].map(@(v){v.to_hex()})
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        str("0"),
                        str("a"),
                        str("10"),
                        str("-a"),
                        str("-10"),
                        str("0.8"),
                    ]),
                )
            },
        )
        .await
        .unwrap();

        test(
            r#"
            <: [
                0.1,
                -0.1,
                1000000.1,
                -1000000.1,
                0.00000000001,
                -0.00000000001,
                1000000.00000000001
                -1000000.00000000001
                0.0000152587890625,
                -0.0000152587890625,
                200.0000152587890625,
                -200.0000152587890625,
            ].map(@(v){v.to_hex()})
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        str("0.1999999999999a"),
                        str("-0.1999999999999a"),
                        str("f4240.199999998"),
                        str("-f4240.199999998"),
                        str("0.000000000afebff0bcb24a8"),
                        str("-0.000000000afebff0bcb24a8"),
                        str("f4240"),
                        str("-f4240"),
                        str("0.0001"),
                        str("-0.0001"),
                        str("c8.0001"),
                        str("-c8.0001"),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }
}

mod str {
    use super::*;

    #[tokio::test]
    async fn len() {
        test(
            r#"
            let str = "hello"
            <: str.len
            "#,
            |res| assert_eq!(res, num(5)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_num() {
        test(
            r#"
            let str = "123"
            <: str.to_num()
            "#,
            |res| assert_eq!(res, num(123)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn upper() {
        test(
            r#"
            let str = "hello"
            <: str.upper()
            "#,
            |res| assert_eq!(res, str("HELLO")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn lower() {
        test(
            r#"
            let str = "HELLO"
            <: str.lower()
            "#,
            |res| assert_eq!(res, str("hello")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn trim() {
        test(
            r#"
            let str = " hello  "
            <: str.trim()
            "#,
            |res| assert_eq!(res, str("hello")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn replace() {
        test(
            r#"
            let str = "hello"
            <: str.replace("l", "x")
            "#,
            |res| assert_eq!(res, str("hexxo")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn index_of() {
        test(
            r#"
            let str = '0123401234'
            <: [
                str.index_of('3') == 3,
                str.index_of('5') == -1,
                str.index_of('3', 3) == 3,
                str.index_of('3', 4) == 8,
                str.index_of('3', -1) == -1,
                str.index_of('3', -2) == 8,
                str.index_of('3', -7) == 3,
                str.index_of('3', 10) == -1,
            ].map(@(v){if (v) '1' else '0'}).join()
            "#,
            |res| assert_eq!(res, str("11111111")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn incl() {
        test(
            r#"
            let str = "hello"
            <: [str.incl("ll"), str.incl("x")]
            "#,
            |res| assert_eq!(res, arr([bool(true), bool(false)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn split() {
        test(
            r#"
            let str = "a,b,c"
            <: str.split(",")
            "#,
            |res| assert_eq!(res, arr([str("a"), str("b"), str("c")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn pick() {
        test(
            r#"
            let str = "hello"
            <: str.pick(1)
            "#,
            |res| assert_eq!(res, str("e")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn slice() {
        test(
            r#"
            let str = "hello"
            <: str.slice(1, 3)
            "#,
            |res| assert_eq!(res, str("el")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn slice_out_of_range() {
        test(
            r#"
            let str = "hello"
            <: str.slice(3, 1)
            "#,
            |res| assert_eq!(res, str("")),
        )
        .await
        .unwrap();

        test(
            r#"
            let str = "hello"
            <: str.slice(-1, 3)
            "#,
            |res| assert_eq!(res, str("hel")),
        )
        .await
        .unwrap();

        test(
            r#"
            let str = "hello"
            <: str.slice(3, -1)
            "#,
            |res| assert_eq!(res, str("")),
        )
        .await
        .unwrap();

        test(
            r#"
            let str = "hello"
            <: str.slice(-1, -3)
            "#,
            |res| assert_eq!(res, str("")),
        )
        .await
        .unwrap();

        test(
            r#"
            let str = "hello"
            <: str.slice(-3, -1)
            "#,
            |res| assert_eq!(res, str("")),
        )
        .await
        .unwrap();

        test(
            r#"
            let str = "hello"
            <: str.slice(11, 13)
            "#,
            |res| assert_eq!(res, str("")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn codepoint_at() {
        test(
            r#"
            let str = "𩸽"
            <: str.codepoint_at(0)
            "#,
            |res| assert_eq!(res, num(171581)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_arr() {
        test(
            r#"
            let str = "𩸽👉🏿👨‍👦"
            <: str.to_arr()
            "#,
            |res| assert_eq!(res, arr([str("𩸽"), str("👉🏿"), str("👨‍👦")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_unicode_arr() {
        test(
            r#"
            let str = "𩸽👉🏿👨‍👦"
            <: str.to_unicode_arr()
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        str("𩸽"),
                        str("👉"),
                        str("\u{1F3FF}"),
                        str("👨"),
                        str("\u{200d}"),
                        str("👦"),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_unicode_codepoint_arr() {
        test(
            r#"
            let str = "𩸽👉🏿👨‍👦"
            <: str.to_unicode_codepoint_arr()
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(171581),
                        num(128073),
                        num(127999),
                        num(128104),
                        num(8205),
                        num(128102),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_char_arr() {
        test(
            r#"
            let str = "abc𩸽👉🏿👨‍👦def"
            <: str.to_char_arr()
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        97, 98, 99, 55399, 56893, 55357, 56393, 55356, 57343, 55357, 56424, 8205,
                        55357, 56422, 100, 101, 102
                    ]
                    .into_iter()
                    .map(|u| str(String::from_utf16_lossy(&[u])))
                    .collect::<Vec<Value>>()),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_charcode_arr() {
        test(
            r#"
            let str = "abc𩸽👉🏿👨‍👦def"
            <: str.to_charcode_arr()
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(97),
                        num(98),
                        num(99),
                        num(55399),
                        num(56893),
                        num(55357),
                        num(56393),
                        num(55356),
                        num(57343),
                        num(55357),
                        num(56424),
                        num(8205),
                        num(55357),
                        num(56422),
                        num(100),
                        num(101),
                        num(102),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn to_utf8_byte_arr() {
        test(
            r#"
            let str = "abc𩸽👉🏿👨‍👦def"
            <: str.to_utf8_byte_arr()
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(97),
                        num(98),
                        num(99),
                        num(240),
                        num(169),
                        num(184),
                        num(189),
                        num(240),
                        num(159),
                        num(145),
                        num(137),
                        num(240),
                        num(159),
                        num(143),
                        num(191),
                        num(240),
                        num(159),
                        num(145),
                        num(168),
                        num(226),
                        num(128),
                        num(141),
                        num(240),
                        num(159),
                        num(145),
                        num(166),
                        num(100),
                        num(101),
                        num(102),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn starts_with_no_index() {
        test(
            r#"
            let str = "hello"
            let empty = ""
            <: [
                str.starts_with(""), str.starts_with("hello"),
                str.starts_with("he"), str.starts_with("ell"),
                empty.starts_with(""), empty.starts_with("he"),
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(false),
                        bool(true),
                        bool(false),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn starts_with_with_index() {
        test(
            r#"
            let str = "hello"
            let empty = ""
            <: [
                str.starts_with("", 4), str.starts_with("he", 0),
                str.starts_with("ll", 2), str.starts_with("lo", 3),
                str.starts_with("lo", -2), str.starts_with("hel", -5),
                str.starts_with("he", 2), str.starts_with("loa", 3),
                str.starts_with("lo", -6), str.starts_with("", -7),
                str.starts_with("lo", 6), str.starts_with("", 7),
                empty.starts_with("", 2), empty.starts_with("ll", 2),
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(false),
                        bool(false),
                        bool(false),
                        bool(true),
                        bool(false),
                        bool(true),
                        bool(true),
                        bool(false),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn ends_with_no_index() {
        test(
            r#"
            let str = "hello"
            let empty = ""
            <: [
                str.ends_with(""), str.ends_with("hello"),
                str.ends_with("lo"), str.ends_with("ell"),
                empty.ends_with(""), empty.ends_with("he"),
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(false),
                        bool(true),
                        bool(false),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn ends_with_with_index() {
        test(
            r#"
            let str = "hello"
            let empty = ""
            <: [
                str.ends_with("", 3), str.ends_with("lo", 5),
                str.ends_with("ll", 4), str.ends_with("he", 2),
                str.ends_with("ll", -1), str.ends_with("he", -3),
                str.ends_with("he", 5), str.ends_with("lo", 3),
                str.ends_with("lo", -6), str.ends_with("", -7),
                str.ends_with("lo", 6), str.ends_with("", 7),
                empty.ends_with("", 2), empty.ends_with("ll", 2),
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(true),
                        bool(false),
                        bool(false),
                        bool(false),
                        bool(true),
                        bool(false),
                        bool(true),
                        bool(true),
                        bool(false),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn pad_start() {
        test(
            r#"
            let str = "abc"
            <: [
                str.pad_start(0), str.pad_start(1), str.pad_start(2),
                str.pad_start(3), str.pad_start(4), str.pad_start(5),
                str.pad_start(0, "0"), str.pad_start(1, "0"), str.pad_start(2, "0"),
                str.pad_start(3, "0"), str.pad_start(4, "0"), str.pad_start(5, "0"),
                str.pad_start(0, "01"), str.pad_start(1, "01"), str.pad_start(2, "01"),
                str.pad_start(3, "01"), str.pad_start(4, "01"), str.pad_start(5, "01"),
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str(" abc"),
                        str("  abc"),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("0abc"),
                        str("00abc"),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("0abc"),
                        str("01abc"),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn pad_end() {
        test(
            r#"
            let str = "abc"
            <: [
                str.pad_end(0), str.pad_end(1), str.pad_end(2),
                str.pad_end(3), str.pad_end(4), str.pad_end(5),
                str.pad_end(0, "0"), str.pad_end(1, "0"), str.pad_end(2, "0"),
                str.pad_end(3, "0"), str.pad_end(4, "0"), str.pad_end(5, "0"),
                str.pad_end(0, "01"), str.pad_end(1, "01"), str.pad_end(2, "01"),
                str.pad_end(3, "01"), str.pad_end(4, "01"), str.pad_end(5, "01"),
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("abc "),
                        str("abc  "),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("abc0"),
                        str("abc00"),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("abc"),
                        str("abc0"),
                        str("abc01"),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }
}

mod arr {
    use super::*;

    #[tokio::test]
    async fn len() {
        test(
            r#"
            let arr = [1, 2, 3]
            <: arr.len
            "#,
            |res| assert_eq!(res, num(3)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn push() {
        test(
            r#"
            let arr = [1, 2, 3]
            arr.push(4)
            <: arr
            "#,
            |res| assert_eq!(res, arr([num(1), num(2), num(3), num(4)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn unshift() {
        test(
            r#"
            let arr = [1, 2, 3]
            arr.unshift(4)
            <: arr
            "#,
            |res| assert_eq!(res, arr([num(4), num(1), num(2), num(3)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn pop() {
        test(
            r#"
            let arr = [1, 2, 3]
            let popped = arr.pop()
            <: [popped, arr]
            "#,
            |res| assert_eq!(res, arr([num(3), arr([num(1), num(2)])])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn shift() {
        test(
            r#"
            let arr = [1, 2, 3]
            let shifted = arr.shift()
            <: [shifted, arr]
            "#,
            |res| assert_eq!(res, arr([num(1), arr([num(2), num(3)])])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn concat() {
        test(
            r#"
            let arr = [1, 2, 3]
            let concated = arr.concat([4, 5])
            <: [concated, arr]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(1), num(2), num(3), num(4), num(5)]),
                        arr([num(1), num(2), num(3)]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn slice() {
        test(
            r#"
            let arr = ["ant", "bison", "camel", "duck", "elephant"]
            let sliced = arr.slice(2, 4)
            <: [sliced, arr]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([str("camel"), str("duck")]),
                        arr([
                            str("ant"),
                            str("bison"),
                            str("camel"),
                            str("duck"),
                            str("elephant"),
                        ]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn slice_out_of_range() {
        test(
            r#"
            let arr = ["ant", "bison", "camel", "duck", "elephant"]
            <: arr.slice(4, 2)
            "#,
            |res| assert_eq!(res, arr([])),
        )
        .await
        .unwrap();

        test(
            r#"
            let arr = ["ant", "bison", "camel", "duck", "elephant"]
            <: arr.slice(-2, 4)
            "#,
            |res| assert_eq!(res, arr([str("duck")])),
        )
        .await
        .unwrap();

        test(
            r#"
            let arr = ["ant", "bison", "camel", "duck", "elephant"]
            <: arr.slice(4, -2)
            "#,
            |res| assert_eq!(res, arr([])),
        )
        .await
        .unwrap();

        test(
            r#"
            let arr = ["ant", "bison", "camel", "duck", "elephant"]
            <: arr.slice(-2, -4)
            "#,
            |res| assert_eq!(res, arr([])),
        )
        .await
        .unwrap();

        test(
            r#"
            let arr = ["ant", "bison", "camel", "duck", "elephant"]
            <: arr.slice(-4, -2)
            "#,
            |res| assert_eq!(res, arr([str("bison"), str("camel")])),
        )
        .await
        .unwrap();

        test(
            r#"
            let arr = ["ant", "bison", "camel", "duck", "elephant"]
            <: arr.slice(12, 14)
            "#,
            |res| assert_eq!(res, arr([])),
        )
        .await
        .unwrap();

        test(
            r#"
            let arr = ["ant", "bison", "camel", "duck", "elephant"]
            <: arr.slice(-14, -12)
            "#,
            |res| assert_eq!(res, arr([])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn join() {
        test(
            r#"
            let arr = ["a", "b", "c"]
            <: arr.join("-")
            "#,
            |res| assert_eq!(res, str("a-b-c")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn map() {
        test(
            r#"
            let arr = [1, 2, 3]
            <: arr.map(@(item) { item * 2 })
            "#,
            |res| assert_eq!(res, arr([num(2), num(4), num(6)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn map_with_index() {
        test(
            r#"
            let arr = [1, 2, 3]
            <: arr.map(@(item, index) { item * index })
            "#,
            |res| assert_eq!(res, arr([num(0), num(2), num(6)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn filter() {
        test(
            r#"
            let arr = [1, 2, 3]
            <: arr.filter(@(item) { item != 2 })
            "#,
            |res| assert_eq!(res, arr([num(1), num(3)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn filter_with_index() {
        test(
            r#"
            let arr = [1, 2, 3, 4]
            <: arr.filter(@(item, index) { item != 2 && index != 3 })
            "#,
            |res| assert_eq!(res, arr([num(1), num(3)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn reduce() {
        test(
            r#"
            let arr = [1, 2, 3, 4]
            <: arr.reduce(@(accumulator, currentValue) { (accumulator + currentValue) })
            "#,
            |res| assert_eq!(res, num(10)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn reduce_with_index() {
        test(
            r#"
            let arr = [1, 2, 3, 4]
            <: arr.reduce(@(accumulator, currentValue, index) { (accumulator + (currentValue * index)) }, 0)
            "#,
            |res| assert_eq!(res, num(20)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn reduce_of_empty_array_without_initial_value() {
        let err = test(
            r#"
            let arr = [1, 2, 3, 4]
            <: [].reduce(@(){})
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Runtime(AiScriptRuntimeError::ReduceWithoutInitialValue) = err else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn find() {
        test(
            r#"
            let arr = ["abc", "def", "ghi"]
            <: arr.find(@(item) { item.incl("e") })
            "#,
            |res| assert_eq!(res, str("def")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn find_with_index() {
        test(
            r#"
            let arr = ["abc1", "def1", "ghi1", "abc2", "def2", "ghi2"]
            <: arr.find(@(item, index) { item.incl("e") && index > 1 })
            "#,
            |res| assert_eq!(res, str("def2")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn incl() {
        test(
            r#"
            let arr = ["abc", "def", "ghi"]
            <: [arr.incl("def"), arr.incl("jkl")]
            "#,
            |res| assert_eq!(res, arr([bool(true), bool(false)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn index_of() {
        test(
            r#"
            let arr = [0,1,2,3,4,0,1,2,3,4]
            <: [
                arr.index_of(3) == 3,
                arr.index_of(5) == -1,
                arr.index_of(3, 3) == 3,
                arr.index_of(3, 4) == 8,
                arr.index_of(3, -1) == -1,
                arr.index_of(3, -2) == 8,
                arr.index_of(3, -7) == 3,
                arr.index_of(3, 10) == -1,
            ].map(@(v){if (v) '1' else '0'}).join()
            "#,
            |res| assert_eq!(res, str("11111111")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn reverse() {
        test(
            r#"
            let arr = [1, 2, 3]
            arr.reverse()
            <: arr
            "#,
            |res| assert_eq!(res, arr([num(3), num(2), num(1)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn copy() {
        test(
            r#"
            let arr = [1, 2, 3]
            let copied = arr.copy()
            copied.reverse()
            <: [copied, arr]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([arr([num(3), num(2), num(1)]), arr([num(1), num(2), num(3)])]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sort_num_array() {
        test(
            r#"
            var arr = [2, 10, 3]
            let comp = @(a, b) { a - b }
            arr.sort(comp)
            <: arr
            "#,
            |res| assert_eq!(res, arr([num(2), num(3), num(10)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sort_string_array_with_str_lt() {
        test(
            r#"
            var arr = ["hoge", "huga", "piyo", "hoge"]
            arr.sort(Str:lt)
            <: arr
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([str("hoge"), str("hoge"), str("huga"), str("piyo")]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sort_string_array_with_str_gt() {
        test(
            r#"
            var arr = ["hoge", "huga", "piyo", "hoge"]
            arr.sort(Str:gt)
            <: arr
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([str("piyo"), str("huga"), str("hoge"), str("hoge")]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sort_object_array() {
        test(
            r#"
            var arr = [{x: 2}, {x: 10}, {x: 3}]
            let comp = @(a, b) { a.x - b.x }

            arr.sort(comp)
            <: arr
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        obj([("x", num(2))]),
                        obj([("x", num(3))]),
                        obj([("x", num(10))]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sort_stable() {
        test(
            r#"
            var arr = [[2, 0], [10, 1], [3, 2], [3, 3], [2, 4]]
			let comp = @(a, b) { a[0] - b[0] }

			arr.sort(comp)
			<: arr
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(2), num(0)]),
                        arr([num(2), num(4)]),
                        arr([num(3), num(2)]),
                        arr([num(3), num(3)]),
                        arr([num(10), num(1)]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn fill() {
        test(
            r#"
            var arr1 = [0, 1, 2]
            let arr2 = arr1.fill(3)
            let arr3 = [0, 1, 2].fill(3, 1)
            let arr4 = [0, 1, 2].fill(3, 1, 2)
            let arr5 = [0, 1, 2].fill(3, -2, -1)
            <: [arr1, arr2, arr3, arr4, arr5]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(3), num(3), num(3)]), //target changed
                        arr([num(3), num(3), num(3)]),
                        arr([num(0), num(3), num(3)]),
                        arr([num(0), num(3), num(2)]),
                        arr([num(0), num(3), num(2)]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn repeat() {
        test(
            r#"
            var arr1 = [0, 1, 2]
            let arr2 = arr1.repeat(3)
            let arr3 = arr1.repeat(0)
            <: [arr1, arr2, arr3]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(0), num(1), num(2)]), // target not changed
                        arr([
                            num(0),
                            num(1),
                            num(2),
                            num(0),
                            num(1),
                            num(2),
                            num(0),
                            num(1),
                            num(2),
                        ]),
                        arr([]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn splice_full() {
        test(
            r#"
            let arr1 = [0, 1, 2, 3]
            let arr2 = arr1.splice(1, 2, [10])
            <: [arr1, arr2]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([arr([num(0), num(10), num(3)]), arr([num(1), num(2)])]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn splice_negative_index() {
        test(
            r#"
            let arr1 = [0, 1, 2, 3]
            let arr2 = arr1.splice(-1, 0, [10, 20])
            <: [arr1, arr2]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(0), num(1), num(2), num(10), num(20), num(3)]),
                        arr([]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn splice_larger_index() {
        test(
            r#"
            let arr1 = [0, 1, 2, 3]
            let arr2 = arr1.splice(4, 100, [10, 20])
            <: [arr1, arr2]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(0), num(1), num(2), num(3), num(10), num(20)]),
                        arr([]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn splice_single_argument() {
        test(
            r#"
            let arr1 = [0, 1, 2, 3]
            let arr2 = arr1.splice(1)
            <: [arr1, arr2]
            "#,
            |res| assert_eq!(res, arr([arr([num(0)]), arr([num(1), num(2), num(3)])])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn flat() {
        test(
            r#"
            var arr1 = [0, [1], [2, 3], [4, [5, 6]]]
            let arr2 = arr1.flat()
            let arr3 = arr1.flat(2)
            <: [arr1, arr2, arr3]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([
                            num(0),
                            arr([num(1)]),
                            arr([num(2), num(3)]),
                            arr([num(4), arr([num(5), num(6)])])
                        ]), // target not changed
                        arr([
                            num(0),
                            num(1),
                            num(2),
                            num(3),
                            num(4),
                            arr([num(5), num(6)]),
                        ]),
                        arr([num(0), num(1), num(2), num(3), num(4), num(5), num(6)]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn flat_map() {
        test(
            r#"
            let arr1 = [0, 1, 2]
            let arr2 = ["a", "b"]
            let arr3 = arr1.flat_map(@(x){ arr2.map(@(y){ [x, y] }) })
            <: [arr1, arr3]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(0), num(1), num(2)]), // target not changed
                        arr([
                            arr([num(0), str("a")]),
                            arr([num(0), str("b")]),
                            arr([num(1), str("a")]),
                            arr([num(1), str("b")]),
                            arr([num(2), str("a")]),
                            arr([num(2), str("b")]),
                        ]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn every() {
        test(
            r#"
            let arr1 = [0, 1, 2, 3]
            let res1 = arr1.every(@(v,i){v==0 || i > 0})
            let res2 = arr1.every(@(v,i){v==0 && i > 0})
            let res3 = [].every(@(v,i){false})
            <: [arr1, res1, res2, res3]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(0), num(1), num(2), num(3)]), // target not changed
                        bool(true),
                        bool(false),
                        bool(true),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn some() {
        test(
            r#"
            let arr1 = [0, 1, 2, 3]
            let res1 = arr1.some(@(v,i){v%2==0 && i <= 2})
            let res2 = arr1.some(@(v,i){v%2==0 && i > 2})
            <: [arr1, res1, res2]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(0), num(1), num(2), num(3)]), // target not changed
                        bool(true),
                        bool(false),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn insert() {
        test(
            r#"
            let arr1 = [0, 1, 2]
            let res = []
            res.push(arr1.insert(3, 10)) // [0, 1, 2, 10]
            res.push(arr1.insert(2, 20)) // [0, 1, 20, 2, 10]
            res.push(arr1.insert(0, 30)) // [30, 0, 1, 20, 2, 10]
            res.push(arr1.insert(-1, 40)) // [30, 0, 1, 20, 2, 40, 10]
            res.push(arr1.insert(-4, 50)) // [30, 0, 1, 50, 20, 2, 40, 10]
            res.push(arr1.insert(100, 60)) // [30, 0, 1, 50, 20, 2, 40, 10, 60]
            res.push(arr1)
            <: res
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        null(),
                        null(),
                        null(),
                        null(),
                        null(),
                        null(),
                        arr([
                            num(30),
                            num(0),
                            num(1),
                            num(50),
                            num(20),
                            num(2),
                            num(40),
                            num(10),
                            num(60),
                        ]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn remove() {
        test(
            r#"
            let arr1 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
            let res = []
            res.push(arr1.remove(9)) // 9 [0, 1, 2, 3, 4, 5, 6, 7, 8]
            res.push(arr1.remove(3)) // 3 [0, 1, 2, 4, 5, 6, 7, 8]
            res.push(arr1.remove(0)) // 0 [1, 2, 4, 5, 6, 7, 8]
            res.push(arr1.remove(-1)) // 8 [1, 2, 4, 5, 6, 7]
            res.push(arr1.remove(-5)) // 2 [1, 4, 5, 6, 7]
            res.push(arr1.remove(100)) // null [1, 4, 5, 6, 7]
            res.push(arr1)
            <: res
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(9),
                        num(3),
                        num(0),
                        num(8),
                        num(2),
                        null(),
                        arr([num(1), num(4), num(5), num(6), num(7)]),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn at_without_default_value() {
        test(
            r#"
            let arr1 = [10, 20, 30]
            <: [
                arr1
                arr1.at(0), arr1.at(1), arr1.at(2)
                arr1.at(-3), arr1.at(-2), arr1.at(-1)
                arr1.at(3), arr1.at(4), arr1.at(5)
                arr1.at(-6), arr1.at(-5), arr1.at(-4)
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(10), num(20), num(30)]),
                        num(10),
                        num(20),
                        num(30),
                        num(10),
                        num(20),
                        num(30),
                        null(),
                        null(),
                        null(),
                        null(),
                        null(),
                        null(),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn at_with_default_value() {
        test(
            r#"
            let arr1 = [10, 20, 30]
            <: [
                arr1
                arr1.at(0, 100), arr1.at(1, 100), arr1.at(2, 100)
                arr1.at(-3, 100), arr1.at(-2, 100), arr1.at(-1, 100)
                arr1.at(3, 100), arr1.at(4, 100), arr1.at(5, 100)
                arr1.at(-6, 100), arr1.at(-5, 100), arr1.at(-4, 100)
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(10), num(20), num(30)]),
                        num(10),
                        num(20),
                        num(30),
                        num(10),
                        num(20),
                        num(30),
                        num(100),
                        num(100),
                        num(100),
                        num(100),
                        num(100),
                        num(100),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn at_fraction() {
        test(
            r#"
            let arr1 = [10, 20, 30]
            <: [
                arr1
                arr1.at(0.1), arr1.at(1.4), arr1.at(2.5)
                arr1.at(-3.1), arr1.at(-2.4), arr1.at(-1.5)
                arr1.at(3.1), arr1.at(4.4), arr1.at(5.5)
                arr1.at(-6.1), arr1.at(-5.4), arr1.at(-4.5)
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        arr([num(10), num(20), num(30)]),
                        num(10),
                        num(20),
                        num(30),
                        num(10),
                        num(20),
                        num(30),
                        null(),
                        null(),
                        null(),
                        null(),
                        null(),
                        null(),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }
}
