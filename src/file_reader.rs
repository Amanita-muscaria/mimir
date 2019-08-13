extern crate regex;
use crate::file_reader::LineType::Directive;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::str;

#[derive(Debug)]
pub struct BufFileReader {
    reader: BufReader<File>,
    pub path: PathBuf,
}

pub enum Directives {
    DeleteProperty,
    DeleteNode,
    Unknown,
}

pub enum Info {
    Node(Option<String>, String),
    Property(String, Option<String>),
    Include(String),
    VersionTag(String),
    MemReserve(String),
    RefNode(String),
    Directive(Directives, Option<String>),
    NodeEnd,
    FileEnd,
}

impl BufFileReader {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self, Error> {
        let path = path.into();
        let file = File::open(&path);
        match file {
            Ok(f) => Ok(BufFileReader {
                reader: BufReader::new(f),
                path: path,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn consume_block_comment(&mut self) -> Result<(), Error> {
        loop {
            let mut buf: Vec<u8> = Vec::new();
            match self.read_until(b'*', &mut buf) {
                Ok(b) => {
                    if b > 0 {
                        let mut next: [u8; 1] = [0];
                        match self.read(&mut next) {
                            Ok(s) => {
                                if s > 0 {
                                    if next[0] == b'/' {
                                        ()
                                    }
                                } else {
                                    return Err(Error::new(ErrorKind::UnexpectedEof, "wat"));
                                }
                            }
                            Err(e) => return Err(e),
                        }
                    } else {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "wat"));
                    }
                }
                Err(e) => return Err(e),
            };
        }
    }

    pub fn get_next(&mut self) -> Result<Info, Error> {
        let mut line = String::new();
        let path = self.path.clone();
        loop {
            match self.read_line(&mut line) {
                Ok(l) => {
                    if l == 0 {
                        return Ok(Info::FileEnd);
                    }
                }
                Err(e) => panic!("failed to read file {:?} error: {}", path, e),
            };

            line = line.trim().to_string();

            if line.starts_with("/*") {
                match self.consume_block_comment() {
                    Ok(_s) => {}
                    Err(e) => panic!("failed to read file {:?} error: {}", path, e),
                }
                continue;
            } else if line.starts_with("//") {
                continue;
            } else if let Some(l) = line.find("//") {
                line.truncate(l + 1);
                break;
            } else if line.is_empty() {
                continue;
            }
        }

        let pos = self.reader.seek(SeekFrom::Current(0)).unwrap();

        match get_line_type(&line) {
            LineType::Node => {
                match parse_node_line(line) {
                    (Some(l), Some(n)) => {
                        return Ok(Info::Node(Some(l), n));
                    }
                    (None, Some(n)) => {
                        return Ok(Info::Node(None, n));
                    }
                    _ => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Node Line parse failed in {:?} pos {}", path, pos),
                        ));
                    }
                };
            }
            LineType::Include => match parse_include_line(line) {
                Some(i) => {
                    return Ok(Info::Include(i));
                }
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Node Line parse failed in {:?} pos {}", path, pos),
                    ));
                }
            },
            LineType::VersionTag => {
                return Ok(Info::VersionTag("vtag".to_string()));
            }
            LineType::MemReserve => {
                return Ok(Info::MemReserve("memrev".to_string()));
            }
            LineType::RefNode => match parse_ref_node_line(line) {
                Some(r) => {
                    return Ok(Info::RefNode(r));
                }
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Ref Node Line parse failed in  {:?} pos {}", path, pos),
                    ));
                }
            },
            LineType::Property => {
                if !line.ends_with(";") {
                    let mut whole = line.into_bytes();
                    match self.read_until(b';', &mut whole) {
                        Ok(l) => {
                            if l == 0 {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    format!(
                                        "Property ended unexpectedly in {:?} pos {}",
                                        path, pos
                                    ),
                                ));
                            } else {
                                line = String::from(str::from_utf8(&whole).unwrap());
                            }
                        }
                        Err(e) => return Err(e),
                    };
                }

                match parse_property_line(line) {
                    (Some(p), Some(v)) => return Ok(Info::Property(p, Some(v))),
                    (Some(p), None) => return Ok(Info::Property(p, None)),
                    _ => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Property Line parse failed in {:?} pos {}", path, pos),
                        ));
                    }
                }
            }
            LineType::Directive => match parse_directive_line(line) {
                (Some(d), Some(t)) => {
                    return Ok(Info::Directive(get_directive_type(d), Some(t)));
                }
                (Some(d), None) => {
                    return Ok(Info::Directive(get_directive_type(d), None));
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Directive Line parse failed in {:?} pos {}", path, pos),
                    ));
                }
            },
            LineType::NodeEnd => {
                return Ok(Info::NodeEnd);
            }
            LineType::Unknown => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Unknown line in {:?} pos {}", path, pos),
                ));
            }
        };
    }
}

