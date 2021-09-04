use crate::ast::{ASTNode, ASTNodeKind, AST};
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::{io, result};

pub use crate::ast::InputPosition;

struct Tape {
    cells: Vec<u8>,
    pos: usize,
}

impl Tape {
    fn with_size(size: usize) -> Tape {
        Tape {
            cells: vec![0; size],
            pos: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RuntimeError {
    pub kind: ErrorKind,
    pub pos: InputPosition,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    OffTapeStart,
    OffTapeEnd(usize),
    IOError,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}:{})", self, self.pos.line, self.pos.pos)
    }
}

impl Error for RuntimeError {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::OffTapeStart => "Tried to move past tape beginning",
            ErrorKind::OffTapeEnd(_) => "Tried to move past end of tape",
            ErrorKind::IOError => "I/O failure",
        }
    }
}

type Result = result::Result<(), RuntimeError>;

pub fn exec(ops: &AST, tape_len: usize, verbose: u8) -> Result {
    let mut tape = Tape::with_size(tape_len);
    exec_ops(&ops.0, &mut tape, verbose)
}

fn exec_ops(ops: &Vec<ASTNode>, tape: &mut Tape, verb: u8) -> Result {
    for op in ops {
        exec_op(&op, tape, verb)?;
    }
    Ok(())
}

fn exec_op(op: &ASTNode, tape: &mut Tape, verb: u8) -> Result {
    match op.kind {
        ASTNodeKind::Loop => exec_loop(op, tape, verb),
        ASTNodeKind::IncTape => inc_tape(op, tape, verb),
        ASTNodeKind::DecTape => dec_tape(op, tape, verb),
        ASTNodeKind::IncVal => inc_val(op, tape, verb),
        ASTNodeKind::DecVal => dec_val(op, tape, verb),
        ASTNodeKind::Read => read(op, tape, verb),
        ASTNodeKind::Write => write(op, tape, verb),
    }
}

fn exec_loop(op: &ASTNode, tape: &mut Tape, verb: u8) -> Result {
    while tape.cells[tape.pos] != 0 {
        if verb > 0 {
            eprintln!(
                "[{},{}] loop check cell {}: {}",
                op.pos.line, op.pos.pos, tape.pos, tape.cells[tape.pos]
            );
        }
        exec_ops(op.ops.as_ref().unwrap(), tape, verb)?;
    }
    if verb > 0 {
        eprintln!(
            "[{},{}] loop end cell {}",
            op.pos.line, op.pos.pos, tape.pos
        );
    }
    Ok(())
}

fn inc_tape(op: &ASTNode, tape: &mut Tape, _verb: u8) -> Result {
    if tape.pos == tape.cells.len() - 1 {
        Err(RuntimeError {
            kind: ErrorKind::OffTapeEnd(tape.cells.len()),
            pos: op.pos.clone(),
        })
    } else {
        tape.pos += 1;
        Ok(())
    }
}

fn dec_tape(op: &ASTNode, tape: &mut Tape, _verb: u8) -> Result {
    if tape.pos == 0 {
        Err(RuntimeError {
            kind: ErrorKind::OffTapeStart,
            pos: op.pos.clone(),
        })
    } else {
        tape.pos -= 1;
        Ok(())
    }
}

fn inc_val(op: &ASTNode, tape: &mut Tape, verb: u8) -> Result {
    tape.cells[tape.pos] = tape.cells[tape.pos].wrapping_add(1);
    if verb > 0 {
        eprintln!(
            "[{}:{}] cell {}: {}, as char: '{}' ",
            op.pos.line, op.pos.pos, tape.pos, tape.cells[tape.pos], tape.cells[tape.pos] as char
        )
    }
    Ok(())
}

fn dec_val(op: &ASTNode, tape: &mut Tape, verb: u8) -> Result {
    tape.cells[tape.pos] = tape.cells[tape.pos].wrapping_sub(1);
    if verb > 0 {
        eprintln!(
            "[{}:{}] cell {}: {}, as char: '{}' ",
            op.pos.line, op.pos.pos, tape.pos, tape.cells[tape.pos], tape.cells[tape.pos] as char
        )
    }
    Ok(())
}

