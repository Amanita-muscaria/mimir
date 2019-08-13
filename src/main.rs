#![feature(core_panic)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate structopt;

mod dt_node;
mod file_reader;
use core::panicking::panic_fmt;
use dt_node::DTNode;
use file_reader::{BufFileReader, Info};
use std::collections::HashMap;
use std::path::PathBuf;
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
    #[structopt(parse(from_os_str), help = "Path to device tree file")]
    input: PathBuf,

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
    let mut file_stack: Vec<BufFileReader> = Vec::new();
    let mut current_path: Vec<String> = Vec::new();
    let mut labels: HashMap<String, Vec<String>> = HashMap::new();
    let mut root = DTNode::new("/");
    let &mut current_node = &mut root;

    file_stack.push(open_file(first_file));

    while !file_stack.is_empty() {
        let mut current_file = file_stack.pop().unwrap();
        match current_file.get_next() {
            Ok(n) => match n {
                Info::Node(l, name) => {
                    current_node = current_node.add_child_get(DTNode::new(&name));
                    current_path.push(name);

                    if let Some(label) = l {
                        labels.insert(label, current_path.clone());
                    }

                    file_stack.push(current_file);
                }
                Info::Property(p, v) => {
                    /*match current_node {
                        Some(n) => current_node = Some(&mut n.add_property(p, v)),
                        None => panic!("Property without node?! file: {:?}", current_file.path),
                    }*/
                    file_stack.push(current_file);
                }
                Info::Include(i) => {
                    file_stack.push(current_file);
                    file_stack.push(open_file(i));
                }
                Info::VersionTag(v) => {
                    file_stack.push(current_file);
                }
                Info::MemReserve(m) => {
                    file_stack.push(current_file);
                }
                Info::RefNode(n) => {
//                    match labels.get(&n) {
//                        Some(p) => match &local_root {
//                            Some(mut r) => match r.get_node(p) {
//                                Some(node) => {
//                                    current_node = Some(node);
//                                }
//                                None => {
//                                    panic!(
//                                        "Couldn't find referenced node {} in file: {:?}",
//                                        n, current_file.path
//                                    );
//                                }
//                            },
//                            None => {
//                                panic!("You have labels without a root? {:?}", current_file.path);
//                            }
//                        },
//                        None => {
//                            panic!(
//                                "Unknown node reference: file: {:?}, node: {}",
//                                current_file.path, n
//                            );
//                        }
//                    }
                    file_stack.push(current_file);
                }
                Info::Directive(d, t) => {
                    match d {
                        DeleteProperty => {

                        },
                        DeleteNode => {

                        },
                        Unknown => {

                        },
                    }
                    file_stack.push(current_file);
                }
                Info::NodeEnd => {
                    file_stack.push(current_file);
                }
                Info::FileEnd => (),
            },
            Err(e) => panic!(
                "Error: {} while attempting to get next symbol in {:?}",
                e, current_file.path
            ),
        }
    }
}

fn open_file<P: Into<PathBuf> + Clone>(p: P) -> BufFileReader {
    let path = p.clone();
    match BufFileReader::new(p) {
        Ok(f) => f,
        Err(e) => panic!("Error: {} when opening file {:?}", e, path.into()),
    }
}

fn output(output: OutputFormat) {
    match output {
        OutputFormat::File => {}
        OutputFormat::DotFile => {}
    }
}
