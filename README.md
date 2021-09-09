[![Rust](https://github.com/scowcron/sateko/actions/workflows/rust.yml/badge.svg)](https://github.com/scowcron/sateko/actions/workflows/rust.yml)
[![Rust](https://img.shields.io/crates/v/sateko.svg)](https://crates.io/crates/sateko)

# sateko brainfuck

sateko is a toy brainfuck compiler.

NOTE: this requires LLVM 12 installed on your machine.

## Installation

Install with:

    $ cargo install sateko

## Compiling

sateko compiles a bf script (e.g. hello.bf) into LLVM IR. The typical process of building an executable
depends on your development environment. Building a typical bf program might look something like this:

    $ sateko hello.bf
    $ llc out.ll
    $ gcc out.s
    $ ./a.out

## About Brainfuck

[Brainfuck][Brainfuck] is an esoteric programming language that models a [Turing machine][Turing machine].
While the language itself is very simple, being productive with it is often less so.

Conceptually, the runtime environment (in this case sateko) provides a tape consisting of
byte-sized cells and a set of operations which manipulate that tape.

### Operations

 * '[' Start a loop
 * ']' End a loop
 * '>' Increase the tape position
 * '<' Decrease the tape position
 * '+' Increase the value at the tape position
 * '-' Decrease the value at the tape position
 * ',' Read a byte from standard in and store value on tape at current position
 * '.' Write value at current tape position to standard output.

### Loops

Each time a loop starts, the interpreter will check the value on tape at the current
position. If that value is 0, the loop will terminate, otherwise, all operations contained
between \[ and \] will be executed.

## Rationale

This is a toy project. I wanted to play with Rust and do a bit of parsing. The brainfuck language is *extremely* simple to
parse, and the number of concerns in the execution envronment is very small.

And, there's a lot of room for experimentation.

Some ideas:
 * replace tokenizing and parsing with a parser-combinator
 * write a proper parser that will handle more than single character commands
 * JIT brainfuck
 * REPL
 * compile to assembly or bytecode   // DONE
 * optimization pass
 * build all the way to executable
 * language extension:
   * add "functions"
   * multi-file programs

## License

sateko uses the 3-clause BSD license. See LICENSE file.


[Brainfuck]: http://www.muppetlabs.com/~breadbox/bf/
[Turing machine]: http://mathworld.wolfram.com/TuringMachine.html