fn read(op: &ASTNode, tape: &mut Tape, _verb: u8) -> Result {
    let mut buf: [u8; 1] = [0];
    match io::stdin().read(&mut buf) {
        Ok(0) => Ok(()),
        Ok(_) => {
            tape.cells[tape.pos] = buf[0];
            Ok(())
        }
        _ => Err(RuntimeError {
            kind: ErrorKind::IOError,
            pos: op.pos.clone(),
        }),
    }
}

fn write(op: &ASTNode, tape: &Tape, _verb: u8) -> Result {
    let buf = [tape.cells[tape.pos]];
    match io::stdout().write(&buf) {
        Ok(1) => Ok(()),
        Ok(_n) => Err(RuntimeError {
            kind: ErrorKind::IOError,
            pos: op.pos.clone(),
        }),
        Err(_e) => Err(RuntimeError {
            kind: ErrorKind::IOError,
            pos: op.pos.clone(),
        }),
    }
}

#[cfg(test)]
mod test {
    use super::{exec, ErrorKind, InputPosition, RuntimeError, Tape};
    use ast::AST;
    use token::tokenize;

    macro_rules! assert_ok {
        ( $x: expr ) => {
            assert_eq!(Ok(()), $x)
        };
    }

    /// Utility
    fn exec_str(s: &str) -> Result<(), RuntimeError> {
        let ts = tokenize(s);
        let ops = AST::from_tokens(&ts).unwrap();
        exec(&ops, 30_000, 0)
    }

    /// Utility
    fn exec_str_with_tape(s: &str, tape: &mut Tape) -> Result<(), RuntimeError> {
        let ts = tokenize(s);
        let ops = AST::from_tokens(&ts).unwrap();
        super::exec_ops(&ops.0, tape, 0)
    }

    #[test]
    fn empty() {
        assert_ok!(exec_str(""));
    }

    #[test]
    fn add() {
        let mut t = Tape::with_size(1);
        for i in 0..255 {
            exec_str_with_tape("+", &mut t).unwrap();
            assert_eq!(t.cells[t.pos], i + 1);
        }
        exec_str_with_tape("+", &mut t).unwrap();
        assert_eq!(t.cells[t.pos], 0);
    }

    #[test]
    fn sub() {
        let mut t = Tape::with_size(1);
        exec_str_with_tape("-", &mut t).unwrap();
        assert_eq!(t.cells[t.pos], 255);
        for i in 0..255 {
            exec_str_with_tape("-", &mut t).unwrap();
            assert_eq!(t.cells[t.pos], 254 - i);
        }
    }

    #[test]
    fn inc_tape() {
        let mut t = Tape::with_size(3);
        exec_str_with_tape(">", &mut t).unwrap();
        assert_eq!(t.pos, 1);
        exec_str_with_tape(">", &mut t).unwrap();
        assert_eq!(t.pos, 2);
        let expect = Err(RuntimeError {
            kind: ErrorKind::OffTapeEnd(3),
            pos: InputPosition { line: 1, pos: 1 },
        });
        assert_eq!(expect, exec_str_with_tape(">", &mut t));
    }

    #[test]
    fn dec_tape() {
        let mut t = Tape::with_size(3);
        t.pos = 2;
        exec_str_with_tape("<", &mut t).unwrap();
        assert_eq!(t.pos, 1);
        exec_str_with_tape("<", &mut t).unwrap();
        assert_eq!(t.pos, 0);
        let expect = Err(RuntimeError {
            kind: ErrorKind::OffTapeStart,
            pos: InputPosition { line: 1, pos: 1 },
        });
        assert_eq!(expect, exec_str_with_tape("<", &mut t));
    }

    #[test]
    fn loop_() {
        let mut t = Tape::with_size(2);
        t.cells[0] = 21;
        exec_str_with_tape("[>++<-]", &mut t).unwrap();
        assert_eq!(t.pos, 0);
        assert_eq!(t.cells[0], 0);
        assert_eq!(t.cells[1], 42);
    }
}
