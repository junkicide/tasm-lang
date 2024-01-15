#![allow(clippy::manual_swap)]

use num::One;
use triton_vm::twenty_first::shared_math::traits::ModPowU32;
use triton_vm::twenty_first::shared_math::x_field_element::XFieldElement;
use triton_vm::BFieldElement;

#[allow(clippy::ptr_arg)]
#[allow(clippy::vec_init_then_push)]
#[allow(dead_code)]
fn xfe_ntt(x: &mut Vec<XFieldElement>, omega: BFieldElement) {
    fn bitreverse(mut n: u32, l: u32) -> u32 {
        let mut r: u32 = 0;
        let mut i: u32 = 0;
        while i < l {
            r = (r << 1) | (n & 1);
            n >>= 1;
            i += 1;
        }

        return r;
    }

    let size: u32 = x.len() as u32;
    let log_2_size: u32 = u32::BITS - size.leading_zeros() - 1;

    {
        let mut k: u32 = 0;
        while k != size {
            let rk: u32 = bitreverse(k, log_2_size);
            if k < rk {
                // TODO: Use `swap` here instead, once it's implemented in `tasm-lib`
                // That will give us a shorter cycle count
                // x.swap(rk as usize, k as usize);
                let rk_val: XFieldElement = x[rk as usize];
                x[rk as usize] = x[k as usize];
                x[k as usize] = rk_val;
            }

            k += 1;
        }
    }

    let mut m: u32 = 1;

    let mut outer_count: u32 = 0;
    while outer_count != log_2_size {
        // for _ in 0..log_2_of_n {
        let w_m: BFieldElement = omega.mod_pow_u32(size / (2 * m));
        let mut k: u32 = 0;
        while k < size {
            let mut w: BFieldElement = BFieldElement::one();
            let mut j: u32 = 0;
            while j != m {
                // for j in 0..m {
                let u: XFieldElement = x[(k + j) as usize];
                let mut v: XFieldElement = x[(k + j + m) as usize];
                v *= w;
                x[(k + j) as usize] = u + v;
                x[(k + j + m) as usize] = u - v;
                w *= w_m;

                j += 1;
            }

            k += 2 * m;
        }

        m *= 2;

        outer_count += 1;
    }

    return;
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use triton_vm::BFieldElement;
    use triton_vm::NonDeterminism;

    use crate::tests_and_benchmarks::test_helpers::shared_test::bfe_lit;
    use crate::tests_and_benchmarks::test_helpers::shared_test::compare_compiled_prop_with_stack_and_memory_and_ins;

    #[test]
    fn fast_xfe_ntt_to_basic_snippet_test() {
        // This first test is only added to illustrate that it is possible to test these OZK functions
        // before generating a `BasicSnippet` implementation from it.
        let compiled = crate::tests_and_benchmarks::ozk::ozk_parsing::compile_for_test(
            "recufier",
            "fast_ntt_to_basic_snippet",
            "xfe_ntt",
            crate::ast_types::ListType::Unsafe,
        );
        let list_pointer = BFieldElement::new(100);
        let list_length = BFieldElement::new(1);
        let root_of_unity = BFieldElement::new(1);
        let init_memory: HashMap<BFieldElement, BFieldElement> = [
            (list_pointer, list_length),
            (list_pointer + BFieldElement::new(1), BFieldElement::new(50)),
            (list_pointer + BFieldElement::new(2), BFieldElement::new(0)),
            (list_pointer + BFieldElement::new(3), BFieldElement::new(0)),
        ]
        .iter()
        .cloned()
        .collect();
        compare_compiled_prop_with_stack_and_memory_and_ins(
            &compiled,
            "xfe_ntt",
            vec![bfe_lit(list_pointer), bfe_lit(root_of_unity)],
            vec![],
            init_memory,
            None,
            vec![],
            NonDeterminism::new(vec![]),
        );

        // Output what we came for: A `BasicSnippet` implementation constructed by the compiler
        let (rust_ast, _, _) =
            crate::tests_and_benchmarks::ozk::ozk_parsing::parse_function_and_structs(
                "recufier",
                "fast_ntt_to_basic_snippet",
                "xfe_ntt",
            );
        let as_bs = crate::tests_and_benchmarks::ozk::ozk_parsing::compile_to_basic_snippet(
            rust_ast,
            HashMap::default(),
            crate::ast_types::ListType::Unsafe,
        );
        println!("{}", as_bs);
    }
}
