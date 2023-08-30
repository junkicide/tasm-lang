// Allows the use of input/output on the native architecture
use crate::tests_and_benchmarks::ozk::rust_shadows as tasm;
use triton_vm::BFieldElement;
use twenty_first::shared_math::x_field_element::XFieldElement;

fn main() {
    let a: (BFieldElement, (XFieldElement, XFieldElement)) = (
        BFieldElement::new(300u64),
        (
            XFieldElement::new([
                BFieldElement::new(101),
                BFieldElement::new(102),
                BFieldElement::new(103),
            ]),
            XFieldElement::new([
                BFieldElement::new(401),
                BFieldElement::new(402),
                BFieldElement::new(403),
            ]),
        ),
    );

    tasm::tasm_io_write_to_stdout_xfe(a.1 .1);
    tasm::tasm_io_write_to_stdout_bfe(a.0);
    tasm::tasm_io_write_to_stdout_xfe(a.1 .0);
    tasm::tasm_io_write_to_stdout_xfe(a.1 .0);
    tasm::tasm_io_write_to_stdout_bfe(a.0);

    return;
}

mod tests {
    use super::*;
    use itertools::Itertools;
    use std::collections::HashMap;
    use triton_vm::{BFieldElement, NonDeterminism};
    use twenty_first::shared_math::bfield_codec::BFieldCodec;

    use crate::tests_and_benchmarks::ozk::{ozk_parsing, rust_shadows};
    use crate::tests_and_benchmarks::test_helpers::shared_test::execute_compiled_with_stack_memory_and_ins_for_test;

    #[test]
    fn nested_tuples_test() {
        let non_determinism = NonDeterminism::new(vec![]);
        let a = (
            BFieldElement::new(300),
            (
                XFieldElement::new(
                    (101u64..104)
                        .map(|x| x.into())
                        .collect_vec()
                        .try_into()
                        .unwrap(),
                ),
                XFieldElement::new(
                    (401u64..404)
                        .map(|x| x.into())
                        .collect_vec()
                        .try_into()
                        .unwrap(),
                ),
            ),
        );

        let expected_output = vec![
            a.1 .1.encode(),
            a.0.encode(),
            a.1 .0.encode(),
            a.1 .0.encode(),
            a.0.encode(),
        ]
        .concat();
        let input = vec![];

        // Run test on host machine
        let native_output =
            rust_shadows::wrap_main_with_io(&main)(input.clone(), non_determinism.clone());
        assert_eq!(native_output, expected_output);

        // Run test on Triton-VM
        let test_program = ozk_parsing::compile_for_test("nested_tuples");
        println!("test_program is:\n{}", test_program.iter().join("\n"));
        let vm_output = execute_compiled_with_stack_memory_and_ins_for_test(
            &test_program,
            vec![],
            &mut HashMap::default(),
            input,
            non_determinism,
            0,
        )
        .unwrap();
        assert_eq!(expected_output, vm_output.output);
    }
}
