use std::io::{BufRead, Write};

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

pub fn main<R: BufRead, W: Write>(mut input: R, mut output: W) -> Result<(), std::io::Error> {
    /*
{{ stub_generator }}
    */
    println!("Hello, world!");
    Ok(())
}

