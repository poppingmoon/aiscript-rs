mod testutils;

use aiscript_v1::{
    Parser,
    ast::*,
    errors::{AiScriptError, AiScriptRuntimeError, AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    values::Value,
};
use testutils::*;

#[tokio::test]
async fn hello_world() {
    test(r#"<: "Hello, world!""#, |res| {
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
                get_count: @() { count },
                count: @() { count = (count + 1) },
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

mod object {
    use super::*;

    #[tokio::test]
    async fn property_access() {
        test(
            r#"
            let obj = {
                a: {
                    b: {
                        c: 42,
                    },
                },
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
                        c: f,
                    },
                },
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
                    ]),
                )
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn string_key() {
        test(
            r#"
            let obj = {
                "藍": 42,
            }

            <: obj."藍"
            "#,
            |res| assert_eq!(res, num(42)),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn string_key_including_colon_and_period() {
        test(
            r#"
            let obj = {
                ":.:": 42,
            }

            <: obj.":.:"
            "#,
            |res| assert_eq!(res, num(42)),
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
            |res| assert_eq!(res, arr([str("ai"), str("taso"), str("kawaii")])),
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
        if let AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange { index, max }) = err {
            assert_eq!(index, 3_f64);
            assert_eq!(max, 2);
        } else {
            panic!("{err}");
        }

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
        if let AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange { index, max }) = err {
            assert_eq!(index, 9_f64);
            assert_eq!(max, 2);
        } else {
            panic!("{err}");
        }
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
        if let AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange { index, max }) = err {
            assert_eq!(index, 1_f64);
            assert_eq!(max, 0);
        } else {
            panic!("{err}");
        }
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
        if let AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange { index, max }) = err {
            assert_eq!(index, 2_f64);
            assert_eq!(max, -1);
        } else {
            panic!("{err}");
        }
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
        if let AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange { index, max }) = err {
            assert_eq!(index, 6.21);
            assert_eq!(max, -1);
        } else {
            panic!("{err}");
        }
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
                    b: [@(name) { name }, @(str) { "chan" }, @() { "kawaii" }],
                },
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
                    b: ["ai", "chan", "kawaii"],
                },
            }

            obj.a.b[1] = "taso"

            <: obj
            "#,
            |res| {
                assert_eq!(
                    res,
                    obj([(
                        "a",
                        obj([("b", arr([str("ai"), str("taso"), str("kawaii")]))]),
                    )]),
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
                    b: ["ai", "chan", "kawaii"],
                },
            }

            var x = null
            x = obj.a.b[1]

            <: x
            "#,
            |res| assert_eq!(res, str("chan")),
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
                    a: 1,
                    b: 2,
                }
            ]

            arr[0].a += 1
            arr[0].b -= 1

            <: arr
            "#,
            |res| assert_eq!(res, arr([obj([("a", num(2)), ("b", num(1))])])),
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
                    b: [1, 2, 3],
                },
            }

            obj.a.b[1] += 1
            obj.a.b[2] -= 1

            <: obj
            "#,
            |res| {
                assert_eq!(
                    res,
                    obj([("a", obj([("b", arr([num(1), num(3), num(2)]))]))]),
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
            |res| assert_eq!(res, arr([str("ai!"), str("chan!"), str("kawaii!")])),
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
        let mut ast = Parser::default()
            .parse(
                r#"
                (a.b).c
                "#,
            )
            .unwrap();
        let line = ast.remove(0);
        if let Node::Expression(Expression::Prop(prop)) = line {
            let Prop { target, name, .. } = *prop;
            assert_eq!(name, "c");
            if let Expression::Prop(prop) = *target {
                let Prop { target, name, .. } = *prop;
                assert_eq!(name, "b");
                if let Expression::Identifier(identifier) = *target {
                    assert_eq!(identifier.name, "a");
                } else {
                    panic!("{target:?}");
                }
            } else {
                panic!("{target:?}");
            }
        } else {
            panic!("{line:?}");
        }
    }

    #[tokio::test]
    async fn index_chain_with_parenthesis() {
        let mut ast = Parser::default()
            .parse(
                r#"
                (a[42]).b
                "#,
            )
            .unwrap();
        let line = ast.remove(0);
        if let Node::Expression(Expression::Prop(prop)) = line {
            let Prop { target, name, .. } = *prop;
            assert_eq!(name, "b");
            if let Expression::Index(index) = *target {
                let Index { target, index, .. } = *index;
                if let Expression::Identifier(identifier) = *target {
                    assert_eq!(identifier.name, "a");
                } else {
                    panic!("{target:?}");
                }
                if let Expression::Num(num) = *index {
                    assert_eq!(num.value, 42.0);
                } else {
                    panic!("{index:?}");
                }
            } else {
                panic!("{target:?}");
            }
        } else {
            panic!("{line:?}");
        }
    }

    #[tokio::test]
    async fn call_chain_with_parenthesis() {
        let mut ast = Parser::default()
            .parse(
                r#"
                (foo(42, 57)).bar
                "#,
            )
            .unwrap();
        let line = ast.remove(0);
        if let Node::Expression(Expression::Prop(prop)) = line {
            let Prop { target, name, .. } = *prop;
            assert_eq!(name, "bar");
            if let Expression::Call(call) = *target {
                let Call { target, args, .. } = *call;
                if let Expression::Identifier(identifier) = *target {
                    assert_eq!(identifier.name, "foo");
                    if let [Expression::Num(num_1), Expression::Num(num_2)] = &args[..] {
                        assert_eq!(num_1.value, 42.0);
                        assert_eq!(num_2.value, 57.0);
                    } else {
                        panic!("{args:?}");
                    }
                } else {
                    panic!("{target:?}");
                }
            } else {
                panic!("{target:?}");
            }
        } else {
            panic!("{line:?}");
        }
    }

    #[tokio::test]
    async fn longer_chain_with_parenthesis() {
        let mut ast = Parser::default()
            .parse(
                r#"
                (a.b.c).d.e
                "#,
            )
            .unwrap();
        let line = ast.remove(0);
        if let Node::Expression(Expression::Prop(prop)) = line {
            let Prop { target, name, .. } = *prop;
            assert_eq!(name, "e");
            if let Expression::Prop(prop) = *target {
                let Prop { target, name, .. } = *prop;
                assert_eq!(name, "d");
                if let Expression::Prop(prop) = *target {
                    let Prop { target, name, .. } = *prop;
                    assert_eq!(name, "c");
                    if let Expression::Prop(prop) = *target {
                        let Prop { target, name, .. } = *prop;
                        assert_eq!(name, "b");
                        if let Expression::Identifier(identifier) = *target {
                            assert_eq!(identifier.name, "a");
                        } else {
                            panic!("{target:?}");
                        }
                    } else {
                        panic!("{target:?}");
                    }
                } else {
                    panic!("{target:?}");
                }
            } else {
                panic!("{target:?}");
            }
        } else {
            panic!("{line:?}");
        }
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
            let Prop { target, name, .. } = *prop;
            assert_eq!(name, "d");
            if let Expression::If(if_) = *target {
                let If {
                    cond, then, else_, ..
                } = *if_;
                if let Expression::Identifier(identifier) = *cond {
                    assert_eq!(identifier.name, "a");
                } else {
                    panic!("{cond:?}");
                }
                if let StatementOrExpression::Expression(Expression::Identifier(identifier)) = *then
                {
                    assert_eq!(identifier.name, "b");
                } else {
                    panic!("{then:?}");
                }
                let else_ = *else_.unwrap();
                if let StatementOrExpression::Expression(Expression::Identifier(identifier)) = else_
                {
                    assert_eq!(identifier.name, "c");
                } else {
                    panic!("{else_:?}");
                }
            } else {
                panic!("{target:?}");
            }
        } else {
            panic!("{line:?}");
        }
    }
}

