use std::fmt;
use std::sync::Arc;

use either::Either;
use simplicity::node::Inner;
use simplicity::types::Final;
use simplicity::Value;
use simplicityhl::debug::{DebugSymbols, FallibleCall, FallibleCallName};
use simplicityhl::either;
use simplicityhl::simplicity::jet::elements::ElementsEnv;
use simplicityhl::SatisfiedProgram;
use simplicityhl::{elements, simplicity};

use crate::jet;
use crate::jet::JetFailed;
use crate::util::Expression;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ErrorKind {
    AssertionFailed,
    FailNode,
    JetFailed,
    WrongType,
    SimplicityHLCallFailed(FallibleCall),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::AssertionFailed => f.write_str("Assertion failed"),
            ErrorKind::FailNode => f.write_str("Universal fail"),
            ErrorKind::JetFailed => f.write_str("Jet failed"),
            ErrorKind::WrongType => {
                f.write_str("The program is ill-typed (this should never happen)")
            }
            ErrorKind::SimplicityHLCallFailed(call) => {
                match call.name() {
                    FallibleCallName::Assert => writeln!(f, "Assertion failed: false")?,
                    FallibleCallName::Panic => writeln!(f, "Explicit panic")?,
                    FallibleCallName::Jet => writeln!(f, "Jet failed")?,
                    FallibleCallName::UnwrapLeft(val) => {
                        writeln!(f, "Called `unwrap_left()` on a `Right` value: {val}")?;
                    }
                    FallibleCallName::UnwrapRight(val) => {
                        writeln!(f, "Called `unwrap_right()` on a `Left` value: {val}")?;
                    }
                    FallibleCallName::Unwrap => writeln!(f, "Called `unwrap()` on a `None` value")?,
                }
                write!(f, "`{}`", call.text())
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Task {
    Execute(Arc<Expression>),
    MoveLeftCompOutput,
    MoveLeftDisconnectOutput,
    MakeLeft(Arc<Final>),
    MakeRight(Arc<Final>),
    MakeProduct,
    ResetActiveSimplicityHLCall,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Runner {
    /// Stack of tasks to run.
    tasks: Vec<Task>,
    /// Stack of input values.
    input: Vec<Value>,
    /// Stack of output values.
    output: Vec<Value>,
    /// simplicityhl debug symbols in the Simplicity target code.
    debug_symbols: DebugSymbols,
    /// simplicityhl call expression that is currently running.
    active_simplicityhl_call: Option<FallibleCall>,
    /// Output from simplicityhl dbg! expressions, in order of occurrence.
    debug_output: Vec<String>,
}

impl Runner {
    pub fn for_program(program: &SatisfiedProgram) -> Self {
        Self {
            tasks: vec![Task::Execute(program.redeem().clone())],
            input: vec![Value::unit()],
            output: vec![],
            debug_symbols: program.debug_symbols().clone(),
            active_simplicityhl_call: None,
            debug_output: vec![],
        }
    }

    pub fn debug_output(self) -> Vec<String> {
        self.debug_output
    }

    pub fn run(&mut self, env: &ElementsEnv<Arc<elements::Transaction>>) -> Result<(), ErrorKind> {
        while let Some(task) = self.tasks.pop() {
            match task {
                Task::Execute(expression) => {
                    let input = self.input.pop().unwrap();
                    match expression.inner() {
                        Inner::Iden => self.output.push(input),
                        Inner::Unit => self.output.push(Value::unit()),
                        Inner::InjL(t) => {
                            let ty_r = expression.arrow().target.as_sum().unwrap().1;
                            self.tasks.push(Task::MakeLeft(Arc::clone(ty_r)));
                            self.tasks.push(Task::Execute(Arc::clone(t)));
                            self.input.push(input);
                        }
                        Inner::InjR(t) => {
                            let ty_l = expression.arrow().target.as_sum().unwrap().0;
                            self.tasks.push(Task::MakeRight(Arc::clone(ty_l)));
                            self.tasks.push(Task::Execute(Arc::clone(t)));
                            self.input.push(input);
                        }
                        Inner::Take(t) => {
                            let (a, _) = input.as_product().ok_or(ErrorKind::WrongType)?;
                            self.tasks.push(Task::Execute(Arc::clone(t)));
                            self.input.push(a.to_value());
                        }
                        Inner::Drop(t) => {
                            let (_, b) = input.as_product().ok_or(ErrorKind::WrongType)?;
                            self.tasks.push(Task::Execute(Arc::clone(t)));
                            self.input.push(b.to_value());
                        }
                        Inner::Comp(s, t) => {
                            self.tasks.push(Task::Execute(Arc::clone(t)));
                            self.tasks.push(Task::MoveLeftCompOutput);
                            self.tasks.push(Task::Execute(Arc::clone(s)));
                            self.input.push(input);
                        }
                        Inner::Pair(s, t) => {
                            self.tasks.push(Task::MakeProduct);
                            self.tasks.push(Task::Execute(Arc::clone(t)));
                            self.tasks.push(Task::Execute(Arc::clone(s)));
                            self.input.push(input.shallow_clone());
                            self.input.push(input);
                        }
                        Inner::Case(..) | Inner::AssertL(..) | Inner::AssertR(..) => {
                            let (sum_a_b, c) = input.as_product().ok_or(ErrorKind::WrongType)?;

                            if let Inner::AssertL(_, cmr) = expression.inner() {
                                if let Some(tracked_call) = self.debug_symbols.get(cmr) {
                                    match tracked_call.map_value(
                                        &simplicityhl::value::StructuralValue::from(c.to_value()),
                                    ) {
                                        Some(Either::Left(fallible_call)) => {
                                            let replaced = self
                                                .active_simplicityhl_call
                                                .replace(fallible_call);
                                            debug_assert!(replaced.is_none());
                                            self.tasks.push(Task::ResetActiveSimplicityHLCall);
                                        }
                                        Some(Either::Right(debug_value)) => {
                                            let s = format!(
                                                "`{}` = `{}`",
                                                debug_value.text(),
                                                debug_value.value()
                                            );
                                            self.debug_output.push(s);
                                        }
                                        None => {}
                                    }
                                }
                            }

                            if let Some(a) = sum_a_b.as_left() {
                                match expression.inner() {
                                    Inner::Case(s, _) | Inner::AssertL(s, _) => {
                                        self.tasks.push(Task::Execute(Arc::clone(s)));
                                        self.input.push(Value::product(a.to_value(), c.to_value()));
                                    }
                                    Inner::AssertR(_, _) => {
                                        return Err(self.error(ErrorKind::AssertionFailed))
                                    }
                                    _ => unreachable!("Covered by outer match statement"),
                                }
                            } else if let Some(b) = sum_a_b.as_right() {
                                match expression.inner() {
                                    Inner::Case(_, t) | Inner::AssertR(_, t) => {
                                        self.tasks.push(Task::Execute(Arc::clone(t)));
                                        self.input.push(Value::product(b.to_value(), c.to_value()));
                                    }
                                    Inner::AssertL(_, _) => {
                                        return Err(self.error(ErrorKind::AssertionFailed))
                                    }
                                    _ => unreachable!("Covered by outer match statement"),
                                }
                            } else {
                                return Err(ErrorKind::WrongType);
                            }
                        }
                        Inner::Disconnect(s, t) => {
                            self.tasks.push(Task::MakeProduct);
                            self.tasks.push(Task::Execute(Arc::clone(t)));
                            self.tasks.push(Task::MoveLeftDisconnectOutput);
                            self.tasks.push(Task::Execute(Arc::clone(s)));
                            let t_cmr = Value::u256(t.cmr().to_byte_array());
                            self.input.push(Value::product(t_cmr, input));
                        }
                        Inner::Witness(value) => self.output.push(value.shallow_clone()),
                        Inner::Fail(_) => return Err(self.error(ErrorKind::FailNode)),
                        Inner::Jet(jet) => match jet::execute_jet_with_env(jet, &input, env) {
                            Ok(output) => self.output.push(output),
                            Err(JetFailed) => return Err(self.error(ErrorKind::JetFailed)),
                        },
                        Inner::Word(word) => self.output.push(word.as_value().shallow_clone()),
                    }
                }
                Task::MoveLeftCompOutput => {
                    let output = self.output.pop().unwrap();
                    self.input.push(output);
                }
                Task::MoveLeftDisconnectOutput => {
                    let prod_b_c = self.output.pop().unwrap();
                    let (b, c) = prod_b_c.as_product().unwrap();
                    self.output.push(b.to_value());
                    self.input.push(c.to_value());
                }
                Task::MakeLeft(ty_r) => {
                    let val_l = self.output.pop().unwrap();
                    self.output.push(Value::left(val_l, ty_r));
                }
                Task::MakeRight(ty_l) => {
                    let val_r = self.output.pop().unwrap();
                    self.output.push(Value::right(ty_l, val_r));
                }
                Task::MakeProduct => {
                    let b = self.output.pop().unwrap();
                    let a = self.output.pop().unwrap();
                    self.output.push(Value::product(a, b));
                }
                Task::ResetActiveSimplicityHLCall => self.active_simplicityhl_call = None,
            }
        }

        debug_assert!(self.input.is_empty());
        debug_assert_eq!(self.output.len(), 1);
        debug_assert!(self.output.pop().unwrap().is_unit());
        Ok(())
    }

    /// Try to return an error with simplicityhl debug information included.
    /// Otherwise, return the original error.
    fn error(&self, error: ErrorKind) -> ErrorKind {
        match &self.active_simplicityhl_call {
            Some(call) => ErrorKind::SimplicityHLCallFailed(call.clone()),
            None => error,
        }
    }
}

#[cfg(test)]
mod tests {
    use simplicityhl::elements::{hashes::Hash, secp256k1_zkp as secp256k1};
    use simplicityhl::CompiledProgram;

    use super::*;
    use crate::examples;
    use crate::examples::Example;
    use crate::util::{HashedData, SigningKeys};

    fn satisfied_and_tx_env(
        example: Example,
        signing_keys: &SigningKeys,
        hashed_data: &HashedData,
    ) -> (SatisfiedProgram, ElementsEnv<Arc<elements::Transaction>>) {
        let arguments = example.arguments(&signing_keys.public_keys, &hashed_data.hashes);
        let compiled = CompiledProgram::new(
            example.template_text(),
            arguments,
            false, /* include debug symbols */
        )
        .expect("example should compile");
        let tx_env = example.params().tx_env(compiled.commit().cmr());
        let sighash_all =
            secp256k1::Message::from_digest(tx_env.c_tx_env().sighash_all().to_byte_array());
        let witness = example.witness(
            &signing_keys.secret_keys,
            &hashed_data.preimages,
            sighash_all,
        );
        let satisfied = compiled
            .satisfy(witness)
            .expect("example should be satisfied");

        (satisfied, tx_env)
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn run_examples() {
        let signing_keys = SigningKeys::default();
        let hashed_data = HashedData::default();

        for name in examples::keys() {
            println!("{name}");
            let example = examples::get(name).unwrap();
            let (satisfied, tx_env) = satisfied_and_tx_env(example, &signing_keys, &hashed_data);
            let mut runner = Runner::for_program(&satisfied);
            if let Err(error) = runner.run(&tx_env) {
                println!("sighash all = {}", tx_env.c_tx_env().sighash_all());
                for debug_line in runner.debug_output() {
                    println!("{debug_line}");
                }
                panic!("Unexpected error: {error}");
            }
        }
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn compare_with_rust_simplicity() {
        let signing_keys = SigningKeys::default();
        let hashed_data = HashedData::default();

        for name in examples::keys() {
            println!("{name}");
            let example = examples::get(name).unwrap();
            let (satisfied, tx_env) = satisfied_and_tx_env(example, &signing_keys, &hashed_data);
            let rust_simplicity_result = simplicity::BitMachine::for_program(satisfied.redeem())
                .expect("program within limits")
                .exec(satisfied.redeem(), &tx_env);
            let webide_result = Runner::for_program(&satisfied).run(&tx_env);
            match (rust_simplicity_result, webide_result) {
                (Ok(..), Err(error)) => {
                    panic!("rust-simplicity accepted but web IDE rejected: {error}")
                }
                (Err(error), Ok(..)) => {
                    panic!("web IDE accepted but rust-simplicity rejected: {error}")
                }
                _ => {}
            }
        }
    }
}
