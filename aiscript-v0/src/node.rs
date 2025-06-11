//! ASTノード
//!
//! ASTノードはCSTノードをインタプリタ等から操作しやすい構造に変形したものです。

use indexmap::IndexMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Loc {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Namespace(Box<Namespace>),
    Meta(Box<Meta>),
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum DefinitionOrNamespace {
    Definition(Box<Definition>),
    Namespace(Box<Namespace>),
}

impl From<DefinitionOrNamespace> for Node {
    fn from(val: DefinitionOrNamespace) -> Self {
        match val {
            DefinitionOrNamespace::Definition(definition) => {
                Node::Statement(Statement::Definition(definition))
            }
            DefinitionOrNamespace::Namespace(namespace) => Node::Namespace(namespace),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementOrExpression {
    Statement(Statement),
    Expression(Expression),
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

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Definition(Box<Definition>),
    Return(Box<Return>),
    Each(Box<Each>),
    For(Box<For>),
    Loop(Box<Loop>),
    Break(Box<Break>),
    Continue(Box<Continue>),
    Assign(Box<Assign>),
    AddAssign(Box<AddAssign>),
    SubAssign(Box<SubAssign>),
}

impl From<Statement> for Node {
    fn from(val: Statement) -> Self {
        Node::Statement(val)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    If(Box<If>),
    Fn(Box<Fn>),
    Match(Box<Match>),
    Block(Box<Block>),
    Exists(Box<Exists>),
    Tmpl(Box<Tmpl>),
    Str(Box<Str>),
    Num(Box<Num>),
    Bool(Box<Bool>),
    Null(Box<Null>),
    Obj(Box<Obj>),
    Arr(Box<Arr>),
    Not(Box<Not>),
    And(Box<And>),
    Or(Box<Or>),
    Identifier(Box<Identifier>),
    Call(Box<Call>),
    Index(Box<Index>),
    Prop(Box<Prop>),
}

impl From<Expression> for Node {
    fn from(val: Expression) -> Self {
        Node::Expression(val)
    }
}

// 名前空間
#[derive(Debug, PartialEq, Clone)]
pub struct Namespace {
    pub name: String,                        // 空間名
    pub members: Vec<DefinitionOrNamespace>, // メンバー
    pub loc: Option<Loc>,
}

// メタデータ定義
#[derive(Debug, PartialEq, Clone)]
pub struct Meta {
    pub name: Option<String>, // 名
    pub value: Expression,    // 値
    pub loc: Option<Loc>,
}

// 変数宣言文
#[derive(Debug, PartialEq, Clone)]
pub struct Definition {
    pub name: String,                 // 変数名
    pub expr: Expression,             // 式
    pub var_type: Option<TypeSource>, // 変数の型
    pub mut_: bool,                   // ミュータブルか否か
    pub attr: Option<Vec<Attribute>>, // 付加された属性
    pub loc: Option<Loc>,
}

// 属性
#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    pub name: String,      // 属性名
    pub value: Expression, // 値
    pub loc: Option<Loc>,
}

// return文
#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub expr: Expression, // 式
    pub loc: Option<Loc>,
}

// each文
#[derive(Debug, PartialEq, Clone)]
pub struct Each {
    pub var: String,                      // イテレータ変数名
    pub items: Expression,                // 配列
    pub for_: Box<StatementOrExpression>, // 本体処理
    pub loc: Option<Loc>,
}

// for文
#[derive(Debug, PartialEq, Clone)]
pub struct For {
    pub var: Option<String>,              // イテレータ変数名
    pub from: Option<Expression>,         // 開始値
    pub to: Option<Expression>,           // 終値
    pub times: Option<Expression>,        // 回数
    pub for_: Box<StatementOrExpression>, // 本体処理
    pub loc: Option<Loc>,
}

// loop文
#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    pub statements: Vec<StatementOrExpression>, // 処理
    pub loc: Option<Loc>,
}

// break文
#[derive(Debug, PartialEq, Clone)]
pub struct Break {
    pub loc: Option<Loc>,
}

// continue文
#[derive(Debug, PartialEq, Clone)]
pub struct Continue {
    pub loc: Option<Loc>,
}

// 加算代入文
#[derive(Debug, PartialEq, Clone)]
pub struct AddAssign {
    pub dest: Expression, // 代入先
    pub expr: Expression, // 式
    pub loc: Option<Loc>,
}

// 減算代入文
#[derive(Debug, PartialEq, Clone)]
pub struct SubAssign {
    pub dest: Expression, // 代入先
    pub expr: Expression, // 式
    pub loc: Option<Loc>,
}

// 代入文
#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub dest: Expression, // 代入先
    pub expr: Expression, // 式
    pub loc: Option<Loc>,
}

// 否定
#[derive(Debug, PartialEq, Clone)]
pub struct Not {
    pub expr: Box<Expression>, // 式
    pub loc: Option<Loc>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct And {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator_loc: Loc,
    pub loc: Option<Loc>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Or {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator_loc: Loc,
    pub loc: Option<Loc>,
}

// if式
#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub cond: Box<Expression>,            // 条件式
    pub then: Box<StatementOrExpression>, // then節
    pub elseif: Vec<Elseif>,
    pub else_: Option<Box<StatementOrExpression>>, // else節
    pub loc: Option<Loc>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Elseif {
    pub cond: Expression,            // elifの条件式
    pub then: StatementOrExpression, // elif節
}

// 関数
#[derive(Debug, PartialEq, Clone)]
pub struct Fn {
    pub args: Vec<Arg>,
    pub ret_type: Option<TypeSource>,         // 戻り値の型
    pub children: Vec<StatementOrExpression>, // 本体処理
    pub loc: Option<Loc>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Arg {
    pub name: String,                 // 引数名
    pub arg_type: Option<TypeSource>, // 引数の型
}

// パターンマッチ
#[derive(Debug, PartialEq, Clone)]
pub struct Match {
    pub about: Box<Expression>, // 対象
    pub qs: Vec<QA>,
    pub default: Option<Box<StatementOrExpression>>, // デフォルト値
    pub loc: Option<Loc>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct QA {
    pub q: Expression,            // 条件
    pub a: StatementOrExpression, // 結果
}

// ブロックまたはeval式
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<StatementOrExpression>,
    pub loc: Option<Loc>,
}

// 変数の存在判定
#[derive(Debug, PartialEq, Clone)]
pub struct Exists {
    pub identifier: Identifier, // 変数名
    pub loc: Option<Loc>,
}

// テンプレート
#[derive(Debug, PartialEq, Clone)]
pub struct Tmpl {
    pub tmpl: Vec<StringOrExpression>, // 処理
    pub loc: Option<Loc>,
}

// 文字列リテラル
#[derive(Debug, PartialEq, Clone)]
pub struct Str {
    pub value: String, // 文字列
    pub loc: Option<Loc>,
}

// 数値リテラル
#[derive(Debug, PartialEq, Clone)]
pub struct Num {
    pub value: f64, // 数値
    pub loc: Option<Loc>,
}

// 真理値リテラル
#[derive(Debug, PartialEq, Clone)]
pub struct Bool {
    pub value: bool, // 真理値
    pub loc: Option<Loc>,
}

// nullリテラル
#[derive(Debug, PartialEq, Clone)]
pub struct Null {
    pub loc: Option<Loc>,
}

// オブジェクト
#[derive(Debug, PartialEq, Clone)]
pub struct Obj {
    pub value: IndexMap<String, Expression>, // プロパティ
    pub loc: Option<Loc>,
}

// 配列
#[derive(Debug, PartialEq, Clone)]
pub struct Arr {
    pub value: Vec<Expression>, // アイテム
    pub loc: Option<Loc>,
}

// 変数などの識別子
#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub name: String, // 変数名
    pub loc: Option<Loc>,
}

// 関数呼び出し
#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub target: Box<Expression>, // 対象
    pub args: Vec<Expression>,   // 引数
    pub loc: Option<Loc>,
}

// 配列要素アクセス
#[derive(Debug, PartialEq, Clone)]
pub struct Index {
    pub target: Box<Expression>, // 対象
    pub index: Box<Expression>,  // インデックス
    pub loc: Option<Loc>,
}

// プロパティアクセス
#[derive(Debug, PartialEq, Clone)]
pub struct Prop {
    pub target: Box<Expression>, // 対象
    pub name: String,            // プロパティ名
    pub loc: Option<Loc>,
}

// Type source

#[derive(Debug, PartialEq, Clone)]
pub enum TypeSource {
    NamedTypeSource(NamedTypeSource),
    FnTypeSource(FnTypeSource),
}

// 名前付き型
#[derive(Debug, PartialEq, Clone)]
pub struct NamedTypeSource {
    pub name: String,                   // 型名
    pub inner: Option<Box<TypeSource>>, // 内側の型
    pub loc: Option<Loc>,
}

// 関数の型
#[derive(Debug, PartialEq, Clone)]
pub struct FnTypeSource {
    pub args: Vec<TypeSource>,   // 引数の型
    pub result: Box<TypeSource>, // 戻り値の型
    pub loc: Option<Loc>,
}
