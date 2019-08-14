#[macro_use]
extern crate structopt;
mod dt_lexer;

use crate::dt_lexer::{DTError, DTInfo};
use dt_lexer::DTLexer;
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
    let mut lex_stack: Vec<DTLexer> = Vec::new();
    let mut file_stack: Vec<(String, String)> = Vec::new();
    let mut f = File::open(&first_file).unwrap();
    let mut file_data = String::new();

    match f.read_to_string(&mut file_data) {
        Ok(_o) => (),
        Err(_e) => panic!("Unknown file {}", first_file),
    };
    file_stack.push((first_file, file_data));
    let mut lex = DTLexer::new(&file_stack[0].1);

    loop {
        let next = lex.next();
        println!("{:?}", next);
        match next {
            Err(e) => match e {
                DTError::UnexpectedEOF(_n) => {
                    println!("UEOF");
                    break;
                }
                _ => continue,
            },
            Ok(i) => {
                if i == DTInfo::EOF {
                    break;
                }
            }
        }
    }
}
