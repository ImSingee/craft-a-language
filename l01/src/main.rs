#[macro_use]
extern crate derive_new;

use std::fmt::Formatter;

/**
 * 第1节
 * 本节的目的是迅速的实现一个最精简的语言的功能，让你了解一门计算机语言的骨架。
 * 知识点：
 * 1.递归下降的方法做词法分析；
 * 2.语义分析中的引用消解（找到函数的定义）；
 * 3.通过遍历AST的方法，执行程序。
 *
 * 本节采用的语法规则是极其精简的，只能定义函数和调用函数。定义函数的时候，还不能有参数。
 * prog = (functionDecl | functionCall)* ;
 * functionDecl: "function" Identifier "(" ")"  functionBody;
 * functionBody : '{' functionCall* '}' ;
 * functionCall : Identifier '(' parameterList? ')' ;
 * parameterList : StringLiteral (',' StringLiteral)* ;
 */

/////////////////////////////////////////////////////////////////////////
// 错误处理
#[derive(Debug)]
struct DecodeError {
    message: String,
}
impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl From<&str> for DecodeError {
    fn from(message: &str) -> Self {
        DecodeError {
            message: message.to_string(),
        }
    }
}
impl From<String> for DecodeError {
    fn from(message: String) -> Self {
        DecodeError { message }
    }
}

enum DecodeResult<T: Statement> {
    Success(T),
    TryNext,
    Error(DecodeError),
}
impl<T: Statement> From<T> for DecodeResult<T> {
    fn from(x: T) -> Self {
        DecodeResult::Success(x)
    }
}
impl<T: Statement> From<DecodeError> for DecodeResult<T> {
    fn from(e: DecodeError) -> Self {
        DecodeResult::Error(e)
    }
}
impl<T: Statement> From<String> for DecodeResult<T> {
    fn from(message: String) -> Self {
        DecodeResult::Error(message.into())
    }
}
impl<T: Statement> From<&str> for DecodeResult<T> {
    fn from(message: &str) -> Self {
        DecodeResult::Error(message.into())
    }
}

/////////////////////////////////////////////////////////////////////////
// 词法分析
// 本节没有提供词法分析器，直接提供了一个Token串。语法分析程序可以从Token串中依次读出
// 一个个Token，也可以重新定位Token串的当前读取位置。

#[derive(Debug, PartialEq, Eq)]
#[allow(unused)]
enum TokenKind {
    Keyword,
    Identifier,
    StringLiteral,
    Seperator,
    Operator,
    EOF,
}

// 代表一个Token的数据结构
#[derive(Debug)]
struct Token {
    kind: TokenKind,
    text: String,
}

// 一个Token数组，代表了下面这段程序做完词法分析后的结果：
/*

//一个函数的声明，这个函数很简单，只打印"Hello World!"
function sayHello(){
    println("Hello World!");
}

//调用刚才声明的函数
sayHello();

*/
fn read_token() -> Vec<Token> {
    vec![
        Token {
            kind: TokenKind::Keyword,
            text: "function".to_string(),
        },
        Token {
            kind: TokenKind::Identifier,
            text: "sayHello".to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: "(".to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: ")".to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: "{".to_string(),
        },
        Token {
            kind: TokenKind::Identifier,
            text: "println".to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: "(".to_string(),
        },
        Token {
            kind: TokenKind::StringLiteral,
            text: "Hello World!".to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: ')'.to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: ';'.to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: '}'.to_string(),
        },
        Token {
            kind: TokenKind::Identifier,
            text: "sayHello".to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: '('.to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: ')'.to_string(),
        },
        Token {
            kind: TokenKind::Seperator,
            text: ';'.to_string(),
        },
        Token {
            kind: TokenKind::EOF,
            text: "".to_string(),
        },
    ]
}

struct Tokenizer {
    tokens: Vec<Token>,
    pos: usize,
}

/**
 * 简化的词法分析器
 * 语法分析器从这里获取Token。
 */
impl Tokenizer {
    fn new(tokens: Vec<Token>) -> Option<Tokenizer> {
        if tokens.len() < 1 || tokens.last().unwrap().kind != TokenKind::EOF {
            None
        } else {
            Some(Tokenizer { tokens, pos: 0 })
        }
    }

