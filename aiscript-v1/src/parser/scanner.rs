use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag, take_till1, take_until},
    character::{
        anychar,
        complete::{alpha1, alphanumeric1, char, digit1, line_ending, multispace1, space0, space1},
    },
    combinator::{consumed, cut, fail, map, not, opt, peek, recognize, rest, success, value},
    multi::{many0, many1},
    sequence::{delimited, preceded, separated_pair, terminated},
};
use nom_locate::LocatedSpan;

use crate::{
    error::{AiScriptSyntaxError, AiScriptSyntaxErrorKind},
    node::Pos,
};

use super::token::{TemplateToken, Token};

type Span<'a> = LocatedSpan<&'a str>;

pub fn read_tokens(input: &str) -> Result<Vec<Token<'_>>, AiScriptSyntaxError> {
    let input = Span::new(input);
    terminated(many0(read_token), space0)
        .parse(input)
        .map_err(|e| match e {
            nom::Err::Incomplete(_) => AiScriptSyntaxError {
                kind: AiScriptSyntaxErrorKind::UnexpectedEof,
                pos: Pos { line: 0, column: 0 },
            },
            nom::Err::Error(nom::error::Error { input, .. })
            | nom::Err::Failure(nom::error::Error { input, .. }) => AiScriptSyntaxError {
                kind: if let Some(c) = input.chars().next() {
                    AiScriptSyntaxErrorKind::InvalidCharacter(c)
                } else {
                    AiScriptSyntaxErrorKind::UnexpectedEof
                },
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
            },
        })
        .and_then(|(input, mut tokens)| {
            let pos = Pos {
                line: input.location_line(),
                column: input.get_column(),
            };
            if let Some(c) = input.chars().next() {
                Err(AiScriptSyntaxError {
                    kind: AiScriptSyntaxErrorKind::InvalidCharacter(c),
                    pos,
                })
            } else {
                tokens.push(Token::Eof {
                    pos,
                    has_left_spacing: false,
                });
                Ok(tokens)
            }
        })
}

fn read_token(input: Span) -> IResult<Span, Token> {
    alt((
        preceded(space1, |input| read_token_no_space(input, true)),
        |input| read_token_no_space(input, false),
    ))
    .parse(input)
}

