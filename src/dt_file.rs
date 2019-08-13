extern crate Logos;
use crate::file_reader::LineType::Directive;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::str;
use logos::Logos;

#[derive(Debug)]
pub struct DTFile {
    reader: BufReader<File>,
    pub path: PathBuf,
    tokens: Vec<Tokens>,
    state: State,
}

impl DTFile {
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
}

impl Read for DTFile {
    fn read(&mut self, arry: &mut [u8]) -> Result<usize, Error> {
        return self.reader.read(arry);
    }
}

impl BufRead for DTFile {
    fn fill_buf(&mut self) -> Result<&[u8], Error> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt);
    }
}


