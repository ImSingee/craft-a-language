use std::iter::Peekable;
use std::str::Chars;

/**
 * 第2节
 * 本节的知识点有两个：
 * 1.学会词法分析；
 * 2.升级语法分析为LL算法，因此需要知道如何使用First和Follow集合。
 *
 * 本节采用的词法规则是比较精简的，比如不考虑Unicode。
 * Identifier: [a-zA-Z_][a-zA-Z0-9_]* ;
 */

/////////////////////////////////////////////////////////////////////////
// 数据流定义

struct CharStream<'a> {
    data: Peekable<Chars<'a>>,
    line: u64,
    col: u64,
}
impl CharStream<'_> {
    fn new(data: &str) -> CharStream {
        CharStream {
            data: data.chars().peekable(),
            line: 1,
            col: 0,
        }
    }

    fn line(&self) -> u64 {
        self.line
    }

    fn col(&self) -> u64 {
        self.col
    }

    fn peek(&mut self) -> Option<&char> {
        self.data.peek()
    }
}
impl Iterator for CharStream<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.data.next();
        if let Some(ch) = ch {
            if ch == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
        }
        ch
    }
}

/////////////////////////////////////////////////////////////////////////
// 词法分析
// 当前支持
// - Identifier, keyword
// - Seperator '(' | ')' | '{' | '}' | ';' | ','
// - StringLiteral
// - Comment (single and block)
// - Operator '/' | '/=' | '+' | '++' | '+=' | '-' | '--' | '-='
// 尚未支持
// - 数字字面量

use l01::{Token, TokenKind};

