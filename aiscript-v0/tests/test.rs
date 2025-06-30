use ::std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use aiscript_v0::{
    Interpreter, Parser,
    ast::*,
    errors::{AiScriptError, AiScriptRuntimeError},
    utils,
    values::Value,
};
use futures::FutureExt;
use indexmap::IndexMap;

async fn test(program: &str, test: fn(Value)) -> Result<Value, AiScriptError> {
    let ast = Parser::default().parse(program)?;
    let test_count = Arc::new(AtomicUsize::new(0));
    let test_count_clone = test_count.clone();
    let aiscript = Interpreter::new(
        [],
        None::<fn(_) -> _>,
        Some(move |value| {
            test(value);
            test_count_clone.fetch_add(1, Ordering::Relaxed);
            async move {}.boxed()
        }),
        None::<fn(_) -> _>,
        Some(9999),
    );
    let result = aiscript.exec(ast).await.map(|value| value.unwrap())?;
    match test_count.load(Ordering::Relaxed) {
        0 => panic!("test has never been called"),
        1 => Ok(result),
        count => panic!("test has been called ${count} times"),
    }
}

fn get_meta(program: &str) -> Result<IndexMap<Option<String>, Option<Value>>, AiScriptError> {
    let ast = Parser::default().parse(program)?;
    let metadata = Interpreter::collect_metadata(ast);
    Ok(metadata)
}

fn null() -> Value {
    Value::null()
}

fn bool(value: bool) -> Value {
    Value::bool(value)
}

fn num(value: impl Into<f64>) -> Value {
    Value::num(value.into())
}

fn str(value: impl Into<String>) -> Value {
    Value::str(value.into())
}

fn arr(value: impl IntoIterator<Item = Value>) -> Value {
    Value::arr(value)
}

fn obj(value: impl IntoIterator<Item = (impl Into<String>, Value)>) -> Value {
    Value::obj(value)
}

fn error(value: impl Into<String>, info: Option<Value>) -> Value {
    Value::error(value, info)
}

#[tokio::test]
async fn hello_world() {
    test("<: \"Hello, world!\"", |res| {
        assert_eq!(res, str("Hello, world!"))
    })
    .await
    .unwrap();
}

#[test]
fn empty_script() {
    let ast = Parser::default().parse("").unwrap();
    assert_eq!(ast, Vec::new());
}

mod interpreter {
    use super::*;

    mod scope {
        use super::*;

        #[tokio::test]
        async fn get_all() {
            let aiscript = Interpreter::default();
            aiscript
                .exec(
                    Parser::default()
                        .parse(
                            r#"
                            let a = 1
                            @b() {
                                let x = a + 1
                                x
                            }
                            if true {
                                var y = 2
                            }
                            var c = true
                            "#,
                        )
                        .unwrap(),
                )
                .await
                .unwrap();
            let vars = aiscript.scope.get_all().await;
            assert_ne!(vars.get("a"), None);
            assert_ne!(vars.get("b"), None);
            assert_ne!(vars.get("a"), None);
            assert_eq!(vars.get("x"), None);
            assert_eq!(vars.get("y"), None);
        }
    }
}

mod ops {
    use super::*;

