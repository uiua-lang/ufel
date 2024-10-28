use std::mem::take;

use ecow::EcoString;

use crate::{
    Array, Assembly, Compiler, DyMod, Dyadic, InputSrc, Mod, Monadic, Node, SigNode, UfelError,
    UfelErrorKind, UfelResult,
};

#[derive(Clone, Default)]
pub struct Ufel {
    pub asm: Assembly,
    stack: Vec<Array>,
    trace: Vec<usize>,
}

impl Ufel {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn run(&mut self, src: InputSrc, text: impl Into<EcoString>) -> UfelResult {
        let mut compiler = Compiler::default();
        compiler.load(src, text.into())?;
        self.asm = compiler.asm;
        self.exec(self.asm.root.clone())?;
        Ok(())
    }
    pub fn run_str(&mut self, text: impl Into<EcoString>) -> UfelResult {
        self.run(InputSrc::Str, text.into())
    }
    pub fn exec(&mut self, node: Node) -> UfelResult {
        match node {
            Node::Run(nodes) => {
                for node in nodes {
                    self.exec(node)?;
                }
            }
            Node::Push(val) => self.push(val),
            Node::Array(len, inner, span) => self.with_span(span, |rt| {
                rt.exec(*inner)?;
                rt.require_height(len)?;
                let start = rt.stack.len() - len;
                let rows: Vec<Array> = rt.stack.drain(start..).rev().collect();
                let arr = Array::from_row_arrays(rows, rt)?;
                rt.push(arr);
                Ok(())
            })?,
            Node::Mon(prim, span) => self.with_span(span, |rt| rt.monadic(prim))?,
            Node::Dy(prim, span) => self.with_span(span, |rt| rt.dyadic(prim))?,
            Node::Mod(prim, f, span) => self.with_span(span, |rt| rt.mon_mod(prim, *f))?,
            Node::DyMod(prim, f, g, span) => self.with_span(span, |rt| rt.dy_mod(prim, *f, *g))?,
        }
        Ok(())
    }
    fn with_span(&mut self, span: usize, f: impl FnOnce(&mut Self) -> UfelResult) -> UfelResult {
        self.trace.push(span);
        let res = f(self);
        self.trace.pop();
        res
    }
    pub fn error(&self, message: impl Into<EcoString>) -> UfelError {
        let span = self.trace.last().copied().unwrap_or(0);
        let span = &self.asm.spans[span];
        let span = self.asm.inputs.error(*span, message.into());
        UfelErrorKind::Run(span).into()
    }
    fn monadic(&mut self, prim: Monadic) -> UfelResult {
        let a = self.pop(1)?;
        let res = match prim {
            Monadic::Identity => a,
            Monadic::Neg => a.neg(),
            Monadic::Not => a.not(),
            Monadic::Abs => a.abs(),
            Monadic::Sign => a.sign(),
            Monadic::Len => todo!(),
            Monadic::Shape => todo!(),
            Monadic::Form => todo!(),
        };
        self.push(res);
        Ok(())
    }
    fn dyadic(&mut self, prim: Dyadic) -> UfelResult {
        let a = self.pop(1)?;
        let b = self.pop(2)?;
        let res = match prim {
            Dyadic::Add => a.add(b, 0, 0, self)?,
            Dyadic::Sub => a.sub(b, 0, 0, self)?,
            Dyadic::Mul => a.mul(b, 0, 0, self)?,
            Dyadic::Div => a.div(b, 0, 0, self)?,
            Dyadic::Mod => a.mod_(b, 0, 0, self)?,
            Dyadic::Eq => a.eq(b, 0, 0, self)?,
            Dyadic::Lt => a.lt(b, 0, 0, self)?,
            Dyadic::Gt => a.gt(b, 0, 0, self)?,
            Dyadic::Min => a.min(b, 0, 0, self)?,
            Dyadic::Max => a.max(b, 0, 0, self)?,
        };
        self.push(res);
        Ok(())
    }
    fn mon_mod(&mut self, prim: Mod, f: SigNode) -> UfelResult {
        match prim {
            Mod::Dip => {
                let a = self.pop(1);
                self.exec(f.node)?;
                self.stack.push(a?);
            }
            Mod::Reduce => todo!(),
            Mod::Scan => todo!(),
        }
        Ok(())
    }
    fn dy_mod(&mut self, prim: DyMod, f: SigNode, g: SigNode) -> UfelResult {
        match prim {
            DyMod::Fork => {
                let f_args = self.copy_n(f.sig.args)?;
                self.exec(g.node)?;
                self.stack.extend(f_args.into_iter().rev());
                self.exec(f.node)?;
            }
        }
        Ok(())
    }
    pub fn push(&mut self, val: impl Into<Array>) {
        self.stack.push(val.into());
    }
    pub fn pop(&mut self, n: usize) -> UfelResult<Array> {
        self.stack
            .pop()
            .ok_or_else(|| self.error(format!("Stack was empty when getting argument {n}")))
    }
    fn copy_n(&self, n: usize) -> UfelResult<Vec<Array>> {
        self.require_height(n)?;
        Ok(self.stack[self.stack.len() - n..].to_vec())
    }
    fn require_height(&self, n: usize) -> UfelResult {
        if self.stack.len() < n {
            return Err(self.error(format!(
                "Stack was empty when getting argument {}",
                n - self.stack.len()
            )));
        }
        Ok(())
    }
    pub fn take_stack(&mut self) -> Vec<Array> {
        take(&mut self.stack)
    }
}
