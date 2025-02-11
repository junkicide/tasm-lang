use crate::tests_and_benchmarks::ozk::rust_shadows as tasm;

#[allow(clippy::needless_else)]
fn main() {
    // https://projecteuler.net/problem=6
    // Let $a = Sum_{i=1}^{100}i**2$, and
    //     $b = (Sum_{i=1}^{100}i)**2$, then the result is
    //     $r = b - a$.

    // We know that $Sum_{i=1}^{N}i = N * (N + 1) / 2$, so let's perform that substitution.
    // then $b = (N * (N + 1) / 2)**2 = N**2*(N + 1)**2 / 4$.
    // The sum of squares in $a$ can be rewritten as:
    // $b = N * (N + 1) * (2N + 1) / 6$

    let n: u32 = 100;
    tasm::tasm_io_write_to_stdout___u32(
        n.pow(2) * (n + 1).pow(2) / 4 - n * (n + 1) * (n * 2 + 1) / 6,
    );

    return;
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use triton_vm::NonDeterminism;

    use crate::tests_and_benchmarks::ozk::ozk_parsing::EntrypointLocation;
    use crate::tests_and_benchmarks::ozk::rust_shadows;
    use crate::tests_and_benchmarks::test_helpers::shared_test::*;

    use super::*;

    #[test]
    fn pe6_test() {
        // Test function on host machine
        let stdin = vec![];
        let non_determinism = NonDeterminism::new(vec![]);
        let native_output =
            rust_shadows::wrap_main_with_io(&main)(stdin.clone(), non_determinism.clone());

        // Test function in Triton VM
        let entrypoint_location = EntrypointLocation::disk("project_euler", "pe6", "main");
        let parsed = entrypoint_location.extract_entrypoint();
        let expected_stack_diff = 0;
        let stack_start = vec![];
        let vm_output =
            execute_with_stack_safe_lists(&parsed, stack_start, expected_stack_diff).unwrap();
        assert_eq!(native_output, vm_output.output);

        println!("vm_output.output: {}", vm_output.output.iter().join(","));
    }
}

mod benches {
    use crate::tests_and_benchmarks::benchmarks::execute_and_write_benchmark;
    use crate::tests_and_benchmarks::benchmarks::profile;
    use crate::tests_and_benchmarks::benchmarks::BenchmarkInput;
    use crate::tests_and_benchmarks::ozk::ozk_parsing::EntrypointLocation;
    use crate::tests_and_benchmarks::test_helpers::shared_test::*;

    #[test]
    fn pe6_bench() {
        let entrypoint_location = EntrypointLocation::disk("project_euler", "pe6", "main");
        let parsed = entrypoint_location.extract_entrypoint();
        let (code, _) = compile_for_run_test(&parsed, crate::ast_types::ListType::Safe);

        let common_case = BenchmarkInput::default();
        let worst_case = BenchmarkInput::default();
        let name = "project_euler_6".to_owned();
        execute_and_write_benchmark(
            name.clone(),
            code.clone(),
            common_case.clone(),
            worst_case,
            0,
        );
        profile(name, code, common_case);
    }
}
