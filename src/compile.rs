use std::mem::take;

use ecow::EcoString;

use crate::{
    ast::*, parse, InputSrc, Inputs, Node, Sp, Span, UfelError, UfelErrorKind, UfelResult,
};

#[derive(Clone, Default)]
pub struct Compiler {
    pub asm: Assembly,
    pub errors: Vec<UfelError>,
}

#[derive(Clone, Default)]
pub struct Assembly {
    pub root: Node,
    pub inputs: Inputs,
    pub spans: Vec<Span>,
}

impl Compiler {
    pub fn load(&mut self, src: InputSrc, text: impl Into<EcoString>) -> UfelResult {
        self.load_impl(src, text.into())
    }
    pub fn load_str(&mut self, text: impl Into<EcoString>) -> UfelResult {
        self.load_impl(InputSrc::Str, text.into())
    }
    fn load_impl(&mut self, src: InputSrc, text: EcoString) -> UfelResult {
        let (items, errors) = parse(src, text, &mut self.asm.inputs);
        if let Some(error) =
            UfelError::from_iter(errors.into_iter().map(|e| self.asm.inputs.human_sp(e)))
        {
            return Err(error);
        }
        let mut added_item_error = false;
        for item in items {
            if let Err(e) = self.item(item) {
                if !added_item_error {
                    self.errors.push(e);
                    added_item_error = true;
                }
            }
        }
        if let Some(error) = UfelError::from_iter(take(&mut self.errors)) {
            Err(error)
        } else {
            Ok(())
        }
    }
    fn item(&mut self, item: Item) -> UfelResult {
        match item {
            Item::Words(words) => {
                let node = self.line(words)?;
                self.asm.root.push(node);
            }
        }
        Ok(())
    }
    fn line(&mut self, words: Vec<Sp<Word>>) -> UfelResult<Node> {
        let mut node = Node::empty();
        for word in words {
            node.push(self.word(word)?);
        }
        Ok(node)
    }
    fn word(&mut self, word: Sp<Word>) -> UfelResult<Node> {
        Ok(match word.value {
            Word::Number(n) => Node::new_push(n),
            Word::Func(func) => {
                let mut node = Node::empty();
                for line in func.lines {
                    node.push(self.line(line)?);
                }
                node
            }
            Word::Array(array) => {
                let mut inner = Node::empty();
                for line in array.lines {
                    inner.push(self.line(line)?);
                }
                let span = self.add_span(word.span);
                let sig = inner.sig();
                Node::Array(sig.outputs, inner.into(), span)
            }
            Word::Mon(monadic) => {
                let span = self.add_span(word.span);
                Node::Mon(monadic, span)
            }
            Word::Dy(dyadic) => {
                let span = self.add_span(word.span);
                Node::Dy(dyadic, span)
            }
            Word::Mod(modified) => return self.modified(modified, word.span),
        })
    }
    fn modified(&mut self, modified: Modified, span: Span) -> UfelResult<Node> {
        if modified.args.len() > modified.modifier.value.arg_count() {
            self.add_error(
                span,
                format!(
                    "{:?} takes {} operands but {} were supplied",
                    modified.modifier.value,
                    modified.modifier.value.arg_count(),
                    modified.args.len()
                ),
            );
        }
        let mut args = modified
            .args
            .into_iter()
            .map(|word| self.word(word).map(Node::sig_node));
        Ok(match modified.modifier.value {
            Modifier::Mon(m) => {
                let f = args.next().transpose()?.unwrap_or_default();
                let span = self.add_span(modified.modifier.span);
                Node::Mod(m, f.into(), span)
            }
            Modifier::Dy(d) => {
                let f = args.next().transpose()?.unwrap_or_default();
                let g = args.next().transpose()?.unwrap_or_default();
                let span = self.add_span(modified.modifier.span);
                Node::DyMod(d, f.into(), g.into(), span)
            }
        })
    }
    fn add_span(&mut self, span: Span) -> usize {
        self.asm.spans.push(span);
        self.asm.spans.len() - 1
    }
    fn add_error(&mut self, span: Span, message: impl Into<EcoString>) {
        self.errors.push(self.error(span, message));
    }
    fn error(&self, span: Span, message: impl Into<EcoString>) -> UfelError {
        UfelErrorKind::Compile(self.asm.inputs.error(span, message.into())).into()
    }
}
