use std::{fmt, ops::Deref, path::PathBuf};

use ecow::EcoString;

use crate::{LocatedError, PrimKind, Primitive};

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
    Primitive(Primitive),
    Number,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenCurly,
    CloseCurly,
    Equals,
    Bar,
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
    fn next_char_exact(&mut self, c: char) -> bool {
        self.next_char_if(|c_| c_ == c).is_some()
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
                '=' => self.end(start, Token::Equals),
                '|' => self.end(start, Token::Bar),
                '\n' => self.end(start, Token::Newline),
                ' ' | '\t' | '\r' => {}
                c if c.is_ascii_digit()
                    || c == '`' && self.next_char_if(|c| c.is_ascii_digit()).is_some() =>
                {
                    while self.next_char_if(|c| c.is_ascii_digit()).is_some() {}
                    if self.next_char_exact('.') {
                        while self.next_char_if(|c| c.is_ascii_digit()).is_some() {}
                    }
                    let reset = self.loc;
                    if self.next_char_exact('e') || self.next_char_exact('E') {
                        self.next_char_exact('`');
                        let mut got_dec = false;
                        while self.next_char_if(|c| c.is_ascii_digit()).is_some() {
                            got_dec = true;
                        }
                        if !got_dec {
                            self.loc = reset;
                        }
                    }
                    self.end(start, Token::Number);
                }
                c => {
                    if let Some(prim) = Primitive::from_glyph(c) {
                        self.end(start, Token::Primitive(prim));
                    } else {
                        return Err(self.span(start).sp(LexError::InvalidChar(c)));
                    }
                }
            }
        }
        Ok(self.tokens)
    }
}

#[derive(Debug, Clone)]
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.src, self.start.char, self.end.char)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start.char, self.end.char)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sp<T, S = Span> {
    pub value: T,
    pub span: S,
}

pub type HumanSp<T> = Sp<T, HumanSpan>;

impl<T, S> Sp<T, S> {
    pub fn new(value: T, span: S) -> Self {
        Self { value, span }
    }
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Sp<U, S> {
        Sp::new(f(self.value), self.span)
    }
}

impl<T, S> fmt::Debug for Sp<T, S>
where
    T: fmt::Debug,
    S: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.span)?;
        self.value.fmt(f)
    }
}

impl<T, S> fmt::Display for Sp<T, S>
where
    T: fmt::Display,
    S: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.span, self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HumanLoc {
    pub line: usize,
    pub col: usize,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct HumanSpan {
    pub start: HumanLoc,
    pub end: HumanLoc,
    pub src: InputSrc,
}

impl fmt::Debug for HumanSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for HumanSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.src {
            InputSrc::File(path) => write!(f, "{}:", path.display())?,
            InputSrc::Str => {}
        }
        write!(f, "{}:{}", self.start.line, self.start.col)
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

#[derive(Debug, Clone, Default)]
pub struct Inputs {
    pub inputs: Vec<Input>,
}

impl Inputs {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, input: Input) {
        self.inputs.push(input);
    }
    pub fn span_text(&self, span: Span) -> &str {
        let text = self.inputs[span.src].text.as_str();
        &text[span.start.byte..span.end.byte]
    }
    pub fn human_loc(&self, loc: Loc, src: usize) -> HumanLoc {
        let text = self.inputs[src].text.as_str();
        let line = text[..loc.byte].chars().filter(|c| *c == '\n').count() + 1;
        let col = text[..loc.byte]
            .chars()
            .rev()
            .take_while(|c| *c != '\n')
            .count()
            + 1;
        HumanLoc { line, col }
    }
    pub fn human_span(&self, span: Span) -> HumanSpan {
        let start = self.human_loc(span.start, span.src);
        let end = self.human_loc(span.end, span.src);
        HumanSpan {
            start,
            end,
            src: self.inputs[span.src].src.clone(),
        }
    }
    pub fn human_sp<T>(&self, sp: Sp<T>) -> HumanSp<T> {
        Sp::new(sp.value, self.human_span(sp.span))
    }
    pub fn error(&self, span: Span, message: impl Into<EcoString>) -> LocatedError {
        let text = self.inputs[span.src].text.as_str();
        let span = self.human_span(span);
        let message = message.into();
        let line = text.split('\n').nth(span.start.line).unwrap();
        LocatedError {
            span,
            message,
            line: line.into(),
        }
    }
}

impl Deref for Inputs {
    type Target = [Input];
    fn deref(&self) -> &Self::Target {
        &self.inputs
    }
}
