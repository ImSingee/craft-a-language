use crate::grammar::{Dumper, Statement};

/**
 * 程序节点，也是AST的根节点
 */
pub struct Prog {
    pub stmts: Vec<Statement>, //程序中可以包含多个语句
}
impl Prog {
    pub fn new(stmts: Vec<Statement>) -> Prog {
        Prog { stmts }
    }
}
impl Dumper for Prog {
    fn dump(&self, prefix: &str) {
        println!("{}Prog", prefix);
        for x in &self.stmts {
            x.dump(&(prefix.to_string() + "\t"))
        }
    }
}
