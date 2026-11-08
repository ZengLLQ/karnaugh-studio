use crate::domain::model::{CellValue, TruthTable};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowError {
    InvalidInput { col: usize, val: String },
    MissingInput { col: usize },
    InvalidOutput { val: String },
    DuplicateRow { duplicate_of: usize },
}

impl RowError {
    pub fn message(&self, var_names: &[&'static str]) -> String {
        match self {
            RowError::InvalidInput { col, val } => {
                let name = var_names.get(*col).unwrap_or(&"?");
                format!("{name} must be 0 or 1 (got '{val}')")
            }
            RowError::MissingInput { col } => {
                let name = var_names.get(*col).unwrap_or(&"?");
                format!("{name} is missing")
            }
            RowError::InvalidOutput { val } => {
                format!("F must be 0, 1, or X (got '{val}')")
            }
            RowError::DuplicateRow { duplicate_of } => {
                format!("Duplicate input tuple of row {}", duplicate_of + 1)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableValidation {
    pub is_valid: bool,
    pub row_errors: HashMap<usize, Vec<RowError>>,
    pub missing_minterms: Vec<usize>,
    pub duplicate_minterm_count: usize,
    /// Maps minterm -> (table_row_index, CellValue)
    pub minterm_map: HashMap<usize, (usize, CellValue)>,
    /// Maps table_row_index -> minterm (if valid row inputs)
    pub row_to_minterm: HashMap<usize, usize>,
    pub summary_errors: Vec<String>,
}

impl TableValidation {
    pub fn validate(table: &TruthTable) -> Self {
        let n = table.var_count.as_usize();
        let expected_count = table.var_count.num_rows();

        let mut row_errors: HashMap<usize, Vec<RowError>> = HashMap::new();
        let mut seen_inputs: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut minterm_map: HashMap<usize, (usize, CellValue)> = HashMap::new();
        let mut row_to_minterm: HashMap<usize, usize> = HashMap::new();
        let mut duplicate_minterm_count = 0;
        let mut summary_errors = Vec::new();

        if table.rows.len() != expected_count {
            summary_errors.push(format!(
                "Table must contain exactly {} rows for {} variables (found {} rows)",
                expected_count,
                n,
                table.rows.len()
            ));
        }

        for (row_idx, row) in table.rows.iter().enumerate() {
            let mut errors = Vec::new();
            let mut parsed_inputs = Vec::with_capacity(n);
            let mut inputs_ok = true;

            for col in 0..n {
                let raw_val = row.inputs.get(col).map(|s| s.trim()).unwrap_or("");
                if raw_val.is_empty() {
                    errors.push(RowError::MissingInput { col });
                    inputs_ok = false;
                } else if raw_val == "0" {
                    parsed_inputs.push(0u8);
                } else if raw_val == "1" {
                    parsed_inputs.push(1u8);
                } else {
                    errors.push(RowError::InvalidInput {
                        col,
                        val: raw_val.to_string(),
                    });
                    inputs_ok = false;
                }
            }

            // Check output
            let output_trimmed = row.output.trim();
            let cell_value = if output_trimmed.is_empty() {
                errors.push(RowError::InvalidOutput {
                    val: "".to_string(),
                });
                None
            } else {
                match CellValue::parse(output_trimmed) {
                    Some(cv) => Some(cv),
                    None => {
                        errors.push(RowError::InvalidOutput {
                            val: output_trimmed.to_string(),
                        });
                        None
                    }
                }
            };

            // Check duplicate inputs
            if inputs_ok && parsed_inputs.len() == n {
                if let Some(&dup_row) = seen_inputs.get(&parsed_inputs) {
                    errors.push(RowError::DuplicateRow {
                        duplicate_of: dup_row,
                    });
                    duplicate_minterm_count += 1;
                } else {
                    seen_inputs.insert(parsed_inputs.clone(), row_idx);
                    let mut minterm = 0usize;
                    for bit in &parsed_inputs {
                        minterm = (minterm << 1) | (*bit as usize);
                    }
                    row_to_minterm.insert(row_idx, minterm);
                    if let Some(cv) = cell_value {
                        minterm_map.insert(minterm, (row_idx, cv));
                    }
                }
            }

            if !errors.is_empty() {
                row_errors.insert(row_idx, errors);
            }
        }

        // Check for missing minterms from 0 to expected_count - 1
        let mut missing_minterms = Vec::new();
        for m in 0..expected_count {
            if !minterm_map.contains_key(&m) {
                missing_minterms.push(m);
            }
        }

        if !missing_minterms.is_empty() {
            if missing_minterms.len() <= 6 {
                let missing_formatted: Vec<String> = missing_minterms
                    .iter()
                    .map(|m| format_minterm_binary(*m, n))
                    .collect();
                summary_errors.push(format!(
                    "Missing {} input combination(s): {}",
                    missing_minterms.len(),
                    missing_formatted.join(", ")
                ));
            } else {
                summary_errors.push(format!(
                    "Missing {} input combinations out of {}",
                    missing_minterms.len(),
                    expected_count
                ));
            }
        }

        if duplicate_minterm_count > 0 {
            summary_errors.push(format!(
                "Table contains {} duplicate input row(s)",
                duplicate_minterm_count
            ));
        }

        let is_valid = row_errors.is_empty()
            && missing_minterms.is_empty()
            && table.rows.len() == expected_count;

        Self {
            is_valid,
            row_errors,
            missing_minterms,
            duplicate_minterm_count,
            minterm_map,
            row_to_minterm,
            summary_errors,
        }
    }

    pub fn get_row_errors(&self, row_idx: usize) -> Option<&Vec<RowError>> {
        self.row_errors.get(&row_idx)
    }

    pub fn row_minterm(&self, row_idx: usize) -> Option<usize> {
        self.row_to_minterm.get(&row_idx).copied()
    }
}

fn format_minterm_binary(m: usize, n: usize) -> String {
    let mut bits = String::with_capacity(n);
    for i in (0..n).rev() {
        bits.push(if (m >> i) & 1 == 1 { '1' } else { '0' });
    }
    format!("{bits} (m{m})")
}
