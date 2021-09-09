use crate::ast::{ASTNode, ASTNodeKind, AST};
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::{io, result};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::basic_block::BasicBlock;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

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

        let void_type = context.void_type();
        let i1_type = context.bool_type();
        let i8_type = context.i8_type();
        let i32_type = context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let putchar_type = i32_type.fn_type(&[BasicTypeEnum::IntType(i32_type)], false);
        let getchar_type = i32_type.fn_type(&[], false);
        let memset_type = void_type.fn_type(&[
            BasicTypeEnum::PointerType(i8_type.ptr_type(inkwell::AddressSpace::Generic)),
            BasicTypeEnum::IntType(i8_type),
            BasicTypeEnum::IntType(i32_type),
            BasicTypeEnum::IntType(i1_type),
        ], false);

        module.add_function("putchar", putchar_type, None);
        module.add_function("getchar", getchar_type, None);
        let memset = module.add_function("llvm.memset.p0i8.i32", memset_type, None);

        let function = module.add_function("main", main_type, None);
        let entry_block = context.append_basic_block(function, "entry");
        builder.position_at_end(entry_block);

        let tape_ptr = builder.build_array_alloca(i8_type, i32_type.const_int(tape_len as u64, false), "tape");
        let active_cell_ptr = builder.build_alloca(i32_type, "active_cell");
        builder.build_call(memset, &[
            BasicValueEnum::PointerValue(tape_ptr),
            BasicValueEnum::IntValue(i8_type.const_int(0, false)),
            BasicValueEnum::IntValue(i32_type.const_int(tape_len as u64, false)),
            BasicValueEnum::IntValue(i1_type.const_int(0, false)),
        ], "");
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

    pub fn build_from_ast(&mut self, ast: &AST) {
        let i32_type = self.context.i32_type();
        let exit_code = i32_type.const_int(0, false);

        for op in &ast.0 {
            self.build_op(&op);
        }

        self.builder.build_return(Some(&exit_code));
    }

    fn build_op(&mut self, op: &ASTNode) -> Option<BasicBlock<'a>> {
        match op.kind {
            ASTNodeKind::Loop => return Some(self.exec_loop(op)),
            ASTNodeKind::IncTape => self.inc_tape(op),
            ASTNodeKind::DecTape => self.dec_tape(op),
            ASTNodeKind::IncVal => self.inc_val(op),
            ASTNodeKind::DecVal => self.dec_val(op),
            ASTNodeKind::Read => self.read(op),
            ASTNodeKind::Write => self.write(op),
        };

        None
    }

    fn exec_loop(&mut self, op: &ASTNode) -> BasicBlock<'a> {
        let i8_type = self.context.i8_type();
        let function = self.module.get_function("main").unwrap();

        let loop_intro_block = self.context.append_basic_block(function, "loop_intro");
        let mut loop_body_block = self.context.append_basic_block(function, "loop_body");
        let loop_out = self.context.append_basic_block(function, "loop_out");

        // jump into loop
        self.builder.build_unconditional_branch(loop_intro_block);

        // check loop condition block
        self.builder.position_at_end(loop_intro_block);
        let i8_zero = i8_type.const_int(0, false);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let cell_ptr = unsafe { self.builder.build_gep(self.tape_ptr, &[active_cell_val], "") };
        let cur_val = self.builder.build_load(cell_ptr, "").into_int_value();
        let check = self.builder.build_int_compare(inkwell::IntPredicate::NE, cur_val, i8_zero, "");
        self.builder.build_conditional_branch(check, loop_body_block, loop_out);

        // loop body - recursively execute each instruction
        self.builder.position_at_end(loop_body_block);
        for op in op.ops.as_ref().unwrap() {
            loop_body_block = self.build_op(op).unwrap_or(loop_body_block);
        }

        // jump back to condition check after loop body
        self.builder.position_at_end(loop_body_block);
        self.builder.build_unconditional_branch(loop_intro_block);

        // continue after loop from next block
        self.builder.position_at_end(loop_out);
        loop_out
    }

    fn inc_tape(&self, op: &ASTNode) {
        let i32_type = self.context.i32_type();

        let i32_one = i32_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let new_cell_val = self.builder.build_int_add(active_cell_val, i32_one, "");
        self.builder.build_store(self.active_cell_ptr, new_cell_val);
    }

    fn dec_tape(&self, op: &ASTNode) {
        let i32_type = self.context.i32_type();

        let i32_one = i32_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let new_cell_val = self.builder.build_int_sub(active_cell_val, i32_one, "");
        self.builder.build_store(self.active_cell_ptr, new_cell_val);
    }


    fn inc_val(&self, op: &ASTNode) {
        let i8_type = self.context.i8_type();

        let i8_one = i8_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let cell_ptr = unsafe { self.builder.build_gep(self.tape_ptr, &[active_cell_val], "") };
        let cur_val = self.builder.build_load(cell_ptr, "").into_int_value();
        let new_val = self.builder.build_int_add(cur_val, i8_one, "");
        self.builder.build_store(cell_ptr, new_val);
    }

    fn dec_val(&self, op: &ASTNode) {
        let i8_type = self.context.i8_type();

        let i32_one = i8_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let cell_ptr = unsafe { self.builder.build_gep(self.tape_ptr, &[active_cell_val], "") };
        let cur_val = self.builder.build_load(cell_ptr, "").into_int_value();
        let new_val = self.builder.build_int_sub(cur_val, i32_one, "");
        self.builder.build_store(cell_ptr, new_val);
    }

    fn read(&self, op: &ASTNode) {
        let i8_type = self.context.i8_type();
        let i32_type = self.context.i32_type();
        let getchar = self.module.get_function("getchar").unwrap();

        let i32_one = i32_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let cell_ptr = unsafe { self.builder.build_gep(self.tape_ptr, &[active_cell_val], "") };
        let new_val = self.builder.build_call(getchar, &[], "").try_as_basic_value().left().unwrap().into_int_value();
        let i8_new_val = self.builder.build_int_truncate(new_val, i8_type, "");
        self.builder.build_store(cell_ptr, i8_new_val);
    }

    fn write(&self, op: &ASTNode) {
        let i32_type = self.context.i32_type();
        let putchar = self.module.get_function("putchar").unwrap();

        let i32_one = i32_type.const_int(1, true);
        let active_cell_val = self.builder.build_load(self.active_cell_ptr, "").into_int_value();
        let cell_ptr = unsafe { self.builder.build_gep(self.tape_ptr, &[active_cell_val], "") };
        let cur_val = self.builder.build_load(cell_ptr, "").into_int_value();
        let i32_cur_val = self.builder.build_int_s_extend(cur_val, i32_type, "").into();
        self.builder.build_call(putchar, &[i32_cur_val], "");
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