fn read_token_no_space(input: Span, has_left_spacing: bool) -> IResult<Span, Token> {
    alt((
        |input| skip_empty_lines(input, has_left_spacing),
        preceded(
            char('!'),
            alt((
                value(
                    Token::NotEq {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char('='),
                ),
                success(Token::Not {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                }),
            )),
        ),
        |input| read_string_literal(input, has_left_spacing),
        preceded(
            char('#'),
            alt((
                value(
                    Token::Sharp3 {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    preceded(char('#'), cut(char('#'))),
                ),
                value(
                    Token::OpenSharpBracket {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char('['),
                ),
                success(Token::Sharp {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                }),
            )),
        ),
        value(
            Token::Percent {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            char('%'),
        ),
        value(
            Token::And2 {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            preceded(char('&'), cut(char('&'))),
        ),
        value(
            Token::OpenParen {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            char('('),
        ),
        value(
            Token::CloseParen {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            char(')'),
        ),
        value(
            Token::Asterisk {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            char('*'),
        ),
        preceded(
            char('+'),
            alt((
                value(
                    Token::PlusEq {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char('='),
                ),
                success(Token::Plus {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                }),
            )),
        ),
        value(
            Token::Comma {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            char(','),
        ),
        preceded(
            char('-'),
            alt((
                value(
                    Token::MinusEq {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char('='),
                ),
                success(Token::Minus {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                }),
            )),
        ),
        value(
            Token::Dot {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            char('.'),
        ),
        preceded(
            char('/'),
            alt((
                preceded(char('*'), preceded(skip_comment_range, read_token)),
                preceded(char('/'), preceded(skip_comment_line, read_token)),
                success(Token::Slash {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                }),
            )),
        ),
        preceded(
            char(':'),
            alt((
                value(
                    Token::Colon2 {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char(':'),
                ),
                success(Token::Colon {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                }),
            )),
        ),
        value(
            Token::SemiColon {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            char(';'),
        ),
        preceded(
            char('<'),
            alt((
                value(
                    Token::LtEq {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char('='),
                ),
                value(
                    Token::Out {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char(':'),
                ),
                success(Token::Lt {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                }),
            )),
        ),
        preceded(
            char('='),
            alt((
                value(
                    Token::Eq2 {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char('='),
                ),
                value(
                    Token::Arrow {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char('>'),
                ),
                success(Token::Eq {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                }),
            )),
        ),
        preceded(
            char('>'),
            alt((
                value(
                    Token::GtEq {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    },
                    char('='),
                ),
                success(Token::Gt {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                }),
            )),
        ),
        alt((
            value(
                Token::Question {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                },
                char('?'),
            ),
            value(
                Token::At {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                },
                char('@'),
            ),
            value(
                Token::OpenBracket {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                },
                char('['),
            ),
            value(
                Token::BackSlash {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                },
                char('\\'),
            ),
            value(
                Token::CloseBracket {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                },
                char(']'),
            ),
            value(
                Token::Hat {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                },
                char('^'),
            ),
            |input| read_template(input, has_left_spacing),
            value(
                Token::OpenBrace {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                },
                char('{'),
            ),
            preceded(
                char('|'),
                alt((
                    value(
                        Token::Or2 {
                            pos: Pos {
                                line: input.location_line(),
                                column: input.get_column(),
                            },
                            has_left_spacing,
                        },
                        char('|'),
                    ),
                    success(Token::Or {
                        pos: Pos {
                            line: input.location_line(),
                            column: input.get_column(),
                        },
                        has_left_spacing,
                    }),
                )),
            ),
            value(
                Token::CloseBrace {
                    pos: Pos {
                        line: input.location_line(),
                        column: input.get_column(),
                    },
                    has_left_spacing,
                },
                char('}'),
            ),
            |input| read_digits(input, has_left_spacing),
            |input| read_word(input, has_left_spacing),
        )),
    ))
    .parse(input)
}

fn read_word(input: Span, has_left_spacing: bool) -> IResult<Span, Token> {
    map(
        recognize((
            alt((alpha1::<Span, _>, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |value| match value.into_fragment() {
            "null" => Token::NullKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "true" => Token::TrueKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "false" => Token::FalseKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "each" => Token::EachKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "for" => Token::ForKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "loop" => Token::LoopKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "do" => Token::DoKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "while" => Token::WhileKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "break" => Token::BreakKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "continue" => Token::ContinueKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "match" => Token::MatchKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "case" => Token::CaseKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "default" => Token::DefaultKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "if" => Token::IfKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "elif" => Token::ElifKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "else" => Token::ElseKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "return" => Token::ReturnKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "eval" => Token::EvalKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "var" => Token::VarKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "let" => Token::LetKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            "exists" => Token::ExistsKeyword {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
            },
            value => Token::Identifier {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
                value,
            },
        },
    )
    .parse(input)
}

fn read_digits(input: Span, has_left_spacing: bool) -> IResult<Span, Token> {
    map(
        recognize(preceded(
            digit1::<Span, _>,
            opt(preceded(char('.'), cut(digit1))),
        )),
        |value| Token::NumberLiteral {
            pos: Pos {
                line: input.location_line(),
                column: input.get_column(),
            },
            has_left_spacing,
            value: value.into_fragment(),
        },
    )
    .parse(input)
}

fn read_string_literal(input: Span, has_left_spacing: bool) -> IResult<Span, Token> {
    fn read_string_literal(
        input: Span,
        has_left_spacing: bool,
        literal_mark: char,
    ) -> IResult<Span, Token> {
        map(
            terminated(
                many0(alt((
                    preceded(char::<Span, _>('\\'), cut(recognize(anychar))),
                    take_till1(|c| c == '\\' || c == literal_mark),
                ))),
                cut(char(literal_mark)),
            ),
            |value| Token::StringLiteral {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
                value: value.into_iter().map(|s| s.into_fragment()).collect(),
            },
        )
        .parse(input)
    }

    alt((
        preceded(
            char('\''),
            cut(|input| read_string_literal(input, has_left_spacing, '\'')),
        ),
        preceded(
            char('"'),
            cut(|input| read_string_literal(input, has_left_spacing, '"')),
        ),
    ))
    .parse(input)
}

fn read_template(input: Span, has_left_spacing: bool) -> IResult<Span, Token> {
    fn template_expr(input: Span) -> IResult<Span, Vec<Token>> {
        map(
            many1(alt((
                map(
                    (
                        map(
                            (space0::<Span<'_>, _>, recognize(char('{'))),
                            |(spaces, c)| Token::OpenBrace {
                                pos: Pos {
                                    line: c.location_line(),
                                    column: c.get_column(),
                                },
                                has_left_spacing: !spaces.is_empty(),
                            },
                        ),
                        template_expr,
                        cut(map(
                            (space0::<Span<'_>, _>, recognize(char('}'))),
                            |(spaces, c)| Token::CloseBrace {
                                pos: Pos {
                                    line: c.location_line(),
                                    column: c.get_column(),
                                },
                                has_left_spacing: !spaces.is_empty(),
                            },
                        )),
                    ),
                    |(open_brace, tokens, close_brace)| {
                        let mut v = Vec::with_capacity(tokens.len() + 2);
                        v.push(open_brace);
                        v.extend(tokens);
                        v.push(close_brace);
                        v
                    },
                ),
                map(
                    preceded(peek(not((space0, char('}')))), read_token),
                    |value| vec![value],
                ),
            ))),
            |children| children.concat(),
        )
        .parse(input)
    }

    delimited(
        char('`'),
        map(
            many0(alt((
                map(
                    (recognize(char::<Span, _>('\\')), cut(recognize(anychar))),
                    |(esc, c)| TemplateToken::TemplateStringElement {
                        pos: Pos {
                            line: esc.location_line(),
                            column: esc.get_column(),
                        },
                        value: &c,
                    },
                ),
                delimited(
                    char('{'),
                    map(
                        separated_pair(consumed(template_expr), space0, recognize(success(()))),
                        |((span, mut children), end_span)| {
                            children.push(Token::Eof {
                                pos: Pos {
                                    line: end_span.location_line(),
                                    column: end_span.get_column(),
                                },
                                has_left_spacing: false,
                            });
                            TemplateToken::TemplateExprElement {
                                pos: Pos {
                                    line: span.location_line(),
                                    column: span.get_column(),
                                },
                                children,
                            }
                        },
                    ),
                    cut(char('}')),
                ),
                map(
                    recognize(take_till1(|c| c == '\\' || c == '{' || c == '`')),
                    |span: Span<'_>| TemplateToken::TemplateStringElement {
                        pos: Pos {
                            line: span.location_line(),
                            column: span.get_column(),
                        },
                        value: &span,
                    },
                ),
            ))),
            |children| Token::Template {
                pos: Pos {
                    line: input.location_line(),
                    column: input.get_column(),
                },
                has_left_spacing,
                children,
            },
        ),
        cut(char('`')),
    )
    .parse(input)
}

fn skip_empty_lines(input: Span, has_left_spacing: bool) -> IResult<Span, Token> {
    value(
        Token::NewLine {
            pos: Pos {
                line: input.location_line(),
                column: input.get_column(),
            },
            has_left_spacing,
        },
        preceded(
            line_ending,
            many0(alt((
                multispace1,
                recognize(preceded(
                    char('/'),
                    alt((
                        preceded(char('*'), skip_comment_range),
                        preceded(char('/'), skip_comment_line),
                    )),
                )),
            ))),
        ),
    )
    .parse(input)
}

fn skip_comment_line(input: Span) -> IResult<Span, ()> {
    value((), is_not("\n")).parse(input)
}

fn skip_comment_range(input: Span) -> IResult<Span, ()> {
    alt((
        value((), (take_until("*/"), tag("*/"))),
        value((), (rest, cut(fail::<_, Span, _>()))),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eof() {
        let tokens = read_tokens("").unwrap();
        assert_eq!(
            tokens[0],
            Token::Eof {
                pos: Pos { line: 1, column: 1 },
                has_left_spacing: false,
            }
        );
    }

    #[test]
    fn keyword() {
        let tokens = read_tokens("if").unwrap();
        assert_eq!(
            tokens[0],
            Token::IfKeyword {
                pos: Pos { line: 1, column: 1 },
                has_left_spacing: false,
            }
        );
        assert_eq!(
            tokens[1],
            Token::Eof {
                pos: Pos { line: 1, column: 3 },
                has_left_spacing: false,
            }
        );
    }

    #[test]
    fn identifier() {
        let tokens = read_tokens("xyz").unwrap();
        assert_eq!(
            tokens[0],
            Token::Identifier {
                pos: Pos { line: 1, column: 1 },
                has_left_spacing: false,
                value: "xyz"
            }
        );
        assert_eq!(
            tokens[1],
            Token::Eof {
                pos: Pos { line: 1, column: 4 },
                has_left_spacing: false,
            }
        );
    }

    #[test]
    fn invalid_token() {
        read_token(Span::new("$")).unwrap_err();
        read_token(Span::new("~")).unwrap_err();
    }

    #[test]
    fn words() {
        let tokens = read_tokens("abc xyz").unwrap();
        assert_eq!(
            tokens[0],
            Token::Identifier {
                pos: Pos { line: 1, column: 1 },
                has_left_spacing: false,
                value: "abc"
            }
        );
        assert_eq!(
            tokens[1],
            Token::Identifier {
                pos: Pos { line: 1, column: 5 },
                has_left_spacing: true,
                value: "xyz"
            }
        );
        assert_eq!(
            tokens[2],
            Token::Eof {
                pos: Pos { line: 1, column: 8 },
                has_left_spacing: false,
            }
        );
    }

    #[test]
    fn stream() {
        let tokens = read_tokens("@abc() { }").unwrap();
        assert_eq!(
            tokens[0],
            Token::At {
                pos: Pos { line: 1, column: 1 },
                has_left_spacing: false,
            }
        );
        assert_eq!(
            tokens[1],
            Token::Identifier {
                pos: Pos { line: 1, column: 2 },
                has_left_spacing: false,
                value: "abc"
            }
        );
        assert_eq!(
            tokens[2],
            Token::OpenParen {
                pos: Pos { line: 1, column: 5 },
                has_left_spacing: false,
            }
        );
        assert_eq!(
            tokens[3],
            Token::CloseParen {
                pos: Pos { line: 1, column: 6 },
                has_left_spacing: false,
            }
        );
        assert_eq!(
            tokens[4],
            Token::OpenBrace {
                pos: Pos { line: 1, column: 8 },
                has_left_spacing: true
            }
        );
        assert_eq!(
            tokens[5],
            Token::CloseBrace {
                pos: Pos {
                    line: 1,
                    column: 10
                },
                has_left_spacing: true
            }
        );
        assert_eq!(
            tokens[6],
            Token::Eof {
                pos: Pos {
                    line: 1,
                    column: 11
                },
                has_left_spacing: false,
            }
        );
    }

    #[test]
    fn multi_lines() {
        let tokens = read_tokens("aaa\nbbb").unwrap();
        assert_eq!(
            tokens[0],
            Token::Identifier {
                pos: Pos { line: 1, column: 1 },
                has_left_spacing: false,
                value: "aaa"
            }
        );
        assert_eq!(
            tokens[1],
            Token::NewLine {
                pos: Pos { line: 1, column: 4 },
                has_left_spacing: false,
            }
        );
        assert_eq!(
            tokens[2],
            Token::Identifier {
                pos: Pos { line: 2, column: 1 },
                has_left_spacing: false,
                value: "bbb"
            }
        );
        assert_eq!(
            tokens[3],
            Token::Eof {
                pos: Pos { line: 2, column: 4 },
                has_left_spacing: false,
            }
        );
    }

    #[test]
    fn empty_lines() {
        let tokens = read_tokens("match 1{\n// comment\n}").unwrap();
        assert_eq!(
            tokens[0],
            Token::MatchKeyword {
                pos: Pos { line: 1, column: 1 },
                has_left_spacing: false,
            }
        );
        assert_eq!(
            tokens[1],
            Token::NumberLiteral {
                pos: Pos { line: 1, column: 7 },
                has_left_spacing: true,
                value: "1"
            }
        );
        assert_eq!(
            tokens[2],
            Token::OpenBrace {
                pos: Pos { line: 1, column: 8 },
                has_left_spacing: false,
            }
        );
        assert_eq!(
            tokens[3],
            Token::NewLine {
                pos: Pos { line: 1, column: 9 },
                has_left_spacing: false,
            }
        );
        assert_eq!(
            tokens[4],
            Token::CloseBrace {
                pos: Pos { line: 3, column: 1 },
                has_left_spacing: false,
            }
        );
        assert_eq!(
            tokens[5],
            Token::Eof {
                pos: Pos { line: 3, column: 2 },
                has_left_spacing: false,
            }
        );
    }
}
