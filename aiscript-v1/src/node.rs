//! ASTノード

use std::sync::Arc;

use indexmap::IndexMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Pos {
    pub line: u32,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Loc {
    pub start: Pos,
    pub end: Pos,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Namespace(Arc<Namespace>),
    Meta(Arc<Meta>),
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum DefinitionOrNamespace {
    Definition(Arc<Definition>),
    Namespace(Arc<Namespace>),
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

impl StatementOrExpression {
    pub fn loc(&self) -> &Loc {
        match self {
            StatementOrExpression::Statement(statement) => statement.loc(),
            StatementOrExpression::Expression(expression) => expression.loc(),
        }
    }
}

// 名前空間
#[derive(Debug, PartialEq, Clone)]
pub struct Namespace {
    pub loc: Loc,
    pub name: String,                        // 空間名
    pub members: Vec<DefinitionOrNamespace>, // メンバー
}

// メタデータ定義
#[derive(Debug, PartialEq, Clone)]
pub struct Meta {
    pub loc: Loc,
    pub name: Option<String>, // 名
    pub value: Expression,    // 値
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Definition(Arc<Definition>),
    Return(Arc<Return>),
    Each(Arc<Each>),
    For(Arc<For>),
    ForLet(Arc<ForLet>),
    Loop(Arc<Loop>),
    Break(Arc<Break>),
    Continue(Arc<Continue>),
    Assign(Arc<Assign>),
    AddAssign(Arc<AddAssign>),
    SubAssign(Arc<SubAssign>),
}

impl From<Statement> for Node {
    fn from(val: Statement) -> Self {
        Node::Statement(val)
    }
}

impl From<Statement> for StatementOrExpression {
    fn from(value: Statement) -> Self {
        StatementOrExpression::Statement(value)
    }
}

impl Statement {
    pub fn loc(&self) -> &Loc {
        match self {
            Statement::Definition(definition) => &definition.loc,
            Statement::Return(return_) => &return_.loc,
            Statement::Each(each) => &each.loc,
            Statement::For(for_) => &for_.loc,
            Statement::ForLet(for_let) => &for_let.loc,
            Statement::Loop(loop_) => &loop_.loc,
            Statement::Break(break_) => &break_.loc,
            Statement::Continue(continue_) => &continue_.loc,
            Statement::Assign(assign) => &assign.loc,
            Statement::AddAssign(add_assign) => &add_assign.loc,
            Statement::SubAssign(sub_assign) => &sub_assign.loc,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    If(Arc<If>),
    Fn(Arc<Fn>),
    Match(Arc<Match>),
    Block(Arc<Block>),
    Exists(Arc<Exists>),
    Tmpl(Arc<Tmpl>),
    Str(Arc<Str>),
    Num(Arc<Num>),
    Bool(Arc<Bool>),
    Null(Arc<Null>),
    Obj(Arc<Obj>),
    Arr(Arc<Arr>),
    Plus(Arc<Plus>),
    Minus(Arc<Minus>),
    Not(Arc<Not>),
    Pow(Arc<Pow>),
    Mul(Arc<Mul>),
    Div(Arc<Div>),
    Rem(Arc<Rem>),
    Add(Arc<Add>),
    Sub(Arc<Sub>),
    Lt(Arc<Lt>),
    Lteq(Arc<Lteq>),
    Gt(Arc<Gt>),
    Gteq(Arc<Gteq>),
    Eq(Arc<Eq>),
    Neq(Arc<Neq>),
    And(Arc<And>),
    Or(Arc<Or>),
    Identifier(Arc<Identifier>),
    Call(Arc<Call>),
    Index(Arc<Index>),
    Prop(Arc<Prop>),
}

impl From<Expression> for Node {
    fn from(val: Expression) -> Self {
        Node::Expression(val)
    }
}

impl From<Expression> for StatementOrExpression {
    fn from(value: Expression) -> Self {
        StatementOrExpression::Expression(value)
    }
}

impl Expression {
    pub fn loc(&self) -> &Loc {
        match self {
            Expression::If(if_) => &if_.loc,
            Expression::Fn(fn_) => &fn_.loc,
            Expression::Match(match_) => &match_.loc,
            Expression::Block(block) => &block.loc,
            Expression::Exists(exists) => &exists.loc,
            Expression::Tmpl(tmpl) => &tmpl.loc,
            Expression::Str(str) => &str.loc,
            Expression::Num(num) => &num.loc,
            Expression::Bool(bool) => &bool.loc,
            Expression::Null(null) => &null.loc,
            Expression::Obj(obj) => &obj.loc,
            Expression::Arr(arr) => &arr.loc,
            Expression::Plus(plus) => &plus.loc,
            Expression::Minus(minus) => &minus.loc,
            Expression::Not(not) => &not.loc,
            Expression::Pow(pow) => &pow.loc,
            Expression::Mul(mul) => &mul.loc,
            Expression::Div(div) => &div.loc,
            Expression::Rem(rem) => &rem.loc,
            Expression::Add(add) => &add.loc,
            Expression::Sub(sub) => &sub.loc,
            Expression::Lt(lt) => &lt.loc,
            Expression::Lteq(lteq) => &lteq.loc,
            Expression::Gt(gt) => &gt.loc,
            Expression::Gteq(gteq) => &gteq.loc,
            Expression::Eq(eq) => &eq.loc,
            Expression::Neq(neq) => &neq.loc,
            Expression::And(and) => &and.loc,
            Expression::Or(or) => &or.loc,
            Expression::Identifier(identifier) => &identifier.loc,
            Expression::Call(call) => &call.loc,
            Expression::Index(index) => &index.loc,
            Expression::Prop(prop) => &prop.loc,
        }
    }
}

// 変数宣言文
#[derive(Debug, PartialEq, Clone)]
pub struct Definition {
    pub loc: Loc,
    pub dest: Expression,             // 宣言式
    pub var_type: Option<TypeSource>, // 変数の型
    pub expr: Expression,             // 式
    pub mut_: bool,                   // ミュータブルか否か
    pub attr: Option<Vec<Attribute>>, // 付加された属性
}

// 属性
#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    pub loc: Loc,
    pub name: String,      // 属性名
    pub value: Expression, // 値
}

// return文
#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub loc: Loc,
    pub expr: Expression, // 式
}

// each文
#[derive(Debug, PartialEq, Clone)]
pub struct Each {
    pub loc: Loc,
    pub label: Option<String>,       // ラベル
    pub var: Expression,             // イテレータ宣言
    pub items: Expression,           // 配列
    pub for_: StatementOrExpression, // 本体処理
}

// for文
#[derive(Debug, PartialEq, Clone)]
pub struct For {
    pub loc: Loc,
    pub label: Option<String>,       // ラベル
    pub times: Expression,           // 回数
    pub for_: StatementOrExpression, // 本体処理
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForLet {
    pub loc: Loc,
    pub label: Option<String>,       // ラベル
    pub var: String,                 // イテレータ変数名
    pub from: Expression,            // 開始値
    pub to: Expression,              // 終値
    pub for_: StatementOrExpression, // 本体処理
}

// loop文
#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    pub loc: Loc,
    pub label: Option<String>,                  // ラベル
    pub statements: Vec<StatementOrExpression>, // 処理
}

// break文
#[derive(Debug, PartialEq, Clone)]
pub struct Break {
    pub loc: Loc,
    pub label: Option<String>,    // ラベル
    pub expr: Option<Expression>, // 式
}

// continue文
#[derive(Debug, PartialEq, Clone)]
pub struct Continue {
    pub loc: Loc,
    pub label: Option<String>, // ラベル
}

// 加算代入文
#[derive(Debug, PartialEq, Clone)]
pub struct AddAssign {
    pub loc: Loc,
    pub dest: Expression, // 代入先
    pub expr: Expression, // 式
}

// 減算代入文
#[derive(Debug, PartialEq, Clone)]
pub struct SubAssign {
    pub loc: Loc,
    pub dest: Expression, // 代入先
    pub expr: Expression, // 式
}

// 代入文
#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub loc: Loc,
    pub dest: Expression, // 代入先
    pub expr: Expression, // 式
}

// 正号
#[derive(Debug, PartialEq, Clone)]
pub struct Plus {
    pub loc: Loc,
    pub expr: Expression, // 式
}

// 負号
#[derive(Debug, PartialEq, Clone)]
pub struct Minus {
    pub loc: Loc,
    pub expr: Expression, // 式
}

// 否定
#[derive(Debug, PartialEq, Clone)]
pub struct Not {
    pub loc: Loc,
    pub expr: Expression, // 式
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pow {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Mul {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Div {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rem {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Add {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Sub {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Lt {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Lteq {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Gt {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Gteq {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Eq {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Neq {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct And {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Or {
    pub loc: Loc,
    pub left: Expression,
    pub right: Expression,
}

// if式
#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub loc: Loc,
    pub label: Option<String>,       // ラベル
    pub cond: Expression,            // 条件式
    pub then: StatementOrExpression, // then節
    pub elseif: Vec<Elseif>,
    pub else_: Option<StatementOrExpression>, // else節
}

#[derive(Debug, PartialEq, Clone)]
pub struct Elseif {
    pub cond: Expression,            // elifの条件式
    pub then: StatementOrExpression, // elif節
}

// 関数
#[derive(Debug, PartialEq, Clone)]
pub struct Fn {
    pub loc: Loc,
    pub type_params: Option<Vec<TypeParam>>,
    pub params: Vec<Param>,
    pub ret_type: Option<TypeSource>,         // 戻り値の型
    pub children: Vec<StatementOrExpression>, // 本体処理
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub dest: Expression, // 引数名
    pub optional: bool,
    pub default: Option<Expression>,  // 引数の初期値
    pub arg_type: Option<TypeSource>, // 引数の型
}

// パターンマッチ
#[derive(Debug, PartialEq, Clone)]
pub struct Match {
    pub loc: Loc,
    pub label: Option<String>, // ラベル
    pub about: Expression,     // 対象
    pub qs: Vec<QA>,
    pub default: Option<StatementOrExpression>, // デフォルト値
}

#[derive(Debug, PartialEq, Clone)]
pub struct QA {
    pub q: Expression,            // 条件
    pub a: StatementOrExpression, // 結果
}

// ブロックまたはeval式
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub loc: Loc,
    pub label: Option<String>,                  // ラベル
    pub statements: Vec<StatementOrExpression>, // 処理
}

// 変数の存在判定
#[derive(Debug, PartialEq, Clone)]
pub struct Exists {
    pub loc: Loc,
    pub identifier: Identifier, // 変数名
}

// テンプレート
#[derive(Debug, PartialEq, Clone)]
pub struct Tmpl {
    pub loc: Loc,
    pub tmpl: Vec<Expression>, // 処理
}

// 文字列リテラル
#[derive(Debug, PartialEq, Clone)]
pub struct Str {
    pub loc: Loc,
    pub value: String, // 文字列
}

// 数値リテラル
#[derive(Debug, PartialEq, Clone)]
pub struct Num {
    pub loc: Loc,
    pub value: f64, // 数値
}

// 真理値リテラル
#[derive(Debug, PartialEq, Clone)]
pub struct Bool {
    pub loc: Loc,
    pub value: bool, // 真理値
}

// nullリテラル
#[derive(Debug, PartialEq, Clone)]
pub struct Null {
    pub loc: Loc,
}

// オブジェクト
#[derive(Debug, PartialEq, Clone)]
pub struct Obj {
    pub loc: Loc,
    pub value: IndexMap<String, Expression>, // プロパティ
}

// 配列
#[derive(Debug, PartialEq, Clone)]
pub struct Arr {
    pub loc: Loc,
    pub value: Vec<Expression>, // アイテム
}

// 変数などの識別子
#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub loc: Loc,
    pub name: String, // 変数名
}

// 関数呼び出し
#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub loc: Loc,
    pub target: Expression,    // 対象
    pub args: Vec<Expression>, // 引数
}

// 配列要素アクセス
#[derive(Debug, PartialEq, Clone)]
pub struct Index {
    pub loc: Loc,
    pub target: Expression, // 対象
    pub index: Expression,  // インデックス
}

// プロパティアクセス
#[derive(Debug, PartialEq, Clone)]
pub struct Prop {
    pub loc: Loc,
    pub target: Expression, // 対象
    pub name: String,       // プロパティ名
}

// Type source

#[derive(Debug, PartialEq, Clone)]
pub enum TypeSource {
    NamedTypeSource(Arc<NamedTypeSource>),
    FnTypeSource(Arc<FnTypeSource>),
    UnionTypeSource(Arc<UnionTypeSource>),
}

// 名前付き型
#[derive(Debug, PartialEq, Clone)]
pub struct NamedTypeSource {
    pub loc: Loc,
    pub name: String,              // 型名
    pub inner: Option<TypeSource>, // 内側の型
}

// 関数の型
#[derive(Debug, PartialEq, Clone)]
pub struct FnTypeSource {
    pub loc: Loc,
    pub type_params: Option<Vec<TypeParam>>, // 型パラメータ
    pub params: Vec<TypeSource>,             // 引数の型
    pub result: TypeSource,                  // 戻り値の型
}

// ユニオン型
#[derive(Debug, PartialEq, Clone)]
pub struct UnionTypeSource {
    pub loc: Loc,
    pub inners: Vec<TypeSource>, // 含まれる型
}

// 型パラメータ
#[derive(Debug, PartialEq, Clone)]
pub struct TypeParam {
    pub loc: Loc,
    pub name: String, // パラメータ名
}
