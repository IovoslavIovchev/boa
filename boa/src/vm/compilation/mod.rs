use super::*;
use crate::syntax::ast::{constant::Const, node::*, Node};
use crate::BoaProfiler;

// this..?
#[derive(Debug, Default)]
pub(crate) struct Compiler {
    res: Vec<Instruction>,
    next_free: u8,
}

// or maybe..
// `impl CodeGen for BinOp` ?
trait CodeGen {
    fn compile(&self, compiler: &mut Compiler);
}

impl Compiler {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn compile(&mut self, list: &StatementList) -> Vec<Instruction> {
        for stmt in list.statements() {
            stmt.compile(self);
        }

        std::mem::replace(&mut self.res, Vec::new())
    }
}

impl CodeGen for Node {
    fn compile(&self, compiler: &mut Compiler) {
        let _timer = BoaProfiler::global().start_event("NodeCodeGen", "codeGen");
        match *self {
            Node::Const(Const::Int(x)) => compiler
                .res
                .push(Instruction::Ld(Reg(compiler.next_free), Value::integer(x))),

            Node::Const(Const::Num(x)) => compiler
                .res
                .push(Instruction::Ld(Reg(compiler.next_free), Value::number(x))),

            Node::BinOp(ref node) => node.compile(compiler),

            Node::ConstDeclList(ref xs) => {
                for x in xs.as_ref() {
                    x.init().compile(compiler);

                    compiler.res.push(Instruction::Bind(
                        Reg(compiler.next_free),
                        x.name().to_owned(), // FIXME: remove .to_owned()
                    ));
                }
            }

            _ => {
                dbg!(self);
                unimplemented!("unsupported Node");
            }
        }
    }
}

impl CodeGen for BinOp {
    fn compile(&self, compiler: &mut Compiler) {
        let dest = compiler.next_free;
        let src = dest + 1;

        self.lhs().compile(compiler);
        compiler.next_free = src;
        self.rhs().compile(compiler);
        compiler.next_free = dest;

        let dest = Reg(dest);
        let src = Reg(src);

        use crate::syntax::ast::op::{BinOp, CompOp, NumOp};
        let i = match self.op() {
            BinOp::Num(num) => match num {
                NumOp::Add => Instruction::Add { dest, src },
                _ => unimplemented!("unsupported NumOp"),
            },
            BinOp::Comp(cmp) => match cmp {
                CompOp::Equal => unimplemented!(),
                _ => unimplemented!(),
            },
            _ => unimplemented!("unsupported BinOp"),
        };

        compiler.res.push(i);
    }
}
