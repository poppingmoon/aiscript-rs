use indexmap::IndexMap;

use crate::node::Loc;

use super::node::*;

peg::parser! {
    pub grammar parser() for str {
        //
        // preprocessor
        //

        pub rule preprocess() -> String
            = s:preprocess_part()* { s.join("") }

        rule preprocess_part() -> String
            = text:$(tmpl()) { text.to_string() }
            / text:$(str()) { text.to_string() }
            / comment()
            / c:[_] { c.to_string() }

        rule comment() -> String
            = text:$("//" (!eol() [_])*) { " ".repeat(text.len()) }
            / text:$("/*" (!"*/" [_])* "*/") { text.replace( |c| c != '\n', " ") }

        //
        // main parser
        //

        pub rule main() -> Vec<Node>
            = _* content:global_statements()? _* { content.unwrap_or_default() }

        rule global_statements() -> Vec<Node>
            = global_statement() ++ (__* lf() _*)

        rule namespace_statements() -> Vec<DefinitionOrNamespace>
            = namespace_statement() ++ (__* lf() _*)

        rule statements() -> Vec<StatementOrExpression>
            = statement() ++ (__* lf() _*)

        // list of global statements

        rule global_statement() -> Node
            = namespace:namespace() { Node::Namespace(namespace) }  // "::"
            / meta:meta() { Node::Meta(meta) }                      // "###"
            / statement:statement() { statement.into() }

        // list of namespace statement

        rule namespace_statement() -> DefinitionOrNamespace
            = var_def:var_def() { DefinitionOrNamespace::Definition(var_def) }
            / fn_def:fn_def() { DefinitionOrNamespace::Definition(fn_def) }
            / namespace:namespace() { DefinitionOrNamespace::Namespace(namespace) }

        // list of statement

        rule statement() -> StatementOrExpression
            = var_def:var_def() { StatementOrExpression::Statement(Statement::Definition(var_def)) }         // "let" NAME | "var" NAME
            / fn_def:fn_def() { StatementOrExpression::Statement(Statement::Definition(fn_def)) }            // "@"
            / out:out() { StatementOrExpression::Expression(Expression::Identifier(out)) }                   // "<:"
            / return_:return() { StatementOrExpression::Statement(Statement::Return(return_)) }              // "return"
            / attr:attr() { StatementOrExpression::Statement(Statement::Attribute(attr)) }                   // "+"
            / each:each() { StatementOrExpression::Statement(Statement::Each(each)) }                        // "each"
            / for_:for() { StatementOrExpression::Statement(Statement::For(for_)) }                          // "for"
            / loop_:loop() { StatementOrExpression::Statement(Statement::Loop(loop_)) }                      // "loop"
            / break_:break() { StatementOrExpression::Statement(Statement::Break(break_)) }                  // "break"
            / continue_:continue() { StatementOrExpression::Statement(Statement::Continue(continue_)) }      // "continue"
            / add_assign:add_assign() { StatementOrExpression::Statement(Statement::AddAssign(add_assign)) } // Expr "+="
            / sub_assign:sub_assign() { StatementOrExpression::Statement(Statement::SubAssign(sub_assign)) } // Expr "-="
            / assign:assign() { StatementOrExpression::Statement(Statement::Assign(assign)) }                // Expr "="
            / expr:expr() { StatementOrExpression::Expression(expr) }

        // list of expression

        #[cache]
        rule expr() -> Expression
            = start:position!() expression:(precedence! {
                left:(@) infix_sp()* start:position!() "&&" end:position!() infix_sp()* right:@ {
                    (
                        Expression::And(
                            And {
                                left: left.0.into(),
                                right: right.0.into(),
                                operator_loc: Loc{ start, end: end - 1 },
                                loc: None,
                            }
                        ),
                        true,
                    )
                }
                left:(@) infix_sp()* start:position!() "||" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Or(
                            Or {
                                left: left.0.into(),
                                right: right.0.into(),
                                operator_loc: Loc{ start, end: end - 1 },
                                loc: None,
                            }
                        ),
                        true,
                    )
                }
                --
                left:(@) infix_sp()* start:position!() "<=" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:lteq".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                left:(@) infix_sp()* start:position!() ">=" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:gteq".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                --
                left:(@) infix_sp()* start:position!() "==" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:eq".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                left:(@) infix_sp()* start:position!() "!=" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:neq".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                left:(@) infix_sp()* start:position!() "<" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:lt".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                left:(@) infix_sp()* start:position!() ">" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:gt".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                --
                left:(@) infix_sp()* start:position!() "+" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:add".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                left:(@) infix_sp()* start:position!() "-" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:sub".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                --
                left:(@) infix_sp()* start:position!() "*" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:mul".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                left:(@) infix_sp()* start:position!() "^" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:pow".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                left:(@) infix_sp()* start:position!() "/" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:div".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                left:(@) infix_sp()* start:position!() "%" end:position!() infix_sp()* right:@ {
                    (
                        Expression::Identifier(
                            Identifier {
                                name: "Core:mod".to_string(),
                                chain: Some(
                                    vec![ChainMember::CallChain(CallChain {
                                        args: vec![left.0, right.0],
                                        loc: None,
                                    })]
                                ),
                                loc: Some(Loc{ start, end: end - 1 }),
                            }
                        ),
                        true,
                    )
                }
                --
                e:expr2() { (e, false) }
            }) end:position!() {
                match expression {
                    (Expression::Identifier(identifier), true) => Expression::Identifier(
                        Identifier {
                            chain: identifier.chain.map(|mut chain| {
                                if let Some(ChainMember::CallChain(call_chain)) = chain.first() {
                                    chain[0] = ChainMember::CallChain(
                                        CallChain {
                                            args: call_chain.args.clone(),
                                            loc: call_chain.loc.clone().or(Some(Loc { start, end })),
                                        }
                                    );
                                }
                                chain
                            }),
                            ..identifier
                        }
                    ),
                    (Expression::And(and), true) => Expression::And(
                        And {
                            loc: and.loc.clone().or(Some(Loc { start, end })),
                            ..and
                        }
                    ),
                    (Expression::Or(or), true) => Expression::Or(
                        Or {
                            loc: or.loc.clone().or(Some(Loc { start, end })),
                            ..or
                        }
                    ),
                    (expression, _) => expression,
                }
            }

        rule expr2() -> Expression
            = if_:if() { Expression::If(if_) } // "if"
            / fn_:fn() { Expression::Fn(fn_) } // "@("
            / chain()                          // Expr3 "(" | Expr3 "[" | Expr3 "."
            / expr3()

        rule expr3() -> Expression
            = match_:match() { Expression::Match(match_) }                   // "match"
            / eval:eval() { Expression::Block(eval) }                        // "eval"
            / exists:exists() { Expression::Exists(exists) }                 // "exists"
            / tmpl:tmpl() { Expression::Tmpl(tmpl) }                         // "`"
            / str:str() { Expression::Str(str) }                             // "\""
            / num:num() { Expression::Num(num) }                             // "+" | "-" | "1"~"9"
            / bool:bool() { Expression::Bool(bool) }                         // "true" | "false"
            / null:null() { Expression::Null(null) }                         // "null"
            / obj:obj() { Expression::Obj(obj) }                             // "{"
            / arr:arr() { Expression::Arr(arr) }                             // "["
            / not:not() { Expression::Not(not) }                             // "!"
            / identifier:identifier() { Expression::Identifier(identifier) } // NAME_WITH_NAMESPACE
            / "(" _* e:expr() _* ")" { e }

        // list of static literal

        rule static_literal() -> Expression
            = num:num() {Expression::Num(num)}
            / str:str() {Expression::Str(str)}
            / bool:bool() {Expression::Bool(bool)}
            / static_arr:static_arr() {Expression::Arr(static_arr)}
            / static_obj:static_obj() {Expression::Obj(static_obj)}
            / null:null() {Expression::Null(null)}

        //
        // global statements ---------------------------------------------------------------------
        //

        // namespace statement

        rule namespace() -> Namespace
            = start:position!() "::" _+ name:name() _+ "{" _* members:namespace_statements()? _* "}" end:position!() {
                Namespace {
                    name,
                    members: members.unwrap_or_default(),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // meta statement

        rule meta() -> Meta
            = start:position!() "###" __* name:name() _* value:static_literal() end:position!() {
                Meta {
                    name: Some(name),
                    value,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }
            / start:position!() "###" __* value:static_literal() end:position!() {
                Meta {
                    name: None,
                    value,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        //
        // statements ----------------------------------------------------------------------------
        //

        // define statement

        rule var_def() -> Definition
            = start:position!() "let" _+ name:name() type_:(_* ":" _* type_:type_() { type_ })? _* "=" _* expr:expr() end:position!() {
                Definition {
                    name,
                    var_type: type_,
                    expr,
                    mut_: false,
                    attr: Some(Vec::new()),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }
            / start:position!() "var" _+ name:name() type_:(_* ":" _* type_:type_() { type_ })? _* "=" _* expr:expr() end:position!() {
                Definition {
                    name,
                    var_type: type_,
                    expr,
                    mut_: true,
                    attr: Some(Vec::new()),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // output statement

        // NOTE: out is syntax sugar for print(expr)
        rule out() -> Identifier
            = start:position!() "<:" _* expr:expr() end:position!() {
                Identifier {
                    name: "print".to_string(),
                    chain: Some(
                        vec![
                            ChainMember::CallChain(
                                CallChain { args: vec![expr], loc: Some(Loc{ start, end: end - 1 }) },
                            ),
                        ],
                    ),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // attribute statement

        // Note: Attribute will be combined with def node when parsing is complete.
        rule attr() -> Attribute
            = start:position!() "#[" _* name:name() value:(_* value:static_literal() { value })? _* "]" end:position!() {
                Attribute {
                    name,
                    value: value.unwrap_or_else(|| Expression::Bool(Bool { value: true, chain: None, loc: None})),
                    loc: Some(Loc{ start, end: end - 1 })
                }
            }

        // each statement

        rule each() -> Each
            = start:position!() "each" _* "(" "let" _+ varn:name() _* ","? _* items:expr() ")" _* x:block_or_statement() end:position!() {
                Each {
                    var: varn,
                    items,
                    for_: x.into(),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }
            / start:position!() "each" _+ "let" _+ varn:name() _* ","? _* items:expr() _+ x:block_or_statement() end:position!() {
                Each {
                    var: varn,
                    items,
                    for_: x.into(),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // for statement

        rule for() -> For
        = start:position!() "for" _* "(" "let" _+ varn:name() _* from_:("=" _* v:expr() { v })? ","? _* to:expr() ")" _* x:block_or_statement() end:position!() {
            For {
                var: Some(varn),
                from: Some(from_.unwrap_or_else(|| Expression::Num(Num { value: 0.0, chain: None, loc: None }))),
                to: Some(to),
                times: None,
                for_: x.into(),
                loc: Some(Loc{ start, end: end - 1 }),
            }
        }
        / start:position!() "for" _+ "let" _+ varn:name() _* from_:("=" _* v:expr() { v })? ","? _* to:expr() _+ x:block_or_statement() end:position!() {
            For {
                var: Some(varn),
                from: Some(from_.unwrap_or_else(|| Expression::Num(Num { value: 0.0, chain: None, loc: None }))),
                to: Some(to),
                times: None,
                for_: x.into(),
                loc: Some(Loc{ start, end: end - 1 }),
            }
        }
        / start:position!() "for" _* "(" times:expr() ")" _* x:block_or_statement() end:position!() {
            For {
                var: None,
                from: None,
                to: None,
                times: Some(times),
                for_: x.into(),
                loc: Some(Loc{ start, end: end - 1 }),
            }
        }
        / start:position!() "for" _+ times:expr() _+ x:block_or_statement() end:position!() {
            For {
                var: None,
                from: None,
                to: None,
                times: Some(times),
                for_: x.into(),
                loc: Some(Loc{ start, end: end - 1 }),
            }
        }

        rule return() -> Return
            = start:position!() "return" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':'] _* expr:expr() end:position!() {
                Return {
                    expr,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule loop() -> Loop
            = start:position!() "loop" _* "{" _* s:statements() _* "}" end:position!() {
                Loop {
                    statements: s,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule break() -> Break
            = start:position!() "break" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':'] end:position!() {
                Break {
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule continue() -> Continue
            = start:position!() "continue" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':'] end:position!() {
                Continue {
                    loc: Some(Loc{ start, end: end - 1 })
                }
            }

        rule add_assign() -> AddAssign
            = start:position!() dest:expr() _* "+=" _* expr:expr() end:position!() {
                AddAssign {
                    dest,
                    expr,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule sub_assign() -> SubAssign
            = start:position!() dest:expr() _* "-=" _* expr:expr() end:position!() {
                SubAssign {
                    dest,
                    expr,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule assign() -> Assign
            = start:position!() dest:expr() _* "=" _* expr:expr() end:position!() {
                Assign {
                    dest,
                    expr,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        //
        // expressions --------------------------------------------------------------------
        //

        // infix expression

        rule infix_sp()
            = "\\" lf()
            / __

        rule not() -> Not
            = start:position!() "!" expr:expr() end:position!() {
                Not {
                    expr: expr.into(),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // chain

        rule chain() -> Expression
            = e:expr3() chain:chain_member()+ {
                match e {
                    Expression::Not(_) => e,
                    Expression::And(_) => e,
                    Expression::Or(_) => e,
                    Expression::If(_) => e,
                    Expression::Fn(fn_) => {
                        let mut c = fn_.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Fn(Fn_ {
                            chain: Some(c),
                            ..fn_
                        })
                    },
                    Expression::Match(match_) => {
                        let mut c = match_.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Match(Match {
                            chain: Some(c),
                            ..match_
                        })
                    },
                    Expression::Block(block) =>  {
                        let mut c = block.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Block(Block {
                            chain: Some(c),
                            ..block
                        })
                    },
                    Expression::Exists(exists) =>  {
                        let mut c = exists.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Exists(Exists {
                            chain: Some(c),
                            ..exists
                        })
                    },
                    Expression::Tmpl(tmpl) =>  {
                        let mut c = tmpl.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Tmpl(Tmpl {
                            chain: Some(c),
                            ..tmpl
                        })
                    },
                    Expression::Str(str) =>  {
                        let mut c = str.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Str(Str {
                            chain: Some(c),
                            ..str
                        })
                    },
                    Expression::Num(num) =>  {
                        let mut c = num.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Num(Num {
                            chain: Some(c),
                            ..num
                        })
                    },
                    Expression::Bool(bool) =>  {
                        let mut c = bool.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Bool(Bool {
                            chain: Some(c),
                            ..bool
                        })
                    },
                    Expression::Null(null) =>  {
                        let mut c = null.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Null(Null {
                            chain: Some(c),
                            ..null
                        })
                    },
                    Expression::Obj(obj) =>  {
                        let mut c = obj.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Obj(Obj {
                            chain: Some(c),
                            ..obj
                        })
                    },
                    Expression::Arr(arr) =>  {
                        let mut c = arr.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Arr(Arr {
                            chain: Some(c),
                            ..arr
                        })
                    },
                    Expression::Identifier(identifier) => {
                        let mut c = identifier.chain.unwrap_or_default();
                        c.extend(chain);
                        Expression::Identifier(Identifier {
                            chain: Some(c),
                            ..identifier
                        })
                    },
                    Expression::Call(_) => e,
                    Expression::Index(_) => e,
                    Expression::Prop(_) => e,
                }
            }

        rule chain_member() -> ChainMember
            = call_chain:call_chain() { ChainMember::CallChain(call_chain) }
            / index_chain:index_chain() { ChainMember::IndexChain(index_chain) }
            / prop_chain:prop_chain() { ChainMember::PropChain(prop_chain) }

        rule call_chain() -> CallChain
            = start:position!() "(" _* args:call_args()? _* ")" end:position!() {
                CallChain {
                    args: args.unwrap_or_default(),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule call_args() -> Vec<Expression>
            = expr() ++ sep()

        rule index_chain() -> IndexChain
            = start:position!() "[" _* index:expr() _* "]" end:position!() {
                IndexChain {
                    index,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule prop_chain() -> PropChain
            = start:position!() "." name:name() end:position!() {
                PropChain {
                    name,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // if statement

        rule if() -> If
            = start:position!()
            "if" _+ cond:expr() _+
            then:block_or_statement()
            elseif:(_+ elseif_blocks:elseif_blocks() { elseif_blocks })?
            else_block:(_+ else_block:else_block() { else_block })? end:position!() {
                If {
                    cond: cond.into(),
                    then: then.into(),
                    elseif: elseif.unwrap_or_default(),
                    else_: else_block.map(Into::into),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule elseif_blocks() -> Vec<Elseif>
            = elseif_block() ++ (_*)

        rule elseif_block() -> Elseif
            = "elif" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':'] _* cond:expr() _* then:block_or_statement() {
                Elseif { cond, then }
            }

        rule else_block() -> StatementOrExpression
            = "else" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':'] _* then:block_or_statement() {
                then
            }

        // match expression

        rule match() -> Match
            = start:position!() "match" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':'] _*
            about:expr() _*
            "{" _*
            qs:(q:expr() _* "=>" _* a:block_or_statement() _* { QA{ q, a } })+
            x:("*" _* "=>" _* x:block_or_statement() { x })? _*
            "}" end:position!() {
                Match {
                    about: about.into(),
                    qs,
                    default: x.map(Into::into),
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // eval expression

        rule eval() -> Block
            = start:position!() "eval" _* "{" _* s:statements() _* "}" end:position!() {
                Block {
                    statements: s,
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // exists expression

        rule exists() -> Exists
            = start:position!() "exists" _+ i:identifier() end:position!() {
                Exists {
                    identifier: i,
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // variable reference expression

        rule identifier() -> Identifier
            = start:position!() name:name_with_namespace() end:position!() {
                Identifier {
                    name,
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        //
        // literals ------------------------------------------------------------------------------
        //

        // template literal

        rule tmpl() -> Tmpl
            = start:position!() "`" items:(!"`" tmpl_embed:tmpl_embed() { tmpl_embed })* "`" end:position!() {
                Tmpl{
                    tmpl: items,
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule tmpl_embed() -> StringOrExpression
            = "{" __* expr:expr() __* "}" { StringOrExpression::Expression(expr) }
            / str:tmpl_atom()+ { StringOrExpression::String(str.into_iter().collect() ) }

        rule tmpl_atom() -> char
            = tmpl_esc()
            / c:([^'`' | '{']) {c}

        rule tmpl_esc() -> char
            = "\\" esc:['{' | '}' | '`'] { esc }

        // string literal

        rule str() -> Str
            = start:position!() "\"" value:(!"\"" c:(str_double_quote_esc() / [_]) {c})* "\"" end:position!() {
                Str {
                    value: value.into_iter().collect(),
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }
            / start:position!() "'" value:(!"'" c:(str_single_quote_esc() / [_]) {c})* "'" end:position!() {
                Str {
                    value: value.into_iter().collect(),
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule str_double_quote_esc() -> char
            = r#"\""# {'"'}

        rule str_single_quote_esc() -> char
            = r#"\'"# {'\''}

        // number literal

        rule num() -> Num
            = float()
            / int()

        rule float() -> Num
            = start:position!() n:$(['+' | '-']? ['1'..='9'] ['0'..='9']+ "." ['0'..='9']+) end:position!() {
                Num {
                    value: n.parse().unwrap(),
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }
            / start:position!() n:$(['+' | '-']? ['0'..='9'] "." ['0'..='9']+) end:position!() {
                Num {
                    value: n.parse().unwrap(),
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule int() -> Num
            = start:position!() n:$(['+' | '-']? ['1'..='9'] ['0'..='9']+) end:position!() {
                Num {
                    value: n.parse().unwrap(),
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }
            / start:position!() n:$(['+' | '-']? ['0'..='9']) end:position!() {
                Num {
                    value: n.parse().unwrap(),
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // boolean literal

        rule bool() -> Bool
            = true()
            / false()

        rule true() -> Bool
            = start:position!() "true" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':'] end:position!() {
                Bool {
                    value: true,
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule false() -> Bool
            = start:position!() "false" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':'] end:position!() {
                Bool {
                    value: false,
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // null literal

        rule null() -> Null
            = start:position!() "null" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':'] end:position!() {
                Null {
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // object literal

        rule obj() -> Obj
            = start:position!() "{" _* kvs:(k:name() _* ":" _+ v:expr() _* ("," / ";")? _* { (k, v) })* _* "}" end:position!() {
                Obj {
                    value: IndexMap::from_iter(kvs),
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // array literal

        rule arr() -> Arr
            = start:position!() "[" _* items:(item:expr() _* ","? _* { item })* _* "]" end:position!() {
                Arr {
                    value: items,
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        //
        // function ------------------------------------------------------------------------------
        //

        rule arg() -> Arg
            = name:name() type_:(_* ":" _* type_:type_() { type_ })? {
                Arg { name, arg_type: type_ }
            }

        rule args() -> Vec<Arg>
            = arg() ++ sep()

        // define function statement

        rule fn_def() -> Definition
            = start:position!()
            "@"
            (quiet!{ !(__+) } / expected!("Cannot use spaces before or after the function name."))
            name:name()
            (quiet!{ !(__+) } / expected!("Cannot use spaces before or after the function name."))
            "(" _*
            args:args()? _*
            ")"
            ret:(_* ":" _* type_:type_() { type_ })? _*
            "{" _*
            content:statements()? _*
            "}"
            end:position!() {
                Definition {
                    name,
                    expr: Expression::Fn(
                        Fn_ {
                            args: args.unwrap_or_default(),
                            ret_type: ret,
                            children: content.unwrap_or_default(),
                            chain: None,
                            loc: Some(Loc{ start, end: end - 1 }),
                        },
                    ),
                    var_type: None,
                    mut_: false,
                    attr: Some(Vec::new()),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        // function expression

        rule fn() -> Fn_
            = start:position!()
            "@(" _* args:args()? _* ")"
            ret:(_* ":" _* type_:type_() { type_ })? _*
            "{" _* content:statements()? _* "}"
            end:position!() {
                Fn_ {
                    args: args.unwrap_or_default(),
                    ret_type: ret,
                    children: content.unwrap_or_default(),
                    chain: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        //
        // static literal ------------------------------------------------------------------------
        //

        // array literal (static)

        rule static_arr() -> Arr
            = start:position!() "[" _* items:(item:static_literal() _* ","? _* { item })* _* "]" end:position!() {
            Arr {
                value: items,
                chain: None,
                loc: Some(Loc{ start, end: end - 1 }),
            }
        }

        // object literal (static)

        rule static_obj() -> Obj
            = start:position!() "{" _* kvs:(k:name() _* ":" _+ v:static_literal() _* ("," / ";")? _* { (k, v) })* "}" end:position!() {
            Obj {
                value: IndexMap::from_iter(kvs),
                chain: None,
                loc: Some(Loc{ start, end: end - 1 }),
            }
        }

        //
        // type ----------------------------------------------------------------------------------
        //

        rule type_() -> TypeSource
            = fn_type:fn_type() { TypeSource::FnTypeSource(fn_type) }
            / named_type:named_type() { TypeSource::NamedTypeSource(named_type) }

        rule fn_type() -> FnTypeSource
            = start:position!() "@(" _* args:arg_types()? _* ")" _* "=>" _* result:type_() end:position!() {
                FnTypeSource{
                    args: args.unwrap_or_default(),
                    result: result.into(),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        rule arg_types() -> Vec<TypeSource>
            = type_() ++ sep()

        rule named_type() -> NamedTypeSource
            = start:position!() name:name() __* "<" __* inner:type_() __* ">" end:position!() {
                NamedTypeSource {
                    name,
                    inner: Some(inner.into()),
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }
            / start:position!() name:name() end:position!() {
                NamedTypeSource {
                    name,
                    inner: None,
                    loc: Some(Loc{ start, end: end - 1 }),
                }
            }

        //
        // general -------------------------------------------------------------------------------
        //

        rule name() -> String
            = text:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) {
                text.to_string()
            }

        rule name_with_namespace() -> String
            = text:$(name() ++ ":") { text.to_string() }

        rule sep()
            = _* "," _*
            / _+

        rule block_or_statement() -> StatementOrExpression
            = start:position!() "{" _* s:statements()? _* "}" end:position!() {
                StatementOrExpression::Expression(
                    Expression::Block(
                        Block {
                            statements: s.unwrap_or_default(),
                            chain: None,
                            loc: Some(Loc{ start, end: end - 1 })
                        }
                    )
                )
            }
            / statement()

        rule lf()
            = "\r\n" / ['\r' | '\n']

        rule eol()
            = ![_] / lf()

        // spacing
        rule _() -> char
            = [' ' | '\t' | '\r' | '\n']

        // spacing (no linebreaks)
        rule __() -> char
            = [' ' | '\t']
    }
}
