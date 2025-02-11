use triton_vm::Digest;

use crate::tests_and_benchmarks::ozk::rust_shadows as tasm;

fn main() {
    let a: Digest = tasm::tasm_io_read_stdin___digest();
    let boxed_a: Box<Digest> = { Box::<Digest>::new(a) };

    let b: Digest = tasm::tasm_io_read_stdin___digest();
    let boxed_b: Box<Digest> = { Box::<Digest>::new(b) };
    let c: Digest = tasm::tasm_io_read_stdin___digest();
    let boxed_c: Box<Digest> = { Box::<Digest>::new(c) };
    let d: Digest = tasm::tasm_io_read_stdin___digest();
    let boxed_d: Box<Digest> = { Box::<Digest>::new(d) };
    let boxed_e: Box<(Digest, Digest)> = Box::<(Digest, Digest)>::new((c, a));

    assert!(a == *boxed_a);
    assert!(b == *boxed_b);
    assert!(c == *boxed_c);
    assert!(d == *boxed_d);
    assert!(a == *boxed_a);

    let e: (Digest, Digest) = *boxed_e;
    assert!(c == e.0);
    assert!(a == e.1);

    tasm::tasm_io_write_to_stdout___digest(a);
    tasm::tasm_io_write_to_stdout___digest(b);
    tasm::tasm_io_write_to_stdout___digest(*boxed_c);
    tasm::tasm_io_write_to_stdout___digest(d);
    tasm::tasm_io_write_to_stdout___digest(*boxed_a);
    tasm::tasm_io_write_to_stdout___digest(*boxed_b);
    tasm::tasm_io_write_to_stdout___digest(c);
    tasm::tasm_io_write_to_stdout___digest(*boxed_d);

    return;
}

#[cfg(test)]
mod test {

    use itertools::Itertools;
    use triton_vm::twenty_first::shared_math::bfield_codec::BFieldCodec;
    use triton_vm::twenty_first::shared_math::other::random_elements;
    use triton_vm::NonDeterminism;

    use crate::tests_and_benchmarks::ozk::ozk_parsing;
    use crate::tests_and_benchmarks::ozk::ozk_parsing::EntrypointLocation;
    use crate::tests_and_benchmarks::ozk::rust_shadows;
    use crate::tests_and_benchmarks::test_helpers::shared_test::*;

    use super::*;

    #[test]
    fn boxed_digest_pair_test() {
        // Test function on host machine
        let rands: Vec<Digest> = random_elements(4);
        let mut std_in = vec![];
        let mut expected_output = vec![];
        for digest in rands {
            let mut elements = digest.encode();
            expected_output.append(&mut elements.clone());
            elements.reverse();
            std_in.append(&mut elements);
        }

        expected_output = [expected_output.clone(), expected_output].concat();
        let non_determinism = NonDeterminism::new(vec![]);
        let native_output =
            rust_shadows::wrap_main_with_io(&main)(std_in.clone(), non_determinism.clone());
        assert_eq!(native_output, expected_output);

        // Test function in Triton VM
        let entrypoint_location = EntrypointLocation::disk("boxed", "digest_pair", "main");
        let test_program =
            ozk_parsing::compile_for_test(&entrypoint_location, crate::ast_types::ListType::Unsafe);
        let expected_stack_diff = 0;
        println!("test_program:\n{}", test_program.iter().join("\n"));
        let vm_output = execute_compiled_with_stack_and_ins_for_test(
            &test_program,
            vec![],
            std_in,
            NonDeterminism::new(vec![]),
            expected_stack_diff,
        )
        .unwrap();
        if expected_output != vm_output.output {
            panic!(
                "expected:\n{}\n\ngot:\n{}",
                expected_output.iter().join(","),
                vm_output.output.iter().join(",")
            );
        }
    }
}
