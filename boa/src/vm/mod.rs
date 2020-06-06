use crate::{
    builtins::value::{undefined, ResultValue, ToValue, Value, ValueData},
    exec::Executor,
    environment::lexical_environment::VariableScope,
    realm::Realm,
    syntax::ast::{
        constant::Const,
        expr::{Expr, ExprDef},
    },
};
use gc::Gc;

#[cfg(test)]
mod tests;

// === Misc
#[derive(Copy, Clone, Debug)]
pub struct Reg(u8);

#[derive(Clone, Debug)]
pub enum In {
    /// Loads a value into a register
    Ld(Reg, Value),
    /// Binds a value from a register to an ident
    Bind(Reg, String),
    /// Adds the values from destination and source and stores the result in destination
    Add { dest: Reg, src: Reg },
}

// === Compilation
#[derive(Default)]
pub struct Compiler {
    res: Vec<In>,
    next_free: u8,
}

impl Compiler {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn compile(&mut self, expr: &Expr) -> Vec<In> {
        self.compile_expr(expr);
        std::mem::replace(&mut self.res, Vec::new())
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match &expr.def {
            ExprDef::Block(exprs) => {
                for e in exprs {
                    self.compile_expr(e);
                }
            }
            ExprDef::ConstDecl(decls) => {
                for (ident, e) in decls {
                    self.compile_expr(e);
                    self.res.push(In::Bind(Reg(self.next_free), ident.clone())) // fix
                }
            }
            ExprDef::Const(Const::Num(x)) => {
                self.res.push(In::Ld(Reg(self.next_free), x.to_value()));
            }
            // fix hardcoded binop
            ExprDef::BinOp(_, l, r) => {
                let free = self.next_free;
                let tmp = free + 1;

                self.compile_expr(l);
                self.next_free = tmp;
                self.compile_expr(r);

                self.res.push(In::Add {
                    dest: Reg(free),
                    src: Reg(tmp),
                })
            }
            _ => {
                dbg!(expr);
                panic!();
            }
        };
    }
}

// === Execution
pub struct VM {
    realm: Realm,
    regs: Vec<Value>,
}

impl VM {
    /// Sets a register's value to `undefined` and returns its previous one
    fn clear(&mut self, reg: Reg) -> Value {
        let v = self.regs[reg.0 as usize].clone();
        self.regs[reg.0 as usize] = undefined();
        v
    }
}

impl Executor<&[In]> for VM {
    fn new(realm: Realm) -> Self {
        VM {
            realm,
            regs: vec![undefined(); 8],
        }
    }

    fn run(&mut self, instrs: &[In]) -> ResultValue {
        let mut idx = 0;

        while idx < instrs.len() {
            match &instrs[idx] {
                In::Ld(r, v) => {
                    self.regs[r.0 as usize] = v.clone();
                }
                In::Add { dest, src } => {
                    let res = (*self.clear(*dest)).clone() + (*self.clear(*src)).clone();
                    self.regs[dest.0 as usize] = Gc::new(res);
                }
                In::Bind(r, ident) => {
                    let val = self.clear(*r);

                    if self.realm.environment.has_binding(ident) {
                        self.realm.environment.set_mutable_binding(ident, val, true);
                    } else {
                        self.realm.environment.create_mutable_binding(
                            ident.clone(), // fix
                            true,
                            VariableScope::Function,
                        );
                        self.realm.environment.initialize_binding(ident, val);
                    }
                }
                _ => {
                    dbg!(&instrs[idx]);
                    panic!();
                }
            }

            idx += 1;
        }

        Ok(self.regs[0].clone())
    }
}
