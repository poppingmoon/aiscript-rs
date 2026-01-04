mod testutils;

use aiscript_v1::{
    errors::{
        AiScriptError, AiScriptNamespaceError, AiScriptNamespaceErrorKind, AiScriptRuntimeError,
        AiScriptSyntaxError, AiScriptSyntaxErrorKind,
    },
    utils,
    values::Value,
};
use indexmap::IndexMap;
use testutils::*;

mod terminator {
    use super::*;

    mod top_level {
        use super::*;

        #[tokio::test]
        async fn newline() {
            test(
                r#"
                :: A {
                    let x = 1
                }
                :: B {
                    let x = 2
                }
                <: A:x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn semi_colon() {
            test(
                r#"
                ::A{let x = 1};::B{let x = 2}
                <: A:x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn semi_colon_of_the_tail() {
            test(
                r#"
                ::A{let x = 1};
                <: A:x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod block {
        use super::*;

        #[tokio::test]
        async fn newline() {
            test(
                r#"
                eval {
                    let x = 1
                    let y = 2
                    <: x + y
                }
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn semi_colon() {
            test(
                r#"
                eval{let x=1;let y=2;<:x+y}
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn semi_colon_of_the_tail() {
            test(
                r#"
                eval{let x=1;<:x;}
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }

    mod namespace {
        use super::*;

        #[tokio::test]
        async fn newline() {
            test(
                r#"
                :: A {
                    let x = 1
                    let y = 2
                }
                <: A:x + A:y
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn semi_colon() {
            test(
                r#"
                ::A{let x=1;let y=2}
                <: A:x + A:y
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn semi_colon_of_the_tail() {
            test(
                r#"
                ::A{let x=1;}
                <: A:x
                "#,
                |res| assert_eq!(res, num(1)),
            )
            .await
            .unwrap();
        }
    }
}

mod separator {
    use super::*;

    mod match_ {
        use super::*;

        #[tokio::test]
        async fn multi_line() {
            test(
                r#"
                let x = 1
                <: match x {
                    case 1 => "a"
                    case 2 => "b"
                }
                "#,
                |res| assert_eq!(res, str("a")),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_comma() {
            test(
                r#"
                let x = 1
                <: match x {
                    case 1 => "a",
                    case 2 => "b"
                }
                "#,
                |res| assert_eq!(res, str("a")),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line() {
            test(
                r#"
                let x = 1
                <:match x{case 1=>"a",case 2=>"b"}
                "#,
                |res| assert_eq!(res, str("a")),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line_with_tail_comma() {
            test(
                r#"
                let x = 1
                <: match x{case 1=>"a",case 2=>"b",}
                "#,
                |res| assert_eq!(res, str("a")),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_default() {
            test(
                r#"
                let x = 3
                <: match x {
                    case 1 => "a"
                    case 2 => "b"
                    default => "c"
                }
                "#,
                |res| assert_eq!(res, str("c")),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_comma_default() {
            test(
                r#"
                let x = 3
                <: match x {
                    case 1 => "a",
                    case 2 => "b",
                    default => "c"
                }
                "#,
                |res| assert_eq!(res, str("c")),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line_default() {
            test(
                r#"
                let x = 3
                <:match x{case 1=>"a",case 2=>"b",default=>"c"}
                "#,
                |res| assert_eq!(res, str("c")),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line_with_tail_comma_default() {
            test(
                r#"
                let x = 3
                <:match x{case 1=>"a",case 2=>"b",default=>"c",}
                "#,
                |res| assert_eq!(res, str("c")),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn no_separator() {
            let err = test(
                r#"
                let x = 1
				<:match x{case 1=>"a" case 2=>"b"}
                "#,
                |_| {},
            )
            .await
            .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::SeparatorExpected,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }

        #[tokio::test]
        async fn no_separator_default() {
            let err = test(
                r#"
                let x = 1
				<:match x{case 1=>"a" default=>"b"}
                "#,
                |_| {},
            )
            .await
            .unwrap_err();
            let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::SeparatorExpected,
                ..
            }) = err
            else {
                panic!("{err}");
            };
        }
    }

    mod call {
        use super::*;

        #[tokio::test]
        async fn multi_line() {
            test(
                r#"
                @f(a, b, c) {
                    a * b + c
                }
                <: f(
                    2
                    3
                    1
                )
                "#,
                |res| assert_eq!(res, num(7)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_comma() {
            test(
                r#"
                @f(a, b, c) {
                    a * b + c
                }
                <: f(
                    2,
                    3,
                    1
                )
                "#,
                |res| assert_eq!(res, num(7)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line() {
            test(
                r#"
                @f(a, b, c) {
                    a * b + c
                }
                <:f(2,3,1)
                "#,
                |res| assert_eq!(res, num(7)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line_with_tail_comma() {
            test(
                r#"
                @f(a, b, c) {
                    a * b + c
                }
                <:f(2,3,1,)
                "#,
                |res| assert_eq!(res, num(7)),
            )
            .await
            .unwrap();
        }
    }

    mod obj {
        use super::*;

        #[tokio::test]
        async fn multi_line() {
            test(
                r#"
                let x = {
                    a: 1
                    b: 2
                }
                <: x.b
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_multi_newlines() {
            test(
                r#"
                let x = {

                    a: 1

                    b: 2

                }
                <: x.b
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_comma() {
            test(
                r#"
                let x = {
                    a: 1,
                    b: 2
                }
                <: x.b
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line() {
            test(
                r#"
                let x={a:1,b:2}
                <: x.b
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line_with_tail_comma() {
            test(
                r#"
                let x={a:1,b:2,}
                <: x.b
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }
    }

    mod arr {
        use super::*;

        #[tokio::test]
        async fn multi_line() {
            test(
                r#"
                let x = [
                    1
                    2
                ]
                <: x[1]
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_multi_newlines() {
            test(
                r#"
                let x = [

                    1

                    2

                ]
                <: x[1]
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_comma() {
            test(
                r#"
                let x = [
                    1,
                    2
                ]
                <: x[1]
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_comma_multi_newlines() {
            test(
                r#"
                let x = [

                    1,

                    2

                ]
                <: x[1]
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_comma_and_tail_comma() {
            test(
                r#"
                let x = [
                    1,
                    2,
                ]
                <: x[1]
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_comma_and_tail_comma_multi_newlines() {
            test(
                r#"
                let x = [

                    1,

                    2,

                ]
                <: x[1]
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line() {
            test(
                r#"
                let x=[1,2]
                <: x[1]
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line_with_tail_comma() {
            test(
                r#"
                let x=[1,2,]
                <: x[1]
                "#,
                |res| assert_eq!(res, num(2)),
            )
            .await
            .unwrap();
        }
    }

    mod function_params {
        use super::*;

        #[tokio::test]
        async fn single_line() {
            test(
                r#"
                @f(a, b) {
                    a + b
                }
                <: f(1, 2)
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn single_line_with_tail_comma() {
            test(
                r#"
                @f(a, b, ) {
                    a + b
                }
                <: f(1, 2)
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line() {
            test(
                r#"
                @f(
                    a
                    b
                ) {
                    a + b
                }
                <: f(1, 2)
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_comma() {
            test(
                r#"
                @f(
                    a,
                    b
                ) {
                    a + b
                }
                <: f(1, 2)
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn multi_line_with_tail_comma() {
            test(
                r#"
                @f(
                    a,
                    b,
                ) {
                    a + b
                }
                <: f(1, 2)
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn destructuring_param() {
            test(
                r#"
                @f([a, b]) {
                    a + b
                }
                <: f([1, 2])
                "#,
                |res| assert_eq!(res, num(3)),
            )
            .await
            .unwrap();
        }
    }
}

mod comment {
    use super::*;

    #[tokio::test]
    async fn single_line_comment() {
        test(
            r#"
            // let a = ...
            let a = 42
            <: a
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn multi_line_comment() {
        test(
            r#"
            /* variable declaration here...
                let a = ...
            */
            let a = 42
            <: a
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn multi_line_comment_2() {
        test(
            r#"
            /* variable declaration here...
                let a = ...
            */
            let a = 42
            /*
                another comment here
            */
            <: a
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn double_slash_as_string() {
        test(
            r#"
            <: "//"
            "#,
            |res| assert_eq!(res, str("//")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn line_tail() {
        test(
            r#"
            let x = 'a' // comment
            let y = 'b'
            <: x
            "#,
            |res| assert_eq!(res, str("a")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn invalid_eof_in_multi_line_comment() {
        let err = test(
            r#"
            /* comment
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn invalid_eof_in_multi_line_comment_2() {
        let err = test("/* comment *", |_| {}).await.unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::UnexpectedEof,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }
}

mod lang_version {
    use super::*;

    #[test]
    fn number() {
        let res = utils::get_lang_version(
            r#"
            /// @2021
            @f(x) {
                x
            }
            "#,
        );
        assert_eq!(res.unwrap(), "2021");
    }

    #[test]
    fn chars() {
        let res = utils::get_lang_version(
            r#"
            /// @ canary
            const a = 1
            @f(x) {
                x
            }
            f(a)
            "#,
        );
        assert_eq!(res.unwrap(), "canary");
    }

    #[test]
    fn complex() {
        let res = utils::get_lang_version(
            r#"
            /// @ 2.0-Alpha
            @f(x) {
                x
            }
            "#,
        );
        assert_eq!(res.unwrap(), "2.0-Alpha");
    }

    #[test]
    fn no_specified() {
        let res = utils::get_lang_version(
            r#"
            @f(x) {
                x
            }
            "#,
        );
        assert_eq!(res, None);
    }
}

mod cannot_put_multiple_statements_in_a_line {
    use super::*;

    #[tokio::test]
    async fn var_def() {
        let err = test(
            r#"
            let a = 42 let b = 11
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::MultipleStatements,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn var_def_op() {
        let err = test(
            r#"
            let a = 13 + 75 let b = 24 + 146
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::MultipleStatements,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn var_def_in_block() {
        let err = test(
            r#"
            eval {
				let a = 42 let b = 11
			}
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::MultipleStatements,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }
}

mod variable_declaration {
    use super::*;

    #[tokio::test]
    async fn let_() {
        test(
            r#"
            let a = 42
            <: a
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn do_not_assign_to_let_issue_328() {
        let err = test(
            r#"
            let hoge = 33
            hoge = 4
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::AssignmentToImmutable(name)) = err {
            assert_eq!(name, "hoge");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn empty_function() {
        test(
            r#"
            @hoge() { }
			<: hoge()
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
    }
}

mod variable_assignment {
    use super::*;

    #[tokio::test]
    async fn simple() {
        test(
            r#"
            var hoge = 25
            hoge = 7
            <: hoge
            "#,
            |res| assert_eq!(res, num(7)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn destructuring_assingment() {
        test(
            r#"
            var hoge = 'foo'
            var fuga = { value: 'bar' }
            [{ value: hoge }, fuga] = [fuga, hoge]
            <: [hoge, fuga]
            "#,
            |res| assert_eq!(res, arr([str("bar"), str("foo")])),
        )
        .await
        .unwrap();
    }

    mod eval_left_hand_once {
        use super::*;

        #[tokio::test]
        async fn add() {
            test(
                r#"
                var index = -1
                let array = [0, 0]
                array[eval { index += 1; index }] += 1
                <: array
                "#,
                |res| assert_eq!(res, arr([num(1), num(0)])),
            )
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn sub() {
            test(
                r#"
                var index = -1
                let array = [0, 0]
                array[eval { index += 1; index }] -= 1
                <: array
                "#,
                |res| assert_eq!(res, arr([num(-1), num(0)])),
            )
            .await
            .unwrap();
        }
    }
}

mod for_ {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            var count = 0
            for (let i, 10) {
                count += i + 1
            }
            <: count
            "#,
            |res| assert_eq!(res, num(55)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn initial_value() {
        test(
            r#"
            var count = 0
            for (let i = 2, 10) {
                count += i
            }
            <: count
            "#,
            |res| assert_eq!(res, num(65)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn without_iterator() {
        test(
            r#"
            var count = 0
            for (10) {
                count = (count + 1)
            }
            <: count
            "#,
            |res| assert_eq!(res, num(10)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn without_brackets() {
        test(
            r#"
            var count = 0
            for let i, 10 {
                count = (count + i)
            }
            <: count
            "#,
            |res| assert_eq!(res, num(45)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn break_() {
        test(
            r#"
            var count = 0
            for (let i, 20) {
                if (i == 11) break
                count += i
            }
            <: count
            "#,
            |res| assert_eq!(res, num(55)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn continue_() {
        test(
            r#"
            var count = 0
            for (let i, 10) {
                if (i == 5) continue
                count = (count + 1)
            }
            <: count
            "#,
            |res| assert_eq!(res, num(9)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn single_statement() {
        test(
            r#"
            var count = 0
            for 10 count += 1
            <: count
            "#,
            |res| assert_eq!(res, num(10)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn var_name_without_space() {
        let err = test(
            r#"
            for (leti, 10) {
                <: i
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
            assert_eq!(token, "Comma");
        } else {
            panic!("{err}");
        }
    }
}

mod each {
    use super::*;

    #[tokio::test]
    async fn standard() {
        test(
            r#"
            let msgs = []
            each let item, ["ai", "chan", "kawaii"] {
                msgs.push([item, "!"].join())
            }
            <: msgs
            "#,
            |res| assert_eq!(res, arr([str("ai!"), str("chan!"), str("kawaii!")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn destructuring_declaration() {
        test(
            r#"
            each let { value: a }, [{ value: 1 }] {
				<: a
			}
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn break_() {
        test(
            r#"
            let msgs = []
            each let item, ["ai", "chan", "kawaii", "yo"] {
                if (item == "kawaii") break
                msgs.push([item, "!"].join())
            }
            <: msgs
            "#,
            |res| assert_eq!(res, arr([str("ai!"), str("chan!")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn single_statement() {
        test(
            r#"
            let msgs = []
            each let item, ["ai", "chan", "kawaii"] msgs.push([item, "!"].join())
            <: msgs
            "#,
            |res| assert_eq!(res, arr([str("ai!"), str("chan!"), str("kawaii!")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn var_name_without_space() {
        let err = test(
            r#"
            each letitem, ["ai", "chan", "kawaii"] {
                <: item
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
            assert_eq!(token, "Identifier");
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn with_label() {
        test(
            r#"
            let msgs = []
            #label: each let item, ["ai", "chan", "kawaii"] {
                msgs.push([item, "!"].join())
            }
            <: msgs
            "#,
            |res| assert_eq!(res, arr([str("ai!"), str("chan!"), str("kawaii!")])),
        )
        .await
        .unwrap();
    }
}

mod while_ {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            var count = 0
            while count < 42 {
                count += 1
            }
            <: count
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn start_false() {
        test(
            r#"
            while false {
                <: 'hoge'
            }
            "#,
            |_| panic!(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_label() {
        test(
            r#"
            var count = 0
            #label: while count < 42 {
                count += 1
            }
            <: count
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }
}

mod do_while {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            var count = 0
            do {
                count += 1
            } while count < 42
            <: count
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn start_false() {
        test(
            r#"
            do {
                <: 'hoge'
            } while false
            "#,
            |res| assert_eq!(res, str("hoge")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_label() {
        test(
            r#"
            var count = 0
            do {
                count += 1
            } while count < 42
            <: count
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }
}

mod loop_ {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            var count = 0
            loop {
                if (count == 10) break
                count = (count + 1)
            }
            <: count
            "#,
            |res| assert_eq!(res, num(10)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_continue() {
        test(
            r#"
            var a = ["ai", "chan", "kawaii", "yo", "!"]
            var b = []
            loop {
                var x = a.shift()
                if (x == "chan") continue
                if (x == "yo") break
                b.push(x)
            }
            <: b
            "#,
            |res| assert_eq!(res, arr([str("ai"), str("kawaii")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_label() {
        test(
            r#"
            var count = 0
            #label: loop {
                if (count == 10) break
                count = (count + 1)
            }
            <: count
            "#,
            |res| assert_eq!(res, num(10)),
        )
        .await
        .unwrap();
    }
}

mod meta {
    use super::*;

    #[test]
    fn default_meta() {
        let res = get_meta(
            r#"
            ### { a: 1, b: 2, c: 3, }
            "#,
        )
        .unwrap();
        assert_eq!(
            res,
            IndexMap::<Option<String>, Option<Value>>::from_iter([(
                None,
                Some(obj([("a", (num(1))), ("b", (num(2))), ("c", (num(3)))]))
            )])
        );
        assert_eq!(
            res.get(&None).cloned(),
            Some(Some(obj([
                ("a", (num(1))),
                ("b", (num(2))),
                ("c", (num(3))),
            ])))
        )
    }

    mod string {
        use super::*;

        #[test]
        fn valid() {
            let res = get_meta(
                r#"
                ### x "hoge"
                "#,
            )
            .unwrap();
            assert_eq!(
                res,
                IndexMap::<Option<String>, Option<Value>>::from_iter([(
                    Some("x".to_string()),
                    Some(str("hoge"))
                )])
            );
        }
    }

    mod number {
        use super::*;

        #[test]
        fn valid() {
            let res = get_meta(
                r#"
                ### x 42
                "#,
            )
            .unwrap();
            assert_eq!(
                res,
                IndexMap::<Option<String>, Option<Value>>::from_iter([(
                    Some("x".to_string()),
                    Some(num(42))
                )])
            );
        }
    }

    mod boolean {
        use super::*;

        #[test]
        fn valid() {
            let res = get_meta(
                r#"
                ### x true
                "#,
            )
            .unwrap();
            assert_eq!(
                res,
                IndexMap::<Option<String>, Option<Value>>::from_iter([(
                    Some("x".to_string()),
                    Some(bool(true))
                )])
            );
        }
    }

    mod null {
        use super::*;

        #[test]
        fn valid() {
            let res = get_meta(
                r#"
                ### x null
                "#,
            )
            .unwrap();
            assert_eq!(
                res,
                IndexMap::<Option<String>, Option<Value>>::from_iter([(
                    Some("x".to_string()),
                    Some(null())
                )])
            );
        }
    }

    mod array {
        use super::*;

        #[test]
        fn valid() {
            let res = get_meta(
                r#"
                ### x [1, 2, 3]
                "#,
            )
            .unwrap();
            assert_eq!(
                res,
                IndexMap::<Option<String>, Option<Value>>::from_iter([(
                    Some("x".to_string()),
                    Some(arr([num(1), num(2), num(3)]))
                )])
            );
        }

        #[test]
        fn invalid() {
            let err = get_meta(
                r#"
                ### x [1, (2 + 2), 3]
                "#,
            )
            .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token),
                ..
            }) = err
            {
                assert_eq!(token, "Plus");
            } else {
                panic!("{err}");
            }
        }
    }

    mod object {
        use super::*;

        #[test]
        fn valid() {
            let res = get_meta(
                r#"
                ### x { a: 1, b: 2, c: 3, }
                "#,
            )
            .unwrap();
            assert_eq!(
                res,
                IndexMap::<Option<String>, Option<Value>>::from_iter([(
                    Some("x".to_string()),
                    Some(obj([("a", num(1)), ("b", num(2)), ("c", num(3))]))
                )])
            );
        }

        #[test]
        fn invalid() {
            let err = get_meta(
                r#"
                ### x { a: 1, b: (2 + 2), c: 3, }
                "#,
            )
            .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token),
                ..
            }) = err
            {
                assert_eq!(token, "Plus");
            } else {
                panic!("{err}");
            }
        }
    }

    mod template {
        use super::*;

        #[test]
        fn invalid() {
            let err = get_meta(
                r#"
                ### x `foo {bar} baz`
                "#,
            )
            .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token),
                ..
            }) = err
            {
                assert_eq!(token, "Template");
            } else {
                panic!("{err}");
            }
        }
    }

    mod expression {
        use super::*;

        #[test]
        fn invalid() {
            let err = get_meta(
                r#"
                ### x (1 + 1)
                "#,
            )
            .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token),
                ..
            }) = err
            {
                assert_eq!(token, "Plus");
            } else {
                panic!("{err}");
            }
        }
    }

    mod labeled_expression {
        use super::*;

        #[test]
        fn invalid() {
            let err = get_meta(
                r#"
                ### x #label: eval { 1 }
                "#,
            )
            .unwrap_err();
            if let AiScriptError::Syntax(AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedToken(token),
                ..
            }) = err
            {
                assert_eq!(token, "Sharp");
            } else {
                panic!("{err}");
            }
        }
    }
}

mod namespace {
    use super::*;

    #[tokio::test]
    async fn standard() {
        test(
            r#"
            <: Foo:bar()

            :: Foo {
                @bar() { "ai" }
            }
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn self_ref() {
        test(
            r#"
            <: Foo:bar()

            :: Foo {
                let ai = "kawaii"
                @bar() { ai }
            }
            "#,
            |res| assert_eq!(res, str("kawaii")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn cannot_declare_mutable_variable() {
        let err = test(
            r#"
            :: Foo {
                var ai = "kawaii"
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Namespace(AiScriptNamespaceError {
            kind: AiScriptNamespaceErrorKind::Mutable(name),
            ..
        }) = err
        {
            assert_eq!(name, "ai");
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn cannot_destructuring_declaration() {
        let err = test(
            r#"
            :: Foo {
				let [a, b] = [1, 2]
			}
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Namespace(AiScriptNamespaceError {
            kind: AiScriptNamespaceErrorKind::DestructuringAssignment,
            ..
        }) = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn nested() {
        test(
            r#"
            <: Foo:Bar:baz()

            :: Foo {
                :: Bar {
                    @baz() { "ai" }
                }
            }
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn nested_ref() {
        test(
            r#"
            <: Foo:baz

            :: Foo {
                let baz = Bar:ai
                :: Bar {
                    let ai = "kawaii"
                }
            }
            "#,
            |res| assert_eq!(res, str("kawaii")),
        )
        .await
        .unwrap();
    }
}

mod operators {
    use super::*;

    #[tokio::test]
    async fn eq() {
        test("<: (1 == 1)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (1 == 2)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (Core:type == Core:type)", |res| {
            assert_eq!(res, bool(true))
        })
        .await
        .unwrap();

        test("<: (Core:type == Core:gt)", |res| {
            assert_eq!(res, bool(false))
        })
        .await
        .unwrap();

        test("<: (@(){} == @(){})", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (Core:eq == @(){})", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test(
            r#"
            let f = @(){}
			let g = f

			<: (f == g)
            "#,
            |res| assert_eq!(res, bool(true)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn neq() {
        test("<: (1 != 2)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (1 != 1)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn and() {
        test("<: (true && true)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (true && false)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (false && true)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (false && false)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (false && null)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        let err = test("<: (true && null)", |_| {}).await.unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::TypeMismatch { expected, actual }) = err
        {
            assert_eq!(expected, "boolean");
            assert_eq!(actual, "null");
        } else {
            panic!("{err}");
        }

        test(
            r#"
            var tmp = null

            @func() {
                tmp = true
                return true
            }

            false && func()

            <: tmp
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();

        test(
            r#"
            var tmp = null

            @func() {
                tmp = true
                return true
            }

            true && func()

            <: tmp
            "#,
            |res| assert_eq!(res, bool(true)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn or() {
        test("<: (true || true)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (true || false)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (false || true)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (false || false)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (true || null)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        let err = test("<: (false || null)", |_| {}).await.unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::TypeMismatch { expected, actual }) = err
        {
            assert_eq!(expected, "boolean");
            assert_eq!(actual, "null");
        } else {
            panic!("{err}");
        }

        test(
            r#"
            var tmp = null

            @func() {
                tmp = true
                return true
            }

            true || func()

            <: tmp
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();

        test(
            r#"
            var tmp = null

            @func() {
                tmp = true
                return true
            }

            false || func()

            <: tmp
            "#,
            |res| assert_eq!(res, bool(true)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn add() {
        test("<: (1 + 1)", |res| assert_eq!(res, num(2)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn sub() {
        test("<: (1 - 1)", |res| assert_eq!(res, num(0)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn mul() {
        test("<: (1 * 1)", |res| assert_eq!(res, num(1)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn pow() {
        test("<: (1 ^ 0)", |res| assert_eq!(res, num(1)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn div() {
        test("<: (1 / 1)", |res| assert_eq!(res, num(1)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn mod_() {
        test("<: (1 % 1)", |res| assert_eq!(res, num(0)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn gt() {
        test("<: (2 > 1)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (1 > 1)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (0 > 1)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn lt() {
        test("<: (2 < 1)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (1 < 1)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (0 < 1)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn gteq() {
        test("<: (2 >= 1)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (1 >= 1)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (0 >= 1)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn lteq() {
        test("<: (2 <= 1)", |res| assert_eq!(res, bool(false)))
            .await
            .unwrap();

        test("<: (1 <= 1)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (0 <= 1)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn precedence() {
        test("<: 1 + 2 * 3 + 4", |res| assert_eq!(res, num(11)))
            .await
            .unwrap();

        test("<: 1 + 4 / 4 + 1", |res| assert_eq!(res, num(3)))
            .await
            .unwrap();

        test("<: 1 + 1 == 2 && 2 * 2 == 4", |res| {
            assert_eq!(res, bool(true))
        })
        .await
        .unwrap();

        test("<: (1 + 1) * 2", |res| assert_eq!(res, num(4)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn negative_numbers() {
        test("<: 1+-1", |res| assert_eq!(res, num(0)))
            .await
            .unwrap();

        test("<: 1--1", |res| assert_eq!(res, num(2)))
            .await
            .unwrap();

        test("<: -1*-1", |res| assert_eq!(res, num(1)))
            .await
            .unwrap();

        test("<: -1==-1", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: 1>-1", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: -1<1", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();
    }
}

mod plus {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            let a = 1
            <: +a
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();
    }
}

mod minus {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            let a = 1
            <: -a
            "#,
            |res| assert_eq!(res, num(-1)),
        )
        .await
        .unwrap();
    }
}

mod not {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            <: !true
            "#,
            |res| assert_eq!(res, bool(false)),
        )
        .await
        .unwrap();
    }
}

mod infix_expression {
    use super::*;

    #[tokio::test]
    async fn simple_infix_expression() {
        test("<: 0 < 1", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: 1 + 1", |res| assert_eq!(res, num(2)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn combination() {
        test("<: 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10", |res| {
            assert_eq!(res, num(55))
        })
        .await
        .unwrap();

        test("<: Core:add(1, 3) * Core:mul(2, 5)", |res| {
            assert_eq!(res, num(40))
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn use_parentheses_to_distinguish_expr() {
        test("<: (1 + 10) * (2 + 5)", |res| assert_eq!(res, num(77)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn syntax_symbols_vs_infix_operators() {
        test(
            r#"
            <: match true {
                case 1 == 1 => "true"
                case 1 < 1 => "false"
            }
            "#,
            |res| assert_eq!(res, str("true")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn number_if_expression() {
        test("<: 1 + if true 1 else 2", |res| assert_eq!(res, num(2)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn number_match_expression() {
        test(
            r#"
            <: 1 + match 2 == 2 {
                case true => 3
                case false => 4
            }
            "#,
            |res| assert_eq!(res, num(4)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn eval_eval() {
        test("<: eval { 1 } + eval { 1 }", |res| assert_eq!(res, num(2)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn disallow_line_break() {
        let err = test(
            r#"
            <: 1 +
            1 + 1
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
            assert_eq!(token, "NewLine");
        } else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn escaped_line_break() {
        test(
            r#"
            <: 1 + \
            1 + 1
            "#,
            |res| assert_eq!(res, num(3)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn infix_to_fncall_on_namespace() {
        test(
            r#"
            :: Hoge {
                @add(x, y) {
                    x + y
                }
            }
            <: Hoge:add(1, 2)
            "#,
            |res| assert_eq!(res, num(3)),
        )
        .await
        .unwrap();
    }
}

mod if_ {
    use super::*;

    #[tokio::test]
    async fn if_() {
        test(
            r#"
            var msg = "ai"
            if true {
                msg = "kawaii"
            }
            <: msg
            "#,
            |res| assert_eq!(res, str("kawaii")),
        )
        .await
        .unwrap();

        test(
            r#"
            var msg = "ai"
            if false {
                msg = "kawaii"
            }
            <: msg
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn else_() {
        test(
            r#"
            var msg = null
            if true {
                msg = "ai"
            } else {
                msg = "kawaii"
            }
            <: msg
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();

        test(
            r#"
            var msg = null
            if false {
                msg = "ai"
            } else {
                msg = "kawaii"
            }
            <: msg
            "#,
            |res| assert_eq!(res, str("kawaii")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn elif() {
        test(
            r#"
            var msg = "bebeyo"
            if false {
                msg = "ai"
            } elif true {
                msg = "kawaii"
            }
            <: msg
            "#,
            |res| assert_eq!(res, str("kawaii")),
        )
        .await
        .unwrap();

        test(
            r#"
            var msg = "bebeyo"
            if false {
                msg = "ai"
            } elif false {
                msg = "kawaii"
            }
            <: msg
            "#,
            |res| assert_eq!(res, str("bebeyo")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn if_elif_else() {
        test(
            r#"
            var msg = null
            if false {
                msg = "ai"
            } elif true {
                msg = "chan"
            } else {
                msg = "kawaii"
            }
            <: msg
            "#,
            |res| assert_eq!(res, str("chan")),
        )
        .await
        .unwrap();

        test(
            r#"
            var msg = null
            if false {
                msg = "ai"
            } elif false {
                msg = "chan"
            } else {
                msg = "kawaii"
            }
            <: msg
            "#,
            |res| assert_eq!(res, str("kawaii")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn expr() {
        test(
            r#"
            <: if true "ai" else "kawaii"
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();

        test(
            r#"
            <: if false "ai" else "kawaii"
            "#,
            |res| assert_eq!(res, str("kawaii")),
        )
        .await
        .unwrap();
    }
}

mod eval {
    use super::*;

    #[tokio::test]
    async fn returns_value() {
        test(
            r#"
            let foo = eval {
                let a = 1
                let b = 2
                (a + b)
            }

            <: foo
            "#,
            |res| assert_eq!(res, num(3)),
        )
        .await
        .unwrap();
    }
}

mod match_ {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            <: match 2 {
                case 1 => "a"
                case 2 => "b"
                case 3 => "c"
            }
            "#,
            |res| assert_eq!(res, str("b")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn when_default_not_provided_returns_null() {
        test(
            r#"
            <: match 42 {
                case 1 => "a"
                case 2 => "b"
                case 3 => "c"
            }
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_default() {
        test(
            r#"
            <: match 42 {
                case 1 => "a"
                case 2 => "b"
                case 3 => "c"
                default => "d"
            }
            "#,
            |res| assert_eq!(res, str("d")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_block() {
        test(
            r#"
            <: match 2 {
                case 1 => 1
                case 2 => {
                    let a = 1
                    let b = 2
                    (a + b)
                }
                case 3 => 3
            }
            "#,
            |res| assert_eq!(res, num(3)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_return() {
        test(
            r#"
            @f(x) {
                match x {
                    case 1 => {
                        return "ai"
                    }
                }
                "foo"
            }
            <: f(1)
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn scope() {
        let err = test(
            r#"
            match 1 { case 1 => let a = 1 }
            <: a
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::NoSuchVariable { name, .. }) = err {
            assert_eq!(name, "a");
        } else {
            panic!("{err}");
        }

        let err = test(
            r#"
            match 1 { default => let a = 1 }
            <: a
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::NoSuchVariable { name, .. }) = err {
            assert_eq!(name, "a");
        } else {
            panic!("{err}");
        }
    }
}

mod exists {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            let foo = null
            <: [(exists foo), (exists bar)]
            "#,
            |res| assert_eq!(res, arr([bool(true), bool(false)])),
        )
        .await
        .unwrap();
    }
}
