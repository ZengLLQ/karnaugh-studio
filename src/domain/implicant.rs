use crate::domain::kmap::{KMapDimensions, KMapRectSegment, decompose_group_segments};
use crate::domain::model::VarCount;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Implicant {
    pub var_count: VarCount,
    pub minterms: BTreeSet<usize>,
    pub r0: usize,
    pub c0: usize,
    pub h: usize,
    pub w: usize,
    /// Fixed variables: (var_idx, value). var_idx 0 is A, 1 is B, etc.
    pub fixed_vars: Vec<(usize, bool)>,
    /// Eliminated variables (indices 0..n)
    pub eliminated_vars: Vec<usize>,
    pub is_essential: bool,
    pub is_wrap: bool,
}

impl Implicant {
    pub fn new(
        var_count: VarCount,
        minterms: BTreeSet<usize>,
        r0: usize,
        c0: usize,
        h: usize,
        w: usize,
    ) -> Self {
        let n = var_count.as_usize();
        let dims = KMapDimensions::for_var_count(var_count);
        let is_wrap = (r0 + h > dims.rows) || (c0 + w > dims.cols);

        let mut fixed_vars = Vec::new();
        let mut eliminated_vars = Vec::new();

        for bit_pos in 0..n {
            // Variable index 0 is MSB (A), 1 is B, etc.
            let shift = n - 1 - bit_pos;
            let mut has_zero = false;
            let mut has_one = false;
            for &m in &minterms {
                let bit = (m >> shift) & 1;
                if bit == 1 {
                    has_one = true;
                } else {
                    has_zero = true;
                }
            }

            if has_zero && !has_one {
                fixed_vars.push((bit_pos, false));
            } else if has_one && !has_zero {
                fixed_vars.push((bit_pos, true));
            } else {
                eliminated_vars.push(bit_pos);
            }
        }

        Self {
            var_count,
            minterms,
            r0,
            c0,
            h,
            w,
            fixed_vars,
            eliminated_vars,
            is_essential: false,
            is_wrap,
        }
    }

    pub fn size(&self) -> usize {
        self.minterms.len()
    }

    pub fn literal_count(&self) -> usize {
        self.fixed_vars.len()
    }

    /// Format as a Sum of Products product term (e.g. "A'B" or "1").
    pub fn to_sop_term(&self) -> String {
        if self.fixed_vars.is_empty() {
            return "1".to_string();
        }
        let var_names = self.var_count.var_names();
        let mut s = String::new();
        for &(var_idx, val) in &self.fixed_vars {
            let name = var_names[var_idx];
            s.push_str(name);
            if !val {
                s.push('\'');
            }
        }
        s
    }

    /// Format as a Product of Sums maxterm factor (e.g. "(A + B')" or "0").
    pub fn to_pos_term(&self) -> String {
        if self.fixed_vars.is_empty() {
            return "0".to_string();
        }
        let var_names = self.var_count.var_names();
        let mut parts = Vec::new();
        for &(var_idx, val) in &self.fixed_vars {
            let name = var_names[var_idx];
            // In PoS: 0 is uncomplemented (A), 1 is complemented (A')
            if val {
                parts.push(format!("{name}'"));
            } else {
                parts.push(name.to_string());
            }
        }
        format!("({})", parts.join(" + "))
    }

    pub fn minterm_list_string(&self, is_sop: bool) -> String {
        let prefix = if is_sop { "m" } else { "M" };
        let list: Vec<String> = self.minterms.iter().map(|m| m.to_string()).collect();
        format!("{prefix}({})", list.join(", "))
    }

    pub fn eliminated_vars_string(&self) -> String {
        if self.eliminated_vars.is_empty() {
            "None".to_string()
        } else {
            let var_names = self.var_count.var_names();
            let names: Vec<&str> = self
                .eliminated_vars
                .iter()
                .map(|&idx| var_names[idx])
                .collect();
            names.join(", ")
        }
    }

    pub fn visual_segments(&self) -> Vec<KMapRectSegment> {
        let dims = KMapDimensions::for_var_count(self.var_count);
        decompose_group_segments(self.r0, self.c0, self.h, self.w, dims)
    }
}
