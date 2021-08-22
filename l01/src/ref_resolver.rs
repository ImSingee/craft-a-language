use crate::grammar::{FunctionDecl, Statement};
use crate::prog::Prog;
use crate::FunctionCall;
use std::collections::HashMap;

pub struct RefResolver {}
impl RefResolver {
    pub fn resolve(prog: &mut Prog) -> Result<(), String> {
        let mut functions: HashMap<String, std::ptr::NonNull<FunctionDecl>> = HashMap::new();

        for x in &mut prog.stmts {
            if let Statement::FunctionDecl(decl) = x {
                functions.insert(decl.name.to_string(), decl.into());
            }
        }

        for x in &mut prog.stmts {
            match x {
                Statement::FunctionDecl(decl) => {
                    for call in &mut decl.body.stmts {
                        RefResolver::resolve_function_call(&functions, call)?
                    }
                }
                Statement::FunctionCall(call) => {
                    RefResolver::resolve_function_call(&functions, call)?
                }
            }
        }

        Ok(())
    }

    fn resolve_function_call(
        functions: &HashMap<String, std::ptr::NonNull<FunctionDecl>>,
        call: &mut FunctionCall,
    ) -> Result<(), String> {
        match functions.get(&call.name) {
            None => match call.name.as_ref() {
                "println" => Ok(()),
                _ => Err(format!("unkown function {}", call.name)),
            },
            Some(ptr) => {
                call.definition = Some(ptr.clone());
                Ok(())
            }
        }
    }
}
