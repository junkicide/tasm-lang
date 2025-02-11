use num::One;
use triton_vm::twenty_first::shared_math::x_field_element::XFieldElement;

use crate::tests_and_benchmarks::ozk::rust_shadows as tasm;

fn main() {
    let xfe: XFieldElement = tasm::tasm_io_read_stdin___xfe();
    let good_xfe: Result<XFieldElement, ()> = Ok(xfe);

    match good_xfe {
        Ok(xfe_again) => {
            assert!(xfe == xfe_again);
            tasm::tasm_io_write_to_stdout___xfe(xfe_again);
        }
        Err(_) => {
            panic!();
        }
    };
    match good_xfe {
        Ok(xfe_again) => {
            assert!(xfe == xfe_again);
            tasm::tasm_io_write_to_stdout___xfe(xfe_again);
        }
        _ => {
            panic!();
        }
    };

    let bad_xfe: Result<XFieldElement, ()> = Err(());
    match bad_xfe {
        Ok(_xfe) => {
            panic!();
        }
        Err(_) => {
            tasm::tasm_io_write_to_stdout___xfe(xfe + XFieldElement::one());
        }
    };

    match bad_xfe {
        Ok(_xfe) => {
            panic!();
        }
        _ => {
            tasm::tasm_io_write_to_stdout___xfe(xfe + XFieldElement::one());
        }
    };

    return;
}

mod test {

    use std::default::Default;

    use rand::random;
    use triton_vm::NonDeterminism;

    use crate::tests_and_benchmarks::ozk::ozk_parsing;
    use crate::tests_and_benchmarks::ozk::ozk_parsing::EntrypointLocation;
    use crate::tests_and_benchmarks::ozk::rust_shadows;
    use crate::tests_and_benchmarks::test_helpers::shared_test::*;

    use super::*;

    #[test]
    fn prelude_match_test() {
        let stdin = vec![random(), random(), random()];
        let non_determinism = NonDeterminism::default();
        let native_output =
            rust_shadows::wrap_main_with_io(&main)(stdin.clone(), non_determinism.clone());
        let entrypoint_location = EntrypointLocation::disk("result_types", "prelude_match", "main");
        let test_program =
            ozk_parsing::compile_for_test(&entrypoint_location, crate::ast_types::ListType::Unsafe);
        let vm_output = execute_compiled_with_stack_and_ins_for_test(
            &test_program,
            vec![],
            stdin,
            NonDeterminism::default(),
            0,
        )
        .unwrap();
        assert_eq!(native_output, vm_output.output);
    }
}
