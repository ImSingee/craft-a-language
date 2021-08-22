pub struct Interpreter {}

use crate::grammar::{FunctionCall, Statement};
use crate::prog::Prog;

impl Interpreter {
    pub fn run(prog: &Prog) -> Result<(), String> {
        for x in &prog.stmts {
            if let Statement::FunctionCall(call) = x {
                Interpreter::run_call(call)?
            }
        }

        Ok(())
    }

    fn run_call(call: &FunctionCall) -> Result<(), String> {
        match call.definition {
            None => {
                if call.name == "println" {
                    println!("{}", call.parameters.join(" "));
                    Ok(())
                } else {
                    Err(format!("Unknown function {}", call.name))
                }
            }
            Some(def) => {
                for x in &{ unsafe { def.as_ref() } }.body.stmts {
                    Interpreter::run_call(x)?
                }

                Ok(())
            }
        }
    }
}
