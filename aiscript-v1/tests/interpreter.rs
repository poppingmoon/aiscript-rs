mod testutils;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use aiscript_v1::{
    Interpreter, Parser,
    ast::Pos,
    errors::{AiScriptError, AiScriptNamespaceError, AiScriptRuntimeError},
    values::Value,
};
use futures::FutureExt;
use testutils::*;

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

mod error_handler {
    use super::*;

    #[tokio::test]
    async fn emit_error() {
        let err_count = Arc::new(AtomicUsize::new(0));
        let aiscript = Interpreter::new(
            [(
                "emitError".to_string(),
                Value::fn_native_sync(|_| Err(AiScriptError::internal("emitError"))),
            )],
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            Some({
                let err_count = err_count.clone();
                move |_| {
                    err_count.fetch_add(1, Ordering::Relaxed);
                    async {}.boxed()
                }
            }),
            None,
        );
        aiscript
            .exec(Parser::default().parse("emitError()").unwrap())
            .await
            .unwrap();
        assert_eq!(err_count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn array_map_calls_the_handler_just_once() {
        let err_count = Arc::new(AtomicUsize::new(0));
        let aiscript = Interpreter::new(
            [],
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            Some({
                let err_count = err_count.clone();
                move |_| {
                    err_count.fetch_add(1, Ordering::Relaxed);
                    async {}.boxed()
                }
            }),
            None,
        );
        aiscript
            .exec(
                Parser::default()
                    .parse("Core:range(1,5).map(@(){ hoge })")
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(err_count.load(Ordering::Relaxed), 1);
    }
}

mod error_location {

    use super::*;

    #[tokio::test]
    async fn non_aiscript_error() {
        let aiscript = Interpreter::new(
            [(
                "emitError".to_string(),
                Value::fn_native_sync(|_| Err(AiScriptError::internal("emitError"))),
            )],
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            None,
        );
        let err = aiscript
            .exec(Parser::default().parse("emitError()").unwrap())
            .await
            .unwrap_err();
        let AiScriptError::Internal(_) = err else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn no_var_in_namespace_declaration() {
        let err = test(
            r#"// vの位置
			:: Ai {
				let chan = 'kawaii'
				var kun = '!?'
			}
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        if let AiScriptError::Namespace(AiScriptNamespaceError { pos, .. }) = err {
            assert_eq!(pos, Pos { line: 4, column: 5 });
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn index_out_of_range() {
        let err = test(
            r#"// [の位置
			let arr = []
			arr[0]
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Runtime(AiScriptRuntimeError::IndexOutOfRange { .. }) = err else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn error_in_passed_function() {
        let err = test(
            r#"// (の位置
			[1, 2, 3].map(@(v){
				if v==1 Core:abort("error")
			})
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Runtime(AiScriptRuntimeError::User(_)) = err else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn no_such_prop() {
        let err = test(
            r#"// .の位置
			[].ai
            "#,
            |_| {},
        )
        .await
        .unwrap_err();
        let AiScriptError::Runtime(AiScriptRuntimeError::NoSuchProperty { .. }) = err else {
            panic!("{err}");
        };
    }
}

mod callstack {
    use super::*;

    #[tokio::test]
    async fn error_in_function() {
        let aiscript = Interpreter::new(
            [(
                "emitError".to_string(),
                Value::fn_native_sync(|_| Err(AiScriptError::internal("emitError"))),
            )],
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            None,
        );
        let err = aiscript
            .exec(
                Parser::default()
                    .parse(
                        r#"
			@function1() { emitError() }
			@function2() { function1() }
			function2()
                        "#,
                    )
                    .unwrap(),
            )
            .await
            .unwrap_err();
        let AiScriptError::Internal(_) = err else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn error_in_function_in_namespace() {
        let aiscript = Interpreter::new(
            [(
                "emitError".to_string(),
                Value::fn_native_sync(|_| Err(AiScriptError::internal("emitError"))),
            )],
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            None,
        );
        let err = aiscript
            .exec(
                Parser::default()
                    .parse(
                        r#"
			:: Ai {
				@function() { emitError() }
			}
			Ai:function()
                        "#,
                    )
                    .unwrap(),
            )
            .await
            .unwrap_err();
        let AiScriptError::Internal(_) = err else {
            panic!("{err}");
        };
    }

    #[tokio::test]
    async fn error_in_anonymous_function() {
        let aiscript = Interpreter::new(
            [(
                "emitError".to_string(),
                Value::fn_native_sync(|_| Err(AiScriptError::internal("emitError"))),
            )],
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            None::<fn(_) -> _>,
            None,
        );
        let err = aiscript
            .exec(
                Parser::default()
                    .parse(
                        r#"
			(@() { emitError() })()
                        "#,
                    )
                    .unwrap(),
            )
            .await
            .unwrap_err();
        let AiScriptError::Internal(_) = err else {
            panic!("{err}");
        };
    }
}

mod attribute {
    use aiscript_v1::values::Attr;

    use super::*;

    #[tokio::test]
    async fn no_attribute() {
        let aiscript = Interpreter::default();
        aiscript
            .exec(
                Parser::default()
                    .parse(
                        r#"
                        @f() {}
                        "#,
                    )
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(aiscript.scope.get("f").await.unwrap().attr, None);
    }

    #[tokio::test]
    async fn single_attribute() {
        let aiscript = Interpreter::default();
        aiscript
            .exec(
                Parser::default()
                    .parse(
                        r#"
                        #[x 42]
                        @f() {}
                        "#,
                    )
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            aiscript.scope.get("f").await.unwrap().attr.unwrap()[..],
            [Attr {
                name: "x".to_string(),
                value: num(42),
            }],
        );
    }

    #[tokio::test]
    async fn multiple_attributes() {
        let aiscript = Interpreter::default();
        aiscript
            .exec(
                Parser::default()
                    .parse(
                        r#"
                        #[o { a: 1, b: 2 }]
                        #[s "ai"]
                        #[b false]
                        @f() {}
                        "#,
                    )
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            aiscript.scope.get("f").await.unwrap().attr.unwrap()[..],
            [
                Attr {
                    name: "o".to_string(),
                    value: obj([("a", num(1)), ("b", num(2))]),
                },
                Attr {
                    name: "s".to_string(),
                    value: str("ai"),
                },
                Attr {
                    name: "b".to_string(),
                    value: bool(false),
                },
            ],
        );
    }

    #[tokio::test]
    async fn single_attribute_without_value() {
        let aiscript = Interpreter::default();
        aiscript
            .exec(
                Parser::default()
                    .parse(
                        r#"
                        #[x]
                        @f() {}
                        "#,
                    )
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            aiscript.scope.get("f").await.unwrap().attr.unwrap()[..],
            [Attr {
                name: "x".to_string(),
                value: bool(true),
            }],
        );
    }

    #[tokio::test]
    async fn attribute_under_namespacee() {
        let aiscript = Interpreter::default();
        aiscript
            .exec(
                Parser::default()
                    .parse(
                        r#"
                        :: Ns {
                            #[x 42]
                            @f() {}
                        }
                        "#,
                    )
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            aiscript.scope.get("Ns:f").await.unwrap().attr.unwrap()[..],
            [Attr {
                name: "x".to_string(),
                value: num(42),
            }],
        );
    }
}
