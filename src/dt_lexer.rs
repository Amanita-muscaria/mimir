extern crate logos;
use crate::dt_lexer::DTToken::NodeStart;
use logos::Logos;

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
    #[token = r#"""#]
    Quote,
    #[token = "<"]
    AngleOpen,
    #[token = ">"]
    AngleClose,
    #[regex = "/([[:alnum:]-]+)/"]
    Directive,
    #[token = "\n"]
    NewLine,

    #[regex = r#"([#[:alnum:]_@,./-]+)"#]
    Text,
}

pub fn lex(file_data: &String) -> Result<Vec<DTInfo>, DTError> {
    let mut lexer = DTToken::lexer(file_data.as_str());
    let mut tokens: Vec<DTInfo> = Vec::new();

    loop {
        match lexer.token {
            DTToken::LineComment => {
                while lexer.token != DTToken::NewLine {
                    lexer.advance();
                    if lexer.token == DTToken::End {
                        return Err(DTError::UnexpectedEOF(lexer.range().start));
                    } else if lexer.token == DTToken::Error {
                        return Err(DTError::UnknownSymbol(
                            lexer.slice().to_string(),
                            lexer.range().start,
                        ));
                    }
                }
            }
            DTToken::BlockCommentStart => {
                while lexer.token != DTToken::BlockCommentEnd {
                    lexer.advance();
                    if lexer.token == DTToken::End {
                        return Err(DTError::UnexpectedEOF(lexer.range().start));
                    } else if lexer.token == DTToken::Error {
                        let slice = lexer.slice().to_string();
                        return Err(DTError::UnknownSymbol(slice, lexer.range().start));
                    }
                }
            }
            DTToken::Define => {
                lexer.advance();
                if lexer.token != DTToken::Text {
                    return Err(DTError::BadDefine(lexer.range().start));
                } else {
                    let data = lexer.slice().to_string();
                    let def: Vec<_> = data.splitn(2, char::is_whitespace).collect();
                    tokens.push(DTInfo::Define(def[0].to_string(), def[1].to_string()));
                }
            }
            DTToken::Text => {
                let lhs = lexer.slice().to_string();
                lexer.advance();
                match lexer.token {
                    DTToken::Equals => {
                        let mut rhs: Vec<String> = Vec::new();
                        lexer.advance();
                        while lexer.token != DTToken::StatementEnd {
                            let slice = lexer.slice().to_string();
                            match lexer.token {
                                DTToken::NewLine => {}
                                DTToken::Text => rhs.push(slice),
                                DTToken::Error => {
                                    return Err(DTError::UnknownSymbol(slice, lexer.range().start))
                                }
                                DTToken::End => {
                                    return Err(DTError::UnexpectedEOF(lexer.range().start))
                                }
                                DTToken::BlockCommentStart => {
                                    while lexer.token != DTToken::BlockCommentEnd {
                                        lexer.advance();
                                    }
                                }
                                DTToken::LineComment => {
                                    while lexer.token != DTToken::NewLine {
                                        lexer.advance();
                                    }
                                }
                                DTToken::RefNode => {
                                    lexer.advance();
                                    let val = format!("{}{}", slice, lexer.slice().to_string());
                                    rhs.push(val);
                                }
                                DTToken::Quote => rhs.push(r#"""#.to_string()),
                                DTToken::AngleOpen => rhs.push("<".to_string()),
                                DTToken::AngleClose => rhs.push(">".to_string()),
                                _ => return Err(DTError::Why(slice, lexer.range().start, line!())),
                            }
                            lexer.advance();
                        }
                        tokens.push(DTInfo::Property(lhs, Some(rhs.join(" "))));
                    }
                    DTToken::StatementEnd => tokens.push(DTInfo::Property(lhs, None)),
                    DTToken::Label => {
                        let label = lhs;
                        let mut name = String::new();
                        lexer.advance();
                        while lexer.token != NodeStart {
                            let slice = lexer.slice().to_string();
                            match lexer.token {
                                DTToken::Text => {
                                    name = slice;
                                }
                                DTToken::BlockCommentStart => {
                                    while lexer.token != DTToken::BlockCommentEnd {
                                        lexer.advance();
                                    }
                                }
                                _ => return Err(DTError::Why(slice, lexer.range().start, line!())),
                            }
                            lexer.advance();
                        }
                        tokens.push(DTInfo::Node(Some(label), name));
                    }
                    DTToken::NodeStart => tokens.push(DTInfo::Node(None, lhs)),
                    _ => {
                        return Err(DTError::Why(
                            lexer.slice().to_string(),
                            lexer.range().start,
                            line!(),
                        ))
                    }
                }
            }
            DTToken::End => {
                tokens.push(DTInfo::EOF);
                break;
            }
            DTToken::Error => {
                return Err(DTError::UnknownSymbol(
                    lexer.slice().to_string(),
                    lexer.range().start,
                ))
            }
            DTToken::NewLine => (),
            DTToken::NodeEnd => tokens.push(DTInfo::NodeEnd),
            DTToken::Include => {
                let mut file: Option<String> = None;
                lexer.advance();
                while lexer.token != DTToken::NewLine {
                    let slice = lexer.slice().to_string();
                    match lexer.token {
                        DTToken::Text => {
                            if file == None {
                                file = Some(slice);
                            } else {
                                return Err(DTError::Why(slice, lexer.range().start, line!()));
                            }
                        }
                        DTToken::Quote => (),
                        DTToken::AngleOpen => (),
                        DTToken::AngleClose => (),
                        _ => return Err(DTError::Why(slice, lexer.range().start, line!())),
                    };
                    lexer.advance();
                }
                tokens.push(DTInfo::Include(file.unwrap()));
            }
            DTToken::Directive => {
                let directive = lexer.slice().to_string();
                lexer.advance();
                let mut rhs: Vec<String> = Vec::new();
                while lexer.token != DTToken::StatementEnd {
                    match lexer.token {
                        DTToken::Text => {
                            rhs.push(lexer.slice().to_string());
                        }
                        DTToken::RefNode => {
                            lexer.advance();
                            let node_name = lexer.slice().to_string();
                            rhs.push(format!("&{}", node_name));
                        }
                        _ => {
                            return Err(DTError::Why(
                                lexer.slice().to_string(),
                                lexer.range().start,
                                line!(),
                            ))
                        }
                    }
                    lexer.advance();
                }
                if !rhs.is_empty() {
                    tokens.push(DTInfo::Directive(directive, Some(rhs.join(""))));
                } else {
                    tokens.push(DTInfo::Directive(directive, None));
                }
            }
            DTToken::RefNode => {
                lexer.advance();
                if lexer.token != DTToken::Text {
                    return Err(DTError::Why(
                        lexer.slice().to_string(),
                        lexer.range().start,
                        line!(),
                    ));
                } else {
                    let node_name = lexer.slice().to_string();
                    lexer.advance();
                    if lexer.token == DTToken::NodeStart {
                        tokens.push(DTInfo::RefNode(node_name));
                    } else {
                        return Err(DTError::Why(
                            lexer.slice().to_string(),
                            lexer.range().start,
                            line!(),
                        ));
                    }
                }
            }
            _ => {
                return Err(DTError::Why(
                    lexer.slice().to_string(),
                    lexer.range().start,
                    line!(),
                ))
            }
        };
        lexer.advance();
    }
    Ok(tokens)
}
