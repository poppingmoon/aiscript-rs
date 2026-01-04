#![allow(uncommon_codepoints, non_snake_case)]

use aiscript_v1::{
    Parser,
    errors::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
};

use paste::paste;

macro_rules! identifier_validation {
    ($(($name:ident, $code:expr$(,)?)),*$(,)?) => {
        $(mod $name {
            use super::*;

            reserved_word_must_be_rejected! {
                ($code, "null"),
                ($code, "true"),
                ($code, "false"),
                ($code, "each"),
                ($code, "for"),
                ($code, "do"),
                ($code, "while"),
                ($code, "loop"),
                ($code, "break"),
                ($code, "continue"),
                ($code, "match"),
                ($code, "case"),
                ($code, "default"),
                ($code, "if"),
                ($code, "elif"),
                ($code, "else"),
                ($code, "return"),
                ($code, "eval"),
                ($code, "var"),
                ($code, "let"),
                ($code, "exists"),
                ($code, "as"),
                ($code, "async"),
                ($code, "attr"),
                ($code, "attribute"),
                ($code, "await"),
                ($code, "catch"),
                ($code, "class"),
                ($code, "component"),
                ($code, "constructor"),
                ($code, "dictionary"),
                ($code, "enum"),
                ($code, "export"),
                ($code, "finally"),
                ($code, "fn"),
                ($code, "hash"),
                ($code, "in"),
                ($code, "interface"),
                ($code, "out"),
                ($code, "private"),
                ($code, "public"),
                ($code, "ref"),
                ($code, "static"),
                ($code, "struct"),
                ($code, "table"),
                ($code, "this"),
                ($code, "throw"),
                ($code, "trait"),
                ($code, "try"),
                ($code, "undefined"),
                ($code, "use"),
                ($code, "using"),
                ($code, "when"),
                ($code, "yield"),
                ($code, "import"),
                ($code, "is"),
                ($code, "meta"),
                ($code, "module"),
                ($code, "namespace"),
                ($code, "new"),
            }

            wordcat_must_be_allowed! {
                ($code, "null"),
                ($code, "true"),
                ($code, "false"),
                ($code, "each"),
                ($code, "for"),
                ($code, "do"),
                ($code, "while"),
                ($code, "loop"),
                ($code, "break"),
                ($code, "continue"),
                ($code, "match"),
                ($code, "case"),
                ($code, "default"),
                ($code, "if"),
                ($code, "elif"),
                ($code, "else"),
                ($code, "return"),
                ($code, "eval"),
                ($code, "var"),
                ($code, "let"),
                ($code, "exists"),
                ($code, "as"),
                ($code, "async"),
                ($code, "attr"),
                ($code, "attribute"),
                ($code, "await"),
                ($code, "catch"),
                ($code, "class"),
                ($code, "component"),
                ($code, "constructor"),
                ($code, "dictionary"),
                ($code, "enum"),
                ($code, "export"),
                ($code, "finally"),
                ($code, "fn"),
                ($code, "hash"),
                ($code, "in"),
                ($code, "interface"),
                ($code, "out"),
                ($code, "private"),
                ($code, "public"),
                ($code, "ref"),
                ($code, "static"),
                ($code, "struct"),
                ($code, "table"),
                ($code, "this"),
                ($code, "throw"),
                ($code, "trait"),
                ($code, "try"),
                ($code, "undefined"),
                ($code, "use"),
                ($code, "using"),
                ($code, "when"),
                ($code, "yield"),
                ($code, "import"),
                ($code, "is"),
                ($code, "meta"),
                ($code, "module"),
                ($code, "namespace"),
                ($code, "new"),
            }

            unicode_identifier_case!($code, "A", true);
            unicode_identifier_case!($code, "Ω", false);
            unicode_identifier_case!($code, "a", true);
            unicode_identifier_case!($code, "β", false);
            unicode_identifier_case!($code, "ǅ", false);
            unicode_identifier_case!($code, "ǈ", false);
            unicode_identifier_case!($code, "ʰ", false);
            unicode_identifier_case!($code, "々", false);
            unicode_identifier_case!($code, "あ", false);
            unicode_identifier_case!($code, "藍", false);
            unicode_identifier_case!($code, "𠮷", false);
            unicode_identifier_case!($code, "ᛮ", false);
            unicode_identifier_case!($code, "Ⅳ", false);
            unicode_identifier_case!($code, "$", false, name = dollar_sign);
            unicode_identifier_case!($code, "_", true);
            unicode_identifier_case!($code, "_A", true);
            unicode_identifier_case!($code, "_Ω", false);
            unicode_identifier_case!($code, "_a", true);
            unicode_identifier_case!($code, "_β", false);
            unicode_identifier_case!($code, "_ǅ", false);
            unicode_identifier_case!($code, "_ǈ", false);
            unicode_identifier_case!($code, "_ʰ", false);
            unicode_identifier_case!($code, "_々", false);
            unicode_identifier_case!($code, "_あ", false);
            unicode_identifier_case!($code, "_藍", false);
            unicode_identifier_case!($code, "_𠮷", false);
            unicode_identifier_case!($code, "_ᛮ", false);
            unicode_identifier_case!($code, "_Ⅳ", false);
            unicode_identifier_case!($code, "_$", false, name = _dollar_sign);
            unicode_identifier_case!($code, "__", true);
            unicode_identifier_case!($code, "á", false);
            unicode_identifier_case!($code, "राम", false);
            unicode_identifier_case!($code, "a0", true);
            unicode_identifier_case!($code, "a๑", false);
            unicode_identifier_case!($code, "a‿b", false);
            unicode_identifier_case!($code, "बि‌ना", false);
            unicode_identifier_case!($code, "क‍्", false);
            unicode_identifier_case!($code, "\\u", false, name = u);
            unicode_identifier_case!($code, "\\u000x", false, name = u000x);
            unicode_identifier_case!($code, "\\u0021", false, name = u0021);
            unicode_identifier_case!($code, "\\u0069\\u0066", false, name = u0069_u0066);
            unicode_identifier_case!($code, "\\ud83e\\udd2f", false, name = ud83e_udd2f);
            unicode_identifier_case!($code, "\\uD83E\\uDD2F", false, name = uD83E_uDD2F);
            unicode_identifier_case!($code, "_\\u", false, name = _u);
            unicode_identifier_case!($code, "_\\u000x", false, name = _u000x);
            unicode_identifier_case!($code, "_\\u0021", false, name = _u0021);
            unicode_identifier_case!($code, "_\\ud83e\\udd2f", false, name = _ud83e_udd2f);
            unicode_identifier_case!($code, "_\\uD83E\\uDD2F", false, name = _uD83E_uDD2F);

            escape_sequence_is_not_allowed! {
                ($code, "\\u0041", name = u0041),
                ($code, "\\u85cd", name = u85cd),
                ($code, "\\u85CD", name = u85CD),
                ($code, "\\ud842\\udfb7", name = ud842_udfb7),
                ($code, "\\uD842\\uDFB7", name = uD842_uDFB7),
                ($code, "_\\u0041", name = _u0041),
                ($code, "_\\u85cd", name = _u85cd),
                ($code, "_\\u85CD", name = _u85CD),
                ($code, "_\\ud842\\udfb7", name = _ud842_udfb7),
                ($code, "_\\uD842\\uDFB7", name = _uD842_uDFB7),
            }
        })*
    }
}

