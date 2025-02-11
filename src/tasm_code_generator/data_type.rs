use tasm_lib;
use triton_vm::instruction::LabelledInstruction;
use triton_vm::op_stack::OpStackElement;
use triton_vm::triton_asm;
use triton_vm::BFieldElement;

use crate::ast_types;
use crate::tasm_code_generator::read_n_words_from_memory;
use crate::tasm_code_generator::CompilerState;

pub mod enum_type;
pub mod struct_type;

impl ast_types::DataType {
    /// BEFORE: _ (*first_word | ∅)
    /// AFTER:  _ [value; value_size]
    // TODO: MAKE PRIVATE if possible
    pub(super) fn copy_words_from_memory(
        static_memory_address: Option<BFieldElement>,
        value_size: usize,
    ) -> Vec<LabelledInstruction> {
        // TODO: Consider making subroutines out of this in order to get shorter programs.
        let mut code = match static_memory_address {
            Some(mem_location) => triton_asm!(push {mem_location.value() + value_size as u64 - 1}),
            None => match value_size {
                0 => panic!("values must never have size 0"),
                1 => triton_asm!(),
                _ => triton_asm!(push {value_size as u64 - 1} add),
            },
        };

        // _ *last_word
        code.extend(read_n_words_from_memory(value_size));

        // _ [value; value_size]
        code
    }

    /// Return the code to copy a value in memory to the stack.
    /// `position` refers to a memory address. If this is `None`, the
    /// memory address is assumed to be on top of the stack.
    /// BEFORE: _ <*value>
    /// AFTER: _ [value]
    pub(super) fn load_from_memory(
        &self,
        static_address: Option<BFieldElement>,
        state: &mut CompilerState,
    ) -> Vec<LabelledInstruction> {
        match self {
            ast_types::DataType::Bool
            | ast_types::DataType::U32
            | ast_types::DataType::U64
            | ast_types::DataType::U128
            | ast_types::DataType::Bfe
            | ast_types::DataType::Xfe
            | ast_types::DataType::Digest
            | ast_types::DataType::Tuple(_) => {
                Self::copy_words_from_memory(static_address, self.stack_size())
            }
            ast_types::DataType::List(_, _)
            | ast_types::DataType::Array(_)
            | ast_types::DataType::Boxed(_) => {
                triton_asm!()
            }
            ast_types::DataType::Enum(enum_type) => enum_type.load_from_memory(state),
            ast_types::DataType::Struct(struct_type) => struct_type.load_from_memory(state),
            ast_types::DataType::VoidPointer => todo!(),
            ast_types::DataType::Function(_) => todo!(),
            ast_types::DataType::Unresolved(_) => todo!(),
        }
    }

    /// Copy a value at a position on the stack to the top
    pub(super) fn dup_value_from_stack_code(
        &self,
        position: OpStackElement,
    ) -> Vec<LabelledInstruction> {
        let elem_size = self.stack_size();

        // the position of the deepest element of the value.
        let n: usize = Into::<usize>::into(position) + elem_size - 1;

        let instrs_as_str = format!("dup {}\n", n);
        let instrs_as_str = instrs_as_str.repeat(elem_size);

        triton_asm!({ instrs_as_str })
    }

    /// Return the code for evaluating equality expression
    pub(super) fn compile_eq_code(&self, state: &mut CompilerState) -> Vec<LabelledInstruction> {
        use ast_types::DataType::*;
        match self {
            Bool | U32 | Bfe | VoidPointer => triton_asm!(eq),
            U64 => triton_asm!(
                // _ a_hi a_lo b_hi b_lo
                swap 3
                eq
                swap 2
                eq
                mul
            ),
            U128 => triton_asm!(
                // _ a_3 a_2 a_1 a_0 b_3 b_2 b_1 b_0
                swap 5
                eq
                // _ a_3 a_2 b_0 a_0 b_3 b_2 (b_1 == a_1)
                swap 5
                eq
                // _ a_3 (b_1 == a_1) b_0 a_0 b_3 (b_2 == a_2)
                swap 5
                eq
                // _ (b_2 == a_2) (b_1 == a_1) b_0 a_0 (b_3 == a_3)
                swap 2
                eq
                // _ (b_2 == a_2) (b_1 == a_1) (b_3 == a_3) (b_0 == a_0)
                mul
                mul
                mul
                // _ (b_2 == a_2) * (b_1 == a_1) * (b_3 == a_3) * (b_0 == a_0)
            ),

            Xfe => triton_asm!(
                 // _ a_2 a_1 a_0 b_2 b_1 b_0
                swap 4 // _ a_2 b_0 a_0 b_2 b_1 a_1
                eq     // _ a_2 b_0 a_0 b_2 (b_1 == a_1)
                swap 4 // _ (b_1 == a_1) b_0 a_0 b_2 a_2
                eq     // _ (b_1 == a_1) b_0 a_0 (b_2 == a_2)
                swap 2 // _ (b_1 == a_1) (b_2 == a_2) a_0 b_0
                eq     // _ (b_1 == a_1) (b_2 == a_2) (a_0 == b_0)
                mul    // _ (b_1 == a_1) ((b_2 == a_2)·(a_0 == b_0))
                mul    // _ ((b_1 == a_1)·(b_2 == a_2)·(a_0 == b_0))
            ),
            Digest => {
                let eq_digest =
                    state.import_snippet(Box::new(tasm_lib::hashing::eq_digest::EqDigest));
                triton_asm!(call { eq_digest })
            }
            List(_, _) => todo!(),
            Tuple(_) => todo!(),
            Array(_) => todo!("Equality for arrays not yet implemented"),
            Function(_) => todo!(),
            Struct(_) => todo!(),
            Boxed(_) => todo!("Comparison of MemPointer not supported yet"),
            Unresolved(name) => panic!("Cannot compare unresolved type {name}"),
            Enum(_) => todo!("Equality for enums not yet implemented"),
        }
    }
}
