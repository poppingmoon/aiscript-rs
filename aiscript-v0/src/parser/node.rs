//! CSTノード
//!
//! パーサーが生成する直接的な処理結果です。
//! パーサーが生成しやすい形式になっているため、インタプリタ等では操作しにくい構造になっていることがあります。
//! この処理結果がプラグインによって処理されるとASTノードとなります。

use indexmap::IndexMap;

use crate::node::{self as ast, Loc};

pub use crate::node::{Arg, Break, Continue, FnTypeSource, NamedTypeSource, TypeSource};

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Namespace(Namespace),
    Meta(Meta),
    Statement(Statement),
    Expression(Expression),
}

impl From<Node> for ast::Node {
    fn from(val: Node) -> Self {
        match val {
            Node::Namespace(namespace) => ast::Node::Namespace(namespace.into()),
            Node::Meta(meta) => ast::Node::Meta(meta.into()),
            Node::Statement(statement) => ast::Node::Statement(statement.into()),
            Node::Expression(expression) => ast::Node::Expression(expression.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Definition(Definition),
    Return(Return),
    Attribute(Attribute), // AST
    Each(Each),
    For(For),
    Loop(Loop),
    Break(Break),
    Continue(Continue),
    Assign(Assign),
    AddAssign(AddAssign),
    SubAssign(SubAssign),
}

impl From<Statement> for ast::Statement {
    fn from(val: Statement) -> Self {
        match val {
            Statement::Definition(definition) => ast::Statement::Definition(definition.into()),
            Statement::Return(return_) => ast::Statement::Return(return_.into()),
            Statement::Attribute(_) => panic!(),
            Statement::Each(each) => ast::Statement::Each(each.into()),
            Statement::For(for_) => ast::Statement::For(for_.into()),
            Statement::Loop(loop_) => ast::Statement::Loop(loop_.into()),
            Statement::Break(break_) => ast::Statement::Break(break_),
            Statement::Continue(continue_) => ast::Statement::Continue(continue_),
            Statement::Assign(assign) => ast::Statement::Assign(assign.into()),
            Statement::AddAssign(addassign) => ast::Statement::AddAssign(addassign.into()),
            Statement::SubAssign(subassign) => ast::Statement::SubAssign(subassign.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Not(Not),
    And(And),
    Or(Or),
    If(If),
    Fn(Fn_),
    Match(Match),
    Block(Block),
    Exists(Exists),
    Tmpl(Tmpl),
    Str(Str),
    Num(Num),
    Bool(Bool),
    Null(Null),
    Obj(Obj),
    Arr(Arr),
    Identifier(Identifier),
    Call(Call),   // IR
    Index(Index), // IR
    Prop(Prop),   // IR
}

impl From<Expression> for ast::Expression {
    fn from(val: Expression) -> Self {
        match val {
            Expression::Not(not) => ast::Expression::Not(not.into()),
            Expression::And(and) => ast::Expression::And(and.into()),
            Expression::Or(or) => ast::Expression::Or(or.into()),
            Expression::If(if_) => ast::Expression::If(if_.into()),
            Expression::Fn(fn_) => ast::Expression::Fn(fn_.into()),
            Expression::Match(match_) => ast::Expression::Match(match_.into()),
            Expression::Block(block) => ast::Expression::Block(block.into()),
            Expression::Exists(exists) => ast::Expression::Exists(exists.into()),
            Expression::Tmpl(tmpl) => ast::Expression::Tmpl(tmpl.into()),
            Expression::Str(str) => ast::Expression::Str(str.into()),
            Expression::Num(num) => ast::Expression::Num(num.into()),
            Expression::Bool(bool) => ast::Expression::Bool(bool.into()),
            Expression::Null(null) => ast::Expression::Null(null.into()),
            Expression::Obj(obj) => ast::Expression::Obj(obj.into()),
            Expression::Arr(arr) => ast::Expression::Arr(arr.into()),
            Expression::Identifier(identifier) => ast::Expression::Identifier(identifier.into()),
            Expression::Call(call) => ast::Expression::Call(call.into()),
            Expression::Index(index) => ast::Expression::Index(index.into()),
            Expression::Prop(prop) => ast::Expression::Prop(prop.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Namespace {
    pub name: String,
    pub members: Vec<DefinitionOrNamespace>,
    pub loc: Option<Loc>,
}

impl From<Namespace> for ast::Namespace {
    fn from(val: Namespace) -> Self {
        ast::Namespace {
            name: val.name,
            members: val.members.into_iter().map(Into::into).collect(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Meta {
    pub name: Option<String>,
    pub value: Expression,
    pub loc: Option<Loc>,
}

impl From<Meta> for ast::Meta {
    fn from(val: Meta) -> Self {
        ast::Meta {
            name: val.name,
            value: val.value.into(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Definition {
    pub name: String,
    pub expr: Expression,
    pub var_type: Option<TypeSource>,
    pub mut_: bool,
    pub attr: Option<Vec<Attribute>>, // IR
    pub loc: Option<Loc>,
}

impl From<Definition> for ast::Definition {
    fn from(val: Definition) -> Self {
        ast::Definition {
            name: val.name,
            expr: val.expr.into(),
            var_type: val.var_type,
            mut_: val.mut_,
            attr: val
                .attr
                .map(|attr| attr.into_iter().map(Into::into).collect()),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: Expression,
    pub loc: Option<Loc>,
}

impl From<Attribute> for ast::Attribute {
    fn from(val: Attribute) -> Self {
        ast::Attribute {
            name: val.name,
            value: val.value.into(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub expr: Expression,
    pub loc: Option<Loc>,
}

impl From<Return> for ast::Return {
    fn from(val: Return) -> Self {
        ast::Return {
            expr: val.expr.into(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Each {
    pub var: String,
    pub items: Expression,
    pub for_: Box<StatementOrExpression>,
    pub loc: Option<Loc>,
}

impl From<Each> for ast::Each {
    fn from(val: Each) -> Self {
        ast::Each {
            var: val.var,
            items: val.items.into(),
            for_: Box::new((*val.for_).into()),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct For {
    pub var: Option<String>,
    pub from: Option<Expression>,
    pub to: Option<Expression>,
    pub times: Option<Expression>,
    pub for_: Box<StatementOrExpression>,
    pub loc: Option<Loc>,
}

impl From<For> for ast::For {
    fn from(val: For) -> Self {
        ast::For {
            var: val.var,
            from: val.from.map(Into::into),
            to: val.to.map(Into::into),
            times: val.times.map(Into::into),
            for_: Box::new((*val.for_).into()),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    pub statements: Vec<StatementOrExpression>,
    pub loc: Option<Loc>,
}

impl From<Loop> for ast::Loop {
    fn from(val: Loop) -> Self {
        ast::Loop {
            statements: val.statements.into_iter().map(Into::into).collect(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AddAssign {
    pub dest: Expression,
    pub expr: Expression,
    pub loc: Option<Loc>,
}

impl From<AddAssign> for ast::AddAssign {
    fn from(val: AddAssign) -> Self {
        ast::AddAssign {
            dest: val.dest.into(),
            expr: val.expr.into(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SubAssign {
    pub dest: Expression,
    pub expr: Expression,
    pub loc: Option<Loc>,
}

impl From<SubAssign> for ast::SubAssign {
    fn from(val: SubAssign) -> Self {
        ast::SubAssign {
            dest: val.dest.into(),
            expr: val.expr.into(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub dest: Expression,
    pub expr: Expression,
    pub loc: Option<Loc>,
}

impl From<Assign> for ast::Assign {
    fn from(val: Assign) -> Self {
        ast::Assign {
            dest: val.dest.into(),
            expr: val.expr.into(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Not {
    pub expr: Box<Expression>,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Not> for ast::Not {
    fn from(val: Not) -> Self {
        ast::Not {
            expr: Box::new((*val.expr).into()),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct And {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator_loc: Loc,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<And> for ast::And {
    fn from(val: And) -> Self {
        ast::And {
            left: Box::new((*val.left).into()),
            right: Box::new((*val.right).into()),
            operator_loc: val.operator_loc,
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Or {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator_loc: Loc,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Or> for ast::Or {
    fn from(val: Or) -> Self {
        ast::Or {
            left: Box::new((*val.left).into()),
            right: Box::new((*val.right).into()),
            operator_loc: val.operator_loc,
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub cond: Box<Expression>,
    pub then: Box<StatementOrExpression>,
    pub elseif: Vec<Elseif>,
    pub else_: Option<Box<StatementOrExpression>>,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<If> for ast::If {
    fn from(val: If) -> Self {
        ast::If {
            cond: Box::new((*val.cond).into()),
            then: Box::new((*val.then).into()),
            elseif: val.elseif.into_iter().map(Into::into).collect(),
            else_: val.else_.map(|else_| Box::new((*else_).into())),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Elseif {
    pub cond: Expression,
    pub then: StatementOrExpression,
}

impl From<Elseif> for ast::Elseif {
    fn from(val: Elseif) -> Self {
        ast::Elseif {
            cond: val.cond.into(),
            then: val.then.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fn_ {
    pub args: Vec<Arg>,
    pub ret_type: Option<TypeSource>,
    pub children: Vec<StatementOrExpression>,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Fn_> for ast::Fn {
    fn from(val: Fn_) -> Self {
        ast::Fn {
            args: val.args,
            ret_type: val.ret_type,
            children: val.children.into_iter().map(Into::into).collect(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Match {
    pub about: Box<Expression>,
    pub qs: Vec<QA>,
    pub default: Option<Box<StatementOrExpression>>,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Match> for ast::Match {
    fn from(val: Match) -> Self {
        ast::Match {
            about: Box::new((*val.about).into()),
            qs: val.qs.into_iter().map(Into::into).collect(),
            default: val.default.map(|default| Box::new((*default).into())),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct QA {
    pub q: Expression,
    pub a: StatementOrExpression,
}

impl From<QA> for ast::QA {
    fn from(val: QA) -> Self {
        ast::QA {
            q: val.q.into(),
            a: val.a.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<StatementOrExpression>,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Block> for ast::Block {
    fn from(val: Block) -> Self {
        ast::Block {
            statements: val.statements.into_iter().map(Into::into).collect(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Exists {
    pub identifier: Identifier,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Exists> for ast::Exists {
    fn from(val: Exists) -> Self {
        ast::Exists {
            identifier: val.identifier.into(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Tmpl {
    pub tmpl: Vec<StringOrExpression>,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Tmpl> for ast::Tmpl {
    fn from(val: Tmpl) -> Self {
        ast::Tmpl {
            tmpl: val.tmpl.into_iter().map(Into::into).collect(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Str {
    pub value: String,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Str> for ast::Str {
    fn from(val: Str) -> Self {
        ast::Str {
            value: val.value,
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Num {
    pub value: f64,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Num> for ast::Num {
    fn from(val: Num) -> Self {
        ast::Num {
            value: val.value,
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bool {
    pub value: bool,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Bool> for ast::Bool {
    fn from(val: Bool) -> Self {
        ast::Bool {
            value: val.value,
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Null {
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Null> for ast::Null {
    fn from(val: Null) -> Self {
        ast::Null { loc: val.loc }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Obj {
    pub value: IndexMap<String, Expression>,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Obj> for ast::Obj {
    fn from(val: Obj) -> Self {
        ast::Obj {
            value: val
                .value
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Arr {
    pub value: Vec<Expression>,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Arr> for ast::Arr {
    fn from(val: Arr) -> Self {
        ast::Arr {
            value: val.value.into_iter().map(Into::into).collect(),
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub name: String,
    pub chain: Option<Vec<ChainMember>>,
    pub loc: Option<Loc>,
}

impl From<Identifier> for ast::Identifier {
    fn from(val: Identifier) -> Self {
        ast::Identifier {
            name: val.name,
            loc: val.loc,
        }
    }
}

// AST
#[derive(Debug, PartialEq, Clone)]
pub enum ChainMember {
    CallChain(CallChain),
    IndexChain(IndexChain),
    PropChain(PropChain),
}

// AST
#[derive(Debug, PartialEq, Clone)]
pub struct CallChain {
    pub args: Vec<Expression>,
    pub loc: Option<Loc>,
}

// AST
#[derive(Debug, PartialEq, Clone)]
pub struct IndexChain {
    pub index: Expression,
    pub loc: Option<Loc>,
}

// AST
#[derive(Debug, PartialEq, Clone)]
pub struct PropChain {
    pub name: String,
    pub loc: Option<Loc>,
}

// IR
#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub target: Box<Expression>,
    pub args: Vec<Expression>,
    pub loc: Option<Loc>,
}

impl From<Call> for ast::Call {
    fn from(val: Call) -> Self {
        ast::Call {
            target: Box::new((*val.target).into()),
            args: val.args.into_iter().map(Into::into).collect(),
            loc: val.loc,
        }
    }
}

// IR
#[derive(Debug, PartialEq, Clone)]
pub struct Index {
    pub target: Box<Expression>,
    pub index: Box<Expression>,
    pub loc: Option<Loc>,
}

impl From<Index> for ast::Index {
    fn from(val: Index) -> Self {
        Self {
            target: Box::new((*val.target).into()),
            index: Box::new((*val.index).into()),
            loc: val.loc,
        }
    }
}

// IR
#[derive(Debug, PartialEq, Clone)]
pub struct Prop {
    pub target: Box<Expression>,
    pub name: String,
    pub loc: Option<Loc>,
}

impl From<Prop> for ast::Prop {
    fn from(val: Prop) -> Self {
        Self {
            target: Box::new((*val.target).into()),
            name: val.name,
            loc: val.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DefinitionOrNamespace {
    Definition(Definition),
    Namespace(Namespace),
}

impl From<DefinitionOrNamespace> for ast::DefinitionOrNamespace {
    fn from(val: DefinitionOrNamespace) -> Self {
        match val {
            DefinitionOrNamespace::Definition(definition) => {
                ast::DefinitionOrNamespace::Definition(definition.into())
            }
            DefinitionOrNamespace::Namespace(namespace) => {
                ast::DefinitionOrNamespace::Namespace(namespace.into())
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementOrExpression {
    Statement(Statement),
    Expression(Expression),
}

impl From<StatementOrExpression> for ast::StatementOrExpression {
    fn from(val: StatementOrExpression) -> Self {
        match val {
            StatementOrExpression::Statement(statement) => {
                ast::StatementOrExpression::Statement(statement.into())
            }
            StatementOrExpression::Expression(expression) => {
                ast::StatementOrExpression::Expression(expression.into())
            }
        }
    }
}

impl From<StatementOrExpression> for Node {
    fn from(val: StatementOrExpression) -> Self {
        match val {
            StatementOrExpression::Statement(statement) => Node::Statement(statement),
            StatementOrExpression::Expression(expression) => Node::Expression(expression),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringOrExpression {
    String(String),
    Expression(Expression),
}

impl From<StringOrExpression> for ast::StringOrExpression {
    fn from(val: StringOrExpression) -> Self {
        match val {
            StringOrExpression::String(string) => ast::StringOrExpression::String(string),
            StringOrExpression::Expression(expression) => {
                ast::StringOrExpression::Expression(expression.into())
            }
        }
    }
}
