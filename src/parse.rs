use std::{fmt, mem::take};

use ecow::EcoString;

use crate::{ast::*, lex, InputSrc, Inputs, LexError, Primitive as Prim, Sp, Span, Token};

use Token::*;

pub fn parse(
    src: InputSrc,
    text: impl Into<EcoString>,
    inputs: &mut Inputs,
) -> (Vec<Item>, Vec<Sp<ParseError>>) {
    parse_impl(src, text.into(), inputs)
}

fn parse_impl(
    src: InputSrc,
    text: EcoString,
    inputs: &mut Inputs,
) -> (Vec<Item>, Vec<Sp<ParseError>>) {
    let tokens = match lex(src, text, inputs) {
        Ok(tokens) => tokens,
        Err(e) => return (Vec::new(), vec![e.map(ParseError::Lex)]),
    };
    let mut parser = Parser {
        inputs,
        tokens,
        curr: 0,
        errors: Vec::new(),
    };
    let items = parser.items();
    let mut errors = parser.errors;
    if parser.curr < parser.tokens.len() {
        errors.push(
            parser.tokens[parser.curr]
                .clone()
                .map(ParseError::UnexpectedToken),
        );
    }
    (items, errors)
}

struct Parser<'a> {
    inputs: &'a Inputs,
    tokens: Vec<Sp<Token>>,
    curr: usize,
    errors: Vec<Sp<ParseError>>,
}

