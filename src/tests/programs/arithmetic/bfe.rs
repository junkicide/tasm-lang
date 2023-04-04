use syn::parse_quote;

use crate::graft::item_fn;

#[allow(dead_code)]
fn instantiate_bfe_with_literal() -> syn::ItemFn {
    item_fn(parse_quote! {
            fn instantiate_bfe() -> (BFieldElement, BFieldElement) {
                let a: BFieldElement = BFieldElement::new(400u64);
                return (a, BFieldElement::new(500u64));
            }
    })
}

#[allow(dead_code)]
fn add_bfe_rast() -> syn::ItemFn {
    item_fn(parse_quote! {
        fn add_bfe(lhs: BFieldElement, rhs: BFieldElement) -> BFieldElement {
            let c: BFieldElement = lhs + rhs;
            return c;
        }
    })
}

#[allow(dead_code)]
fn mul_bfe_rast() -> syn::ItemFn {
    item_fn(parse_quote! {
        fn add_bfe(lhs: BFieldElement, rhs: BFieldElement) -> BFieldElement {
            return lhs * rhs;
        }
    })
}

#[allow(dead_code)]
pub fn div_bfe_rast() -> syn::ItemFn {
    item_fn(parse_quote! {
        fn div_bfe(numerator: BFieldElement, divisor: BFieldElement) ->  BFieldElement {
            return numerator / divisor;
        }
    })
}

#[allow(dead_code)]
fn cast_from_bool_rast() -> syn::ItemFn {
    item_fn(parse_quote! {
        fn cast_from_bool(input: bool) -> BFieldElement {
            return input as BFieldElement;
        }
    })
}

#[allow(dead_code)]
fn long_bfe_expr_rast() -> syn::ItemFn {
    item_fn(parse_quote! {
        fn long_bfe_expr(
            input_0: BFieldElement,
            input_1: BFieldElement,
            input_2: BFieldElement,
            input_3: BFieldElement,
            input_4: BFieldElement,
            input_5: BFieldElement,
        ) -> (BFieldElement, BFieldElement, BFieldElement) {
            let val0: u64 = 0;
            let val1: BFieldElement = BFieldElement::new(0u64);
            let val2: BFieldElement = BFieldElement::new(0u64);
            let mut res0: BFieldElement = input_0 * BFieldElement::new(2u64)
                + (input_1 * BFieldElement::new(2u64)
                    + (input_2 * BFieldElement::new(2u64)
                        + (input_3 * BFieldElement::new(2u64)
                            + (input_4 * BFieldElement::new(2u64) + input_5 * BFieldElement::new(2u64)))))
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + BFieldElement::new(2u64) * input_1
                + (true || false) as BFieldElement;
            let res1: BFieldElement = res0;
            let val3: BFieldElement = BFieldElement::new(0u64);
            let val4: BFieldElement = BFieldElement::new(0u64);
            let val5: BFieldElement = BFieldElement::new(0u64);
            let val6: BFieldElement = BFieldElement::new(0u64);
            let val7: BFieldElement = BFieldElement::new(0u64);
            let val8: BFieldElement = BFieldElement::new(0u64);
            let val9: BFieldElement = BFieldElement::new(0u64);
            let val10: BFieldElement = BFieldElement::new(0u64);
            let val11: BFieldElement = BFieldElement::new(0u64);
            let val12: BFieldElement = BFieldElement::new(0u64);
            let val13: BFieldElement = BFieldElement::new(0u64);
            let val14: BFieldElement = BFieldElement::new(0u64);
            let val15: BFieldElement = BFieldElement::new(0u64);
            let val16: BFieldElement = BFieldElement::new(0u64);
            let val17: BFieldElement = BFieldElement::new(0u64);
            let val18: BFieldElement = BFieldElement::new(0u64);
            res0 = res0 + input_0 + input_1 + input_2 + input_3 + input_4 + input_5 + res0;
            res0 = res0 * BFieldElement::new(2u64)
                + input_0 * BFieldElement::new(2u64)
                + ((input_1 * BFieldElement::new(2u64) + input_2 * BFieldElement::new(2u64)))
                + (input_3 * BFieldElement::new(2u64) + input_4 * BFieldElement::new(2u64) + input_5 * BFieldElement::new(2u64))
                    * BFieldElement::new(2u64)
                + input_0
                + input_1
                + input_2;
            let res2: BFieldElement = res0 * res0 + 2;

            return (res0, res1, res2);
        }
    })
}

#[cfg(test)]
mod compile_and_typecheck_tests {
    use super::*;
    use crate::tests::shared_test::graft_check_compile_prop;

    #[test]
    fn add_bfe_test() {
        graft_check_compile_prop(&add_bfe_rast());
    }
}

#[cfg(test)]
mod run_tests {
    use twenty_first::shared_math::b_field_element::BFieldElement;

    use super::*;
    use crate::tests::shared_test::*;

    #[test]
    fn instantiate_bfe_test() {
        compare_prop_with_stack(
            &instantiate_bfe_with_literal(),
            vec![],
            vec![bfe_lit(400u64.into()), bfe_lit(500u64.into())],
        );
    }

    #[test]
    fn add_bfe_test() {
        compare_prop_with_stack(
            &add_bfe_rast(),
            vec![bfe_lit(1000u64.into()), bfe_lit(BFieldElement::MAX.into())],
            vec![bfe_lit(999u64.into())],
        );
    }

    #[test]
    fn mul_bfe_test() {
        compare_prop_with_stack(
            &mul_bfe_rast(),
            vec![
                bfe_lit(10_000_000_000u64.into()),
                bfe_lit(10_000_000_000u64.into()),
            ],
            vec![bfe_lit(7766279652927078395u64.into())],
        );
    }

    #[test]
    fn div_bfe_test() {
        compare_prop_with_stack(
            &div_bfe_rast(),
            vec![bfe_lit(1u64.into()), bfe_lit(8561862112314395584u64.into())],
            vec![bfe_lit(17307602810081694772u64.into())],
        );
        compare_prop_with_stack(
            &div_bfe_rast(),
            vec![
                bfe_lit(1u64.into()),
                bfe_lit(17307602810081694772u64.into()),
            ],
            vec![bfe_lit(8561862112314395584u64.into())],
        );
        compare_prop_with_stack(
            &div_bfe_rast(),
            vec![bfe_lit(1u64.into()), bfe_lit(7u64.into())],
            vec![bfe_lit(2635249152773512046u64.into())],
        );
    }

    #[test]
    fn cast_from_bool_test() {
        compare_prop_with_stack(
            &cast_from_bool_rast(),
            vec![bool_lit(false)],
            vec![bfe_lit(0u64.into())],
        );
        compare_prop_with_stack(
            &cast_from_bool_rast(),
            vec![bool_lit(true)],
            vec![bfe_lit(1u64.into())],
        );
    }

    #[test]
    fn long_bfe_expr_test() {
        compare_prop_with_stack(
            &long_bfe_expr_rast(),
            vec![
                bfe_lit(3u64.into()),
                bfe_lit(4u64.into()),
                bfe_lit(5u64.into()),
                bfe_lit(6u64.into()),
                bfe_lit(7u64.into()),
                bfe_lit(8u64.into()),
            ],
            vec![
                bfe_lit(902u64.into()),
                bfe_lit(179u64.into()),
                bfe_lit(813606u64.into()),
            ],
        );
    }
}
