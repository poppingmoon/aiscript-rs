mod testutils;

use aiscript_v1::values::Value;
use testutils::*;

mod empty_lines {
    use super::*;

    mod match_ {
        use super::*;

        #[tokio::test]
        async fn empty_line() {
            test(
                r#"
                <: match 1 {
                    // comment
                }
                "#,
                |res| assert_eq!(res, null()),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_before_case() {
            test(
                r#"
                <: match 1 {
                    // comment
                    case 1 => 1
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_after_case() {
            test(
                r#"
                <: match 1 {
                    case 1 => 1
                    // comment
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_before_default() {
            test(
                r#"
                <: match 1 {
                    // comment
                    default => 1
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_after_default() {
            test(
                r#"
                <: match 1 {
                    default => 1
                    // comment
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod call {
        use super::*;

        #[tokio::test]
        async fn empty_line() {
            test(
                r#"
                @f() {
                    1
                }
                <:f(
                    // comment
                )
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_before() {
            test(
                r#"
                @f(a) {
                    a
                }
                <:f(
                    // comment
                    1
                )
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_after() {
            test(
                r#"
                @f(a) {
                    a
                }
                <:f(
                    1
                    // comment
                )
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod type_params {
        use super::*;

        mod function {
            use super::*;

            #[tokio::test]
            async fn empty_line_before() {
                test(
                    r#"
                    @f<
                        // comment
                        T
                    >(v: T): T {
                        v
                    }
                    <: f(1)
                    "#,
                    |res| assert_eq!(res, num(1)),
                )
                .await
                .unwrap();
            }

            #[tokio::test]
            async fn empty_line_after() {
                test(
                    r#"
                    @f<
                        T
                        // comment
                    >(v: T): T {
                        v
                    }
                    <: f(1)
                    "#,
                    |res| assert_eq!(res, num(1)),
                )
                .await
                .unwrap();
            }
        }

        mod function_type {
            use super::*;

            #[tokio::test]
            async fn empty_line_before() {
                test(
                    r#"
                    let f: @<
                        // comment
                        T
                    >(T) => T = @(v) {
                        v
                    }
                    <: f(1)
                    "#,
                    |res| assert_eq!(res, num(1)),
                )
                .await
                .unwrap();
            }

            #[tokio::test]
            async fn empty_line_after() {
                test(
                    r#"
                    let f: @<
                        T
                        // comment
                    >(T) => T = @(v) {
                        v
                    }
                    <: f(1)
                    "#,
                    |res| assert_eq!(res, num(1)),
                )
                .await
                .unwrap();
            }
        }
    }

    mod function_params {
        use super::*;

        #[tokio::test]
        async fn empty_line() {
            test(
                r#"
                @f(
                    // comment
                ) {
                    1
                }
                <: f()
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_before() {
            test(
                r#"
                @f(
                    // comment
                    a
                ) {
                    a
                }
                <: f(1)
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_after() {
            test(
                r#"
                @f(
                    a
                    // comment
                ) {
                    a
                }
                <: f(1)
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod if_ {
        use super::*;

        #[tokio::test]
        async fn empty_line_between_if_elif() {
            test(
                r#"
                <: if true {
                    1
                }
                // comment
                elif true {
                    2
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_between_if_elif_elif() {
            test(
                r#"
                <: if true {
                    1
                }
                // comment
                elif true {
                    2
                }
                // comment
                elif true {
                    3
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_between_if_else() {
            test(
                r#"
                <: if true {
                    1
                }
                // comment
                else {
                    2
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_between_if_elif_else() {
            test(
                r#"
                <: if true {
                    1
                }
                // comment
                elif true {
                    2
                }
                // comment
                else {
                    3
                }
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod unary_operation {
        use super::*;

        #[tokio::test]
        async fn empty_line_after() {
            test(
                r#"
                ! \
                // comment
                true
                "#,
                |res| assert_eq!(res, bool(false)),
            )
            .await
            .unwrap();
        }
    }

    mod binary_operation {
        use super::*;

        #[tokio::test]
        async fn empty_line_before() {
            test(
                r#"
                <: 2 \
                // comment
                * 3
                "#,
                |res| assert_eq!(res, num(6)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_after() {
            test(
                r#"
                <: 2 * \
                // comment
                3
                "#,
                |res| assert_eq!(res, num(6)),
            )
            .await
            .unwrap();
        }
    }

    mod variable_definition {
        use super::*;

        #[tokio::test]
        async fn empty_line_after_equal() {
            test(
                r#"
                let a =
                // comment
                1
                <: a
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod attribute {
        use super::*;

        #[tokio::test]
        async fn empty_line_after() {
            test(
                r#"
                #[abc]
                // comment
                let a = 1
                <: a
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod obj_literal {
        use super::*;

        #[tokio::test]
        async fn empty_line() {
            test(
                r#"
                <: {
                    // comment
                }
                "#,
                |res| assert_eq!(res, obj([] as [(String, Value); 0])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_before() {
            test(
                r#"
                let x = {
                    // comment
                    a: 1
                }
                <: x.a
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_after() {
            test(
                r#"
                let x = {
                    a: 1
                    // comment
                }
                <: x.a
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod arr_literal {
        use super::*;

        #[tokio::test]
        async fn empty_line() {
            test(
                r#"
                <: [
                    // comment
                ]
                "#,
                |res| assert_eq!(res, arr([])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_before() {
            test(
                r#"
                let x = [
                    // comment
                    1
                ]
                <: x[0]
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn empty_line_after() {
            test(
                r#"
                let x = [
                    1
                    // comment
                ]
                <: x[0]
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }
}