impl Read for BufFileReader {
    fn read(&mut self, arry: &mut [u8]) -> Result<usize, Error> {
        return self.reader.read(arry);
    }
}

impl BufRead for BufFileReader {
    fn fill_buf(&mut self) -> Result<&[u8], Error> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt);
    }
}

fn get_directive_type(d: String) -> Directives {
    match d.as_str() {
        "delete-node" => Directives::DeleteNode,
        "delete-property" => Directives::DeleteProperty,
        _ => Directives::Unknown,
    }
}

#[derive(Debug)]
enum LineType {
    Node,
    Include,
    VersionTag,
    MemReserve,
    RefNode,
    Property,
    Directive,
    NodeEnd,
    Unknown,
}

fn get_line_type(ln: &String) -> LineType {
    if ln.starts_with("#include ") || ln.starts_with("/include/ ") {
        return LineType::Include;
    } else if ln == "/dts-v1/;" {
        return LineType::VersionTag;
    } else if ln.starts_with("/") && ln[1..].contains("/") && ln.ends_with(";") {
        return LineType::Directive;
    } else if ln.starts_with("memreserve") && ln.ends_with(";") {
        return LineType::MemReserve;
    } else if ln.starts_with("&") && ln.ends_with("{") {
        return LineType::RefNode;
    } else if ln.ends_with("{") {
        return LineType::Node;
    } else if ln == "};" {
        return LineType::NodeEnd;
    } else if ln.chars().next().unwrap().is_alphabetic() && (ln.contains("=") || ln.ends_with(";"))
    {
        return LineType::Property;
    }
    LineType::Unknown
}

fn parse_node_line(ln: String) -> (Option<String>, Option<String>) {
    lazy_static! {
        static ref NODE_REG_EX: Regex =
            Regex::new(r"((?P<l>[[:alpha:]][\w]+):)?\s*(?P<n>[/\w,._+@-]+)\s*\{").unwrap();
    };
    let matches = NODE_REG_EX.captures(&ln);

    match matches {
        Some(n) => {
            let label = match n.name("l") {
                Some(l) => Some(l.as_str().to_string()),
                None => None,
            };
            let node_name = match n.name("n") {
                Some(n) => Some(n.as_str().to_string()),
                None => None,
            };
            (label, node_name)
        }
        None => (None, None),
    }
}

fn parse_include_line(ln: String) -> Option<String> {
    lazy_static! {
        static ref INCLUDE_REG_EX: Regex = Regex::new(r#"["<]([\w_.]+)[">]"#).unwrap();
    }
    match INCLUDE_REG_EX.captures(&ln) {
        Some(i) => Some(i[1].to_string()),
        None => None,
    }
}

fn parse_ref_node_line(ln: String) -> Option<String> {
    lazy_static! {
        static ref REF_NODE_REG_EX: Regex = Regex::new(r"&([[:alpha:]][\w_]+)\s*\{").unwrap();
    }

    match REF_NODE_REG_EX.captures(&ln) {
        Some(n) => Some(n[1].to_string()),
        None => None,
    }
}

fn parse_directive_line(ln: String) -> (Option<String>, Option<String>) {
    lazy_static! {
        static ref DIR_LINE_REG_EX: Regex =
            Regex::new(r"/([[:alpha:]-]+)/ ([[:alnum:]_-]+);").unwrap();
    }
    let matches = DIR_LINE_REG_EX.captures(&ln);
    let mut directive = None;
    let mut target = None;

    match matches {
        Some(d) => {
            directive = Some(d[1].to_string());
            target = Some(d[2].to_string());
        }
        None => (),
    };

    (directive, target)
}

fn parse_property_line(ln: String) -> (Option<String>, Option<String>) {
    lazy_static! {
        static ref PROP_LINE_REG_EX: Regex = Regex::new(
            r#"(?P<n>[#[:alnum:],._+-]+)(\s*=\s*(<|"|\[)(?P<p>[[:alnum:]&,\s]+)(>|"|\])+;)"#
        )
        .unwrap();
    }
    let matches = PROP_LINE_REG_EX.captures(&ln);

    match matches {
        Some(m) => {
            let prop_name = match m.name("n") {
                Some(n) => Some(n.as_str().to_string()),
                None => None,
            };
            let prop = match m.name("p") {
                Some(p) => Some(p.as_str().to_string()),
                None => None,
            };
            (prop_name, prop)
        }
        None => (None, None),
    }
}
