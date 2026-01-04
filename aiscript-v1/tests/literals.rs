mod testutils;

use aiscript_v1::errors::{AiScriptError, AiScriptSyntaxError, AiScriptSyntaxErrorKind};
use testutils::*;

mod literal {
    use super::*;

    #[tokio::test]
    async fn string_single_quote() {
        test(
            r#"
            <: 'foo'
            "#,
            |res| assert_eq!(res, str("foo")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn string_double_quote() {
        test(
            r#"
            <: "foo"
            "#,
            |res| assert_eq!(res, str("foo")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn escaped_double_quote() {
        test(r#"<: "ai saw a note \"bebeyo\".""#, |res| {
            assert_eq!(res, str(r#"ai saw a note "bebeyo"."#))
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn escaped_single_quote() {
        test(r#"<: 'ai saw a note \'bebeyo\'.'"#, |res| {
            assert_eq!(res, str("ai saw a note 'bebeyo'."))
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn bool_true() {
        test(
            r#"
            <: true
            "#,
            |res| assert_eq!(res, bool(true)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn bool_false() {
        test(
            r#"
            <: false
            "#,
            |res| assert_eq!(res, bool(false)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn number_int() {
        test(
            r#"
            <: 10
            "#,
            |res| assert_eq!(res, num(10)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn number_float() {
        test(
            r#"
            <: 0.5
            "#,
            |res| assert_eq!(res, num(0.5)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn arr_separated_by_comma() {
        test(
            r#"
            <: [1, 2, 3]
            "#,
            |res| assert_eq!(res, arr([num(1), num(2), num(3)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn arr_separated_by_comma_with_trailing_comma() {
        test(
            r#"
            <: [1, 2, 3,]
            "#,
            |res| assert_eq!(res, arr([num(1), num(2), num(3)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn arr_separated_by_line_break() {
        test(
            r#"
            <: [
                1
                2
                3
            ]
            "#,
            |res| assert_eq!(res, arr([num(1), num(2), num(3)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn arr_separated_by_line_break_and_comma() {
        test(
            r#"
            <: [
                1,
                2,
                3
            ]
            "#,
            |res| assert_eq!(res, arr([num(1), num(2), num(3)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn arr_separated_by_line_break_and_comma_with_trailing_comma() {
        test(
            r#"
            <: [
                1,
                2,
                3,
            ]
            "#,
            |res| assert_eq!(res, arr([num(1), num(2), num(3)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn obj_separated_by_comma() {
        test(
            r#"
            <: { a: 1, b: 2, c: 3 }
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(2)), ("c", num(3))])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn obj_separated_by_comma_with_trailing_comma() {
        test(
            r#"
            <: { a: 1, b: 2, c: 3, }
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(2)), ("c", num(3))])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn obj_separated_by_line_break() {
        test(
            r#"
            <: {
                a: 1
                b: 2
                c: 3
            }
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(2)), ("c", num(3))])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn obj_string_key() {
        test(
            r#"
            <: {
                "藍": 42,
            }
            "#,
            |res| assert_eq!(res, obj([("藍", num(42))])),
        )
        .await
        .unwrap();
    }

    mod obj_reserved_word_as_key {
        use super::*;

        #[tokio::test]
        async fn key_null() {
            test(
                r#"
                <: {
                    null: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("null", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_true() {
            test(
                r#"
                <: {
                    true: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("true", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_false() {
            test(
                r#"
                <: {
                    false: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("false", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_each() {
            test(
                r#"
                <: {
                    each: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("each", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_for() {
            test(
                r#"
                <: {
                    for: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("for", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_loop() {
            test(
                r#"
                <: {
                    loop: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("loop", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_do() {
            test(
                r#"
                <: {
                    do: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("do", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_break() {
            test(
                r#"
                <: {
                    break: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("break", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_continue() {
            test(
                r#"
                <: {
                    continue: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("continue", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_match() {
            test(
                r#"
                <: {
                    match: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("match", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_case() {
            test(
                r#"
                <: {
                    case: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("case", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_default() {
            test(
                r#"
                <: {
                    default: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("default", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_if() {
            test(
                r#"
                <: {
                    if: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("if", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_elif() {
            test(
                r#"
                <: {
                    elif: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("elif", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_else() {
            test(
                r#"
                <: {
                    else: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("else", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_return() {
            test(
                r#"
                <: {
                    return: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("return", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_eval() {
            test(
                r#"
                <: {
                    eval: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("eval", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_var() {
            test(
                r#"
                <: {
                    var: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("var", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_let() {
            test(
                r#"
                <: {
                    let: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("let", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_exists() {
            test(
                r#"
                <: {
                    exists: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("exists", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_as() {
            test(
                r#"
                <: {
                    as: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("as", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_async() {
            test(
                r#"
                <: {
                    async: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("async", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_attr() {
            test(
                r#"
                <: {
                    attr: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("attr", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_attribute() {
            test(
                r#"
                <: {
                    attribute: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("attribute", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_await() {
            test(
                r#"
                <: {
                    await: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("await", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_catch() {
            test(
                r#"
                <: {
                    catch: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("catch", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_class() {
            test(
                r#"
                <: {
                    class: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("class", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_component() {
            test(
                r#"
                <: {
                    component: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("component", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_constructor() {
            test(
                r#"
                <: {
                    constructor: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("constructor", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_dictionary() {
            test(
                r#"
                <: {
                    dictionary: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("dictionary", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_enum() {
            test(
                r#"
                <: {
                    enum: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("enum", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_export() {
            test(
                r#"
                <: {
                    export: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("export", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_finally() {
            test(
                r#"
                <: {
                    finally: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("finally", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_fn() {
            test(
                r#"
                <: {
                    fn: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("fn", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_hash() {
            test(
                r#"
                <: {
                    hash: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("hash", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_in() {
            test(
                r#"
                <: {
                    in: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("in", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_interface() {
            test(
                r#"
                <: {
                    interface: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("interface", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_out() {
            test(
                r#"
                <: {
                    out: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("out", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_private() {
            test(
                r#"
                <: {
                    private: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("private", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_public() {
            test(
                r#"
                <: {
                    public: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("public", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_ref() {
            test(
                r#"
                <: {
                    ref: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("ref", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_static() {
            test(
                r#"
                <: {
                    static: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("static", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_struct() {
            test(
                r#"
                <: {
                    struct: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("struct", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_table() {
            test(
                r#"
                <: {
                    table: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("table", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_this() {
            test(
                r#"
                <: {
                    this: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("this", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_throw() {
            test(
                r#"
                <: {
                    throw: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("throw", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_trait() {
            test(
                r#"
                <: {
                    trait: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("trait", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_try() {
            test(
                r#"
                <: {
                    try: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("try", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_undefined() {
            test(
                r#"
                <: {
                    undefined: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("undefined", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_use() {
            test(
                r#"
                <: {
                    use: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("use", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_using() {
            test(
                r#"
                <: {
                    using: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("using", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_when() {
            test(
                r#"
                <: {
                    when: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("when", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_while() {
            test(
                r#"
                <: {
                    while: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("while", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_yield() {
            test(
                r#"
                <: {
                    yield: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("yield", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_import() {
            test(
                r#"
                <: {
                    import: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("import", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_is() {
            test(
                r#"
                <: {
                    is: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("is", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_meta() {
            test(
                r#"
                <: {
                    meta: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("meta", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_module() {
            test(
                r#"
                <: {
                    module: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("module", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_namespace() {
            test(
                r#"
                <: {
                    namespace: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("namespace", num(42))])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn key_new() {
            test(
                r#"
                <: {
                    new: 42,
                }
                "#,
                |res| assert_eq!(res, obj([("new", num(42))])),
            )
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    async fn obj_escaped_reserved_word_as_key() {
        let err = test(
            r#"
            <: {
                \\u0064\\u0065\\u0066\\u0061\\u0075\\u006c\\u0074: 42,
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token),
            ..
        }) = err
        {
            assert_eq!(token, "BackSlash");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn obj_duplicate_key() {
        let err = test(
            r#"
            <: { hoge: 1, hoge: 2 }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::DuplicateKey(key),
            ..
        }) = err
        {
            assert_eq!(key, "hoge");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn obj_invalid_key() {
        let err = test(
            r#"
            <: {
                42: 42,
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedToken(token),
            ..
        }) = err
        {
            assert_eq!(token, "NumberLiteral");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn obj_and_arr_separated_by_line_break() {
        test(
            r#"
            <: {
                a: 1
                b: [
                    1
                    2
                    3
                ]
                c: 3
            }
            "#,
            |res| {
                assert_eq!(
                    res,
                    obj([
                        ("a", num(1)),
                        ("b", arr([num(1), num(2), num(3)])),
                        ("c", num(3)),
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }
}

mod template_syntax {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            let str = "kawaii"
            <: `Ai is {str}!`
            "#,
            |res| assert_eq!(res, str("Ai is kawaii!")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn convert_to_str() {
        test(
            r#"
            <: `1 + 1 = {(1 + 1)}`
            "#,
            |res| assert_eq!(res, str("1 + 1 = 2")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn invalid() {
        test(
            r#"
            <: `{hoge}`
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn escape() {
        test(
            r#"
            let message = "Hello"
            <: `\`a\{b\}c\``
            "#,
            |res| assert_eq!(res, str("`a{b}c`")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn nested_brackets() {
        test(
            r#"
            <: `{if true {1} else {2}}`
            "#,
            |res| assert_eq!(res, str("1")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn new_line_before() {
        test(
            r#"
            <: `{"Hello"
            // comment
            }`
            "#,
            |res| assert_eq!(res, str("Hello")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn new_line_after() {
        test(
            r#"
            <: `{
            // comment
            "Hello"}`
            "#,
            |res| assert_eq!(res, str("Hello")),
        )
        .await
        .unwrap();
    }
}