    #[tokio::test]
    async fn eq() {
        test("<: (1 == 1)", |res| assert_eq!(res, bool(true)))
            .await
            .unwrap();

        test("<: (1 == 2)", |res| assert_eq!(res, bool(false)))
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
        assert!(matches!(err, AiScriptError::Runtime(_)));

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
        assert!(matches!(err, AiScriptError::Runtime(_)));

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
        test("<: (1 ^ 1)", |res| assert_eq!(res, num(1)))
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
                1 == 1 => "true"
                1 < 1 => "false"
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
                true => 3
                false => 4
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
        test(
            r#"
            <: 1 +
            1 + 1
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
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
}

#[tokio::test]
async fn expression_containing_collon_is_not_object() {
    test(
        r#"
        <: eval {
            Core:eq("ai", "ai")
        }
        "#,
        |res| assert_eq!(res, bool(true)),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn inc() {
    test(
        r#"
        var a = 0
        a += 1
        a += 2
        a += 3
        <: a
        "#,
        |res| assert_eq!(res, num(6)),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn dec() {
    test(
        r#"
        var a = 0
        a -= 1
        a -= 2
        a -= 3
        <: a
        "#,
        |res| assert_eq!(res, num(-6)),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn reference_is_not_chained() {
    test(
        r#"
        var f = @() { "a" }
        var g = f
        f = @() { "b" }

        <: g()
        "#,
        |res| assert_eq!(res, str("a")),
    )
    .await
    .unwrap();
}

mod cannot_put_multiple_statements_in_a_line {
    use super::*;

    #[tokio::test]
    async fn var_def() {
        test(
            r#"
            let a = 42 let b = 11
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn var_def_op() {
        test(
            r#"
            let a = 13 + 75 let b = 24 + 146
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }
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

#[tokio::test]
async fn empty_lambda() {
    test(
        r#"
        let hoge = @() { }
        <: hoge()
        "#,
        |res| assert_eq!(res, null()),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn lambda_that_returns_an_object() {
    test(
        r#"
        let hoge = @() {{}}
        <: hoge()
        "#,
        |res| assert_eq!(res, obj([] as [(String, Value); 0])),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn closure() {
    test(
        r#"
        @store(v) {
            let state = v
            @() {
                state
            }
        }
        let s = store("ai")
        <: s()
        "#,
        |res| assert_eq!(res, str("ai")),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn closure_counter() {
    test(
        r#"
        @create_counter() {
            var count = 0
            {
                get_count: @() { count };
                count: @() { count = (count + 1) };
            }
        }

        let counter = create_counter()
        let get_count = counter.get_count
        let count = counter.count

        count()
        count()
        count()

        <: get_count()
        "#,
        |res| assert_eq!(res, num(3)),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn recursion() {
    test(
        r#"
        @fact(n) {
            if (n == 0) { 1 } else { (fact((n - 1)) * n) }
        }

        <: fact(5)
        "#,
        |res| assert_eq!(res, num(120)),
    )
    .await
    .unwrap();
}

mod var_name_starts_with_reserved_word {
    use super::*;

    #[tokio::test]
    async fn let_() {
        test(
            r#"
            @f() {
                let letcat = "ai"
                letcat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn var() {
        test(
            r#"
            @f() {
                let varcat = "ai"
                varcat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn return_() {
        test(
            r#"
            @f() {
                let returncat = "ai"
                returncat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn each() {
        test(
            r#"
            @f() {
                let eachcat = "ai"
                eachcat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn for_() {
        test(
            r#"
            @f() {
                let forcat = "ai"
                forcat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn loop_() {
        test(
            r#"
            @f() {
                let loopcat = "ai"
                loopcat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn break_() {
        test(
            r#"
            @f() {
                let breakcat = "ai"
                breakcat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn continue_() {
        test(
            r#"
            @f() {
                let continuecat = "ai"
                continuecat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn if_() {
        test(
            r#"
            @f() {
                let ifcat = "ai"
                ifcat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn match_() {
        test(
            r#"
            @f() {
                let matchcat = "ai"
                matchcat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn true_() {
        test(
            r#"
            @f() {
                let truecat = "ai"
                truecat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn false_() {
        test(
            r#"
            @f() {
                let falsecat = "ai"
                falsecat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn null() {
        test(
            r#"
            @f() {
                let nullcat = "ai"
                nullcat
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }
}

mod name_validation_of_reserved_word {
    use super::*;

    #[tokio::test]
    async fn def() {
        test(
            r#"
            let let = 1
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn attr() {
        test(
            r#"
            #[let 1]
            @f() { 1 }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn ns() {
        test(
            r#"
            :: let {
                @f() { 1 }
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn var() {
        test(
            r#"
            let
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn prop() {
        test(
            r#"
            let x = { let: 1 }
            x.let
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn meta() {
        test(
            r#"
            ### let 1
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn fn_() {
        test(
            r#"
            @let() { 1 }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }
}

mod object {
    use super::*;

    #[tokio::test]
    async fn property_access() {
        test(
            r#"
            let obj = {
                a: {
                    b: {
                        c: 42;
                    };
                };
            }

            <: obj.a.b.c
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn property_access_fn_call() {
        test(
            r#"
            @f() { 42 }

            let obj = {
                a: {
                    b: {
                        c: f;
                    };
                };
            }

            <: obj.a.b.c()
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn property_assign() {
        test(
            r#"
            let obj = {
                a: 1
                b: {
                    c: 2
                    d: {
                        e: 3
                    }
                }
            }

            obj.a = 24
            obj.b.d.e = 42

            <: obj
            "#,
            |res| {
                assert_eq!(
                    res,
                    obj([
                        ("a", num(24)),
                        ("b", obj([("c", num(2)), ("d", obj([("e", num(42))]))]))
                    ])
                )
            },
        )
        .await
        .unwrap();
    }
}

mod array {

    use super::*;

    #[tokio::test]
    async fn array_item_access() {
        test(
            r#"
            let arr = ["ai", "chan", "kawaii"]

            <: arr[1]
            "#,
            |res| assert_eq!(res, str("chan")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn array_item_assign() {
        test(
            r#"
            let arr = ["ai", "chan", "kawaii"]

            arr[1] = "taso"

            <: arr
            "#,
            |res| assert_eq!(res, arr([str("ai"), str("taso"), str("kawaii"),])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn assign_array_item_to_out_of_range() {
        let err = test(
            r#"
            let arr = [1, 2, 3]

            arr[3] = 4

            <: null
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange(_, _))
        ));

        let err = test(
            r#"
            let arr = [1, 2, 3]

            arr[9] = 10

            <: null
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange(_, _))
        ));
    }

    #[tokio::test]
    async fn index_out_of_range_error() {
        let err = test(
            r#"
            <: [42][1]
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange(_, _))
        ));
    }

    #[tokio::test]
    async fn index_out_of_range_on_assignment() {
        let err = test(
            r#"
            var a = []
            a[2] = 'hoge'
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange(_, _))
        ));
    }

    #[tokio::test]
    async fn non_integer_indexed_assignment() {
        let err = test(
            r#"
            var a = []
            a[6.21] = 'hoge'
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange(_, _))
        ));
    }
}

mod chain {
    use super::*;

    #[tokio::test]
    async fn chain_access_prop_index_call() {
        test(
            r#"
            let obj = {
                a: {
                    b: [@(name) { name }, @(str) { "chan" }, @() { "kawaii" }];
                };
            }

            <: obj.a.b[0]("ai")
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn chained_assign_left_side_prop_index() {
        test(
            r#"
            let obj = {
                a: {
                    b: ["ai", "chan", "kawaii"];
                };
            }

            obj.a.b[1] = "taso"

            <: obj
            "#,
            |res| {
                assert_eq!(
                    res,
                    obj([(
                        "a",
                        obj([("b", arr([str("ai"), str("taso"), str("kawaii"),]))])
                    )])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn chained_assign_right_side_prop_index_call() {
        test(
            r#"
            let obj = {
                a: {
                    b: ["ai", "chan", "kawaii"];
                };
            }

            var x = null
            x = obj.a.b[1]

            <: x
            "#,
            |res| assert_eq!(res, str("chan"),),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn chained_inc_dec_left_side_index_prop() {
        test(
            r#"
            let arr = [
                {
                    a: 1;
                    b: 2;
                }
            ]

            arr[0].a += 1
            arr[0].b -= 1

            <: arr
            "#,
            |res| assert_eq!(res, arr([obj([("a", num(2)), ("b", num(1)),])])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn chained_inc_dec_left_side_prop_index() {
        test(
            r#"
            let obj = {
                a: {
                    b: [1, 2, 3];
                };
            }

            obj.a.b[1] += 1
            obj.a.b[2] -= 1

            <: obj
            "#,
            |res| {
                assert_eq!(
                    res,
                    obj([("a", obj([("b", arr([num(1), num(3), num(2),]))]))])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn prop_in_def() {
        test(
            r#"
            let x = @() {
                let obj = {
                    a: 1
                }
                obj.a
            }

            <: x()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn prop_in_return() {
        test(
            r#"
            let x = @() {
                let obj = {
                    a: 1
                }
                return obj.a
                2
            }

            <: x()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn prop_in_each() {
        test(
            r#"
            let msgs = []
            let x = { a: ["ai", "chan", "kawaii"] }
            each let item, x.a {
                let y = { a: item }
                msgs.push([y.a, "!"].join())
            }
            <: msgs
            "#,
            |res| assert_eq!(res, arr([str("ai!"), str("chan!"), str("kawaii!"),])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn prop_in_for() {
        test(
            r#"
            let x = { times: 10, count: 0 }
            for (let i, x.times) {
                x.count = (x.count + i)
            }
            <: x.count
            "#,
            |res| assert_eq!(res, num(45)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn object_with_index() {
        test(
            r#"
            let ai = {a: {}}['a']
            ai['chan'] = 'kawaii'
            <: ai[{a: 'chan'}['a']]
            "#,
            |res| assert_eq!(res, str("kawaii")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn property_chain_with_parenthesis() {
        let ast = Parser::default()
            .parse(
                r#"
                (a.b).c
                "#,
            )
            .unwrap();
        let line = ast.first().unwrap().clone();
        if let Node::Expression(Expression::Prop(prop)) = line {
            assert_eq!(prop.name, "c".to_string());
            if let Expression::Prop(prop) = *prop.target {
                assert_eq!(prop.name, "b".to_string());
                if let Expression::Identifier(identifier) = *prop.target {
                    assert_eq!(identifier.name, "a".to_string());
                    return;
                }
            }
        }
        panic!();
    }

    #[tokio::test]
    async fn index_chain_with_parenthesis() {
        let ast = Parser::default()
            .parse(
                r#"
                (a[42]).b
                "#,
            )
            .unwrap();
        let line = ast.first().unwrap().clone();
        if let Node::Expression(Expression::Prop(prop)) = line {
            assert_eq!(prop.name, "b".to_string());
            if let Expression::Index(index) = *prop.target {
                if let (Expression::Identifier(identifier), Expression::Num(num)) =
                    (*index.target, *index.index)
                {
                    assert_eq!(identifier.name, "a".to_string());
                    assert_eq!(num.value, 42.0);
                    return;
                }
            }
        }
        panic!();
    }

    #[tokio::test]
    async fn call_chain_with_parenthesis() {
        let ast = Parser::default()
            .parse(
                r#"
                (foo(42, 57)).bar
                "#,
            )
            .unwrap();
        let line = ast.first().unwrap().clone();
        if let Node::Expression(Expression::Prop(prop)) = line {
            assert_eq!(prop.name, "bar".to_string());
            if let Expression::Call(call) = *prop.target {
                if let Expression::Identifier(identifier) = *call.target {
                    assert_eq!(identifier.name, "foo".to_string());
                    if let [Expression::Num(num_1), Expression::Num(num_2)] = &call.args[..] {
                        assert_eq!(num_1.value, 42.0);
                        assert_eq!(num_2.value, 57.0);
                        return;
                    }
                }
            }
        }
        panic!();
    }

    #[tokio::test]
    async fn longer_chain_with_parenthesis() {
        let ast = Parser::default()
            .parse(
                r#"
                (a.b.c).d.e
                "#,
            )
            .unwrap();
        let line = ast.first().unwrap().clone();
        if let Node::Expression(Expression::Prop(prop)) = line {
            assert_eq!(prop.name, "e".to_string());
            if let Expression::Prop(prop) = *prop.target {
                assert_eq!(prop.name, "d".to_string());
                if let Expression::Prop(prop) = *prop.target {
                    assert_eq!(prop.name, "c".to_string());
                    if let Expression::Prop(prop) = *prop.target {
                        assert_eq!(prop.name, "b".to_string());
                        if let Expression::Identifier(identifier) = *prop.target {
                            assert_eq!(identifier.name, "a".to_string());
                            return;
                        }
                    }
                }
            }
        }
        panic!();
    }

    #[tokio::test]
    async fn property_chain_with_if() {
        let ast = Parser::default()
            .parse(
                r#"
                (if a b else c).d
                "#,
            )
            .unwrap();
        let line = ast.first().unwrap().clone();
        if let Node::Expression(Expression::Prop(prop)) = line.clone() {
            assert_eq!(prop.name, "d".to_string());
            if let Expression::If(if_) = *prop.target {
                if let (
                    Expression::Identifier(cond),
                    StatementOrExpression::Expression(Expression::Identifier(then)),
                    StatementOrExpression::Expression(Expression::Identifier(else_)),
                ) = (*if_.cond, *if_.then, *if_.else_.unwrap())
                {
                    assert_eq!(cond.name, "a".to_string());
                    assert_eq!(then.name, "b".to_string());
                    assert_eq!(else_.name, "c".to_string());
                    return;
                }
            }
        }
        panic!();
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
}

#[tokio::test]
async fn throws_error_when_divided_by_zero() {
    test(
        r#"
        <: (0 / 0)
        "#,
        |_| {},
    )
    .await
    .unwrap_err();
}

mod function_call {
    use super::*;

    #[tokio::test]
    async fn without_args() {
        test(
            r#"
            @f() {
                42
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_args() {
        test(
            r#"
            @f(x) {
                x
            }
            <: f(42)
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_args_separated_by_comma() {
        test(
            r#"
            @f(x, y) {
                (x + y)
            }
            <: f(1, 1)
            "#,
            |res| assert_eq!(res, num(2)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn with_args_separated_by_space() {
        test(
            r#"
            @f(x y) {
                (x + y)
            }
            <: f(1 1)
            "#,
            |res| assert_eq!(res, num(2)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn std_throw_aiscript_error_when_required_arg_missing() {
        let err = test(
            r#"
            <: Core:eq(1)
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AiScriptError::Runtime(_)));
    }

    #[tokio::test]
    async fn omitted_args() {
        test(
            r#"
            @f(x, y) {
                [x, y]
            }
            <: f(1)
            "#,
            |res| assert_eq!(res, arr([num(1), null()])),
        )
        .await
        .unwrap();
    }
}

mod return_ {
    use super::*;

    #[tokio::test]
    async fn early_return() {
        test(
            r#"
            @f() {
                if true {
                    return "ai"
                }

                "pope"
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn early_return_nested() {
        test(
            r#"
            @f() {
                if true {
                    if true {
                        return "ai"
                    }
                }

                "pope"
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn early_return_nested_2() {
        test(
            r#"
            @f() {
                if true {
                    return "ai"
                }

                "pope"
            }

            @g() {
                if (f() == "ai") {
                    return "kawaii"
                }

                "pope"
            }

            <: g()
            "#,
            |res| assert_eq!(res, str("kawaii")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn early_return_without_block() {
        test(
            r#"
            @f() {
                if true return "ai"

                "pope"
            }
            <: f()
            "#,
            |res| assert_eq!(res, str("ai")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn return_inside_for() {
        test(
            r#"
            @f() {
                var count = 0
                for (let i, 100) {
                    count += 1
                    if (i == 42) {
                        return count
                    }
                }
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(43)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn return_inside_for_2() {
        test(
            r#"
            @f() {
                for (let i, 10) {
                    return 1
                }
                2
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn return_inside_loop() {
        test(
            r#"
            @f() {
                var count = 0
                loop {
                    count += 1
                    if (count == 42) {
                        return count
                    }
                }
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn return_inside_loop_2() {
        test(
            r#"
            @f() {
                loop {
                    return 1
                }
                2
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn return_inside_each() {
        test(
            r#"
            @f() {
                var count = 0
                each (let item, ["ai", "chan", "kawaii"]) {
                    count += 1
                    if (item == "chan") {
                        return count
                    }
                }
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(2)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn return_inside_each_2() {
        test(
            r#"
            @f() {
                each (let item, ["ai", "chan", "kawaii"]) {
                    return 1
                }
                2
            }
            <: f()
            "#,
            |res| assert_eq!(res, num(1)),
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

mod exists {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            let foo = null
            <: [(exists foo) (exists bar)]
            "#,
            |res| assert_eq!(res, arr([bool(true), bool(false)])),
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

mod match_ {
    use super::*;

    #[tokio::test]
    async fn basic() {
        test(
            r#"
            <: match 2 {
                1 => "a"
                2 => "b"
                3 => "c"
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
                1 => "a"
                2 => "b"
                3 => "c"
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
                1 => "a"
                2 => "b"
                3 => "c"
                * => "d"
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
                1 => 1
                2 => {
                    let a = 1
                    let b = 2
                    (a + b)
                }
                3 => 3
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
                    1 => {
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
            var a = ["ai" "chan" "kawaii" "yo" "!"]
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
        test(
            r#"
            for (leti, 10) {
                <: i
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
    }
}

mod for_of {
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
    async fn break_() {
        test(
            r#"
            let msgs = []
            each let item, ["ai", "chan", "kawaii" "yo"] {
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
        test(
            r#"
            each letitem, ["ai", "chan", "kawaii"] {
                <: item
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
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
        test(
            r#"
            :: Foo {
                var ai = "kawaii"
            }
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
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
        test(
            r#"
            <: "ai saw a note \"bebeyo\"."
            "#,
            |res| assert_eq!(res, str(r#"ai saw a note "bebeyo"."#)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn escaped_single_quote() {
        test(
            r#"
            <: 'ai saw a note \'bebeyo\'.'
            "#,
            |res| assert_eq!(res, str("ai saw a note 'bebeyo'.")),
        )
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
    async fn obj_separated_by_semicolon() {
        test(
            r#"
            <: { a: 1; b: 2; c: 3 }
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(2)), ("c", num(3))])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn obj_separated_by_semicolon_with_trailing_semicolon() {
        test(
            r#"
            <: { a: 1; b: 2; c: 3; }
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
    async fn obj_separated_by_line_break_and_semicolon() {
        test(
            r#"
            <: {
                a: 1;
                b: 2;
                c: 3
            }
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(2)), ("c", num(3))])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn obj_separated_by_line_break_and_semicolon_with_trailing_semicolon() {
        test(
            r#"
            <: {
                a: 1;
                b: 2;
                c: 3;
            }
            "#,
            |res| assert_eq!(res, obj([("a", num(1)), ("b", num(2)), ("c", num(3))])),
        )
        .await
        .unwrap();
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
                        ("c", num(3))
                    ])
                )
            },
        )
        .await
        .unwrap();
    }
}

mod type_declaration {
    use super::*;

    #[tokio::test]
    async fn def() {
        test(
            r#"
            let abc: num = 1
            var xyz: str = "abc"
            <: [abc xyz]
            "#,
            |res| assert_eq!(res, arr([num(1), str("abc")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn fn_def() {
        test(
            r#"
            @f(x: arr<num>, y: str, z: @(num) => bool): arr<num> {
                x.push(0)
                y = "abc"
                var r: bool = z(x[0])
                x.push(if r 5 else 10)
                x
            }

            <: f([1, 2, 3], "a", @(n) { n == 1 })
            "#,
            |res| assert_eq!(res, arr([num(1), num(2), num(3), num(0), num(5)])),
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
            ### { a: 1; b: 2; c: 3; }
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
                ("c", (num(3)))
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
                ### x [1 2 3]
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
            get_meta(
                r#"
                ### x [1 (2 + 2) 3]
                "#,
            )
            .unwrap_err();
        }
    }

    mod object {
        use super::*;

        #[test]
        fn valid() {
            let res = get_meta(
                r#"
                ### x { a: 1; b: 2; c: 3; }
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
            get_meta(
                r#"
                ### x { a: 1; b: (2 + 2); c: 3; }
                "#,
            )
            .unwrap_err();
        }
    }

    mod template {
        use super::*;

        #[test]
        fn invalid() {
            get_meta(
                r#"
                ### x `foo {bar} baz`
                "#,
            )
            .unwrap_err();
        }
    }

    mod expression {
        use super::*;

        #[test]
        fn invalid() {
            get_meta(
                r#"
                ### x (1 + 1)
                "#,
            )
            .unwrap_err();
        }
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
        assert_eq!(res, Some("2021".to_string()));
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
        assert_eq!(res, Some("canary".to_string()));
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
        assert_eq!(res, Some("2.0-Alpha".to_string()));
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

mod attribute {
    use super::*;

    #[test]
    fn single_attribute_with_function_str() {
        let nodes = Parser::default()
            .parse(
                r#"
                #[Event "Received"]
                @onReceived(data) {
                    data
                }
                "#,
            )
            .unwrap();
        if let [Node::Statement(Statement::Definition(definition))] = &nodes[..] {
            assert_eq!(definition.name, "onReceived");
            if let Some(attr) = &definition.attr {
                if let [
                    Attribute {
                        name,
                        value: Expression::Str(str),
                        ..
                    },
                ] = &attr[..]
                {
                    assert_eq!(name, "Event");
                    assert_eq!(str.value, "Received");
                    return;
                }
            }
        }
        panic!();
    }

    #[test]
    fn multiple_attributes_with_function_obj_str_bool() {
        let nodes = Parser::default()
            .parse(
                r#"
                #[Endpoint { path: "/notes/create"; }]
                #[Desc "Create a note."]
                #[Cat true]
                @createNote(text) {
                    <: text
                }
                "#,
            )
            .unwrap();
        if let [Node::Statement(Statement::Definition(definition))] = &nodes[..] {
            assert_eq!(definition.name, "createNote");
            if let Some(attr) = &definition.attr {
                if let [
                    Attribute {
                        name: name1,
                        value: Expression::Obj(obj),
                        ..
                    },
                    Attribute {
                        name: name2,
                        value: Expression::Str(str),
                        ..
                    },
                    Attribute {
                        name: name3,
                        value: Expression::Bool(bool),
                        ..
                    },
                ] = &attr[..]
                {
                    assert_eq!(name1, "Endpoint");
                    if let [(key, Expression::Str(str))] =
                        obj.value.iter().collect::<Vec<(&String, &Expression)>>()[..]
                    {
                        assert_eq!(key, "path");
                        assert_eq!(str.value, "/notes/create");
                        return;
                    };
                    assert_eq!(name2, "Desc");
                    assert_eq!(str.value, "Create a note.");
                    assert_eq!(name3, "Cat");
                    assert!(bool.value);
                }
            }
        }
        panic!();
    }

    #[test]
    fn single_attribute_no_value() {
        let nodes = Parser::default()
            .parse(
                r#"
                #[serializable]
                let data = 1
                "#,
            )
            .unwrap();
        if let [Node::Statement(Statement::Definition(definition))] = &nodes[..] {
            assert_eq!(definition.name, "data");
            if let Some(attr) = &definition.attr {
                if let [
                    Attribute {
                        name,
                        value: Expression::Bool { .. },
                        ..
                    },
                ] = &attr[..]
                {
                    assert_eq!(name, "serializable");
                    return;
                }
            }
        }
        panic!();
    }
}

mod location {
    use super::*;

    #[test]
    fn function() {
        let nodes = Parser::default()
            .parse(
                r#"
		@f(a) { a }
                "#,
            )
            .unwrap();
        if let [Node::Statement(Statement::Definition(definition))] = &nodes[..] {
            if let Some(Loc { start, end }) = definition.loc {
                assert_eq!(start.clone(), 3);
                assert_eq!(end.clone(), 13);
                return;
            }
        }
        panic!();
    }

    #[test]
    fn comment() {
        let nodes = Parser::default()
            .parse(
                r#"
		/*
		*/
		// hoge
		@f(a) { a }
                "#,
            )
            .unwrap();
        if let [Node::Statement(Statement::Definition(definition))] = &nodes[..] {
            if let Some(Loc { start, end }) = definition.loc {
                assert_eq!(start.clone(), 23);
                assert_eq!(end.clone(), 33);
                return;
            }
        }
        panic!();
    }
}

mod variable_declaration {
    use super::*;

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
        assert!(matches!(err, AiScriptError::Runtime(_)));
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
}

mod primitive_props {
    use super::*;

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
                            str("👦")
                        ])
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
                            num(128102)
                        ])
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
                            97, 98, 99, 55399, 56893, 55357, 56393, 55356, 57343, 55357, 56424,
                            8205, 55357, 56422, 100, 101, 102
                        ]
                        .into_iter()
                        .map(|u| str(String::from_utf16_lossy(&[u])))
                        .collect::<Vec<Value>>())
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
                        ])
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
                        ])
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
                        ])
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
                        ])
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
                        ])
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
                        ])
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
                        ])
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
                        ])
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
                            arr([num(1), num(2), num(3)])
                        ])
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
                                str("elephant")
                            ])
                        ])
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
                <: arr.reduce(@(accumulator, currentValue, index) { (accumulator + (currentValue * index)) } 0)
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
            assert!(matches!(
                err,
                AiScriptError::Runtime(AiScriptRuntimeError::Runtime(message))
                    if &message == "Reduce of empty array without initial value"
            ));
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
                        arr([arr([num(3), num(2), num(1)]), arr([num(1), num(2), num(3)])])
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
                        arr([str("hoge"), str("hoge"), str("huga"), str("piyo")])
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
                        arr([str("piyo"), str("huga"), str("hoge"), str("hoge")])
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
                            obj([("x", num(10))])
                        ])
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
                        ])
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
                        ])
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
                        arr([arr([num(0), num(10), num(3)]), arr([num(1), num(2)]),])
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
                        ])
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
                        ])
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
                |res| assert_eq!(res, arr([arr([num(0)]), arr([num(1), num(2), num(3)]),])),
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
                            arr([num(0), num(1), num(2), num(3), num(4), num(5), num(6),]),
                        ])
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
                        ])
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
                        ])
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
                        ])
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
                                num(60)
                            ])
                        ])
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
                            arr([num(1), num(4), num(5), num(6), num(7)])
                        ])
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
                        ])
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
                        ])
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
                        ])
                    )
                },
            )
            .await
            .unwrap();
        }
    }
}

mod std {
    use super::*;

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
                        num(10)
                    ])
                )
            })
            .await
            .unwrap();

            test("<: Core:range(1, 1)", |res| assert_eq!(res, arr([num(1),])))
                .await
                .unwrap();

            test("<: Core:range(9, 7)", |res| {
                assert_eq!(res, arr([num(9), num(8), num(7),]))
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
            assert!(matches!(
                err,
                AiScriptError::Runtime(AiScriptRuntimeError::User(message))
                    if message == "hoge"
            ));
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
    }

    mod obj {
        use super::*;

        #[tokio::test]
        async fn keys() {
            test(
                r#"
                let o = { a: 1; b: 2; c: 3; }

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
                let o = { _nul: null; _num: 24; _str: 'hoge'; _arr: []; _obj: {}; }

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
                            obj([] as [(String, Value); 0])
                        ])
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
                let o = { a: 1; b: 2; c: 3; }

                <: Obj:kvs(o)
                "#,
                |res| {
                    assert_eq!(
                        res,
                        arr([
                            arr([str("a"), num(1)]),
                            arr([str("b"), num(2)]),
                            arr([str("c"), num(3)])
                        ])
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
                let o1 = { a: 1; b: 2; }
                let o2 = { b: 3; c: 4; }

                <: Obj:merge(o1, o2)
                "#,
                |res| assert_eq!(res, obj([("a", num(1)), ("b", num(3)), ("c", num(4)),])),
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
                        ])
                    )
                },
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
                        str("https://example.com/?q=%E3%81%82%E3%81%84%E3%81%A1%E3%82%83%E3%82%93")
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
                    Json:stringify(Json:parse(''))
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
                    Json:stringify(Json:parse('hoge'))
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
                    Json:stringify(Json:parse('['))
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
                            num(2024)
                        ])
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
                        arr([num(Local.timestamp_millis_opt(0).unwrap().month()), num(1)])
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
                        arr([num(Local.timestamp_millis_opt(0).unwrap().day()), num(2)])
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
                        arr([num(Local.timestamp_millis_opt(0).unwrap().hour()), num(3)])
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
                        arr([num(Local.timestamp_millis_opt(0).unwrap().minute()), num(4)])
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
                        arr([num(Local.timestamp_millis_opt(0).unwrap().second()), num(5)])
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
                            num(6)
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
                    .captures(&s1);
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
                            str("2024-04-11T11:29:46.021-05:18")
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
                    let res = <Vec<Value>>::try_from(res).unwrap();
                    assert_eq!(res[..4], vec![num(0), num(0), num(0), num(0)]);
                    assert!(f64::try_from(res[4].clone()).unwrap().is_nan())
                },
            )
            .await
            .unwrap();
        }
    }
}

mod unicode {
    use super::*;

    #[tokio::test]
    async fn len() {
        test(
            r#"
            <: "👍🏽🍆🌮".len
            "#,
            |res| assert_eq!(res, num(3)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn pick() {
        test(
            r#"
            <: "👍🏽🍆🌮".pick(1)
            "#,
            |res| assert_eq!(res, str("🍆")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn slice() {
        test(
            r#"
            <: "Emojis 👍🏽 are 🍆 poison. 🌮s are bad.".slice(7, 14)
            "#,
            |res| assert_eq!(res, str("👍🏽 are 🍆")),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn split() {
        test(
            r#"
            <: "👍🏽🍆🌮".split()
            "#,
            |res| assert_eq!(res, arr([str("👍🏽"), str("🍆"), str("🌮")])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn charcode_at() {
        test(
            r#"
            <: [
                "👍🏽🍆🌮".charcode_at(0),
                "👍🏽🍆🌮".charcode_at(1),
                "👍🏽🍆🌮".charcode_at(2),
                "👍🏽🍆🌮".charcode_at(3),
                "👍🏽🍆🌮".charcode_at(4),
                "👍🏽🍆🌮".charcode_at(5),
                "👍🏽🍆🌮".charcode_at(6),
                "👍🏽🍆🌮".charcode_at(7),
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(55357),
                        num(56397),
                        num(55356),
                        num(57341),
                        num(55356),
                        num(57158),
                        num(55356),
                        num(57134),
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn codepoint_at() {
        test(
            r#"
            <: [
                "👍🏽🍆🌮".codepoint_at(0),
                "👍🏽🍆🌮".codepoint_at(1),
                "👍🏽🍆🌮".codepoint_at(2),
                "👍🏽🍆🌮".codepoint_at(3),
                "👍🏽🍆🌮".codepoint_at(4),
                "👍🏽🍆🌮".codepoint_at(5),
                "👍🏽🍆🌮".codepoint_at(6),
                "👍🏽🍆🌮".codepoint_at(7),
            ]
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(128077),
                        num(56397),
                        num(127997),
                        num(57341),
                        num(127814),
                        num(57158),
                        num(127790),
                        num(57134),
                    ])
                )
            },
        )
        .await
        .unwrap();
    }
}

mod security {
    use super::*;

    #[tokio::test]
    async fn cannot_access_js_native_property_via_var() {
        let err = test(
            r#"
            <: constructor
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AiScriptError::Runtime(_)));

        let err = test(
            r#"
            <: prototype
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AiScriptError::Runtime(_)));

        let err = test(
            r#"
            <: __proto__
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AiScriptError::Runtime(_)));
    }

    #[tokio::test]
    async fn cannot_access_js_native_property_via_object() {
        test(
            r#"
            let obj = {}

            <: obj.constructor
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();

        test(
            r#"
            let obj = {}

            <: obj.prototype
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();

        test(
            r#"
            let obj = {}

            <: obj.__proto__
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn cannot_access_js_native_property_via_primitive_prop() {
        let err = test(
            r#"
            <: "".constructor
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AiScriptError::Runtime(_)));

        let err = test(
            r#"
            <: "".prototype
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AiScriptError::Runtime(_)));

        let err = test(
            r#"
            <: "".__proto__
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AiScriptError::Runtime(_)));
    }
}

mod extra {
    use super::*;

    #[tokio::test]
    async fn fizz_buzz() {
        test(
            r#"
            let res = []
            for (let i = 1, 15) {
                let msg =
                    if (i % 15 == 0) "FizzBuzz"
                    elif (i % 3 == 0) "Fizz"
                    elif (i % 5 == 0) "Buzz"
                    else i
                res.push(msg)
            }
            <: res
            "#,
            |res| {
                assert_eq!(
                    res,
                    arr([
                        num(1),
                        num(2),
                        str("Fizz"),
                        num(4),
                        str("Buzz"),
                        str("Fizz"),
                        num(7),
                        num(8),
                        str("Fizz"),
                        str("Buzz"),
                        num(11),
                        str("Fizz"),
                        num(13),
                        num(14),
                        str("FizzBuzz"),
                    ])
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn ski() {
        test(
            r#"
            let s = @(x) { @(y) { @(z) {
                //let f = x(z) f(@(a){ let g = y(z) g(a) })
                let f = x(z)
                f(y(z))
            }}}
            let k = @(x){ @(y) { x } }
            let i = @(x){ x }

            // combine
            @c(l) {
                // extract
                @x(v) {
                    if (Core:type(v) == "arr") { c(v) } else { v }
                }

                // rec
                @r(f, n) {
                    if (n < l.len) {
                        r(f(x(l[n])), (n + 1))
                    } else { f }
                }

                r(x(l[0]), 1)
            }

            let sksik = [s, [k, [s, i]], k]
            c([sksik, "foo", print])
            "#,
            |res| assert_eq!(res, str("foo")),
        )
        .await
        .unwrap();
    }
}
