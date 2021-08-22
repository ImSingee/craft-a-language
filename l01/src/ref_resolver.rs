use crate::grammar::{FunctionDecl, Statement};
use crate::prog::Prog;
use std::collections::HashMap;

pub struct RefResolver {}
impl RefResolver {
    pub fn resolve(prog: &mut Prog) -> Result<(), String> {
        let mut functions: HashMap<String, std::ptr::NonNull<FunctionDecl>> = HashMap::new();

        for x in &mut prog.stmts {
            match x {
                Statement::FunctionDecl(decl) => {
                    functions.insert(decl.name.to_string(), decl.into());
                }
                Statement::FunctionCall(call) => match functions.get(&call.name) {
                    None => match call.name.as_ref() {
                        "println" => {}
                        _ => return Err(format!("unkown name {}", call.name)),
                    },
                    Some(ptr) => call.definition = Some(ptr.clone()),
                },
            }
        }

        Ok(())
    }
}
