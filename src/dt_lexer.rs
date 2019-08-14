extern crate logos;
use crate::dt_lexer::DTToken::NodeStart;
use logos::{Lexer, Logos};

#[derive(Debug, PartialEq)]
pub enum DTInfo {
    Include(String),
    Directive(String, Option<String>),
    Node(Option<String>, String),
    NodeEnd,
    Property(String, Option<String>),
    Define(String, String),
    EOF,
    RefNode(String),
}

#[derive(Debug)]
pub enum DTError {
    UnexpectedEOF(usize),
    UnknownSymbol(String, usize),
    BadDefine(usize),
    Why(String, usize, u32),
}

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
enum DTToken {
    #[end]
    End,
    #[error]
    Error,

    #[regex = r#"(#include)|(/include/)"#]
    Include,
    #[token = "#define"]
    Define,

    #[token = "/*"]
    BlockCommentStart,
    #[token = "*/"]
    BlockCommentEnd,
    #[token = "//"]
    LineComment,
    #[token = "&"]
    RefNode,
    #[token = ":"]
    Label,
    #[token = "{"]
    NodeStart,
    #[token = "};"]
    NodeEnd,
    #[token = "="]
    Equals,
    #[token = ";"]
    StatementEnd,
    #[regex = "/([[:alnum:]-]+)/"]
    Directive,
    #[token = "\n"]
    NewLine,

    #[regex = r#"([#[:alnum:]_@<>,./"-]+)"#]
    Text,
}

pub struct DTLexer<'a> {
    lex: Lexer<DTToken, &'a str>,
}

impl<'a> DTLexer<'a> {
    pub fn new(data: &'a str) -> Self {
        DTLexer {
            lex: DTToken::lexer(data),
        }
    }

