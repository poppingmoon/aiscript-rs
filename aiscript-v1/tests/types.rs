mod testutils;

use aiscript_v1::errors::{AiScriptError, AiScriptSyntaxError, AiScriptSyntaxErrorKind};
use testutils::*;

mod function_types {
    use super::*;

    #[tokio::test]
    async fn multiple_params() {
        test(
            r#"
            let f: @(str, num) => bool = @() { true }
            <: f('abc', 123)
            "#,
            |res| assert_eq!(res, bool(true)),
        )
        .await
        .unwrap();
    }
}

mod generics {
    use super::*;

    mod function {
        use super::*;

        #[tokio::test]
        async fn expr() {
            test(
                r#"
                let f = @<T>(v: T): void {}
                <: f("a")
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn consumer() {
            test(
                r#"
                @f<T>(v: T): void {}
                <: f("a")
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn identity_function() {
            test(
                r#"
                @f<T>(v: T): T { v }
                <: f(1)
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn use_as_inner_type() {
            test(
                r#"
                @vals<T>(v: obj<T>): arr<T> {
                    Obj:vals(v)
                }
                <: vals({ a: 1, b: 2, c: 3 })
                "#,
                |res| assert_eq!(res, arr([num(1), num(2), num(3)])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn use_as_variable_type() {
            test(
                r#"
                @f<T>(v: T): void {
                    let v2: T = v
                }
                <: f(1)
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn use_as_function_type() {
            test(
                r#"
                @f<T>(v: T): @() => T {
                    let g: @() => T = @() { v }
                    g
                }
                <: f(1)()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn curried() {
            test(
                r#"
                @concat<A>(a: A): @<B>(B) => str {
                    @<B>(b: B) {
                        `{a}{b}`
                    }
                }
                <: concat("abc")(123)
                "#,
                |res| assert_eq!(res, str("abc123")),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn new_lines() {
            test(
                r#"
                @f<
                    T
                    U
                >(x: T, y: U): arr<T | U> {
                    [x, y]
                }
                <: f("abc", 123)
                "#,
                |res| assert_eq!(res, arr([str("abc"), num(123)])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn duplicate() {
            let err = test(
                r#"
                @f<T, T>(v: T) {}
                "#,
                |_| {},
            )
            .await
            .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::DuplicateTypeParameterName(name),
                ..
            }) = err
            {
                assert_eq!(name, "T");
            } else {
                panic!("{err}");
            }
        }

        #[tokio::test]
        async fn duplicate_no_param_and_ret_types() {
            let err = test(
                r#"
                @f<T, T>() {}
                "#,
                |_| {},
            )
            .await
            .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::DuplicateTypeParameterName(name),
                ..
            }) = err
            {
                assert_eq!(name, "T");
            } else {
                panic!("{err}");
            }
        }

        #[tokio::test]
        async fn empty() {
            let err = test(
                r#"
                @f<>() {}
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
                assert_eq!(token, "Gt");
            } else {
                panic!("{err}");
            }
        }

        #[tokio::test]
        async fn cannot_have_inner_type() {
            let err = test(
                r#"
                @f<T>(v: T<num>) {}
                "#,
                |_| {},
            )
            .await
            .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnknownType(name),
                ..
            }) = err
            {
                assert_eq!(name, "T<num>");
            } else {
                panic!("{err}");
            }
        }
    }
}

mod union {
    use super::*;

    #[tokio::test]
    async fn variable_type() {
        test(
            r#"
            let a: num | null = null
            <: a
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn more_inners() {
        test(
            r#"
            let a: str | num | null = null
            <: a
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn inner_type() {
        test(
            r#"
            let a: arr<num | str> = ["abc", 123]
            <: a
            "#,
            |res| assert_eq!(res, arr([str("abc"), num(123)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn param_type() {
        test(
            r#"
            @f(x: num | str): str {
                `{x}`
            }
            <: f(1)
            "#,
            |res| assert_eq!(res, str("1")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn return_type() {
        test(
            r#"
            @f(): num | str { 1 }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn type_parameter() {
        test(
            r#"
            @f<T>(v: T): T | null { null }
            <: f(1)
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn function_type() {
        test(
            r#"
            let f: @(num | str) => str = @(x) { `{x}` }
            <: f(1)
            "#,
            |res| assert_eq!(res, str("1")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn invalid_inner() {
        let err = test(
            r#"
            let a: ThisIsAnInvalidTypeName | null = null
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnknownType(name),
            ..
        }) = err
        {
            assert_eq!(name, "ThisIsAnInvalidTypeName");
        } else {
            panic!("{err}");
        }
    }
}

mod simple {
    use super::*;

    #[tokio::test]
    async fn error() {
        test(
            r#"
            let a: error = Error:create("Ai")
            <: a
            "#,
            |res| assert_eq!(res, testutils::error("Ai", None)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn never() {
        test(
            r#"
            @f() {
                let a: never = eval {
                    return 1
                }
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn in_break() {
    let err = test(
        r#"
        #l: eval {
            break #l eval {
                let x: invalid = 0
            }
        }
        "#,
        |_| {},
    )
    .await
    .unwrap_err();
    if let AiScriptError::Syntax(AiScriptSyntaxError {
        kind: AiScriptSyntaxErrorKind::UnknownType(name),
        ..
    }) = err
    {
        assert_eq!(name, "invalid");
    } else {
        panic!("{err}");
    }
}
