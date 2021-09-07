#![allow(unused)]

mod ast;
mod exec;
mod token;

use argparse::{ArgumentParser, IncrBy, Store};
use std::fs::File;
use std::io::Read;
use inkwell::context::Context;
use crate::exec::IrBuilder;

const LLVM_OUTPUT: &'static str = "out.ll";
const COMPILER: &'static str = "llc";


fn main() {
    let mut fname = String::new();
    let mut tape_len = 30_000;
    let mut verbose = 0;
    let mut compiler = String::new();

    {
        let mut args = ArgumentParser::new();
        args.set_description("sateko brainfuck.");
        args.refer(&mut fname)
            .add_argument("FILE", Store, "path to script")
            .required();
        args.refer(&mut tape_len).add_option(
            &["-t", "--tape-length"],
            Store,
            "number of cells on tape",
        );
        args.refer(&mut verbose)
            .add_option(&["-d", "--debug"], IncrBy(1), "enable debug output");
        args.refer(&mut compiler)
            .add_option(&["-c", "--compiler"], Store, "llvm compiler path");

        args.parse_args_or_exit();
    }

    let mut raw = String::new();
    let mut f = match File::open(&fname) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open \"{}\": {}", fname, e);
            return;
        }
    };
    if let Err(e) = f.read_to_string(&mut raw) {
        println!("Failed to read \"{}\": {}", fname, e);
        return;
    };

    let ts = token::tokenize(&raw);
    let ops = match ast::AST::from_tokens(&ts) {
        Ok(ops) => ops,
        Err(e) => {
            println!("Parse failed: {}", e);
            return;
        }
    };

    let context = Context::create();
    let mut irbuilder = IrBuilder::create(&context, tape_len);
    irbuilder.build_from_ast(&ops);
    let module = irbuilder.get_module();
    module.set_name(&fname);
    module.set_source_file_name(&fname);
    if let Err(e) = module.print_to_file(LLVM_OUTPUT) {
        println!("Failed to generate LLVM IR: {}", e);
        return;
    };

    // out.ll -> out.s
    let compiler = "llc";
    std::process::Command::new(compiler)
        .arg(LLVM_OUTPUT);

    // out.s -> a.out

}
