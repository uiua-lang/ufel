use std::{fmt, path::PathBuf};

use ecow::EcoString;

pub fn lex(
    src: InputSrc,
    text: impl Into<EcoString>,
    inputs: &mut Inputs,
) -> Result<Vec<Sp<Token>>, Sp<LexError>> {
    lex_impl(src, text.into(), inputs)
}

fn lex_impl(
    src: InputSrc,
    text: EcoString,
    inputs: &mut Inputs,
) -> Result<Vec<Sp<Token>>, Sp<LexError>> {
    inputs.push(Input::new(src, text));
    let src = inputs.len() - 1;
    let text = inputs.last().unwrap().text.as_str();
    Lexer {
        text,
        src,
        loc: Loc { char: 0, byte: 0 },
        tokens: Vec::new(),
    }
    .lex()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenCurly,
    CloseCurly,
    Newline,
}

struct Lexer<'a> {
    text: &'a str,
    src: usize,
    loc: Loc,
    tokens: Vec<Sp<Token>>,
}

impl<'a> Lexer<'a> {
    fn next_char_if(&mut self, f: impl FnOnce(char) -> bool) -> Option<char> {
        let c = self.text[self.loc.byte..].chars().next()?;
        if !f(c) {
            return None;
        }
        self.loc.char += 1;
        self.loc.byte += c.len_utf8();
        Some(c)
    }
    fn end(&mut self, start: Loc, token: impl Into<Token>) {
        let end = self.loc;
        let span = Span {
            start,
            end,
            src: self.src,
        };
        self.tokens.push(span.sp(token.into()));
    }
    fn span(&self, start: Loc) -> Span {
        Span {
            start,
            end: self.loc,
            src: self.src,
        }
    }
    fn lex(mut self) -> Result<Vec<Sp<Token>>, Sp<LexError>> {
        loop {
            let start = self.loc;
            let Some(c) = self.next_char_if(|_| true) else {
                break;
            };
            match c {
                '(' => self.end(start, Token::OpenParen),
                ')' => self.end(start, Token::CloseParen),
                '[' => self.end(start, Token::OpenBracket),
                ']' => self.end(start, Token::CloseBracket),
                '{' => self.end(start, Token::OpenCurly),
                '}' => self.end(start, Token::CloseCurly),
                ' ' | '\t' | '\r' => {}
                c => return Err(self.span(start).sp(LexError::InvalidChar(c))),
            }
        }
        Ok(self.tokens)
    }
}

#[derive(Debug)]
pub enum LexError {
    InvalidChar(char),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::InvalidChar(c) => write!(f, "Invalid chararacter: {c:?}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Loc {
    pub char: usize,
    pub byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Loc,
    pub end: Loc,
    pub src: usize,
}

impl Span {
    pub fn merge(self, other: Self) -> Self {
        assert_eq!(
            self.src, other.src,
            "Cannot merge spans from different inputs"
        );
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            src: self.src,
        }
    }
    pub fn sp<T>(self, value: T) -> Sp<T> {
        Sp::new(value, self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sp<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Sp<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Sp<U> {
        Sp::new(f(self.value), self.span)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputSrc {
    File(PathBuf),
    Str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Input {
    pub src: InputSrc,
    pub text: EcoString,
}

impl Input {
    pub fn new(src: InputSrc, text: impl Into<EcoString>) -> Self {
        Self {
            src,
            text: text.into(),
        }
    }
}

pub type Inputs = Vec<Input>;
