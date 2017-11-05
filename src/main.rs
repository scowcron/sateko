extern crate argparse;

mod token;
mod ast;
mod exec;

use std::fs::File;
use std::io::Read;
use argparse::{ArgumentParser, Store};

fn main() {
    let mut fname = String::new();
    let mut tape_len = 30_000;

    {
        let mut args = ArgumentParser::new();
        args.set_description("sateko brainfuck.");
        args.refer(&mut fname)
            .add_argument("FILE", Store, "path to script")
            .required();
        args.refer(&mut tape_len)
            .add_option(&["-t", "--tape-length"], Store,
                        "number of cells on tape");
        args.parse_args_or_exit();
    }

    let mut raw = String::new();
    let mut f = File::open(&fname).unwrap();
    f.read_to_string(&mut raw).unwrap();

    let ts = token::tokenize(&raw);
    let ops = match ast::AST::from_tokens(&ts) {
        Ok(ops) => ops,
        Err(e) => {
            println!("Parse failed: {}", e);
            return;
        }
    };

    if let Err(e) = exec::exec(&ops, tape_len){
        println!("Runtime error: {}", e);
        return;
    }
}