macro_rules! identifier_validation_on_obj_key {
    ($(($name:ident, $code:expr$(,)?)),*$(,)?) => {
        $(mod $name {
            use super::*;

            reserved_word_must_be_allowed! {
                ($code, "null"),
                ($code, "true"),
                ($code, "false"),
                ($code, "each"),
                ($code, "for"),
                ($code, "do"),
                ($code, "while"),
                ($code, "loop"),
                ($code, "break"),
                ($code, "continue"),
                ($code, "match"),
                ($code, "case"),
                ($code, "default"),
                ($code, "if"),
                ($code, "elif"),
                ($code, "else"),
                ($code, "return"),
                ($code, "eval"),
                ($code, "var"),
                ($code, "let"),
                ($code, "exists"),
                ($code, "as"),
                ($code, "async"),
                ($code, "attr"),
                ($code, "attribute"),
                ($code, "await"),
                ($code, "catch"),
                ($code, "class"),
                ($code, "component"),
                ($code, "constructor"),
                ($code, "dictionary"),
                ($code, "enum"),
                ($code, "export"),
                ($code, "finally"),
                ($code, "fn"),
                ($code, "hash"),
                ($code, "in"),
                ($code, "interface"),
                ($code, "out"),
                ($code, "private"),
                ($code, "public"),
                ($code, "ref"),
                ($code, "static"),
                ($code, "struct"),
                ($code, "table"),
                ($code, "this"),
                ($code, "throw"),
                ($code, "trait"),
                ($code, "try"),
                ($code, "undefined"),
                ($code, "use"),
                ($code, "using"),
                ($code, "when"),
                ($code, "yield"),
                ($code, "import"),
                ($code, "is"),
                ($code, "meta"),
                ($code, "module"),
                ($code, "namespace"),
                ($code, "new"),
            }
        })*
    };
}

macro_rules! reserved_word_must_be_rejected {
    ($(($code:expr, $word:expr)),*$(,)?) => {
        paste! {
            $(#[test]
            fn [<$word _must_be_rejected>]() {
                let err = Parser::default()
                    .parse(&format!($code, name = $word))
                    .unwrap_err();
                assert!(matches!(
                    err,
                    AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::MultipleStatements
                            | AiScriptSyntaxErrorKind::ReservedWord(_)
                            | AiScriptSyntaxErrorKind::UnexpectedToken(_),
                        ..
                    }
                ));
            })*
        }
    };
}

