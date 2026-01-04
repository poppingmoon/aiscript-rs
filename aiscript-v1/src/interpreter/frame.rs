use indexmap::IndexMap;

use crate::node::{Attribute, Elseif, Expression, QA, Statement, StatementOrExpression};

use super::{
    scope::Scope,
    value::{VArr, VFn, VObj, Value},
};

#[derive(Debug)]
pub enum Frame {
    Statement(Statement),
    Expression(Expression),
    Definition1 {
        dest: Expression,
        mut_: bool,
        attr: Option<Vec<Attribute>>,
    },
    Definition2 {
        dest: Expression,
        value: Value,
        mut_: bool,
    },
    Return,
    Each1 {
        label: Option<String>,
        var: Expression,
        for_: StatementOrExpression,
    },
    Each2 {
        label: Option<String>,
        var: Expression,
        items: Vec<Value>,
        for_: StatementOrExpression,
    },
    Each3 {
        label: Option<String>,
        var: Expression,
        items: Vec<Value>,
        for_: StatementOrExpression,
    },
    For1 {
        label: Option<String>,
        for_: StatementOrExpression,
    },
    For2 {
        label: Option<String>,
        i: f64,
        times: f64,
        for_: StatementOrExpression,
    },
    For3 {
        label: Option<String>,
        i: f64,
        times: f64,
        for_: StatementOrExpression,
    },
    ForLet1 {
        label: Option<String>,
        var: String,
        to: Expression,
        for_: StatementOrExpression,
    },
    ForLet2 {
        label: Option<String>,
        var: String,
        from: f64,
        for_: StatementOrExpression,
    },
    ForLet3 {
        label: Option<String>,
        var: String,
        i: f64,
        until: f64,
        for_: StatementOrExpression,
    },
    ForLet4 {
        label: Option<String>,
        var: String,
        i: f64,
        until: f64,
        for_: StatementOrExpression,
    },
    Loop1 {
        label: Option<String>,
        statements: Vec<StatementOrExpression>,
    },
    Loop2 {
        label: Option<String>,
        statements: Vec<StatementOrExpression>,
    },
    Break {
        label: Option<String>,
    },
    Assign {
        dest: Expression,
        expr: Option<Expression>,
        op: Option<AssignmentOperator>,
    },
    AssignIdentifier {
        name: String,
        op: Option<AssignmentOperator>,
    },
    AssignIndex1 {
        index: Expression,
        expr: Option<Expression>,
        op: Option<AssignmentOperator>,
    },
    AssignIndex2 {
        assignee: Value,
        expr: Option<Expression>,
        op: Option<AssignmentOperator>,
    },
    AssignIndexArr {
        assignee: VArr,
        index: usize,
        op: Option<AssignmentOperator>,
    },
    AssignProp1 {
        name: String,
        expr: Option<Expression>,
        op: Option<AssignmentOperator>,
    },
    AssignProp2 {
        assignee: VObj,
        name: String,
        op: Option<AssignmentOperator>,
    },
    AssignArr {
        len: usize,
        op: Option<AssignmentOperator>,
    },
    AssignObj {
        keys: Vec<String>,
        op: Option<AssignmentOperator>,
    },
    If1 {
        label: Option<String>,
        then: StatementOrExpression,
        elseif: Vec<Elseif>,
        else_: Option<Box<StatementOrExpression>>,
    },
    If2 {
        label: Option<String>,
        is_statement: bool,
    },
    Match1 {
        label: Option<String>,
        qs: Vec<QA>,
        default: Option<Box<StatementOrExpression>>,
    },
    Match2 {
        label: Option<String>,
        about: Value,
        qs: Vec<QA>,
        default: Option<Box<StatementOrExpression>>,
    },
    Match3 {
        label: Option<String>,
        about: Value,
        a: Box<StatementOrExpression>,
        qs: Vec<QA>,
        default: Option<Box<StatementOrExpression>>,
    },
    Match4 {
        label: Option<String>,
        is_statement: bool,
    },
    Block {
        label: Option<String>,
    },
    Tmpl1 {
        tmpl: Vec<Expression>,
        str: String,
    },
    Tmpl2 {
        tmpl: Vec<Expression>,
        str: String,
    },
    Obj1 {
        obj: Box<IndexMap<String, Expression>>,
        value: Box<IndexMap<String, Value>>,
    },
    Obj2 {
        obj: Box<IndexMap<String, Expression>>,
        value: Box<IndexMap<String, Value>>,
        k: String,
    },
    Arr1 {
        arr: Vec<Expression>,
        value: Vec<Value>,
    },
    Arr2 {
        arr: Vec<Expression>,
        value: Vec<Value>,
    },
    Plus,
    Minus,
    Not,
    BinOp1 {
        callee: VFn,
        right: Expression,
    },
    BinOp2 {
        callee: VFn,
        left: Value,
    },
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
        args: Vec<Expression>,
        value: Vec<Value>,
    },
    Call3 {
        callee: VFn,
        args: Vec<Expression>,
        value: Vec<Value>,
    },
    Call4 {
        callee: VFn,
        args: Vec<Value>,
    },
    Call5 {
        scope: Scope,
    },
    Index1 {
        index: Expression,
    },
    Index2 {
        target: Value,
    },
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

#[derive(Debug)]
pub enum AssignmentOperator {
    Add,
    Sub,
}
