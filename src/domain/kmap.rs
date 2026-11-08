use crate::domain::model::VarCount;
use std::collections::BTreeSet;

pub const GRAY_2BIT: [usize; 4] = [0b00, 0b01, 0b11, 0b10];
pub const GRAY_2BIT_STR: [&str; 4] = ["00", "01", "11", "10"];

pub const GRAY_1BIT: [usize; 2] = [0b0, 0b1];
pub const GRAY_1BIT_STR: [&str; 2] = ["0", "1"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KMapDimensions {
    pub rows: usize,
    pub cols: usize,
}

impl KMapDimensions {
    pub fn for_var_count(var_count: VarCount) -> Self {
        match var_count {
            VarCount::Two => Self { rows: 2, cols: 2 },
            VarCount::Three => Self { rows: 2, cols: 4 },
            VarCount::Four => Self { rows: 4, cols: 4 },
        }
    }
}

/// Convert (row, col) grid coordinates in Gray code order to minterm index.
pub fn coords_to_minterm(var_count: VarCount, r: usize, c: usize) -> usize {
    match var_count {
        VarCount::Two => {
            let a = GRAY_1BIT[r];
            let b = GRAY_1BIT[c];
            (a << 1) | b
        }
        VarCount::Three => {
            let a = GRAY_1BIT[r];
            let bc = GRAY_2BIT[c];
            (a << 2) | bc
        }
        VarCount::Four => {
            let ab = GRAY_2BIT[r];
            let cd = GRAY_2BIT[c];
            (ab << 2) | cd
        }
    }
}

/// Convert minterm index to (row, col) grid coordinates.
pub fn minterm_to_coords(var_count: VarCount, minterm: usize) -> (usize, usize) {
    match var_count {
        VarCount::Two => {
            let a = (minterm >> 1) & 1;
            let b = minterm & 1;
            let r = if a == 0 { 0 } else { 1 };
            let c = if b == 0 { 0 } else { 1 };
            (r, c)
        }
        VarCount::Three => {
            let a = (minterm >> 2) & 1;
            let bc = minterm & 0b11;
            let r = if a == 0 { 0 } else { 1 };
            let c = match bc {
                0b00 => 0,
                0b01 => 1,
                0b11 => 2,
                0b10 => 3,
                _ => unreachable!(),
            };
            (r, c)
        }
        VarCount::Four => {
            let ab = (minterm >> 2) & 0b11;
            let cd = minterm & 0b11;
            let r = match ab {
                0b00 => 0,
                0b01 => 1,
                0b11 => 2,
                0b10 => 3,
                _ => unreachable!(),
            };
            let c = match cd {
                0b00 => 0,
                0b01 => 1,
                0b11 => 2,
                0b10 => 3,
                _ => unreachable!(),
            };
            (r, c)
        }
    }
}

pub fn get_row_labels(var_count: VarCount) -> Vec<(&'static str, usize)> {
    match var_count {
        VarCount::Two | VarCount::Three => {
            vec![("0", 0), ("1", 1)]
        }
        VarCount::Four => vec![("00", 0), ("01", 1), ("11", 2), ("10", 3)],
    }
}

pub fn get_col_labels(var_count: VarCount) -> Vec<(&'static str, usize)> {
    match var_count {
        VarCount::Two => vec![("0", 0), ("1", 1)],
        VarCount::Three | VarCount::Four => {
            vec![("00", 0), ("01", 1), ("11", 2), ("10", 3)]
        }
    }
}

pub fn row_var_header(var_count: VarCount) -> &'static str {
    match var_count {
        VarCount::Two | VarCount::Three => "A",
        VarCount::Four => "AB",
    }
}

pub fn col_var_header(var_count: VarCount) -> &'static str {
    match var_count {
        VarCount::Two => "B",
        VarCount::Three => "BC",
        VarCount::Four => "CD",
    }
}

/// A contiguous visual rectangular segment of a K-map group on the grid canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KMapRectSegment {
    pub r_start: usize,
    pub r_end: usize, // inclusive
    pub c_start: usize,
    pub c_end: usize, // inclusive
    pub open_top: bool,
    pub open_bottom: bool,
    pub open_left: bool,
    pub open_right: bool,
}

/// Decompose a toroidal group (r0, c0, H, W) into 1, 2, or 4 visual grid segments with open-edge flags.
pub fn decompose_group_segments(
    r0: usize,
    c0: usize,
    h: usize,
    w: usize,
    dims: KMapDimensions,
) -> Vec<KMapRectSegment> {
    let wraps_v = r0 + h > dims.rows;
    let wraps_h = c0 + w > dims.cols;

    let v_ranges: Vec<(usize, usize, bool, bool)> = if !wraps_v {
        // [r0, r0 + h - 1], fully closed vertical range since it does not wrap
        vec![(r0, r0 + h - 1, false, false)]
    } else {
        // Two vertical parts: [r0, dims.rows - 1] (open bottom) and [0, (r0 + h - 1) % dims.rows] (open top)
        vec![
            (r0, dims.rows - 1, false, true),
            (0, (r0 + h - 1) % dims.rows, true, false),
        ]
    };

    let h_ranges: Vec<(usize, usize, bool, bool)> = if !wraps_h {
        // [c0, c0 + w - 1], fully closed horizontal range since it does not wrap
        vec![(c0, c0 + w - 1, false, false)]
    } else {
        // Two horizontal parts: [c0, dims.cols - 1] (open right) and [0, (c0 + w - 1) % dims.cols] (open left)
        vec![
            (c0, dims.cols - 1, false, true),
            (0, (c0 + w - 1) % dims.cols, true, false),
        ]
    };

    let mut segments = Vec::new();
    for &(r_start, r_end, open_top, open_bottom) in &v_ranges {
        for &(c_start, c_end, open_left, open_right) in &h_ranges {
            segments.push(KMapRectSegment {
                r_start,
                r_end,
                c_start,
                c_end,
                open_top,
                open_bottom,
                open_left,
                open_right,
            });
        }
    }
    segments
}

/// Collect the set of minterms in a toroidal group (r0, c0, H, W).
pub fn group_minterms(
    var_count: VarCount,
    r0: usize,
    c0: usize,
    h: usize,
    w: usize,
) -> BTreeSet<usize> {
    let dims = KMapDimensions::for_var_count(var_count);
    let mut minterms = BTreeSet::new();
    for i in 0..h {
        let r = (r0 + i) % dims.rows;
        for j in 0..w {
            let c = (c0 + j) % dims.cols;
            minterms.insert(coords_to_minterm(var_count, r, c));
        }
    }
    minterms
}
