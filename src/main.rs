#![feature(core_panic)]

#[macro_use]
extern crate structopt;

mod dt_lexer;
mod root;

use dt_lexer::{lex, DTInfo};
use root::Root;
use std::fs::File;
use std::io::Read;
use std::str;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
enum OutputFormat {
    File,
    DotFile,
}

impl str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "file" => Ok(Self::File),
            "dot" => Ok(Self::DotFile),
            "dotfile" => Ok(Self::DotFile),
            _ => Err("Invalid output format".to_string()),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Mimir",
    about = "Pull together information from a device tree file and its includes"
)]
struct Opt {
    #[structopt(help = "Path to device tree file")]
    input: String,

    #[structopt(
        short = "o",
        long = "output",
        help = "Output File Format",
        default_value = "File"
    )]
    output: OutputFormat,
}

fn main() {
    let opt = Opt::from_args();
    let first_file = opt.input;
    let mut r = Root::new();
    let mut files: Vec<Vec<DTInfo>> = Vec::new();
    let mut path: Vec<String> = Vec::new();

    match File::open(&first_file) {
        Ok(mut f) => {
            let mut d = String::new();
            match f.read_to_string(&mut d) {
                Ok(_o) => match lex(&d) {
                    Ok(t) => files.push(t),
                    Err(e) => panic!("Err {:?} when lexxing {}", e, first_file),
                },
                Err(e) => panic!("Could not read file {}. Error: {}", first_file, e),
            }
        }
        Err(e) => panic!("could not open {}. Error: {}", first_file, e),
    }

    while !files.is_empty() {
        let mut current = files.pop().unwrap();
        loop {
            let token = current.remove(0);
            match token {
                DTInfo::Include(i) => match File::open(&i) {
                    Ok(mut f) => {
                        let mut d = String::new();
                        match f.read_to_string(&mut d) {
                            Ok(_o) => match lex(&d) {
                                Ok(t) => {
                                    files.push(current);
                                    files.push(t);
                                    break;
                                }
                                Err(e) => panic!("Err {:?} when lexxing {}", e, i),
                            },
                            Err(e) => panic!("Error reading {}: {:?}", i, e),
                        }
                    }
                    Err(e) => panic!("Error opening {}: {:?}", i, e),
                },
                DTInfo::Directive(d, t) => {
                    match d.as_str() {
                        "delete-node" => {
                            match t {
                                Some(mut t) => {
                                    if t.starts_with("&") {
                                        let name = t.split_off(1);
                                        r = match r.delete_from_label(name) {
                                            Err(e) => panic!("Error while deleting node: {:?}", e),
                                            Ok(s) => s,
                                        };
                                    } else {
                                        let new = t.split("/").map(|i| i.to_string());
                                        let mut p = path.clone();
                                        p.append(&mut new.collect());
                                        r = match r.delete_node(p) {
                                            Err(e) => panic!("Error while deleting node: {:?}", e),
                                            Ok(s) => s,
                                        };
                                    }
                                }
                                None => panic!("Unknown node to delete"),
                            };
                        }
                        _ => println!("directive: {}", d),
                    };
                }
                DTInfo::Node(label, name) => {
                    r = match r.add_node(&path, &name) {
                        Ok(s) => s,
                        Err(e) => panic!("Error when adding node: {:?}", e),
                    };
                    path.push(name);
                    if let Some(l) = label {
                        r.add_path(l, &path);
                    }
                }
                DTInfo::NodeEnd => {
                    path.pop();
                }
                DTInfo::Property(p, v) => {
                    r = match r.add_property(&path, (p, v)) {
                        Ok(s) => s,
                        Err(e) => panic!("Error when adding property: {:?}", e),
                    };
                }
                DTInfo::Define(n, v) => {
                    r = match r.add_define(n, v) {
                        Ok(s) => s,
                        Err(e) => panic!("Error when adding define: {:?}", e),
                    };
                }
                DTInfo::EOF => {
                    break;
                }
                DTInfo::RefNode(n) => {
                    match r.get_path(n) {
                        Ok(p) => path = p,
                        Err(e) => panic!("{:?}", e),
                    };
                }
            }
        }
    }

    println!("{:#?}", r);
}
