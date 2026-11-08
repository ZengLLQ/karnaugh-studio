use karnaugh_studio::domain::minimizer::Minimizer;
use karnaugh_studio::domain::model::{CellValue, RawRow, TruthTable, VarCount};
use karnaugh_studio::domain::validation::{RowError, TableValidation};
use std::collections::HashMap;

#[test]
fn test_truth_table_validation_valid_canonical() {
    let table = TruthTable::new(VarCount::Three);
    let val = TableValidation::validate(&table);
    assert!(val.is_valid);
    assert!(val.row_errors.is_empty());
    assert!(val.missing_minterms.is_empty());
    assert_eq!(val.minterm_map.len(), 8);
}

#[test]
fn test_truth_table_validation_unordered_valid_rows() {
    let mut table = TruthTable::new(VarCount::Two);
    // Rows out of canonical order: m3, m0, m2, m1
    table.rows = vec![
        RawRow::new(0, vec!["1".into(), "1".into()], "1".into()), // m3
        RawRow::new(1, vec!["0".into(), "0".into()], "0".into()), // m0
        RawRow::new(2, vec!["1".into(), "0".into()], "1".into()), // m2
        RawRow::new(3, vec!["0".into(), "1".into()], "X".into()), // m1
    ];

    let val = TableValidation::validate(&table);
    assert!(
        val.is_valid,
        "Unordered rows with all minterms must be valid"
    );
    assert_eq!(val.minterm_map.get(&3).unwrap().1, CellValue::One);
    assert_eq!(val.minterm_map.get(&0).unwrap().1, CellValue::Zero);
    assert_eq!(val.minterm_map.get(&2).unwrap().1, CellValue::One);
    assert_eq!(val.minterm_map.get(&1).unwrap().1, CellValue::DontCare);
}

#[test]
fn test_truth_table_validation_duplicates_and_omissions() {
    let mut table = TruthTable::new(VarCount::Two);
    table.rows = vec![
        RawRow::new(0, vec!["0".into(), "0".into()], "0".into()), // m0
        RawRow::new(1, vec!["0".into(), "0".into()], "1".into()), // duplicate m0!
        RawRow::new(2, vec!["1".into(), "0".into()], "0".into()), // m2
        RawRow::new(3, vec!["1".into(), "1".into()], "1".into()), // m3
    ];

    let val = TableValidation::validate(&table);
    assert!(!val.is_valid);
    assert_eq!(val.duplicate_minterm_count, 1);
    // m1 was omitted
    assert_eq!(val.missing_minterms, vec![1]);
    let row1_errors = val
        .get_row_errors(1)
        .expect("Row 1 must have duplicate error");
    assert!(
        row1_errors
            .iter()
            .any(|e| matches!(e, RowError::DuplicateRow { duplicate_of: 0 }))
    );
}

#[test]
fn test_truth_table_validation_invalid_input_and_output() {
    let mut table = TruthTable::new(VarCount::Two);
    table.rows = vec![
        RawRow::new(0, vec!["2".into(), "0".into()], "0".into()), // invalid input
        RawRow::new(1, vec!["0".into(), "1".into()], "Z".into()), // invalid output
        RawRow::new(2, vec!["".into(), "0".into()], "1".into()),  // missing input
        RawRow::new(3, vec!["1".into(), "1".into()], "".into()),  // missing output
    ];

    let val = TableValidation::validate(&table);
    assert!(!val.is_valid);
    assert!(val.get_row_errors(0).is_some());
    assert!(val.get_row_errors(1).is_some());
    assert!(val.get_row_errors(2).is_some());
    assert!(val.get_row_errors(3).is_some());
}

#[test]
fn test_four_corners_torus_wrap_minimization() {
    // 4-variable K-map four corners: m0 (0000), m2 (0010), m8 (1000), m10 (1010)
    // Fixed variables: B=0, D=0. A and C eliminated.
    // Result SoP: B'D'
    let mut map = HashMap::new();
    for m in 0..16 {
        map.insert(m, CellValue::Zero);
    }
    map.insert(0, CellValue::One);
    map.insert(2, CellValue::One);
    map.insert(8, CellValue::One);
    map.insert(10, CellValue::One);

    let res = Minimizer::minimize_sop(VarCount::Four, &map);
    assert_eq!(res.groups.len(), 1);
    let g = &res.groups[0];
    assert_eq!(g.size(), 4);
    assert!(g.is_wrap);
    assert!(g.is_essential);
    assert_eq!(res.expression, "B'D'");

    // For PoS, test with 0s at corners and 1s everywhere else:
    let mut map_pos = HashMap::new();
    for m in 0..16 {
        map_pos.insert(m, CellValue::One);
    }
    map_pos.insert(0, CellValue::Zero);
    map_pos.insert(2, CellValue::Zero);
    map_pos.insert(8, CellValue::Zero);
    map_pos.insert(10, CellValue::Zero);

    let res_pos = Minimizer::minimize_pos(VarCount::Four, &map_pos);
    assert_eq!(res_pos.groups.len(), 1);
    // B=0, D=0 -> In PoS: (B + D)
    assert_eq!(res_pos.expression, "(B + D)");
}

