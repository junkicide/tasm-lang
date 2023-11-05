use crate::tests_and_benchmarks::ozk::rust_shadows as tasm;
use twenty_first::shared_math::x_field_element::XFieldElement;

pub enum SimpleEnum {
    A(XFieldElement),
    B,
}

fn main() {
    let a: SimpleEnum = SimpleEnum::A(tasm::tasm_io_read_stdin___xfe());
    let b: SimpleEnum = SimpleEnum::B;

    match b {
        SimpleEnum::A(xfe) => {
            assert!(false);
        }
        SimpleEnum::B => {
            assert!(true);
        }
    };

    match a {
        SimpleEnum::A(xfe) => {
            assert!(true);
            tasm::tasm_io_write_to_stdout___xfe(xfe);
        }
        SimpleEnum::B => {
            assert!(false);
        }
    };

    return;
}

mod tests {
    use super::*;
    use itertools::Itertools;
    use rand::random;
    use std::collections::HashMap;
    use triton_vm::NonDeterminism;
    use twenty_first::shared_math::bfield_codec::BFieldCodec;

    use crate::tests_and_benchmarks::ozk::{ozk_parsing, rust_shadows};
    use crate::tests_and_benchmarks::test_helpers::shared_test::execute_compiled_with_stack_memory_and_ins_for_test;

    #[test]
    fn two_variants_one_data_test() {
        let non_determinism = NonDeterminism::default();
        let random_xfe: XFieldElement = random();
        let mut std_in = random_xfe.encode();
        std_in.reverse();
        let expected_output = random_xfe.encode();
        // let expected_output = vec![];

        // Run test on host machine
        let native_output =
            rust_shadows::wrap_main_with_io(&main)(std_in.clone(), non_determinism.clone());
        assert_eq!(native_output, expected_output);

        // Run test on Triton-VM
        let test_program = ozk_parsing::compile_for_test(
            "enums",
            "two_variants_one_data",
            "main",
            crate::ast_types::ListType::Unsafe,
        );
        println!("executing:\n{}", test_program.iter().join("\n"));
        let vm_output = execute_compiled_with_stack_memory_and_ins_for_test(
            &test_program,
            vec![],
            &mut HashMap::default(),
            std_in,
            non_determinism,
            0,
        )
        .unwrap();
        // assert_eq!(expected_output, vm_output.output);
        if expected_output != vm_output.output {
            panic!(
                "expected_output:\n {}, got:\n{}. Code was:\n{}",
                expected_output.iter().join(", "),
                vm_output.output.iter().join(", "),
                test_program.iter().join("\n")
            );
        }
    }
}