struct Tokenizer<'a> {
    stream: CharStream<'a>,
    eof: bool,
}
impl Tokenizer<'_> {
    fn new(code: &str) -> Peekable<Tokenizer> {
        return Tokenizer {
            stream: CharStream::new(code),
            eof: false,
        }
        .peekable();
    }
    fn from_stream(stream: CharStream) -> Peekable<Tokenizer> {
        return Tokenizer { stream, eof: false }.peekable();
    }

    // 从字符串流中获取一个新Token
    fn next_token(&mut self) -> Option<Token> {
        if self.eof {
            return None;
        }

        // 忽略所有的空白符
        self.skip_whitespaces();

        match self.stream.peek() {
            None => {
                self.eof = true;
                Some(Token {
                    kind: TokenKind::EOF,
                    text: "".to_string(),
                })
            }
            Some(&ch) => {
                match ch {
                    '"' => return Some(self.parse_string_literal().unwrap()),
                    '(' | ')' | '{' | '}' | ';' | ',' => {
                        return Some(Token {
                            kind: TokenKind::Seperator,
                            text: self.stream.next().unwrap().to_string(),
                        })
                    }
                    '+' => {
                        // 可能是 +, ++, +=
                        self.stream.next();

                        return match self.stream.peek() {
                            Some('+') => Some(Token {
                                kind: TokenKind::Operator,
                                text: "++".to_string(),
                            }),
                            Some('=') => Some(Token {
                                kind: TokenKind::Operator,
                                text: "+=".to_string(),
                            }),
                            _ => Some(Token {
                                kind: TokenKind::Operator,
                                text: "+".to_string(),
                            }),
                        };
                    }
                    '-' => {
                        // 可能是 -, --, -=
                        self.stream.next();

                        return match self.stream.peek() {
                            Some('-') => Some(Token {
                                kind: TokenKind::Operator,
                                text: "--".to_string(),
                            }),
                            Some('=') => Some(Token {
                                kind: TokenKind::Operator,
                                text: "-=".to_string(),
                            }),
                            _ => Some(Token {
                                kind: TokenKind::Operator,
                                text: "-".to_string(),
                            }),
                        };
                    }
                    '*' => {
                        // 可能是 *, *=
                        self.stream.next();

                        return match self.stream.peek() {
                            Some('=') => Some(Token {
                                kind: TokenKind::Operator,
                                text: "*=".to_string(),
                            }),
                            _ => Some(Token {
                                kind: TokenKind::Operator,
                                text: "*".to_string(),
                            }),
                        };
                    }
                    '/' => {
                        // 可能是 /*, //, /, /=
                        self.stream.next();

                        return match self.stream.peek() {
                            Some('/') => {
                                self.skip_line();
                                self.next_token()
                            }
                            Some('*') => {
                                self.skip_block_comment().unwrap();
                                self.next_token()
                            }
                            Some('=') => Some(Token {
                                kind: TokenKind::Operator,
                                text: "/=".to_string(),
                            }),
                            _ => Some(Token {
                                kind: TokenKind::Operator,
                                text: "/".to_string(),
                            }),
                        };
                    }
                    _ => {}
                }

                if ch.is_alphabetic() {
                    return Some(self.parse_identifier());
                }
                if ch == '/' {}

                // 无法识别，作为 identifier
                panic!(format!(
                    "Invalid token {} at {}:{}",
                    ch,
                    self.stream.line(),
                    self.stream.col()
                ))
            }
        }
    }

    fn skip_whitespaces(&mut self) {
        while matches!(self.stream.peek(), Some(c) if c.is_whitespace()) {
            self.stream.next();
        }
    }

    // 跳过整行，在解析到 // 后使用
    fn skip_line(&mut self) {
        while matches!(self.stream.peek(), Some(&c) if c != '\n') {
            self.stream.next();
        }
    }

    // 跳过段注释
    // 如果一直到 EOF 都没有读到 */ 则返回错误
    fn skip_block_comment(&mut self) -> Result<(), String> {
        self.stream.next();

        while let Some(&c) = self.stream.peek() {
            self.stream.next();

            if c == '*' {
                if let Some(&c) = self.stream.peek() {
                    if c == '/' {
                        return Ok(());
                    }
                } else {
                    break; // will return error
                }
            }
        }

        return Err("No */ found until EOF".to_string());
    }

    // identifier 为以字母开头，后接若干数字/字符串/下划线
    fn parse_identifier(&mut self) -> Token {
        let mut text: String = self.stream.next().unwrap().into(); // 由上层调用保证当前是一个合法的 identifier 开头

        while matches!(self.stream.peek(), Some(x) if Tokenizer::is_identifier_char(x)) {
            text.push(self.stream.next().unwrap());
        }

        match text.as_ref() {
            "function" => Token {
                kind: TokenKind::Keyword,
                text: text.to_string(),
            },
            _ => Token {
                kind: TokenKind::Identifier,
                text: text.to_string(),
            },
        }
    }

    fn is_identifier_char(c: &char) -> bool {
        *c == '_' || c.is_alphanumeric()
    }

    // 字符串字面量，表现为 "xxx"
    // 当引号未闭合时返回 error
    fn parse_string_literal(&mut self) -> Result<Token, String> {
        self.stream.next(); // 忽略起始引号
        let mut text = String::new();

        while let Some(x) = self.stream.peek() {
            match x {
                '\n' => {
                    return Err(format!(
                        "Unexpected line break at {}:{}",
                        self.stream.line(),
                        self.stream.col()
                    ))
                }
                '\\' => {
                    self.stream.next();
                    match self.stream.peek() {
                        Some('n') => {
                            self.stream.next();
                            text.push('\n');
                        }
                        Some('\\') => {
                            self.stream.next();
                            text.push('\\');
                        }
                        _ => {
                            return Err(format!(
                                "Unexpected {} at {}:{}",
                                '\\',
                                self.stream.line(),
                                self.stream.col()
                            ))
                        }
                    }
                }
                '"' => {
                    self.stream.next();
                    return Ok(Token {
                        kind: TokenKind::StringLiteral,
                        text,
                    });
                }
                _ => text.push(self.stream.next().unwrap()),
            }
        }

        Err(format!(
            "Expecting {} at {}:{}",
            "\"",
            self.stream.line(),
            self.stream.col()
        ))
    }
}
impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

/////////////////////////////////////////////////////////////////////////
// 主程序
fn compile_and_run(code: &str) -> Result<(), String> {
    // 词法分析（模拟）
    let tokenizer = Tokenizer::new(code);
    {
        let tokenizer = Tokenizer::new(dbg!(code));
        println!("\nAST:");
        for token in tokenizer {
            println!("{:?}", token);
        }
    }

    // // 语法分析
    // let mut prog = Parser::new(tokenizer).parse_prog()?;
    // println!("\n语法分析后的AST:");
    // prog.dump("");
    //
    // // 语义分析
    // RefResolver::resolve(&mut prog)?;
    // println!("\n语义分析后的AST:");
    // prog.dump("");
    //
    // // 运行程序
    // println!("\n运行程序");
    // Interpreter::run(&prog)?;

    Ok(())
}

const DEFAULT_CODE: &str = include_str!("default.ps");

fn main() -> Result<(), String> {
    compile_and_run(DEFAULT_CODE)
}