    fn eof(&self) -> bool {
        if self.pos >= self.tokens.len() {
            true
        } else if self.tokens.get(self.pos).unwrap().kind == TokenKind::EOF {
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> &Token {
        if self.pos >= self.tokens.len() {
            self.tokens.last().unwrap()
        } else {
            let v = self.tokens.get(self.pos).unwrap();
            self.pos += 1;
            v
        }
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn trace_back(&mut self, new_pos: usize) -> bool {
        if new_pos > self.pos {
            false
        } else {
            self.pos = new_pos;
            true
        }
    }
}

/////////////////////////////////////////////////////////////////////////
// 语法分析
// 包括了AST的数据结构和递归下降的语法解析程序

/**
 * 基类
 */
trait AstNode {
    //打印对象信息，prefix是前面填充的字符串，通常用于缩进显示
    fn dump(&self, prefix: &str);
}

/**
 * 语句
 * 其子类包括函数声明和函数调用
 */
trait Statement: AstNode {}

/**
 * 程序节点，也是AST的根节点
 */
#[derive(new)]
struct Prog {
    stmts: Vec<Box<dyn Statement>>, //程序中可以包含多个语句
}
impl AstNode for Prog {
    fn dump(&self, prefix: &str) {
        println!("{}Prog", prefix);
        for x in &self.stmts {
            x.dump(&(prefix.to_string() + "\t"))
        }
    }
}

/**
 * 函数声明节点
 */
#[derive(new)]
struct FunctionDecl {
    name: String,       //函数名称
    body: FunctionBody, //函数体
}
impl AstNode for FunctionDecl {
    fn dump(&self, prefix: &str) {
        println!("{}FunctionDecl {}", prefix, self.name);
        self.body.dump(&(prefix.to_string() + "\t"));
    }
}
impl Statement for FunctionDecl {}

/**
 * 函数体
 */
#[derive(new)]
struct FunctionBody {
    stmts: Vec<FunctionCall>,
}
impl AstNode for FunctionBody {
    fn dump(&self, prefix: &str) {
        println!("{}FunctionBody", prefix);
        for x in &self.stmts {
            x.dump(&*format!("{}\t", prefix))
        }
    }
}
impl Statement for FunctionBody {}

/**
 * 函数调用
 */
struct FunctionCall {
    name: String,
    parameters: Vec<String>,
    definition: Option<FunctionDecl>, // 指向函数的声明
}
impl FunctionCall {
    fn new(name: String, parameters: Vec<String>) -> FunctionCall {
        FunctionCall {
            name,
            parameters,
            definition: None,
        }
    }
}
impl AstNode for FunctionCall {
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
impl Statement for FunctionCall {}

#[derive(new)]
struct Parser {
    tokenizer: Tokenizer,
}
impl Parser {
    fn parse_prog(mut self) -> Result<Prog, DecodeError> {
        let mut stmts: Vec<Box<dyn Statement>> = Vec::new();
        while !self.tokenizer.eof() {
            // 每次循环解析一个语句

            // 尝试一下函数声明
            match self.parse_function_decl() {
                DecodeResult::Success(stmt) => {
                    stmts.push(Box::new(stmt));
                    continue;
                } // next statement
                DecodeResult::TryNext => {} // continue
                DecodeResult::Error(e) => return Err(e),
            }

            // 如果前一个尝试不成功，那么再尝试一下函数调用
            match self.parse_function_call() {
                DecodeResult::Success(stmt) => {
                    stmts.push(Box::new(stmt));
                    continue;
                } // next statement
                DecodeResult::TryNext => {} // continue
                DecodeResult::Error(e) => return Err(e),
            }

            //如果都没成功，那就失败结束
            return Err("unknown statement".into());
        }

        Ok(Prog::new(stmts))
    }

    /**
     * 解析函数声明
     * 语法规则：
     * functionDecl: "function" Identifier "(" ")"  functionBody;
     */
    fn parse_function_decl(&mut self) -> DecodeResult<FunctionDecl> {
        let old_pos = self.tokenizer.position();
        let t = self.tokenizer.next();

        if t.kind == TokenKind::Keyword && t.text == "function" {
            // "function"

            let t = self.tokenizer.next(); // Identifier
            if t.kind != TokenKind::Identifier {
                return format!("expect Identifier but got {:?}", t).into();
            }
            let function_name = t.text.to_string();

            // "(",
            let t = self.tokenizer.next();
            if t.kind != TokenKind::Seperator || t.text != "(" {
                return format!("expect Seperator '(' but got {:?}", t).into();
            }
            // 暂时不支持参数
            // ")"
            let t = self.tokenizer.next();
            if t.kind != TokenKind::Seperator || t.text != ")" {
                return format!("expect Seperator ')' but got {:?}", t).into();
            }

            // 解析函数体
            let function_body;
            match self.parse_function_body() {
                DecodeResult::Success(x) => function_body = x,
                DecodeResult::TryNext => {
                    return format!("expect FunctionBody, but not found").into()
                }
                DecodeResult::Error(e) => return e.into(),
            }

            // 解析成功
            return FunctionDecl {
                name: function_name,
                body: function_body,
            }
            .into();
        }

        //如果解析不成功，回溯，继续尝试
        self.tokenizer.trace_back(old_pos);
        DecodeResult::TryNext
    }

    /**
     * 解析函数体
     * 语法规则：
     * functionBody : '{' functionCall* '}' ;
     *
     * 该函数永远不会返回 TryNext
     */
    fn parse_function_body(&mut self) -> DecodeResult<FunctionBody> {
        let t = self.tokenizer.next();
        if t.kind != TokenKind::Seperator || t.text != "{" {
            return format!("expect Seperator '{}' but got {:?}", '{', t).into();
        }

        let mut stmts = Vec::new();
        loop {
            match self.parse_function_call() {
                DecodeResult::Success(x) => stmts.push(x),
                DecodeResult::TryNext => break,
                DecodeResult::Error(e) => return e.into(),
            }
        }

        let t = self.tokenizer.next();
        if t.kind != TokenKind::Seperator || t.text != "}" {
            return format!("expect Seperator '{}' but got {:?}", '}', t).into();
        }

        return FunctionBody::new(stmts).into();
    }

    fn parse_function_call(&mut self) -> DecodeResult<FunctionCall> {
        let old_pos = self.tokenizer.position();

        let t = self.tokenizer.next();
        if t.kind == TokenKind::Identifier {
            let function_name = t.text.to_string();
            let t = self.tokenizer.next();
            if t.kind == TokenKind::Seperator && t.text == "(" {
                // function call
                let mut function_parameters = Vec::new();
                // parameter, parameter, ... )
                let mut t = self.tokenizer.next();
                while t.kind != TokenKind::Seperator || t.text != ")" {
                    // t should be StringLiteral
                    if t.kind != TokenKind::StringLiteral {
                        return format!("expect string parameter '(' but got {:?}", t).into();
                    }
                    function_parameters.push(t.text.to_string());

                    // next should be Seperator, ',' or ')'
                    t = self.tokenizer.next();
                    if t.kind != TokenKind::Seperator || (t.text != "," && t.text != ")") {
                        return format!("expect Seperator ',' or ')' but got {:?}", t).into();
                    }
                    if t.text == "," {
                        // simple skip
                        t = self.tokenizer.next();
                    }
                }
                // 末尾分号
                let t = self.tokenizer.next();
                if t.kind != TokenKind::Seperator || t.text != ";" {
                    return format!("expect Seperator ';' but got {:?}", t).into();
                }

                // 解析成功
                return FunctionCall::new(function_name, function_parameters).into();
            }
        }

        // 回溯
        self.tokenizer.trace_back(old_pos);
        DecodeResult::TryNext
    }
}

/////////////////////////////////////////////////////////////////////////
// 主程序
fn compileAndRun(tokens: Vec<Token>) -> Result<(), DecodeError> {
    // 词法分析（模拟）
    let tokenizer = Tokenizer::new(dbg!(tokens)).unwrap();

    // 语法分析
    let prog = Parser::new(tokenizer).parse_prog()?;
    println!("语法分析后的AST:");
    prog.dump("");

    // 语义分析

    // 运行程序
    Ok(())
}

fn main() -> Result<(), DecodeError> {
    compileAndRun(read_token())
}
