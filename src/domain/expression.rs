use crate::domain::model::VarCount;

/// Evaluates a Boolean expression (either SoP or PoS) for a specific minterm or input assignment.
#[derive(Debug, Clone)]
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// Evaluates a SoP expression string (e.g. "A'B + CD", "1", "0") for a given minterm.
    pub fn evaluate_sop(expr: &str, var_count: VarCount, minterm: usize) -> bool {
        let trimmed = expr.trim();
        if trimmed == "0" {
            return false;
        }
        if trimmed == "1" {
            return true;
        }

        let terms = trimmed.split('+');
        for term in terms {
            let term = term.trim();
            if term.is_empty() {
                continue;
            }
            if Self::evaluate_product_term(term, var_count, minterm) {
                return true;
            }
        }
        false
    }

    /// Evaluates a PoS expression string (e.g. "(A + B')(C + D)", "1", "0") for a given minterm.
    pub fn evaluate_pos(expr: &str, var_count: VarCount, minterm: usize) -> bool {
        let trimmed = expr.trim();
        if trimmed == "0" {
            return false;
        }
        if trimmed == "1" {
            return true;
        }

        // Split by factors enclosed in parentheses or split by ")("
        let factors = Self::parse_pos_factors(trimmed);
        if factors.is_empty() {
            return true;
        }

        for factor in factors {
            if !Self::evaluate_sum_term(&factor, var_count, minterm) {
                return false;
            }
        }
        true
    }

    /// Evaluates a single AND product term (e.g. "A'B" or "A")
    fn evaluate_product_term(term: &str, var_count: VarCount, minterm: usize) -> bool {
        let n = var_count.as_usize();
        let var_names = var_count.var_names();
        let chars: Vec<char> = term.chars().filter(|c| !c.is_whitespace()).collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            let mut complemented = false;
            if i + 1 < chars.len() && (chars[i + 1] == '\'' || chars[i + 1] == '’') {
                complemented = true;
                i += 2;
            } else {
                i += 1;
            }

            let var_name = ch.to_string();
            if let Some(var_idx) = var_names.iter().position(|&v| v == var_name) {
                let shift = n - 1 - var_idx;
                let bit_val = ((minterm >> shift) & 1) == 1;
                let lit_val = if complemented { !bit_val } else { bit_val };
                if !lit_val {
                    return false;
                }
            }
        }
        true
    }

    /// Parse factors in a PoS string. Examples: "(A + B)(C + D')", "(A)(B')", "(A + B')", "A + B'"
    fn parse_pos_factors(expr: &str) -> Vec<String> {
        let trimmed = expr.trim();
        if !trimmed.contains('(') {
            // No parentheses: could be single sum term e.g. "A + B'" or literals e.g. "A"
            if !trimmed.is_empty() {
                return vec![trimmed.to_string()];
            }
            return Vec::new();
        }

        let mut factors = Vec::new();
        let mut current = String::new();
        let mut inside = false;

        for ch in trimmed.chars() {
            if ch == '(' {
                inside = true;
                current.clear();
            } else if ch == ')' {
                inside = false;
                let factor = current.trim();
                if !factor.is_empty() {
                    factors.push(factor.to_string());
                }
                current.clear();
            } else if inside {
                current.push(ch);
            }
        }

        factors
    }

    /// Evaluates a single OR sum term (e.g. "A + B'").
    fn evaluate_sum_term(term: &str, var_count: VarCount, minterm: usize) -> bool {
        let n = var_count.as_usize();
        let var_names = var_count.var_names();
        let parts = term.split('+');

        for part in parts {
            let lit = part.trim();
            if lit.is_empty() {
                continue;
            }
            let is_complemented = lit.ends_with('\'') || lit.ends_with('’');
            let var_char = lit.chars().next().unwrap_or('?');
            let var_name = var_char.to_string();

            if let Some(var_idx) = var_names.iter().position(|&v| v == var_name) {
                let shift = n - 1 - var_idx;
                let bit_val = ((minterm >> shift) & 1) == 1;
                // In a sum term: A evaluates to bit_val, A' evaluates to !bit_val
                let lit_val = if is_complemented { !bit_val } else { bit_val };
                if lit_val {
                    return true;
                }
            }
        }
        false
    }
}
