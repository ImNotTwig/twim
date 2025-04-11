#![feature(pattern)]
#![feature(iterator_try_collect)]

mod tokenizer;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use tokenizer::Tokenizer;

fn main() -> anyhow::Result<()> {
    let buf = read_file("./src/stupid.md")?;
    let lines = buf.lines().try_collect::<Vec<_>>()?;

    let mut tkn = Tokenizer {
        tokens: vec![],
        lines: lines.into_boxed_slice(),
    };

    tkn.chunk()?;

    for token in tkn.tokens {
        println!("{token:?}");
    }

    Ok(())
}

fn read_file(path: &str) -> anyhow::Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}