macro_rules! wordcat_must_be_allowed {
    ($(($code:expr, $word:expr)),*$(,)?) => {
        paste! {
            $(#[test]
            fn [<$word cat_must_be_allowed>]() {
                let wordcat = $word.to_string() + "cat";
                Parser::default()
                    .parse(&format!($code, name = wordcat))
                    .unwrap();
            })*
        }
    };
}

macro_rules! unicode_identifier_case {
    ($code:expr, $word:expr, true) => {
        paste! {
            #[test]
            fn [<$word _must_be_allowed>]() {
                Parser::default()
                    .parse(&format!($code, name = $word))
                    .unwrap();
            }
        }
    };

    ($code:expr, $word:expr, true, name = $name:ident) => {
        paste! {
            #[test]
            fn [<$name _must_be_allowed>]() {
                Parser::default()
                    .parse(&format!($code, name = $word))
                    .unwrap();
            }
        }
    };

    ($code:expr, $word:expr, false) => {
        paste! {
            #[test]
            fn [<$word _must_be_rejected>]() {
                let err = Parser::default()
                    .parse(&format!($code, name = $word))
                    .unwrap_err();
                assert!(matches!(
                    err,
                    AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::InvalidCharacter(_)
                            | AiScriptSyntaxErrorKind::UnexpectedToken(_),
                        ..
                    }
                ));
            }
        }
    };

    ($code:expr, $word:expr, false, name = $name:ident) => {
        paste! {
            #[test]
            fn [<$name _must_be_rejected>]() {
                let err = Parser::default()
                    .parse(&format!($code, name = $word))
                    .unwrap_err();
                assert!(matches!(
                    err,
                    AiScriptSyntaxError {
                        kind: AiScriptSyntaxErrorKind::InvalidCharacter(_)
                            | AiScriptSyntaxErrorKind::UnexpectedToken(_),
                        ..
                    }
                ));
            }
        }
    };
}

macro_rules! escape_sequence_is_not_allowed {
    ($(($code:expr, $word:expr, name = $name:ident)),*$(,)?) => {
        paste! {
            $(#[test]
            fn [<escape_sequence_is_not_allowed_ $name>]() {
                let err = Parser::default()
                    .parse(&format!($code, name = $word))
                    .unwrap_err();
                if let AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::UnexpectedToken(value),
                    ..
                } = err
                {
                    assert_eq!(value, "BackSlash");
                    return;
                }
                panic!();
            })*
        }
    };
}

macro_rules! reserved_word_must_be_allowed {
    ($(($code:expr, $word:expr)),*$(,)?) => {
        paste! {
            $(#[test]
            fn [<reserved_word_ $word _must_be_allowed>]() {
                Parser::default()
                    .parse(&format!($code, name = $word))
                    .unwrap();
            })*
        }
    };
}

mod identifier_validation {
    use super::*;

    identifier_validation! {
        (
            variable,
            r#"
            let {name} = "ai"
            <: {name}
            "#,
        ),
        (
            function,
            r#"
            @{name}() {{ 'ai' }}
            <: {name}()
            "#,
        ),
        (
            attribute,
            r#"
            #[{name} 1]
            @f() {{ 1 }}
            "#,
        ),
        (
            namespace,
            r#"
            :: {name} {{
                @f() {{ 1 }}
            }}
            <: {name}:f()
            "#,
        ),
        (
            meta,
            r#"
            ### {name} 1
            "#,
        ),
        (
            for_break,
            r#"
            #{name}: for 1 {{
                break #{name}
            }}
            "#,
        ),
        (
            each_break,
            r#"
            #{name}: each let v, [0] {{
                break #{name}
            }}
            "#,
        ),
        (
            while_break,
            r#"
            #{name}: while false {{
                break #{name}
            }}
            "#,
        ),
        (
            for_continue,
            r#"
            #{name}: for 1 {{
                continue #{name}
            }}
            "#,
        ),
        (
            each_continue,
            r#"
            #{name}: each let v, [0] {{
                break #{name}
            }}
            "#,
        ),
        (
            while_continue,
            r#"
            var flag = true
            #{name}: while flag {{
                flag = false
                continue #{name}
            }}
            "#,
        ),
        (
            type_param,
            r#"
            @f<{name}>(x): {name} {{ x }}
            "#,
        ),
    }
}

mod identifier_validation_on_obj_key {
    use super::*;

    identifier_validation_on_obj_key! {
        (
            literal,
            r#"
            let x = {{ {name}: 1 }}
            <: x["{name}"]
            "#,
        ),
        (
            prop,
            r#"
            let x = {{}}
            x."{name}" = 1
            <: x."{name}"
            "#,
        ),
    }
}

#[test]
fn keyword_cannot_contain_escape_characters() {
    let err = Parser::default()
        .parse(
            r#"
            \\u0069\\u0066 true {
                <: 1
            }
            "#,
        )
        .unwrap_err();
    if let AiScriptSyntaxError {
        kind: AiScriptSyntaxErrorKind::UnexpectedToken(value),
        ..
    } = err
    {
        assert_eq!(value, "BackSlash");
        return;
    }
    panic!();
}
