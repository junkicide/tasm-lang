#![allow(clippy::needless_borrow)]

use triton_vm::BFieldElement;

use crate::tests_and_benchmarks::ozk::rust_shadows as tasm;

#[derive(Clone, Copy, Debug)]
struct DazeFieldElement(u64);

impl DazeFieldElement {
    fn new(value: u64) -> DazeFieldElement {
        return DazeFieldElement(DazeFieldElement::montyred(
            value as u128 * 0xFFFFFFFE00000001u128,
        ));
    }

    fn montyred(x: u128) -> u64 {
        let xl: u64 = x as u64;
        let xh: u64 = (x >> 64) as u64;
        let add_res: (u64, bool) = xl.overflowing_add(xl << 32);

        let b: u64 = add_res
            .0
            .wrapping_sub(add_res.0 >> 32)
            .wrapping_sub(add_res.1 as u64);

        let sub_res: (u64, bool) = xh.overflowing_sub(b);

        return sub_res
            .0
            .wrapping_sub((1 + !0xffff_ffff_0000_0001u64) * sub_res.1 as u64);
    }

    fn canonical_representation(self) -> u64 {
        return DazeFieldElement::montyred(self.0 as u128);
    }

    fn valued(self) -> u64 {
        return self.canonical_representation();
    }

    fn mul(self, rhs: DazeFieldElement) -> DazeFieldElement {
        return DazeFieldElement(DazeFieldElement::montyred(
            tasm::tasm_arithmetic_u64_mul_two_u64s_to_u128_u64(self.0, rhs.0),
        ));
    }
}

fn main() {
    let a: DazeFieldElement = DazeFieldElement::new(tasm::tasm_io_read_stdin___bfe().value());
    let b: DazeFieldElement = DazeFieldElement::new(tasm::tasm_io_read_stdin___bfe().value());
    let res: DazeFieldElement = a.mul(b);
    tasm::tasm_io_write_to_stdout___bfe(BFieldElement::new(res.valued()));
    tasm::tasm_io_write_to_stdout___u64(res.valued());

    return;
}

#[cfg(test)]
mod test {

    use itertools::Itertools;
    use rand::random;
    use triton_vm::twenty_first::shared_math::bfield_codec::BFieldCodec;
    use triton_vm::BFieldElement;
    use triton_vm::NonDeterminism;

    use crate::tests_and_benchmarks::ozk::ozk_parsing;
    use crate::tests_and_benchmarks::ozk::rust_shadows;
    use crate::tests_and_benchmarks::test_helpers::shared_test::*;

    use super::*;

    #[test]
    fn dazefield_element_test() {
        // Test function on host machine
        let non_determinism = NonDeterminism::new(vec![]);

        for _ in 0..4 {
            let a: BFieldElement = random();
            let b: BFieldElement = random();
            let res = a * b;
            let stdin: Vec<BFieldElement> = vec![a, b];
            let expected_output = [vec![res], res.value().encode()].concat();
            let native_output =
                rust_shadows::wrap_main_with_io(&main)(stdin.clone(), non_determinism.clone());
            assert_eq!(native_output, expected_output);

            // Test function in Triton VM
            let entrypoint_location = ozk_parsing::EntrypointLocation::disk(
                "arithmetic",
                "dazefield_element_mul",
                "main",
            );
            let test_program = ozk_parsing::compile_for_test(
                &entrypoint_location,
                crate::ast_types::ListType::Unsafe,
            );
            let expected_stack_diff = 0;
            let vm_output = execute_compiled_with_stack_and_ins_for_test(
                &test_program,
                vec![],
                stdin,
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
}

mod benches {
    use triton_vm::BFieldElement;

    use crate::tests_and_benchmarks::benchmarks::execute_and_write_benchmark;
    use crate::tests_and_benchmarks::benchmarks::profile;
    use crate::tests_and_benchmarks::benchmarks::BenchmarkInput;
    use crate::tests_and_benchmarks::ozk::ozk_parsing;

    #[test]
    fn dazefield_element_bench() {
        let worst_case_input = BenchmarkInput {
            std_in: vec![
                BFieldElement::new(1u64 << 40),
                BFieldElement::new(1u64 << 40),
            ],
            ..Default::default()
        };
        let common_case = BenchmarkInput {
            std_in: vec![
                BFieldElement::new(BFieldElement::MAX),
                BFieldElement::new(BFieldElement::MAX),
            ],
            ..Default::default()
        };

        let entrypoint_location =
            ozk_parsing::EntrypointLocation::disk("arithmetic", "dazefield_element_mul", "main");
        let code =
            ozk_parsing::compile_for_test(&entrypoint_location, crate::ast_types::ListType::Unsafe);

        let name = "dazefield_element_mul".to_owned();
        execute_and_write_benchmark(
            name.clone(),
            code.clone(),
            common_case.clone(),
            worst_case_input,
            0,
        );
        profile(name, code, common_case);
    }
}
