#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarCount {
    Two = 2,
    Three = 3,
    Four = 4,
}

impl VarCount {
    pub fn as_usize(self) -> usize {
        self as usize
    }

    pub fn num_rows(self) -> usize {
        1 << self.as_usize()
    }

    pub fn var_names(self) -> Vec<&'static str> {
        match self {
            VarCount::Two => vec!["A", "B"],
            VarCount::Three => vec!["A", "B", "C"],
            VarCount::Four => vec!["A", "B", "C", "D"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellValue {
    Zero,
    One,
    DontCare,
}

impl CellValue {
    pub fn as_char(self) -> char {
        match self {
            CellValue::Zero => '0',
            CellValue::One => '1',
            CellValue::DontCare => 'X',
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            CellValue::Zero => "0",
            CellValue::One => "1",
            CellValue::DontCare => "X",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed == "0" {
            Some(CellValue::Zero)
        } else if trimmed == "1" {
            Some(CellValue::One)
        } else if trimmed.eq_ignore_ascii_case("x") || trimmed == "-" {
            Some(CellValue::DontCare)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRow {
    pub id: usize,
    pub inputs: Vec<String>,
    pub output: String,
}

impl RawRow {
    pub fn new(id: usize, inputs: Vec<String>, output: String) -> Self {
        Self { id, inputs, output }
    }

    pub fn canonical(id: usize, minterm: usize, var_count: VarCount, output: CellValue) -> Self {
        let n = var_count.as_usize();
        let mut inputs = Vec::with_capacity(n);
        for i in (0..n).rev() {
            let bit = (minterm >> i) & 1;
            inputs.push(bit.to_string());
        }
        Self {
            id,
            inputs,
            output: output.as_str().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthTable {
    pub var_count: VarCount,
    pub rows: Vec<RawRow>,
    next_row_id: usize,
}

impl TruthTable {
    pub fn new(var_count: VarCount) -> Self {
        let mut table = Self {
            var_count,
            rows: Vec::new(),
            next_row_id: 0,
        };
        table.populate_canonical(CellValue::Zero);
        table
    }

    pub fn populate_canonical(&mut self, default_output: CellValue) {
        self.rows.clear();
        let num_rows = self.var_count.num_rows();
        for m in 0..num_rows {
            let row = RawRow::canonical(self.next_row_id, m, self.var_count, default_output);
            self.next_row_id += 1;
            self.rows.push(row);
        }
    }

    pub fn set_var_count(&mut self, var_count: VarCount) {
        if self.var_count != var_count {
            self.var_count = var_count;
            self.populate_canonical(CellValue::Zero);
        }
    }

    pub fn set_all_outputs(&mut self, val: CellValue) {
        for row in &mut self.rows {
            row.output = val.as_str().to_string();
        }
    }

    pub fn reset(&mut self) {
        self.populate_canonical(CellValue::Zero);
    }

    pub fn is_canonical(&self) -> bool {
        let n = self.var_count.as_usize();
        let expected_rows = self.var_count.num_rows();
        if self.rows.len() != expected_rows {
            return false;
        }

        for (m, row) in self.rows.iter().enumerate() {
            if row.inputs.len() != n {
                return false;
            }
            for i in 0..n {
                let bit = (m >> (n - 1 - i)) & 1;
                if row.inputs[i].trim() != bit.to_string() {
                    return false;
                }
            }
        }
        true
    }

    pub fn is_dirty(&self) -> bool {
        // True if not in standard canonical zero state
        if !self.is_canonical() {
            return true;
        }
        self.rows.iter().any(|r| r.output.trim() != "0")
    }

    pub fn parsed_minterm_for_row(&self, row_idx: usize) -> Option<usize> {
        let row = self.rows.get(row_idx)?;
        let n = self.var_count.as_usize();
        if row.inputs.len() != n {
            return None;
        }
        let mut minterm = 0;
        for input in &row.inputs {
            let trimmed = input.trim();
            if trimmed == "1" {
                minterm = (minterm << 1) | 1;
            } else if trimmed == "0" {
                minterm <<= 1;
            } else {
                return None;
            }
        }
        Some(minterm)
    }
}
