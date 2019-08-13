#[macro_use]
extern crate structopt;

use structopt::StructOpt;

mod dt_file;
use dt_file;

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
}