#[test]
fn test_special_case_all_zero_and_all_one() {
    let mut all_zeros = HashMap::new();
    for m in 0..8 {
        all_zeros.insert(m, CellValue::Zero);
    }

    let sop_zero = Minimizer::minimize_sop(VarCount::Three, &all_zeros);
    assert_eq!(sop_zero.expression, "0");
    assert_eq!(sop_zero.groups.len(), 0);

    let pos_zero = Minimizer::minimize_pos(VarCount::Three, &all_zeros);
    assert_eq!(pos_zero.expression, "0");
    assert_eq!(pos_zero.groups.len(), 1);
    assert_eq!(pos_zero.groups[0].size(), 8);
    assert!(pos_zero.groups[0].is_essential);

    let mut all_ones = HashMap::new();
    for m in 0..8 {
        all_ones.insert(m, CellValue::One);
    }

    let sop_one = Minimizer::minimize_sop(VarCount::Three, &all_ones);
    assert_eq!(sop_one.expression, "1");
    assert_eq!(sop_one.groups.len(), 1);
    assert_eq!(sop_one.groups[0].size(), 8);
    assert!(sop_one.groups[0].is_essential);

    let pos_one = Minimizer::minimize_pos(VarCount::Three, &all_ones);
    assert_eq!(pos_one.expression, "1");
    assert_eq!(pos_one.groups.len(), 0);
}

#[test]
fn test_all_dont_care_special_case() {
    let mut all_dc = HashMap::new();
    for m in 0..16 {
        all_dc.insert(m, CellValue::DontCare);
    }

    let sop = Minimizer::minimize_sop(VarCount::Four, &all_dc);
    assert_eq!(sop.expression, "0");
    assert_eq!(sop.groups.len(), 0);

    let pos = Minimizer::minimize_pos(VarCount::Four, &all_dc);
    assert_eq!(pos.expression, "1");
    assert_eq!(pos.groups.len(), 0);
}

#[test]
fn test_single_minterm() {
    let mut map = HashMap::new();
    for m in 0..4 {
        map.insert(m, CellValue::Zero);
    }
    map.insert(3, CellValue::One); // A=1, B=1

    let res = Minimizer::minimize_sop(VarCount::Two, &map);
    assert_eq!(res.expression, "AB");
    assert_eq!(res.groups.len(), 1);
    assert_eq!(res.groups[0].size(), 1);
}

#[test]
fn test_dont_care_expansion() {
    // 3 variables: m1 (001), m3 (011) are 1; m5 (101), m7 (111) are X
    // Without X: group {1, 3} gives A'C (size 2)
    // With X: group {1, 3, 5, 7} expands to size 4, eliminating A and B, giving C!
    let mut map = HashMap::new();
    for m in 0..8 {
        map.insert(m, CellValue::Zero);
    }
    map.insert(1, CellValue::One);
    map.insert(3, CellValue::One);
    map.insert(5, CellValue::DontCare);
    map.insert(7, CellValue::DontCare);

    let res = Minimizer::minimize_sop(VarCount::Three, &map);
    assert_eq!(res.expression, "C");
    assert_eq!(res.groups.len(), 1);
    assert_eq!(res.groups[0].size(), 4);
}

#[test]
fn test_essential_prime_implicant_selection() {
    // F(A, B, C) = Σ(0, 1, 2, 5)
    // Pairs:
    // (0, 1) -> A'B'
    // (0, 2) -> A'C'
    // (1, 5) -> B'C
    // Minterm 2 is covered only by (0, 2) -> (0, 2) is Essential!
    // Minterm 5 is covered only by (1, 5) -> (1, 5) is Essential!
    // (0, 2) and (1, 5) cover: {0, 2, 1, 5} -> All minterms covered!
    // (0, 1) is redundant and must not be included.
    let mut map = HashMap::new();
    for m in 0..8 {
        map.insert(m, CellValue::Zero);
    }
    map.insert(0, CellValue::One);
    map.insert(1, CellValue::One);
    map.insert(2, CellValue::One);
    map.insert(5, CellValue::One);

    let res = Minimizer::minimize_sop(VarCount::Three, &map);
    assert_eq!(res.groups.len(), 2);
    assert!(res.groups.iter().all(|g| g.is_essential));
    // The two essential prime implicants are A'C' and B'C
    let terms: Vec<String> = res.groups.iter().map(|g| g.to_sop_term()).collect();
    assert!(terms.contains(&"A'C'".to_string()));
    assert!(terms.contains(&"B'C".to_string()));
}

#[test]
fn test_integration_editing_truth_table_updates_kmap_and_expressions() {
    let mut table = TruthTable::new(VarCount::Two);
    // Initially all 0s
    let val0 = TableValidation::validate(&table);
    assert!(val0.is_valid);
    let minterms0: HashMap<usize, CellValue> = val0
        .minterm_map
        .iter()
        .map(|(&m, &(_, cv))| (m, cv))
        .collect();
    let sop0 = Minimizer::minimize_sop(VarCount::Two, &minterms0);
    assert_eq!(sop0.expression, "0");

    // User edits row 3 (which is minterm 3: 11) to '1'
    table.rows[3].output = "1".to_string();
    let val1 = TableValidation::validate(&table);
    assert!(val1.is_valid);
    assert_eq!(val1.minterm_map.get(&3).unwrap().1, CellValue::One);
    let minterms1: HashMap<usize, CellValue> = val1
        .minterm_map
        .iter()
        .map(|(&m, &(_, cv))| (m, cv))
        .collect();
    let sop1 = Minimizer::minimize_sop(VarCount::Two, &minterms1);
    assert_eq!(sop1.expression, "AB");

    // User edits row 2 (which is minterm 2: 10) to '1' -> group {2, 3} simplifies to A!
    table.rows[2].output = "1".to_string();
    let val2 = TableValidation::validate(&table);
    assert!(val2.is_valid);
    let minterms2: HashMap<usize, CellValue> = val2
        .minterm_map
        .iter()
        .map(|(&m, &(_, cv))| (m, cv))
        .collect();
    let sop2 = Minimizer::minimize_sop(VarCount::Two, &minterms2);
    assert_eq!(sop2.expression, "A");
    assert_eq!(sop2.groups.len(), 1);
    assert_eq!(sop2.groups[0].size(), 2);
}