#[tokio::test]
async fn does_not_throw_error_when_divided_by_zero() {
    test(
        r#"
        <: (0 / 0)
        "#,
        |res| assert!(f64::try_from(res).unwrap().is_nan()),
    )
    .await
    .unwrap();
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
    async fn optional_args() {
        test(
            r#"
            @f(x, y?, z?) {
                [x, y, z]
            }
            <: f(true)
            "#,
            |res| assert_eq!(res, arr([bool(true), null(), null()])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn args_with_default_value() {
        test(
            r#"
            @f(x, y=1, z=2) {
                [x, y, z]
            }
            <: f(5, 3)
            "#,
            |res| assert_eq!(res, arr([num(5), num(3), num(2)])),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn args_must_not_be_both_optional_and_default_valued() {
        let err = Parser::default()
            .parse(
                r#"
                @func(a? = 1){}
                "#,
            )
            .unwrap_err();
        let AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::SeparatorExpected,
            ..
        } = err
        else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn missing_arg() {
        let err = test(
            r#"
            @func(a){}
			func()
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Runtime(AiScriptRuntimeError::ExpectAny) = err else {
            panic!("{err}");
        };
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
        let AiScriptError::Runtime(AiScriptRuntimeError::ExpectAny) = err else {
            panic!("{err}");
        };
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

mod type_declaration {
    use super::*;

    #[tokio::test]
    async fn def() {
        test(
            r#"
            let abc: num = 1
            var xyz: str = "abc"
            <: [abc, xyz]
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

    #[tokio::test]
    async fn def_null() {
        test(
            r#"
            let a: null = null
            <: a
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn fn_def_null() {
        test(
            r#"
            @f(): null {}
            <: f()
            "#,
            |res| assert_eq!(res, null()),
        )
        .await
        .unwrap();
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
            let Definition { dest, attr, .. } = *definition.clone();
            if let Expression::Identifier(identifier) = dest {
                assert_eq!(identifier.name, "onReceived");
            } else {
                panic!("{dest:?}");
            }
            let attr = attr.unwrap();
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
            } else {
                panic!("{attr:?}");
            }
        } else {
            panic!("{nodes:?}");
        }
    }

    #[test]
    fn multiple_attributes_with_function_obj_str_bool() {
        let nodes = Parser::default()
            .parse(
                r#"
                #[Endpoint { path: "/notes/create" }]
                #[Desc "Create a note."]
                #[Cat true]
                @createNote(text) {
                    <: text
                }
                "#,
            )
            .unwrap();
        if let [Node::Statement(Statement::Definition(definition))] = &nodes[..] {
            let Definition { dest, attr, .. } = *definition.clone();
            if let Expression::Identifier(identifier) = dest {
                assert_eq!(identifier.name, "createNote");
            } else {
                panic!("{dest:?}");
            }
            let attr = attr.unwrap();
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
                let obj = obj.value.iter().collect::<Vec<(&String, &Expression)>>();
                if let [(key, Expression::Str(str))] = obj[..] {
                    assert_eq!(key, "path");
                    assert_eq!(str.value, "/notes/create");
                } else {
                    panic!("{obj:?}");
                };
                assert_eq!(name2, "Desc");
                assert_eq!(str.value, "Create a note.");
                assert_eq!(name3, "Cat");
                assert!(bool.value);
            } else {
                panic!("{attr:?}");
            }
        } else {
            panic!("{nodes:?}");
        }
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
            let Definition { dest, attr, .. } = *definition.clone();
            if let Expression::Identifier(identifier) = dest {
                assert_eq!(identifier.name, "data");
            } else {
                panic!("{dest:?}");
            }
            let attr = attr.unwrap();
            if let [
                Attribute {
                    name,
                    value: Expression::Bool { .. },
                    ..
                },
            ] = &attr[..]
            {
                assert_eq!(name, "serializable");
            } else {
                panic!("{attr:?}");
            }
        } else {
            panic!("{nodes:?}");
        }
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
            assert_eq!(
                definition.loc,
                Loc {
                    start: Pos { line: 2, column: 4 },
                    end: Pos {
                        line: 2,
                        column: 15
                    }
                },
            );
        } else {
            panic!("{nodes:?}");
        }
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
            assert_eq!(
                definition.loc,
                Loc {
                    start: Pos { line: 5, column: 3 },
                    end: Pos {
                        line: 5,
                        column: 14
                    }
                },
            );
        } else {
            panic!("{nodes:?}");
        }
    }

    #[test]
    fn template() {
        let nodes = Parser::default()
            .parse(
                r#"
			`hoge{1}fuga`
                "#,
            )
            .unwrap();
        if let [Node::Expression(Expression::Tmpl(tmpl))] = &nodes[..] {
            let Tmpl { loc, tmpl } = *tmpl.clone();
            assert_eq!(
                loc,
                Loc {
                    start: Pos { line: 2, column: 4 },
                    end: Pos {
                        line: 2,
                        column: 17
                    }
                },
            );
            if let [
                Expression::Str(elem1),
                Expression::Num(elem2),
                Expression::Str(elem3),
            ] = &tmpl[..]
            {
                assert_eq!(
                    elem1.loc,
                    Loc {
                        start: Pos { line: 2, column: 5 },
                        end: Pos {
                            line: 2,
                            column: 10,
                        },
                    },
                );
                assert_eq!(
                    elem2.loc,
                    Loc {
                        start: Pos {
                            line: 2,
                            column: 10,
                        },
                        end: Pos {
                            line: 2,
                            column: 11,
                        },
                    },
                );
                assert_eq!(
                    elem3.loc,
                    Loc {
                        start: Pos {
                            line: 2,
                            column: 12,
                        },
                        end: Pos {
                            line: 2,
                            column: 17,
                        },
                    },
                );
            } else {
                panic!("{tmpl:?}");
            }
        } else {
            panic!("{nodes:?}");
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
                    ]),
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
                    ]),
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
        if let AiScriptError::Syntax(AiScriptSyntaxError {
            kind: AiScriptSyntaxErrorKind::ReservedWord(value),
            ..
        }) = err
        {
            assert_eq!(value, "constructor");
        } else {
            panic!("{err}");
        }

        let err = test(
            r#"
            <: prototype
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::NoSuchVariable { name, .. }) = err {
            assert_eq!(name, "prototype");
        } else {
            panic!("{err}");
        }

        let err = test(
            r#"
            <: __proto__
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::NoSuchVariable { name, .. }) = err {
            assert_eq!(name, "__proto__");
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn cannot_access_js_native_property_via_object() {
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
        if let AiScriptError::Runtime(AiScriptRuntimeError::NoSuchProperty { name, target_type }) =
            err
        {
            assert_eq!(name, "constructor");
            assert_eq!(target_type, "str");
        } else {
            panic!("{err}");
        }

        let err = test(
            r#"
            <: "".prototype
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::NoSuchProperty { name, target_type }) =
            err
        {
            assert_eq!(name, "prototype");
            assert_eq!(target_type, "str");
        } else {
            panic!("{err}");
        }

        let err = test(
            r#"
            <: "".__proto__
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Runtime(AiScriptRuntimeError::NoSuchProperty { name, target_type }) =
            err
        {
            assert_eq!(name, "__proto__");
            assert_eq!(target_type, "str");
        } else {
            panic!("{err}");
        }
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
                    ]),
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
