use karnaugh_studio::domain::expression::ExpressionEvaluator;
use karnaugh_studio::domain::minimizer::Minimizer;
use karnaugh_studio::domain::model::{CellValue, VarCount};
use std::collections::HashMap;

fn verify_equivalence(
    var_count: VarCount,
    truth_table_map: &HashMap<usize, CellValue>,
    sop_expr: &str,
    pos_expr: &str,
) {
    let total = var_count.num_rows();
    for m in 0..total {
        let expected = truth_table_map.get(&m).copied().unwrap_or(CellValue::Zero);
        let sop_eval = ExpressionEvaluator::evaluate_sop(sop_expr, var_count, m);
        let pos_eval = ExpressionEvaluator::evaluate_pos(pos_expr, var_count, m);

        match expected {
            CellValue::One => {
                assert!(
                    sop_eval,
                    "SoP failed at minterm {m} for expr '{sop_expr}'. Expected 1, got 0"
                );
                assert!(
                    pos_eval,
                    "PoS failed at minterm {m} for expr '{pos_expr}'. Expected 1, got 0"
                );
            }
            CellValue::Zero => {
                assert!(
                    !sop_eval,
                    "SoP failed at minterm {m} for expr '{sop_expr}'. Expected 0, got 1"
                );
                assert!(
                    !pos_eval,
                    "PoS failed at minterm {m} for expr '{pos_expr}'. Expected 0, got 1"
                );
            }
            CellValue::DontCare => {
                // Don't-care values can be satisfied by either 0 or 1
            }
        }
    }
}

#[test]
fn test_equivalence_half_adder_sum_and_carry() {
    let vc = VarCount::Two;

    // Sum = A XOR B: m1, m2 = 1; m0, m3 = 0
    let mut sum_map = HashMap::new();
    sum_map.insert(0, CellValue::Zero);
    sum_map.insert(1, CellValue::One);
    sum_map.insert(2, CellValue::One);
    sum_map.insert(3, CellValue::Zero);

    let sop = Minimizer::minimize_sop(vc, &sum_map);
    let pos = Minimizer::minimize_pos(vc, &sum_map);
    verify_equivalence(vc, &sum_map, &sop.expression, &pos.expression);

    // Carry = A AND B: m3 = 1; m0, m1, m2 = 0
    let mut carry_map = HashMap::new();
    carry_map.insert(0, CellValue::Zero);
    carry_map.insert(1, CellValue::Zero);
    carry_map.insert(2, CellValue::Zero);
    carry_map.insert(3, CellValue::One);

    let sop_c = Minimizer::minimize_sop(vc, &carry_map);
    let pos_c = Minimizer::minimize_pos(vc, &carry_map);
    verify_equivalence(vc, &carry_map, &sop_c.expression, &pos_c.expression);
}

#[test]
fn test_equivalence_3_variable_majority() {
    // Majority of 3: m3 (011), m5 (101), m6 (110), m7 (111) are 1, others 0
    let vc = VarCount::Three;
    let mut map = HashMap::new();
    for m in 0..8 {
        let ones = (m & 1) + ((m >> 1) & 1) + ((m >> 2) & 1);
        if ones >= 2 {
            map.insert(m, CellValue::One);
        } else {
            map.insert(m, CellValue::Zero);
        }
    }

    let sop = Minimizer::minimize_sop(vc, &map);
    let pos = Minimizer::minimize_pos(vc, &map);
    verify_equivalence(vc, &map, &sop.expression, &pos.expression);
}