impl<'a> Parser<'a> {
    fn next_token_map<T>(&mut self, f: impl FnOnce(&Token, &str) -> Option<T>) -> Option<Sp<T>> {
        let token = self.tokens.get(self.curr)?;
        let s = self.inputs.span_text(token.span);
        let res = f(&token.value, s)?;
        self.curr += 1;
        Some(token.span.sp(res))
    }
    fn next_token_exact(&mut self, token: Token) -> Option<Span> {
        self.next_token_map(|t, _| if t == &token { Some(()) } else { None })
            .map(|sp| sp.span)
    }
    fn items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        while let Some(item) = self.item() {
            items.push(item);
            self.newline();
        }
        items
    }
    fn item(&mut self) -> Option<Item> {
        let words = self.words()?;
        Some(Item::Words(words))
    }
    fn words(&mut self) -> Option<Vec<Sp<Word>>> {
        let mut words = Vec::new();
        while let Some(word) = self.word() {
            words.push(word);
        }
        if words.is_empty() {
            None
        } else {
            Some(words)
        }
    }
    fn word(&mut self) -> Option<Sp<Word>> {
        self.term()
    }
    fn term(&mut self) -> Option<Sp<Word>> {
        Some(if let Some(num) = self.number() {
            num.map(Word::Number)
        } else if let Some(mon) = self.next_token_map(|t, _| match t {
            Token::Primitive(Prim::Mon(p)) => Some(*p),
            _ => None,
        }) {
            mon.map(Word::Mon)
        } else if let Some(dy) = self.next_token_map(|t, _| match t {
            Token::Primitive(Prim::Dy(p)) => Some(*p),
            _ => None,
        }) {
            dy.map(Word::Dy)
        } else if let Some(m) = self.modified() {
            m.map(Word::Mod)
        } else if let Some(Ok(func)) = self.func(false) {
            func
        } else if let Some(open) = self.next_token_exact(OpenBracket) {
            self.newline();
            let mut lines = Vec::new();
            while let Some(line) = self.words() {
                lines.push(line);
                self.newline();
            }
            let close = self.expect(CloseBracket);
            let span = open.merge(close);
            span.sp(Word::Array(Array { open, lines, close }))
        } else {
            return None;
        })
    }
    fn func(&mut self, allow_pack: bool) -> Option<Result<Sp<Word>, Vec<Sp<Word>>>> {
        let open = self.next_token_exact(OpenParen)?;
        self.newline();
        let mut first_lines = Vec::new();
        while let Some(line) = self.words() {
            first_lines.push(line);
            self.newline();
        }
        let mut other_lines = Vec::new();
        if allow_pack {
            while let Some(bar_span) = self.next_token_exact(Bar) {
                self.newline();
                let mut lines = Vec::new();
                while let Some(line) = self.words() {
                    lines.push(line);
                    self.newline();
                }
                other_lines.push((bar_span, lines));
                self.newline();
            }
        }
        let close = self.expect(CloseParen);
        Some(if other_lines.is_empty() {
            Ok(open.merge(close).sp(Word::Func(Func {
                open,
                lines: first_lines,
                close: Some(close),
            })))
        } else {
            let mut lines = Vec::new();
            let first_span = open.merge(
                (first_lines.iter().flatten().last().map(|w| w.span))
                    .unwrap_or_else(|| other_lines.first().unwrap().0),
            );
            lines.push(first_span.sp(Word::Func(Func {
                open,
                lines: first_lines,
                close: None,
            })));
            let other_len = other_lines.len();
            for i in 0..other_len {
                let &(bar_span, _) = other_lines.get(i).unwrap();
                let span = bar_span.merge(
                    other_lines
                        .get(i + 1)
                        .map(|&(bar_span, _)| bar_span)
                        .unwrap_or(close),
                );
                let (bar_span, other) = other_lines.get_mut(i).unwrap();
                lines.push(span.sp(Word::Func(Func {
                    open: *bar_span,
                    lines: take(other),
                    close: if i == other_len - 1 {
                        None
                    } else {
                        Some(close)
                    },
                })));
            }
            Err(lines)
        })
    }
    fn modified(&mut self) -> Option<Sp<Modified>> {
        let (modifier, margs) = if let Some(mon) = self.next_token_map(|t, _| match t {
            Token::Primitive(Prim::MonMod(p)) => Some(*p),
            _ => None,
        }) {
            (mon.map(Modifier::Mon), 1)
        } else if let Some(dy) = self.next_token_map(|t, _| match t {
            Token::Primitive(Prim::DyMod(p)) => Some(*p),
            _ => None,
        }) {
            (dy.map(Modifier::Dy), 2)
        } else {
            return None;
        };
        let mut pack = false;
        let args = match self.func(true) {
            Some(Err(args)) => {
                pack = true;
                args
            }
            Some(Ok(first)) => {
                let mut args = vec![first];
                for _ in 0..margs - 1 {
                    if let Some(arg) = self.word() {
                        args.push(arg);
                    } else {
                        break;
                    }
                }
                args
            }
            None => {
                let mut args = Vec::new();
                for _ in 0..margs {
                    if let Some(arg) = self.word() {
                        args.push(arg);
                    } else {
                        break;
                    }
                }
                args
            }
        };
        let mut span = modifier.span;
        if let Some(arg) = args.last() {
            span = span.merge(arg.span);
        }
        Some(span.sp(Modified {
            modifier,
            args,
            pack,
        }))
    }
    fn newline(&mut self) -> bool {
        let mut newline = false;
        while self.next_token_exact(Newline).is_some() {
            newline = true;
        }
        newline
    }
    fn number(&mut self) -> Option<Sp<f64>> {
        self.next_token_map(|t, s| match t {
            Token::Number => s.replace('`', "-").parse().ok(),
            _ => None,
        })
    }
    fn curr_span(&self) -> Span {
        self.tokens
            .get(self.curr)
            .unwrap_or_else(|| self.tokens.last().unwrap())
            .span
    }
    fn expect(&mut self, token: Token) -> Span {
        self.next_token_exact(token.clone()).unwrap_or_else(|| {
            let span = self.curr_span();
            self.errors.push(span.sp(ParseError::ExpectedToken(token)));
            span
        })
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    Lex(LexError),
    ExpectedToken(Token),
    UnexpectedToken(Token),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Lex(e) => write!(f, "{e}"),
            ParseError::ExpectedToken(t) => write!(f, "Expected {t:?}"),
            ParseError::UnexpectedToken(t) => write!(f, "Unexpected {t:?}"),
        }
    }
}
