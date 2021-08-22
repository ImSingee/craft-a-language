use std::ptr::NonNull;

pub trait Dumper {
    //打印对象信息，prefix是前面填充的字符串，通常用于缩进显示
    fn dump(&self, prefix: &str);
}

pub enum Statement {
    FunctionDecl(FunctionDecl),
    FunctionCall(FunctionCall),
}
impl Dumper for Statement {
    fn dump(&self, prefix: &str) {
        match self {
            Statement::FunctionDecl(x) => x.dump(prefix),
            Statement::FunctionCall(x) => x.dump(prefix),
        }
    }
}

/**
 * 函数声明节点
 */
pub struct FunctionDecl {
    pub name: String,       //函数名称
    pub body: FunctionBody, //函数体
}
impl FunctionDecl {
    pub fn new(name: String, body: FunctionBody) -> FunctionDecl {
        FunctionDecl { name, body }
    }
}
impl Dumper for FunctionDecl {
    fn dump(&self, prefix: &str) {
        println!("{}FunctionDecl {}", prefix, self.name);
        self.body.dump(&(prefix.to_string() + "\t"));
    }
}

/**
 * 函数体
 */
pub struct FunctionBody {
    pub stmts: Vec<FunctionCall>,
}
impl FunctionBody {
    pub fn new(stmts: Vec<FunctionCall>) -> FunctionBody {
        FunctionBody { stmts }
    }
}
impl Dumper for FunctionBody {
    fn dump(&self, prefix: &str) {
        println!("{}FunctionBody", prefix);
        for x in &self.stmts {
            x.dump(&*format!("{}\t", prefix))
        }
    }
}

/**
 * 函数调用
 */
pub struct FunctionCall {
    pub name: String,
    pub parameters: Vec<String>,
    pub definition: Option<NonNull<FunctionDecl>>, // 指向函数的声明
}
impl FunctionCall {
    pub fn new(name: String, parameters: Vec<String>) -> FunctionCall {
        FunctionCall {
            name,
            parameters,
            definition: None,
        }
    }
}
impl Dumper for FunctionCall {
    fn dump(&self, prefix: &str) {
        println!(
            "{}FunctionCall {}, {}",
            prefix,
            self.name,
            match self.definition {
                Some(_) => "resolved",
                None => "not resolved",
            }
        );

        for x in &self.parameters {
            println!("{}\tParameter: {}", prefix, x)
        }
    }
}
