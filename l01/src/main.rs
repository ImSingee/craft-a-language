#[macro_use]
extern crate derive_new;

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
use l01::DecodeError;

/////////////////////////////////////////////////////////////////////////
// 词法分析
// 本节没有提供词法分析器，直接提供了一个Token串。语法分析程序可以从Token串中依次读出
// 一个个Token，也可以重新定位Token串的当前读取位置。
use l01::{Token, TokenKind};

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

use l01::{Dumper, FunctionBody, FunctionCall, FunctionDecl, Prog, Statement};

#[derive(new)]
struct Parser {
    tokenizer: Tokenizer,
}
impl Parser {
    fn parse_prog(mut self) -> Result<Prog, DecodeError> {
        let mut stmts: Vec<Statement> = Vec::new();
        while !self.tokenizer.eof() {
            // 每次循环解析一个语句

            // 尝试一下函数声明
            match self.parse_function_decl() {
                Ok(stmt) => {
                    stmts.push(Statement::FunctionDecl(stmt));
                    continue;
                }
                Err(DecodeError::TryNext) => {} // continue
                Err(DecodeError::Fatal(e)) => return Err(e.into()),
            }

            // 如果前一个尝试不成功，那么再尝试一下函数调用
            match self.parse_function_call() {
                Ok(stmt) => {
                    stmts.push(Statement::FunctionCall(stmt));
                    continue;
                }
                Err(DecodeError::TryNext) => {} // continue
                Err(DecodeError::Fatal(e)) => return Err(e.into()),
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
    fn parse_function_decl(&mut self) -> Result<FunctionDecl, DecodeError> {
        let old_pos = self.tokenizer.position();
        let t = self.tokenizer.next();

        if t.kind == TokenKind::Keyword && t.text == "function" {
            // "function"

            let t = self.tokenizer.next(); // Identifier
            if t.kind != TokenKind::Identifier {
                return Err(format!("expect Identifier but got {:?}", t).into());
            }
            let function_name = t.text.to_string();

            // "(",
            let t = self.tokenizer.next();
            if t.kind != TokenKind::Seperator || t.text != "(" {
                return Err(format!("expect Seperator '(' but got {:?}", t).into());
            }
            // 暂时不支持参数
            // ")"
            let t = self.tokenizer.next();
            if t.kind != TokenKind::Seperator || t.text != ")" {
                return Err(format!("expect Seperator ')' but got {:?}", t).into());
            }

            // 解析函数体
            let function_body;
            match self.parse_function_body() {
                Ok(x) => function_body = x,
                Err(e) => return Err(e.into()),
            }

            // 解析成功
            return Ok(FunctionDecl::new(function_name, function_body));
        }

        //如果解析不成功，回溯，继续尝试
        self.tokenizer.trace_back(old_pos);
        Err(DecodeError::TryNext)
    }

    /**
     * 解析函数体
     * 语法规则：
     * functionBody : '{' functionCall* '}' ;
     */
    fn parse_function_body(&mut self) -> Result<FunctionBody, String> {
        let t = self.tokenizer.next();
        if t.kind != TokenKind::Seperator || t.text != "{" {
            return Err(format!("expect Seperator '{}' but got {:?}", '{', t));
        }

        let mut stmts = Vec::new();
        loop {
            match self.parse_function_call() {
                Ok(x) => stmts.push(x),
                Err(DecodeError::TryNext) => break,
                Err(DecodeError::Fatal(e)) => return Err(format!("{}", e)),
            }
        }

        let t = self.tokenizer.next();
        if t.kind != TokenKind::Seperator || t.text != "}" {
            return Err(format!("expect Seperator '{}' but got {:?}", '}', t).into());
        }

        return Ok(FunctionBody::new(stmts));
    }

    fn parse_function_call(&mut self) -> Result<FunctionCall, DecodeError> {
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
                        return Err(format!("expect string parameter '(' but got {:?}", t).into());
                    }
                    function_parameters.push(t.text.to_string());

                    // next should be Seperator, ',' or ')'
                    t = self.tokenizer.next();
                    if t.kind != TokenKind::Seperator || (t.text != "," && t.text != ")") {
                        return Err(format!("expect Seperator ',' or ')' but got {:?}", t).into());
                    }
                    if t.text == "," {
                        // simple skip
                        t = self.tokenizer.next();
                    }
                }
                // 末尾分号
                let t = self.tokenizer.next();
                if t.kind != TokenKind::Seperator || t.text != ";" {
                    return Err(format!("expect Seperator ';' but got {:?}", t).into());
                }

                // 解析成功
                return Ok(FunctionCall::new(function_name, function_parameters));
            }
        }

        // 回溯
        self.tokenizer.trace_back(old_pos);
        Err(DecodeError::TryNext)
    }
}

/////////////////////////////////////////////////////////////////////////
// 语义分析
use l01::{Interpreter, RefResolver};

/////////////////////////////////////////////////////////////////////////
// 主程序
fn compile_and_run(tokens: Vec<Token>) -> Result<(), DecodeError> {
    // 词法分析（模拟）
    let tokenizer = Tokenizer::new(dbg!(tokens)).unwrap();

    // 语法分析
    let mut prog = Parser::new(tokenizer).parse_prog()?;
    println!("\n语法分析后的AST:");
    prog.dump("");

    // 语义分析
    RefResolver::resolve(&mut prog)?;
    println!("\n语义分析后的AST:");
    prog.dump("");

    // 运行程序
    println!("\n运行程序");
    Interpreter::run(&prog)?;

    Ok(())
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

fn main() -> Result<(), DecodeError> {
    compile_and_run(read_token())
}
