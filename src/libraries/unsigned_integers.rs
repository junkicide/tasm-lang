use tasm_lib::traits::basic_snippet::BasicSnippet;
use triton_vm::triton_asm;

use crate::ast;
use crate::ast_types;
use crate::graft::Graft;
use crate::tasm_code_generator::CompilerState;
use crate::type_checker::is_u32_based_type;

use super::Library;
use super::LibraryFunction;

#[derive(Clone, Debug)]
pub struct UnsignedIntegersLib {
    pub list_type: ast_types::ListType,
}

const LEADING_ZEROS_METHOD: &str = "leading_zeros";
const COUNT_ONES_METHOD: &str = "count_ones";
const POW_METHOD: &str = "pow";
const OVERFLOWING_ADD_METHOD: &str = "overflowing_add";
const OVERFLOWING_SUB_METHOD: &str = "overflowing_sub";
const WRAPPING_SUB_METHOD: &str = "wrapping_sub";

impl Library for UnsignedIntegersLib {
    fn get_function_name(&self, _full_name: &str) -> Option<String> {
        None
    }

    fn get_method_name(
        &self,
        method_name: &str,
        receiver_type: &ast_types::DataType,
    ) -> Option<String> {
        if is_u32_based_type(receiver_type)
            && matches!(
                method_name,
                LEADING_ZEROS_METHOD
                    | COUNT_ONES_METHOD
                    | POW_METHOD
                    | OVERFLOWING_ADD_METHOD
                    | OVERFLOWING_SUB_METHOD
                    | WRAPPING_SUB_METHOD
            )
        {
            return Some(method_name.to_owned());
        }

        None
    }

    fn method_name_to_signature(
        &self,
        method_name: &str,
        receiver_type: &ast_types::DataType,
        _args: &[ast::Expr<super::Annotation>],
        _type_checker_state: &crate::type_checker::CheckState,
    ) -> ast::FnSignature {
        if !is_u32_based_type(receiver_type) {
            panic!("Cannot call unsigned integer method on non-u32 based value. Receiver type was: {receiver_type}");
        }

        if method_name == COUNT_ONES_METHOD && ast_types::DataType::U32 == *receiver_type {
            return get_count_ones_u32_method().signature;
        }

        let snippet = name_to_tasm_lib_snippet(method_name, receiver_type)
            .unwrap_or_else(|| panic!("Unknown function name {method_name}"));

        let name = snippet.entrypoint();

        let mut args: Vec<ast_types::AbstractArgument> = vec![];
        for (ty, name) in snippet.inputs().into_iter() {
            let fn_arg = ast_types::AbstractValueArg {
                name,
                data_type: ast_types::DataType::from_tasm_lib_datatype(ty, self.list_type),
                mutable: true,
            };
            args.push(ast_types::AbstractArgument::ValueArgument(fn_arg));
        }

        let mut output_types: Vec<ast_types::DataType> = vec![];
        for (ty, _name) in snippet.outputs() {
            output_types.push(ast_types::DataType::from_tasm_lib_datatype(
                ty,
                self.list_type,
            ));
        }

        let output = match output_types.len() {
            1 => output_types[0].clone(),
            0 => ast_types::DataType::Tuple(vec![].into()),
            _ => ast_types::DataType::Tuple(output_types.into()),
        };

        ast::FnSignature {
            name,
            args,
            output,
            arg_evaluation_order: Default::default(),
        }
    }

    fn function_name_to_signature(
        &self,
        _fn_name: &str,
        _type_parameter: Option<ast_types::DataType>,
        _args: &[ast::Expr<super::Annotation>],
    ) -> ast::FnSignature {
        panic!("unsigned_integers lib does not contain any functions");
    }

    fn call_method(
        &self,
        method_name: &str,
        receiver_type: &ast_types::DataType,
        _args: &[ast::Expr<super::Annotation>],
        state: &mut CompilerState,
    ) -> Vec<triton_vm::instruction::LabelledInstruction> {
        if method_name == COUNT_ONES_METHOD && ast_types::DataType::U32 == *receiver_type {
            return get_count_ones_u32_method().body;
        }

        let snippet = name_to_tasm_lib_snippet(method_name, receiver_type)
            .unwrap_or_else(|| panic!("Unknown function name {method_name}"));
        let entrypoint = snippet.entrypoint();
        state.import_snippet(snippet);

        triton_asm!(call { entrypoint })
    }

