mod testutils;

use aiscript_v1::errors::{AiScriptError, AiScriptSyntaxError, AiScriptSyntaxErrorKind};
use testutils::*;

mod return_ {
    use super::*;

    #[tokio::test]
    async fn as_statement() {
        test(
            r#"
            @f() {
                return 1
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();

        let err = test("return 1", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_eval() {
        test(
            r#"
            @f() {
                let a = eval {
                    return 1
                }
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();

        let err = test("<: eval { return 1 }", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    mod in_if {
        use super::*;

        #[tokio::test]
        async fn cond() {
            test(
                r#"
                @f() {
                    let a = if eval { return true } {}
                }
                <: f()
                "#,
                |res| assert_eq!(res, bool(true)),
            )
            .await
            .unwrap();

            let err = test("<: if eval { return true } {}", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn then() {
            test(
                r#"
                @f() {
                    let a = if true {
                        return 1
                    }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("<: if true { return 1 }", |_| {}).await.unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn elif_cond() {
            test(
                r#"
                @f() {
                    let a = if false {} elif eval { return true } {}
                }
                <: f()
                "#,
                |res| assert_eq!(res, bool(true)),
            )
            .await
            .unwrap();

            let err = test("<: if false {} elif eval { return true } {}", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn elif_then() {
            test(
                r#"
                @f() {
                    let a = if false {
                    } elif true {
                        return 1
                    }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("<: if false {} elif true eval { return true }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn else_() {
            test(
                r#"
                @f() {
                    let a = if false {
                    } else {
                        return 1
                    }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("<: if false {} else eval { return true }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_match {
        use super::*;

        #[tokio::test]
        async fn about() {
            test(
                r#"
                @f() {
                    let a = match eval { return 1 } {}
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("<: match eval { return 1 } {}", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn case_q() {
            test(
                r#"
                @f() {
                    let a = match 0 {
                        case eval { return 0 } => {
                            return 1
                        }
                    }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(0)),
            )
            .await
            .unwrap();

            let err = test("<: match 0 { case eval { return 0 } => {} }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn case_a() {
            test(
                r#"
                @f() {
                    let a = match 0 {
                        case 0 => {
                            return 1
                        }
                    }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("<: match 0 { case 0 => { return 1 } }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn default() {
            test(
                r#"
                @f() {
                    let a = match 0 {
                        default => {
                            return 1
                        }
                    }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("<: match 0 { default => { return 1 } }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_binary_operation {
        use super::*;

        #[tokio::test]
        async fn left() {
            test(
                r#"
                @f() {
                    eval { return 1 } + 2
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("<: eval { return 1 } + 2", |_| {}).await.unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn right() {
            test(
                r#"
                @f() {
                    1 + eval { return 2 }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();

            let err = test("<: 1 + eval { return 2 }", |_| {}).await.unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_call {
        use super::*;

        #[tokio::test]
        async fn callee() {
            test(
                r#"
                @f() {
                    eval { return print }('Hello, world!')
                }
                f()('Hi')
                "#,
                |res| assert_eq!(res, str("Hi")),
            )
            .await
            .unwrap();

            let err = test("eval { return print }('Hello, world!')", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn arg() {
            test(
                r#"
                @f() {
                    print(eval { return 'Hello, world!' })
                }
                <: f()
                "#,
                |res| assert_eq!(res, str("Hello, world!")),
            )
            .await
            .unwrap();

            let err = test("print(eval { return 'Hello, world' })", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_for {
        use super::*;

        #[tokio::test]
        async fn times() {
            test(
                r#"
                @f() {
                    for eval { return 1 } {}
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("for eval { return 1 } {}", |_| {}).await.unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn from() {
            test(
                r#"
                @f() {
                    for let i = eval { return 1 }, 2 {}
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("for let i = eval { return 1 }, 2 {}", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn to() {
            test(
                r#"
                @f() {
                    for let i = 0, eval { return 1 } {}
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("for let i = 0, eval { return 1 } {}", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn for_() {
            test(
                r#"
                @f() {
                    for 1 {
                        return 1
                    }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("for 1 { return 1 }", |_| {}).await.unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_each {
        use super::*;

        #[tokio::test]
        async fn items() {
            test(
                r#"
                @f() {
                    each let v, [eval { return 1 }] {}
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("each let v, [eval { return 1 }] {}", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn for_() {
            test(
                r#"
                @f() {
                    each let v, [0] {
                        return 1
                    }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("each let v, [0] { return 1 }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_assign {
        use aiscript_v1::values::Value;

        use super::*;

        #[tokio::test]
        async fn expr() {
            test(
                r#"
                @f() {
                    var a = null
                    a = eval { return 1 }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("var a = null; a = eval { return 1 }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn index_target() {
            test(
                r#"
                @f() {
                    let a = [null]
                    eval { return a }[0] = 1
                }
                <: f()
                "#,
                |res| assert_eq!(res, testutils::arr([null()])),
            )
            .await
            .unwrap();

            let err = test("let a = [null]; eval { return a }[0] = 1", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn index() {
            test(
                r#"
                @f() {
                    let a = [null]
                    a[eval { return 0 }] = 1
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(0)),
            )
            .await
            .unwrap();

            let err = test("let a = [null]; a[eval { return 0 }] = 1", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn prop_target() {
            test(
                r#"
                @f() {
                    let o = {}
                    eval { return o }.p = 1
                }
                <: f()
                "#,
                |res| assert_eq!(res, testutils::obj([] as [(String, Value); 0])),
            )
            .await
            .unwrap();

            let err = test("let o = {}; eval { return o }.p = 1", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn arr() {
            test(
                r#"
                @f() {
                    let o = {}
                    [eval { return o }.p] = [1]
                }
                <: f()
                "#,
                |res| assert_eq!(res, testutils::obj([] as [(String, Value); 0])),
            )
            .await
            .unwrap();

            let err = test("let o = {}; [eval { return o }.p] = [1]", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn obj() {
            test(
                r#"
                @f() {
                    let o = {}
                    { a: eval { return o }.p } = { a: 1 }
                }
                <: f()
                "#,
                |res| assert_eq!(res, testutils::obj([] as [(String, Value); 0])),
            )
            .await
            .unwrap();

            let err = test("let o = {}; { a: eval { return o }.p } = { a: 1 }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_add_assign {
        use super::*;

        #[tokio::test]
        async fn dest() {
            test(
                r#"
                @f() {
                    let a = [0]
                    a[eval { return 0 }] += 1
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(0)),
            )
            .await
            .unwrap();

            let err = test("let a = [0]; a[eval { return 0 }] += 1", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn expr() {
            test(
                r#"
                @f() {
                    let a = 0
                    a += eval { return 1 }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("let a = 0; a += eval { return 1 }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_sub_assign {
        use super::*;

        #[tokio::test]
        async fn dest() {
            test(
                r#"
                @f() {
                    let a = [0]
                    a[eval { return 0 }] -= 1
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(0)),
            )
            .await
            .unwrap();

            let err = test("let a = [0]; a[eval { return 0 }] -= 1", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn expr() {
            test(
                r#"
                @f() {
                    let a = 0
                    a -= eval { return 1 }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("let a = 0; a -= eval { return 1 }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    #[tokio::test]
    async fn in_array() {
        test(
            r#"
            @f() {
                let a = [eval { return 1 }]
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();

        let err = test("<: [eval { return 1 }]", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_object() {
        test(
            r#"
            @f() {
                let o = {
                    p: eval { return 1 }
                }
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();

        let err = test("<: { p: eval { return 1 } }", |_| {})
            .await
            .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_prop() {
        test(
            r#"
            @f() {
                let p = {
                    p: eval { return 1 }
                }.p
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();

        let err = test("<: { p: eval { return 1 } }.p", |_| {})
            .await
            .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    mod in_index {
        use super::*;

        #[tokio::test]
        async fn target() {
            test(
                r#"
                @f() {
                    let v = [eval { return 1 }][0]
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("<: [eval { return 1 }][0]", |_| {}).await.unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn index() {
            test(
                r#"
                @f() {
                    let v = [1][eval { return 0 }]
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(0)),
            )
            .await
            .unwrap();

            let err = test("<: [0][eval { return 1 }]", |_| {}).await.unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    #[tokio::test]
    async fn in_not() {
        test(
            r#"
            @f() {
                let b = !eval { return true }
            }
            <: f()
            "#,
            |res| assert_eq!(res, bool(true)),
        )
        .await
        .unwrap();

        let err = test("<: !eval { return true }", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_function_default_param() {
        test(
            r#"
            @f() {
                let g = @(x = eval { return 1 }) {}
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();

        let err = test("<: @(x = eval { return 1 }){}", |_| {})
            .await
            .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };

        let err = test("<: @(a = @(b = eval { return 0 }){}){}", |_| {})
            .await
            .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_template() {
        test(
            r#"
            @f() {
                let s = `{eval { return 1 }}`
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();

        let err = test("<: `{eval {return 1}}`", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_return() {
        test(
            r#"
            @f() {
                return eval { return 1 } + 2
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();

        let err = test("return eval { return 1 } + 2", |_| {})
            .await
            .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    mod in_break {
        use super::*;

        #[tokio::test]
        async fn valid() {
            test(
                r#"
                @f() {
                    #l: eval {
                        break #l eval { return 1 }
                    }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn invalid() {
            let err = test(
                r#"
                #l: eval {
                    break #l eval { return 1 }
                }
                "#,
                |_| {},
            )
            .await
            .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_and {
        use super::*;

        #[tokio::test]
        async fn left() {
            test(
                r#"
                @f() {
                    eval { return 1 } && false
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("eval { return 1 } && false", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn right() {
            test(
                r#"
                @f() {
                    true && eval { return 1 }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("true && eval { return 1 }", |_| {}).await.unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod in_or {
        use super::*;

        #[tokio::test]
        async fn left() {
            test(
                r#"
                @f() {
                    eval { return 1 } || false
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("eval { return 1 } || false", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn right() {
            test(
                r#"
                @f() {
                    false || eval { return 1 }
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();

            let err = test("false || eval { return 1 }", |_| {})
                .await
                .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::ReturnOutsideFunction,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }
}

mod break_ {
    use super::*;

    #[tokio::test]
    async fn as_statement() {
        test(
            r#"
            var x = 0
            for 1 {
                break
                x += 1
            }
            <: x
            "#,
            |res| assert_eq!(res, num(0)),
        )
        .await
        .unwrap();

        let err = test("break", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnlabeledBreakOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };

        let err = test("@() { break }()", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnlabeledBreakOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_eval() {
        test(
            r#"
            var x = 0
            for 1 {
                let a = eval {
                    break
                }
                x += 1
            }
            <: x
            "#,
            |res| assert_eq!(res, num(0)),
        )
        .await
        .unwrap();

        let err = test("<: eval { break }", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnlabeledBreakOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_if() {
        test(
            r#"
            var x = 0
            for 1 {
                let a = if true {
                    break
                }
                x += 1
            }
            <: x
            "#,
            |res| assert_eq!(res, num(0)),
        )
        .await
        .unwrap();

        let err = test("<: if true { break }", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnlabeledBreakOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_match() {
        test(
            r#"
            var x = 0
            for 1 {
                let a = match 0 {
                    default => break
                }
                x += 1
            }
            <: x
            "#,
            |res| assert_eq!(res, num(0)),
        )
        .await
        .unwrap();

        let err = test("<: match 0 { default => break }", |_| {})
            .await
            .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnlabeledBreakOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_function() {
        let err = test(
            r#"
            for 1 {
                @f() {
                    break;
                }
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnlabeledBreakOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn invalid_label() {
        let err = test(
            r#"
            for 1 {
                break #l
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
            ..
        }) = err
        {
            assert_eq!(label, "l");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn invalid_value() {
        let err = test(
            r#"
            #l: for 1 {
                break #l 1
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::BreakToStatementWithValue,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn break_corresponding_to_each_is_not_allowed_in_the_target() {
        let err = test("#l: each let i, eval { break #l } {}", |_| {})
            .await
            .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
            ..
        }) = err
        {
            assert_eq!(label, "l");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn break_corresponding_to_for_is_not_allowed_in_the_count() {
        let err = test("#l: for eval { break #l } {}", |_| {})
            .await
            .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
            ..
        }) = err
        {
            assert_eq!(label, "l");
        } else {
            panic!("{err}");
        };
    }

    mod break_corresponding_to_for_is_not_allowed_in_the_range {
        use super::*;

        #[tokio::test]
        async fn from() {
            let err = test("#l: for let i = eval { break #l }, 0 {}", |_| {})
                .await
                .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
                ..
            }) = err
            {
                assert_eq!(label, "l");
            } else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn to() {
            let err = test("#l: for let i = 0, eval { break #l } {}", |_| {})
                .await
                .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
                ..
            }) = err
            {
                assert_eq!(label, "l");
            } else {
                panic!("{err}");
            };
        }
    }

    mod break_corresponding_to_if_is_not_allowed_in_the_condition {
        use super::*;

        #[tokio::test]
        async fn if_() {
            let err = test("#l: if eval { break #l } {}", |_| {})
                .await
                .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
                ..
            }) = err
            {
                assert_eq!(label, "l");
            } else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn elif() {
            let err = test("#l: if false {} elif eval { break #l } {}", |_| {})
                .await
                .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
                ..
            }) = err
            {
                assert_eq!(label, "l");
            } else {
                panic!("{err}");
            };
        }
    }

    #[tokio::test]
    async fn break_corresponding_to_match_is_not_allowed_in_the_target() {
        let err = test("#l: match eval { break #l } {}", |_| {})
            .await
            .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
            ..
        }) = err
        {
            assert_eq!(label, "l");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn break_corresponding_to_match_is_not_allowed_in_the_pattern() {
        let err = test("#l: match 0 { case eval { break #l } => 1 }", |_| {})
            .await
            .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
            ..
        }) = err
        {
            assert_eq!(label, "l");
        } else {
            panic!("{err}");
        };
    }

    mod labeled_each {
        use super::*;

        #[tokio::test]
        async fn inner_each() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    each let v, [0] {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_for() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    for 1 {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_loop() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    loop {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_do_while() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    do {
                        x = 1
                        break #l
                    } while false
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    while true {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_for {
        use super::*;

        #[tokio::test]
        async fn inner_each() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    each let v, [0] {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_for() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    for 1 {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_loop() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    loop {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_do_while() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    do {
                        x = 1
                        break #l
                    } while false
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    while true {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_loop {
        use super::*;

        #[tokio::test]
        async fn inner_each() {
            test(
                r#"
                var x = 0
                #l: loop {
                    each let v, [0] {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_for() {
            test(
                r#"
                var x = 0
                #l: loop {
                    for 1 {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_loop() {
            test(
                r#"
                var x = 0
                #l: loop {
                    loop {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_do_while() {
            test(
                r#"
                var x = 0
                #l: loop {
                    do {
                        x = 1
                        break #l
                    } while false
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                var x = 0
                #l: loop {
                    while true {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_do_while {
        use super::*;

        #[tokio::test]
        async fn inner_each() {
            test(
                r#"
                var x = 0
                #l: do {
                    each let v, [0] {
                        x = 1
                        break #l
                    }
                    x = 2
                } while false
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_for() {
            test(
                r#"
                var x = 0
                #l: do {
                    for 1 {
                        x = 1
                        break #l
                    }
                    x = 2
                } while false
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_loop() {
            test(
                r#"
                var x = 0
                #l: do {
                    loop {
                        x = 1
                        break #l
                    }
                    x = 2
                } while false
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_do_while() {
            test(
                r#"
                var x = 0
                #l: do {
                    do {
                        x = 1
                        break #l
                    } while false
                    x = 2
                } while false
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                var x = 0
                #l: do {
                    while true {
                        x = 1
                        break #l
                    }
                    x = 2
                } while false
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_while {
        use super::*;

        #[tokio::test]
        async fn inner_each() {
            test(
                r#"
                var x = 0
                #l: while true {
                    each let v, [0] {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_for() {
            test(
                r#"
                var x = 0
                #l: while true {
                    for 1 {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_loop() {
            test(
                r#"
                var x = 0
                #l: while true {
                    loop {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_do_while() {
            test(
                r#"
                var x = 0
                #l: while true {
                    do {
                        x = 1
                        break #l
                    } while false
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                var x = 0
                #l: while true {
                    while true {
                        x = 1
                        break #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_if {
        use super::*;

        #[tokio::test]
        async fn simple_break() {
            test(
                r#"
                <: #l: if true {
                    break #l
                    2
                }
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn break_with_value() {
            test(
                r#"
                <: #l: if true {
                    break #l 1
                    2
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_match {
        use super::*;

        #[tokio::test]
        async fn simple_break() {
            test(
                r#"
                <: #l: match 0 {
                    default => {
                        break #l
                        2
                    }
                }
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn break_with_value() {
            test(
                r#"
                <: #l: match 0 {
                    default => {
                        break #l 1
                        2
                    }
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_eval {
        use super::*;

        #[tokio::test]
        async fn simple_break() {
            test(
                r#"
                <: #l: eval {
                    break #l
                    2
                }
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn break_with_value() {
            test(
                r#"
                <: #l: eval {
                    break #l 1
                    2
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                <: #l: eval {
                    while true {
                        if true break #l 1
                    }
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }
}

mod continue_ {
    use super::*;

    #[tokio::test]
    async fn as_statement() {
        test(
            r#"
            var x = 0
            for 1 {
                continue
                x += 1
            }
            <: x
            "#,
            |res| assert_eq!(res, num(0)),
        )
        .await
        .unwrap();

        let err = test("continue", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };

        let err = test("@() { continue }()", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_eval() {
        test(
            r#"
            var x = 0
            for 1 {
                let a = eval {
                    continue
                }
                x += 1
            }
            <: x
            "#,
            |res| assert_eq!(res, num(0)),
        )
        .await
        .unwrap();

        let err = test("<: eval { continue }", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_if() {
        test(
            r#"
            var x = 0
            for 1 {
                let a = if true {
                    continue
                }
                x += 1
            }
            <: x
            "#,
            |res| assert_eq!(res, num(0)),
        )
        .await
        .unwrap();

        let err = test("<: if true { continue }", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_match() {
        test(
            r#"
            var x = 0
            for 1 {
                let a = match 0 {
                    default => continue
                }
                x += 1
            }
            <: x
            "#,
            |res| assert_eq!(res, num(0)),
        )
        .await
        .unwrap();

        let err = test("<: match 0 { default => continue }", |_| {})
            .await
            .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn in_function() {
        let err = test(
            r#"
            for 1 {
                @f() {
                    continue;
                }
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn invalid_label() {
        let err = test(
            r#"
            for 1 {
                continue #l
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
            ..
        }) = err
        {
            assert_eq!(label, "l");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn invalid_block() {
        let err = test("#l: if true { continue #l }", |_| {})
            .await
            .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };

        let err = test("#l: match 0 { default => continue #l }", |_| {})
            .await
            .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };

        let err = test("#l: eval { continue #l }", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ContinueOutsideLoop,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn continue_corresponding_to_each_is_not_allowed_in_the_target() {
        let err = test("#l: each let i, eval { continue #l } {}", |_| {})
            .await
            .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
            ..
        }) = err
        {
            assert_eq!(label, "l");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn continue_corresponding_to_for_is_not_allowed_in_the_count() {
        let err = test("#l: for eval { continue #l } {}", |_| {})
            .await
            .unwrap_err();
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
            ..
        }) = err
        {
            assert_eq!(label, "l");
        } else {
            panic!("{err}");
        };
    }

    mod continue_corresponding_to_for_is_not_allowed_in_the_range {
        use super::*;

        #[tokio::test]
        async fn from() {
            let err = test("#l: for let i = eval { continue #l }, 0 {}", |_| {})
                .await
                .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
                ..
            }) = err
            {
                assert_eq!(label, "l");
            } else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn to() {
            let err = test("#l: for let i = 0, eval { continue #l } {}", |_| {})
                .await
                .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UndefinedLabel(label),
                ..
            }) = err
            {
                assert_eq!(label, "l");
            } else {
                panic!("{err}");
            };
        }
    }

    mod labeled_each {
        use super::*;

        #[tokio::test]
        async fn inner_each() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    each let v, [0] {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_for() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    for 1 {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_loop() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    loop {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_do_while() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    do {
                        x = 1
                        continue #l
                    } while false
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                var x = 0
                #l: each let v, [0] {
                    while true {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_for {
        use super::*;

        #[tokio::test]
        async fn inner_each() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    each let v, [0] {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_for() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    for 1 {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_loop() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    loop {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_do_while() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    do {
                        x = 1
                        continue #l
                    } while false
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                var x = 0
                #l: for 1 {
                    while true {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_while {
        use super::*;

        #[tokio::test]
        async fn inner_each() {
            test(
                r#"
                var x = 0
                #l: while x == 0 {
                    each let v, [0] {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_for() {
            test(
                r#"
                var x = 0
                #l: while x == 0 {
                    for 1 {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_loop() {
            test(
                r#"
                var x = 0
                #l: while x == 0 {
                    loop {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_do_while() {
            test(
                r#"
                var x = 0
                #l: while x == 0 {
                    do {
                        x = 1
                        continue #l
                    } while false
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                var x = 0
                #l: while x == 0 {
                    while true {
                        x = 1
                        continue #l
                    }
                    x = 2
                }
                <: x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_if {
        use super::*;

        #[tokio::test]
        async fn simple_break() {
            test(
                r#"
                <: #l: if true {
                    break #l
                    2
                }
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn break_with_value() {
            test(
                r#"
                <: #l: if true {
                    break #l 1
                    2
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_match {
        use super::*;

        #[tokio::test]
        async fn simple_break() {
            test(
                r#"
                <: #l: match 0 {
                    default => {
                        break #l
                        2
                    }
                }
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn break_with_value() {
            test(
                r#"
                <: #l: match 0 {
                    default => {
                        break #l 1
                        2
                    }
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod labeled_eval {
        use super::*;

        #[tokio::test]
        async fn simple_break() {
            test(
                r#"
                <: #l: eval {
                    break #l
                    2
                }
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn break_with_value() {
            test(
                r#"
                <: #l: eval {
                    break #l 1
                    2
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn inner_while() {
            test(
                r#"
                <: #l: eval {
                    while true {
                        if true break #l 1
                    }
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }
}

mod label {
    use super::*;

    #[tokio::test]
    async fn invalid_statement() {
        let err = test("#l: null", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::InvalidStatementWithLabel,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn invalid_expression() {
        let err = test("let a = #l: null", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::InvalidExpressionWithLabel,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn invalid_space() {
        let err = test("# l: eval { null }", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::SpaceInLabel,
            ..
        }) = err
        else {
            panic!("{err}");
        };

        let err = test("#l: eval { break # l }", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::SpaceInLabel,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }
}
