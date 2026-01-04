use crate::node::Pos;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Eof {
        pos: Pos,
        has_left_spacing: bool,
    },
    NewLine {
        pos: Pos,
        has_left_spacing: bool,
    },
    Identifier {
        pos: Pos,
        has_left_spacing: bool,
        value: &'a str,
    },

    // literal
    NumberLiteral {
        pos: Pos,
        has_left_spacing: bool,
        value: &'a str,
    },
    StringLiteral {
        pos: Pos,
        has_left_spacing: bool,
        value: Vec<&'a str>,
    },

    // template string
    Template {
        pos: Pos,
        has_left_spacing: bool,
        children: Vec<TemplateToken<'a>>,
    },

    // keyword
    NullKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    TrueKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    FalseKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    EachKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    ForKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    LoopKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    DoKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    WhileKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    BreakKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    ContinueKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    MatchKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    CaseKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    DefaultKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    IfKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    ElifKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    ElseKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    ReturnKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    EvalKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    VarKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    LetKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },
    ExistsKeyword {
        pos: Pos,
        has_left_spacing: bool,
    },

    /// "!"
    Not {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "!="
    NotEq {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "#"
    Sharp {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "#["
    OpenSharpBracket {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "###"
    Sharp3 {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "%"
    Percent {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "&&"
    And2 {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "("
    OpenParen {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// ")"
    CloseParen {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "*"
    Asterisk {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "+"
    Plus {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "+="
    PlusEq {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// ","
    Comma {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "-"
    Minus {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "-="
    MinusEq {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "."
    Dot {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "/"
    Slash {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// ":"
    Colon {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "::"
    Colon2 {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// ";"
    SemiColon {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "<"
    Lt {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "<="
    LtEq {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "<:"
    Out {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "="
    Eq {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "=="
    Eq2 {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "=>"
    Arrow {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// ">"
    Gt {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// ">="
    GtEq {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "?"
    Question {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "@"
    At {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "["
    OpenBracket {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "\\"
    BackSlash {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "]"
    CloseBracket {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "^"
    Hat {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "{"
    OpenBrace {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "|"
    Or {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "||"
    Or2 {
        pos: Pos,
        has_left_spacing: bool,
    },
    /// "}"
    CloseBrace {
        pos: Pos,
        has_left_spacing: bool,
    },
}

impl Token<'_> {
    pub fn pos(&self) -> &Pos {
        match self {
            Token::Eof { pos, .. }
            | Token::NewLine { pos, .. }
            | Token::Identifier { pos, .. }
            | Token::NumberLiteral { pos, .. }
            | Token::StringLiteral { pos, .. }
            | Token::Template { pos, .. }
            | Token::NullKeyword { pos, .. }
            | Token::TrueKeyword { pos, .. }
            | Token::FalseKeyword { pos, .. }
            | Token::EachKeyword { pos, .. }
            | Token::ForKeyword { pos, .. }
            | Token::LoopKeyword { pos, .. }
            | Token::DoKeyword { pos, .. }
            | Token::WhileKeyword { pos, .. }
            | Token::BreakKeyword { pos, .. }
            | Token::ContinueKeyword { pos, .. }
            | Token::MatchKeyword { pos, .. }
            | Token::CaseKeyword { pos, .. }
            | Token::DefaultKeyword { pos, .. }
            | Token::IfKeyword { pos, .. }
            | Token::ElifKeyword { pos, .. }
            | Token::ElseKeyword { pos, .. }
            | Token::ReturnKeyword { pos, .. }
            | Token::EvalKeyword { pos, .. }
            | Token::VarKeyword { pos, .. }
            | Token::LetKeyword { pos, .. }
            | Token::ExistsKeyword { pos, .. }
            | Token::Not { pos, .. }
            | Token::NotEq { pos, .. }
            | Token::Sharp { pos, .. }
            | Token::OpenSharpBracket { pos, .. }
            | Token::Sharp3 { pos, .. }
            | Token::Percent { pos, .. }
            | Token::And2 { pos, .. }
            | Token::OpenParen { pos, .. }
            | Token::CloseParen { pos, .. }
            | Token::Asterisk { pos, .. }
            | Token::Plus { pos, .. }
            | Token::PlusEq { pos, .. }
            | Token::Comma { pos, .. }
            | Token::Minus { pos, .. }
            | Token::MinusEq { pos, .. }
            | Token::Dot { pos, .. }
            | Token::Slash { pos, .. }
            | Token::Colon { pos, .. }
            | Token::Colon2 { pos, .. }
            | Token::SemiColon { pos, .. }
            | Token::Lt { pos, .. }
            | Token::LtEq { pos, .. }
            | Token::Out { pos, .. }
            | Token::Eq { pos, .. }
            | Token::Eq2 { pos, .. }
            | Token::Arrow { pos, .. }
            | Token::Gt { pos, .. }
            | Token::GtEq { pos, .. }
            | Token::Question { pos, .. }
            | Token::At { pos, .. }
            | Token::OpenBracket { pos, .. }
            | Token::BackSlash { pos, .. }
            | Token::CloseBracket { pos, .. }
            | Token::Hat { pos, .. }
            | Token::OpenBrace { pos, .. }
            | Token::Or { pos, .. }
            | Token::Or2 { pos, .. }
            | Token::CloseBrace { pos, .. } => pos,
        }
    }

    pub fn into_pos(self) -> Pos {
        match self {
            Token::Eof { pos, .. }
            | Token::NewLine { pos, .. }
            | Token::Identifier { pos, .. }
            | Token::NumberLiteral { pos, .. }
            | Token::StringLiteral { pos, .. }
            | Token::Template { pos, .. }
            | Token::NullKeyword { pos, .. }
            | Token::TrueKeyword { pos, .. }
            | Token::FalseKeyword { pos, .. }
            | Token::EachKeyword { pos, .. }
            | Token::ForKeyword { pos, .. }
            | Token::LoopKeyword { pos, .. }
            | Token::DoKeyword { pos, .. }
            | Token::WhileKeyword { pos, .. }
            | Token::BreakKeyword { pos, .. }
            | Token::ContinueKeyword { pos, .. }
            | Token::MatchKeyword { pos, .. }
            | Token::CaseKeyword { pos, .. }
            | Token::DefaultKeyword { pos, .. }
            | Token::IfKeyword { pos, .. }
            | Token::ElifKeyword { pos, .. }
            | Token::ElseKeyword { pos, .. }
            | Token::ReturnKeyword { pos, .. }
            | Token::EvalKeyword { pos, .. }
            | Token::VarKeyword { pos, .. }
            | Token::LetKeyword { pos, .. }
            | Token::ExistsKeyword { pos, .. }
            | Token::Not { pos, .. }
            | Token::NotEq { pos, .. }
            | Token::Sharp { pos, .. }
            | Token::OpenSharpBracket { pos, .. }
            | Token::Sharp3 { pos, .. }
            | Token::Percent { pos, .. }
            | Token::And2 { pos, .. }
            | Token::OpenParen { pos, .. }
            | Token::CloseParen { pos, .. }
            | Token::Asterisk { pos, .. }
            | Token::Plus { pos, .. }
            | Token::PlusEq { pos, .. }
            | Token::Comma { pos, .. }
            | Token::Minus { pos, .. }
            | Token::MinusEq { pos, .. }
            | Token::Dot { pos, .. }
            | Token::Slash { pos, .. }
            | Token::Colon { pos, .. }
            | Token::Colon2 { pos, .. }
            | Token::SemiColon { pos, .. }
            | Token::Lt { pos, .. }
            | Token::LtEq { pos, .. }
            | Token::Out { pos, .. }
            | Token::Eq { pos, .. }
            | Token::Eq2 { pos, .. }
            | Token::Arrow { pos, .. }
            | Token::Gt { pos, .. }
            | Token::GtEq { pos, .. }
            | Token::Question { pos, .. }
            | Token::At { pos, .. }
            | Token::OpenBracket { pos, .. }
            | Token::BackSlash { pos, .. }
            | Token::CloseBracket { pos, .. }
            | Token::Hat { pos, .. }
            | Token::OpenBrace { pos, .. }
            | Token::Or { pos, .. }
            | Token::Or2 { pos, .. }
            | Token::CloseBrace { pos, .. } => pos,
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Token::Eof { .. } => "Eof",
            Token::NewLine { .. } => "NewLine",
            Token::Identifier { .. } => "Identifier",
            Token::NumberLiteral { .. } => "NumberLiteral",
            Token::StringLiteral { .. } => "StringLiteral",
            Token::Template { .. } => "Template",
            Token::NullKeyword { .. } => "NullKeyword",
            Token::TrueKeyword { .. } => "TrueKeyword",
            Token::FalseKeyword { .. } => "FalseKeyword",
            Token::EachKeyword { .. } => "EachKeyword",
            Token::ForKeyword { .. } => "ForKeyword",
            Token::LoopKeyword { .. } => "LoopKeyword",
            Token::DoKeyword { .. } => "DoKeyword",
            Token::WhileKeyword { .. } => "WhileKeyword",
            Token::BreakKeyword { .. } => "BreakKeyword",
            Token::ContinueKeyword { .. } => "ContinueKeyword",
            Token::MatchKeyword { .. } => "MatchKeyword",
            Token::CaseKeyword { .. } => "CaseKeyword",
            Token::DefaultKeyword { .. } => "DefaultKeyword",
            Token::IfKeyword { .. } => "IfKeyword",
            Token::ElifKeyword { .. } => "ElifKeyword",
            Token::ElseKeyword { .. } => "ElseKeyword",
            Token::ReturnKeyword { .. } => "ReturnKeyword",
            Token::EvalKeyword { .. } => "EvalKeyword",
            Token::VarKeyword { .. } => "VarKeyword",
            Token::LetKeyword { .. } => "LetKeyword",
            Token::ExistsKeyword { .. } => "ExistsKeyword",
            Token::Not { .. } => "Not",
            Token::NotEq { .. } => "NotEq",
            Token::Sharp { .. } => "Sharp",
            Token::OpenSharpBracket { .. } => "OpenSharpBracket",
            Token::Sharp3 { .. } => "Sharp3",
            Token::Percent { .. } => "Percent",
            Token::And2 { .. } => "And2",
            Token::OpenParen { .. } => "OpenParen",
            Token::CloseParen { .. } => "CloseParen",
            Token::Asterisk { .. } => "Asterisk",
            Token::Plus { .. } => "Plus",
            Token::PlusEq { .. } => "PlusEq",
            Token::Comma { .. } => "Comma",
            Token::Minus { .. } => "Minus",
            Token::MinusEq { .. } => "MinusEq",
            Token::Dot { .. } => "Dot",
            Token::Slash { .. } => "Slash",
            Token::Colon { .. } => "Colon",
            Token::Colon2 { .. } => "Colon2",
            Token::SemiColon { .. } => "SemiColon",
            Token::Lt { .. } => "Lt",
            Token::LtEq { .. } => "LtEq",
            Token::Out { .. } => "Out",
            Token::Eq { .. } => "Eq",
            Token::Eq2 { .. } => "Eq2",
            Token::Arrow { .. } => "Arrow",
            Token::Gt { .. } => "Gt",
            Token::GtEq { .. } => "GtEq",
            Token::Question { .. } => "Question",
            Token::At { .. } => "At",
            Token::OpenBracket { .. } => "OpenBracket",
            Token::BackSlash { .. } => "BackSlash",
            Token::CloseBracket { .. } => "CloseBracket",
            Token::Hat { .. } => "Hat",
            Token::OpenBrace { .. } => "OpenBrace",
            Token::Or { .. } => "Or",
            Token::Or2 { .. } => "Or2",
            Token::CloseBrace { .. } => "CloseBrace",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TemplateToken<'a> {
    TemplateStringElement { pos: Pos, value: &'a str },
    TemplateExprElement { pos: Pos, children: Vec<Token<'a>> },
}

pub type Tokens<'a> = Vec<Token<'a>>;

pub trait TokensExt<'a> {
    fn pop_token(&mut self) -> Token<'a>;

    fn peek(&self) -> &Token<'a>;

    fn lookahead(&self, offset: usize) -> &Token<'a>;
}

impl<'a> TokensExt<'a> for Tokens<'a> {
    fn pop_token(&mut self) -> Token<'a> {
        self.pop().unwrap_or(Token::Eof {
            pos: Pos { line: 0, column: 0 },
            has_left_spacing: false,
        })
    }

    fn peek(&self) -> &Token<'a> {
        self.last().unwrap_or(&Token::Eof {
            pos: Pos { line: 0, column: 0 },
            has_left_spacing: false,
        })
    }

    fn lookahead(&self, offset: usize) -> &Token<'a> {
        self.get(self.len() - offset - 1).unwrap_or(&Token::Eof {
            pos: Pos { line: 0, column: 0 },
            has_left_spacing: false,
        })
    }
}
