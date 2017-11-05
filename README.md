# sateko brainfuck

sateko is a simple brainfuck interpreter.

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

Additinally, there's a lot of room for experimentation.

Some ideas:
 * replace tokenizing and parsing with a parser-combinator
 * write a proper parser that will handle more than single character commands
 * JIT brainfuck
 * REPL
 * compile to assembly or bytecode
 * language extension:
   * add "functions"
   * multi-file programs

## License

sateko is 0bsd licensed. Do with it as you please.


[Brainfuck]: http://www.muppetlabs.com/~breadbox/bf/
[Turing machine]: http://mathworld.wolfram.com/TuringMachine.html