#[test]
fn test_equivalence_4_variable_multiplexer() {
    // 2-to-1 Mux with 4 variables:
    // S = A, I0 = B, I1 = C, Enable = D (active high)
    // If D = 0 -> 0
    // If D = 1, S = 0 -> B
    // If D = 1, S = 1 -> C
    let vc = VarCount::Four;
    let mut map = HashMap::new();
    for m in 0..16 {
        let a = (m >> 3) & 1;
        let b = (m >> 2) & 1;
        let c = (m >> 1) & 1;
        let d = m & 1;

        let out = if d == 0 {
            0
        } else if a == 0 {
            b
        } else {
            c
        };

        map.insert(
            m,
            if out == 1 {
                CellValue::One
            } else {
                CellValue::Zero
            },
        );
    }

    let sop = Minimizer::minimize_sop(vc, &map);
    let pos = Minimizer::minimize_pos(vc, &map);
    verify_equivalence(vc, &map, &sop.expression, &pos.expression);
}

#[test]
fn test_equivalence_with_dont_cares_4_variable() {
    // BCD to Excess-3 or invalid BCD digits:
    // Valid BCD: 0..9. Invalid BCD 10..15 are Don't Care (X).
    let vc = VarCount::Four;
    let mut map = HashMap::new();
    for m in 0..10 {
        // Output 1 for odd numbers: 1, 3, 5, 7, 9
        if m % 2 == 1 {
            map.insert(m, CellValue::One);
        } else {
            map.insert(m, CellValue::Zero);
        }
    }
    for m in 10..16 {
        map.insert(m, CellValue::DontCare);
    }

    let sop = Minimizer::minimize_sop(vc, &map);
    let pos = Minimizer::minimize_pos(vc, &map);
    verify_equivalence(vc, &map, &sop.expression, &pos.expression);
}

#[test]
fn test_equivalence_exhaustive_2_variable_functions() {
    // All 16 possible 2-variable Boolean functions!
    let vc = VarCount::Two;
    for func_id in 0..16 {
        let mut map = HashMap::new();
        for m in 0..4 {
            let bit = (func_id >> m) & 1;
            map.insert(
                m,
                if bit == 1 {
                    CellValue::One
                } else {
                    CellValue::Zero
                },
            );
        }

        let sop = Minimizer::minimize_sop(vc, &map);
        let pos = Minimizer::minimize_pos(vc, &map);
        verify_equivalence(vc, &map, &sop.expression, &pos.expression);
    }
}

#[test]
fn test_equivalence_exhaustive_3_variable_functions() {
    // All 256 possible 3-variable Boolean functions (2^8 = 256)
    let vc = VarCount::Three;
    for func_id in 0..256 {
        let mut map = HashMap::new();
        for m in 0..8 {
            let bit = (func_id >> m) & 1;
            map.insert(
                m,
                if bit == 1 {
                    CellValue::One
                } else {
                    CellValue::Zero
                },
            );
        }

        let sop = Minimizer::minimize_sop(vc, &map);
        let pos = Minimizer::minimize_pos(vc, &map);
        verify_equivalence(vc, &map, &sop.expression, &pos.expression);
    }
}

#[test]
fn test_equivalence_4_variable_patterns() {
    let vc = VarCount::Four;

    // Pattern 1: Parity checker (odd parity for 4 variables: XOR sum)
    let mut parity_map = HashMap::new();
    for m in 0..16 {
        let ones = (m as u32).count_ones();
        parity_map.insert(
            m,
            if ones % 2 == 1 {
                CellValue::One
            } else {
                CellValue::Zero
            },
        );
    }
    let sop_p = Minimizer::minimize_sop(vc, &parity_map);
    let pos_p = Minimizer::minimize_pos(vc, &parity_map);
    verify_equivalence(vc, &parity_map, &sop_p.expression, &pos_p.expression);

    // Pattern 2: Prime numbers < 16: 2, 3, 5, 7, 11, 13
    let mut primes_map = HashMap::new();
    let primes = [2, 3, 5, 7, 11, 13];
    for m in 0..16 {
        primes_map.insert(
            m,
            if primes.contains(&m) {
                CellValue::One
            } else {
                CellValue::Zero
            },
        );
    }
    let sop_pr = Minimizer::minimize_sop(vc, &primes_map);
    let pos_pr = Minimizer::minimize_pos(vc, &primes_map);
    verify_equivalence(vc, &primes_map, &sop_pr.expression, &pos_pr.expression);
}