    fn call_function(
        &self,
        _fn_name: &str,
        _type_parameter: Option<ast_types::DataType>,
        _args: &[ast::Expr<super::Annotation>],
        _state: &mut CompilerState,
    ) -> Vec<triton_vm::instruction::LabelledInstruction> {
        panic!("unsigned_integers lib does not contain any functions");
    }

    fn get_graft_function_name(&self, _full_name: &str) -> Option<String> {
        None
    }

    fn graft_function(
        &self,
        _graft_config: &mut Graft,
        _fn_name: &str,
        _args: &syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>,
        _function_type_parameter: Option<ast_types::DataType>,
    ) -> Option<ast::Expr<super::Annotation>> {
        panic!("unsigned_integers lib cannot graft");
    }

    fn graft_method(
        &self,
        _graft_config: &mut Graft,
        _rust_method_call: &syn::ExprMethodCall,
    ) -> Option<ast::Expr<super::Annotation>> {
        None
    }
}

fn get_count_ones_u32_method() -> LibraryFunction {
    let fn_signature = ast::FnSignature {
        name: COUNT_ONES_METHOD.to_owned(),
        args: vec![ast_types::AbstractArgument::ValueArgument(
            ast_types::AbstractValueArg {
                name: "value".to_owned(),
                data_type: ast_types::DataType::U32,
                mutable: false,
            },
        )],
        output: ast_types::DataType::U32,
        arg_evaluation_order: Default::default(),
    };

    LibraryFunction {
        signature: fn_signature,
        body: triton_asm!(pop_count),
    }
}

/// Map list-function or method name to the TASM lib snippet type
fn name_to_tasm_lib_snippet(
    public_name: &str,
    receiver_type: &ast_types::DataType,
) -> Option<Box<dyn BasicSnippet>> {
    match public_name {
        LEADING_ZEROS_METHOD => match receiver_type {
            ast_types::DataType::U32 => Some(Box::new(
                tasm_lib::arithmetic::u32::leadingzeros::Leadingzeros,
            )),
            ast_types::DataType::U64 => Some(Box::new(
                tasm_lib::arithmetic::u64::leading_zeros_u64::LeadingZerosU64,
            )),
            _ => panic!("Dont know `{public_name}` for {receiver_type}"),
        },
        COUNT_ONES_METHOD => match receiver_type {
            ast_types::DataType::U64 => Some(Box::new(
                tasm_lib::arithmetic::u64::popcount_u64::PopCountU64,
            )),
            _ => panic!("Dont know `{public_name}` for {receiver_type}"),
        },
        POW_METHOD => match receiver_type {
            ast_types::DataType::U32 => Some(Box::new(tasm_lib::arithmetic::u32::safepow::Safepow)),
            _ => panic!("Dont know `{public_name}` for {receiver_type}"),
        },
        OVERFLOWING_ADD_METHOD => match receiver_type {
            ast_types::DataType::U64 => Some(Box::new(
                tasm_lib::arithmetic::u64::overflowing_add_u64::OverflowingAdd,
            )),
            _ => panic!("Dont know `{public_name}` for {receiver_type}"),
        },
        OVERFLOWING_SUB_METHOD => match receiver_type {
            ast_types::DataType::U64 => Some(Box::new(
                tasm_lib::arithmetic::u64::overflowing_sub_u64::OverflowingSub,
            )),
            _ => panic!("Dont know `{public_name}` for {receiver_type}"),
        },
        WRAPPING_SUB_METHOD => match receiver_type {
            ast_types::DataType::U64 => Some(Box::new(
                tasm_lib::arithmetic::u64::wrapping_sub_u64::WrappingSub,
            )),
            _ => panic!("Dont know `{public_name}` for {receiver_type}"),
        },
        _ => None,
    }
}
