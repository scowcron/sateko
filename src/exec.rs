use crate::ast::{ASTNode, ASTNodeKind, AST};
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::{io, result};
use inkwell::context::Context;
use inkwell::module::Module;

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

pub struct IrBuilder<'a> {
    context: &'a Context,
    module: Module<'a>,
    builder: inkwell::builder::Builder<'a>,
    tape_ptr: inkwell::values::PointerValue<'a>,
    active_cell_ptr: inkwell::values::PointerValue<'a>,
    tape_len: u64,
}

impl<'a> IrBuilder<'a> {
    pub fn create(context: &'a Context, tape_len: u32) -> Self {
        // FIXME should probably be name of input file
        let module = context.create_module("sateko");
        let builder = context.create_builder();
        // TODO ?machine info

        let i8_type = context.i8_type();
        let i32_type = context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let putc_type = i32_type.fn_type(&[inkwell::types::BasicTypeEnum::IntType(i32_type)], false);

        module.add_function("putchar", putc_type, None);

        let function = module.add_function("main", main_type, None);
        let basic_block = context.append_basic_block(function, "entry");
        builder.position_at_end(basic_block);

        let tape_ptr = builder.build_array_alloca(i8_type, i32_type.const_int(tape_len as u64, false), "tape");
        let active_cell_ptr = builder.build_alloca(i32_type, "active_cell");
        builder.build_store(active_cell_ptr, i32_type.const_int(0, false));

        Self {
            context,
            module,
            builder,
            tape_ptr,
            active_cell_ptr,
            tape_len: tape_len as u64,
        }
    }

    pub fn build_from_ast(&self, ast: &AST) {
        let i32_type = self.context.i32_type();
        let exit_code = i32_type.const_int(0, false);

        for op in &ast.0 {
            self.exec_op(&op);
        }

        self.builder.build_return(Some(&exit_code));
    }

    fn exec_op(&self, op: &ASTNode) {
        match op.kind {
            ASTNodeKind::Loop => self.exec_loop(op),
            ASTNodeKind::IncTape => self.inc_tape(op),
            ASTNodeKind::DecTape => self.dec_tape(op),
            ASTNodeKind::IncVal => self.inc_val(op),
            ASTNodeKind::DecVal => self.dec_val(op),
            ASTNodeKind::Read => self.read(op),
            ASTNodeKind::Write => self.write(op),
        }
    }

    fn exec_loop(&self, op: &ASTNode) {
        // TODO
    }

    fn inc_tape(&self, op: &ASTNode) {
        let i32_type = self.context.i32_type();

        let i32_one = i32_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let new_cell_val = self.builder.build_int_add(i32_one, active_cell_val, "");
        self.builder.build_store(self.active_cell_ptr, new_cell_val);
    }

    fn dec_tape(&self, op: &ASTNode) {
        let i32_type = self.context.i32_type();

        let i32_one = i32_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let new_cell_val = self.builder.build_int_sub(i32_one, active_cell_val, "");
        self.builder.build_store(self.active_cell_ptr, new_cell_val);
    }


    fn inc_val(&self, op: &ASTNode) {
        let i8_type = self.context.i8_type();

        let i8_one = i8_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let cell_ptr = unsafe { self.builder.build_gep(self.tape_ptr, &[active_cell_val], "") };
        let cur_val = self.builder.build_load(cell_ptr, "").into_int_value();
        let new_val = self.builder.build_int_add(i8_one, cur_val, "");
        self.builder.build_store(cell_ptr, new_val);
    }

    fn dec_val(&self, op: &ASTNode) {
        let i8_type = self.context.i8_type();

        let i32_one = i8_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let cell_ptr = unsafe { self.builder.build_gep(self.tape_ptr, &[active_cell_val], "") };
        let cur_val = self.builder.build_load(cell_ptr, "").into_int_value();
        let new_val = self.builder.build_int_sub(i32_one, cur_val, "");
        self.builder.build_store(cell_ptr, new_val);
    }

    fn read(&self, op: &ASTNode) {
        /*
        let i32_type = self.context.i32_type();
        let getchar = self.module.get_function("getchar").unwrap();

        let i32_one = i32_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let cell_ptr = unsafe { self.builder.build_gep(self.tape_ptr, &[active_cell_val], "") };
        let new_val = self.builder.build_call(getchar, &[], "");
        self.builder.build_store(cell_ptr, new_val);
        */

        // TOOD
    }

    fn write(&self, op: &ASTNode) {
        /*
        let i32_type = self.context.i32_type();
        let putchar = self.module.get_function("putchar").unwrap();

        let i32_one = i32_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let cell_ptr = unsafe { self.builder.build_gep(self.tape_ptr, &[active_cell_val], "") };
        let cur_val = self.builder.build_load(cell_ptr, "");
        self.builder.build_call(putchar, &[cur_val], "");
        */
    }

    pub fn get_module(&self) -> &Module<'a> {
        &self.module
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
        //exec_ops(op.ops.as_ref().unwrap(), tape, verb)?;
    }
    if verb > 0 {
        eprintln!(
            "[{},{}] loop end cell {}",
            op.pos.line, op.pos.pos, tape.pos
        );
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