    pub fn next(&mut self) -> Result<DTInfo, DTError> {
        let lex = &mut self.lex;
        loop {
            lex.advance();
            match lex.token {
                DTToken::LineComment => while lex.token != DTToken::NewLine {
                    lex.advance();
                    println!("line comment: {}", lex.slice());
                    if lex.token == DTToken::End {
                        return Err(DTError::UnexpectedEOF(lex.range().start));
                    } else if lex.token == DTToken::Error {
                        return Err(DTError::UnknownSymbol(
                            lex.slice().to_string(),
                            lex.range().start,
                        ));
                    }
                },
                DTToken::BlockCommentStart => while lex.token != DTToken::BlockCommentEnd {
                    lex.advance();
                    println!("line comment: {}", lex.slice());
                    if lex.token == DTToken::End {
                        return Err(DTError::UnexpectedEOF(lex.range().start));
                    } else if lex.token == DTToken::Error {
                        let slice = lex.slice().to_string();
                        return Err(DTError::UnknownSymbol(slice, lex.range().start));
                    }
                },
                DTToken::Define => {
                    lex.advance();
                    if lex.token != DTToken::Text {
                        return Err(DTError::BadDefine(lex.range().start));
                    } else {
                        let data = lex.slice().to_string();
                        let def: Vec<_> = data.splitn(2, char::is_whitespace).collect();
                        return Ok(DTInfo::Define(def[0].to_string(), def[1].to_string()));
                    }
                }
                DTToken::Text => {
                    let lhs = lex.slice().to_string();

                    lex.advance();
                    match lex.token {
                        DTToken::Equals => {
                            let mut rhs: Vec<String> = Vec::new();
                            lex.advance();
                            while lex.token != DTToken::StatementEnd {
                                let slice = lex.slice().to_string();
                                match lex.token {
                                    DTToken::NewLine => {},
                                    DTToken::Text => rhs.push(slice),
                                    DTToken::Error => {
                                        return Err(DTError::UnknownSymbol(
                                            slice,
                                            lex.range().start,
                                        ))
                                    }
                                    DTToken::End => {
                                        return Err(DTError::UnexpectedEOF(lex.range().start))
                                    }
                                    DTToken::BlockCommentStart => {
                                        while lex.token != DTToken::BlockCommentEnd {
                                            lex.advance();
                                        }
                                    }
                                    DTToken::LineComment => {
                                        while lex.token != DTToken::NewLine {
                                            lex.advance();
                                        }
                                    }
                                    DTToken::RefNode => {
                                        lex.advance();
                                        let val = format!("{}{}", slice, lex.slice().to_string());
                                        rhs.push(val);
                                    }
                                    _ => {
                                        return Err(DTError::Why(
                                            slice,
                                            lex.range().start,
                                            line!(),
                                        ))
                                    }
                                }
                                lex.advance();
                            }
                            return Ok(DTInfo::Property(lhs, Some(rhs.join(" "))));
                        }
                        DTToken::StatementEnd => return Ok(DTInfo::Property(lhs, None)),
                        DTToken::Label => {
                            let label = lhs;
                            let mut name = String::new();
                            lex.advance();
                            while lex.token != NodeStart {
                                let slice = lex.slice().to_string();
                                match lex.token {
                                    DTToken::Text => {
                                        name = slice;
                                    }
                                    DTToken::BlockCommentStart => {
                                        while lex.token != DTToken::BlockCommentEnd {
                                            lex.advance();
                                        }
                                    }
                                    _ => {
                                        return Err(DTError::Why(
                                            slice,
                                            lex.range().start,
                                            line!(),
                                        ))
                                    }
                                }
                                lex.advance();
                            }
                            return Ok(DTInfo::Node(Some(label), name));
                        }
                        DTToken::NodeStart => return Ok(DTInfo::Node(None, lhs)),
                        _ => {
                            return Err(DTError::Why(
                                lex.slice().to_string(),
                                lex.range().start,
                                line!(),
                            ))
                        }
                    }
                }
                DTToken::End => return Ok(DTInfo::EOF),
                DTToken::Error => {
                    return Err(DTError::UnknownSymbol(
                        lex.slice().to_string(),
                        lex.range().start,
                    ))
                }
                DTToken::NewLine => continue,
                DTToken::NodeEnd => return Ok(DTInfo::NodeEnd),
                DTToken::Include => {
                    let mut file: Option<String> = None;
                    lex.advance();
                    while lex.token != DTToken::NewLine {
                        let slice = lex.slice().to_string();
                        match lex.token {
                            DTToken::Text => {
                                if file == None {
                                    file = Some(slice);
                                } else {
                                    return Err(DTError::Why(
                                        slice,
                                        lex.range().start,
                                        line!(),
                                    ));
                                }
                            }
                            _ => {
                                return Err(DTError::Why(
                                    slice,
                                    lex.range().start,
                                    line!(),
                                ))
                            }
                        }
                        lex.advance();
                    }
                    return Ok(DTInfo::Include(file.unwrap()));
                }
                DTToken::Directive => {
                    let directive = lex.slice().to_string();
                    lex.advance();
                    let mut rhs: Vec<String> = Vec::new();
                    while lex.token != DTToken::StatementEnd {
                        match lex.token {
                            DTToken::Text => {
                                rhs.push(lex.slice().to_string());
                            }
                            DTToken::RefNode => {
                                lex.advance();
                                let node_name = lex.slice().to_string();
                                rhs.push(format!("&{}", node_name));
                            }
                            _ => {
                                return Err(DTError::Why(
                                    lex.slice().to_string(),
                                    lex.range().start,
                                    line!(),
                                ))
                            }
                        }
                        lex.advance();
                    }
                    if !rhs.is_empty() {
                        return Ok(DTInfo::Directive(directive, Some(rhs.join(""))));
                    } else {
                        return Ok(DTInfo::Directive(directive, None));
                    }
                }
                DTToken::RefNode => {
                    lex.advance();
                    if lex.token != DTToken::Text {
                        return Err(DTError::Why(
                            lex.slice().to_string(),
                            lex.range().start,
                            line!(),
                        ));
                    } else {
                        let node_name = lex.slice().to_string();
                        lex.advance();
                        if lex.token == DTToken::NodeStart {
                            return Ok(DTInfo::RefNode(node_name));
                        } else {
                            return Err(DTError::Why(
                                lex.slice().to_string(),
                                lex.range().start,
                                line!(),
                            ));
                        }
                    }
                }
                _ => {
                    return Err(DTError::Why(
                        lex.slice().to_string(),
                        lex.range().start,
                        line!(),
                    ))
                }
            };
        }
    }
}
