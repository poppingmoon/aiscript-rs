use indexmap::IndexMap;

use crate::node::{Elseif, Expression, QA, Statement, StatementOrExpression, StringOrExpression};

use super::{
    scope::Scope,
    value::{VFn, Value},
};

#[derive(Debug)]
pub enum Frame {
    Statement(Statement),
    Expression(Expression),
    Definition {
        name: String,
        mut_: bool,
    },
    Return,
    Each1 {
        var: String,
        for_: StatementOrExpression,
    },
    Each2 {
        var: String,
        items: Vec<Value>,
        for_: StatementOrExpression,
    },
    Each3 {
        var: String,
        items: Vec<Value>,
        for_: StatementOrExpression,
    },
    For1 {
        for_: StatementOrExpression,
    },
    For2 {
        i: f64,
        times: f64,
        for_: StatementOrExpression,
    },
    For3 {
        i: f64,
        times: f64,
        for_: StatementOrExpression,
    },
    ForLet1 {
        var: String,
        to: Expression,
        for_: StatementOrExpression,
    },
    ForLet2 {
        var: String,
        from: f64,
        for_: StatementOrExpression,
    },
    ForLet3 {
        var: String,
        i: f64,
        until: f64,
        for_: StatementOrExpression,
    },
    ForLet4 {
        var: String,
        i: f64,
        until: f64,
        for_: StatementOrExpression,
    },
    Loop1 {
        statements: Vec<StatementOrExpression>,
    },
    Loop2 {
        statements: Vec<StatementOrExpression>,
    },
    Assign1 {
        dest: Expression,
    },
    Assign2 {
        dest: Expression,
        value: Value,
    },
    AssignIndex {
        value: Value,
    },
    AssignProp {
        name: String,
        value: Value,
    },
    AddAssign1 {
        dest: Expression,
        expr: Expression,
    },
    AddAssign2 {
        dest: Expression,
        target: f64,
    },
    SubAssign1 {
        dest: Expression,
        expr: Expression,
    },
    SubAssign2 {
        dest: Expression,
        target: f64,
    },
    If {
        then: StatementOrExpression,
        elseif: Vec<Elseif>,
        else_: Option<Box<StatementOrExpression>>,
    },
    Match1 {
        qs: Vec<QA>,
        default: Option<Box<StatementOrExpression>>,
    },
    Match2 {
        about: Value,
        qs: Vec<QA>,
        default: Option<Box<StatementOrExpression>>,
    },
    Match3 {
        about: Value,
        a: Box<StatementOrExpression>,
        qs: Vec<QA>,
        default: Option<Box<StatementOrExpression>>,
    },
    Block,
    Tmpl1 {
        tmpl: Vec<StringOrExpression>,
        str: String,
    },
    Tmpl2 {
        tmpl: Vec<StringOrExpression>,
        str: String,
    },
    Obj1 {
        obj: Box<IndexMap<String, Expression>>,
        map: Box<IndexMap<String, Value>>,
    },
    Obj2 {
        obj: Box<IndexMap<String, Expression>>,
        map: Box<IndexMap<String, Value>>,
        k: String,
    },
    Not,
    And1 {
        right: Expression,
    },
    And2,
    Or1 {
        right: Expression,
    },
    Or2,
    Call1 {
        args: Vec<Expression>,
    },
    Call2 {
        callee: VFn,
        args: Vec<Value>,
    },
    Call3 {
        scope: Scope,
    },
    Index,
    Prop {
        name: String,
    },
    Run,
    Unwind,
    Eval,
}

impl From<StatementOrExpression> for Frame {
    fn from(value: StatementOrExpression) -> Self {
        match value {
            StatementOrExpression::Statement(statement) => Frame::Statement(statement),
            StatementOrExpression::Expression(expression) => Frame::Expression(expression),
        }
    }
}

impl From<Expression> for Frame {
    fn from(value: Expression) -> Self {
        Frame::Expression(value)
    }
}
