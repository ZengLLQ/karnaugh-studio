use karnaugh_studio::domain::kmap::{
    KMapDimensions, coords_to_minterm, decompose_group_segments, get_col_labels, get_row_labels,
    minterm_to_coords,
};
use karnaugh_studio::domain::model::VarCount;

#[test]
fn test_2_var_kmap_mapping() {
    let vc = VarCount::Two;
    let dims = KMapDimensions::for_var_count(vc);
    assert_eq!(dims.rows, 2);
    assert_eq!(dims.cols, 2);

    // Row 0 (A=0): col 0 (B=0) -> m0 (00), col 1 (B=1) -> m1 (01)
    // Row 1 (A=1): col 0 (B=0) -> m2 (10), col 1 (B=1) -> m3 (11)
    assert_eq!(coords_to_minterm(vc, 0, 0), 0);
    assert_eq!(coords_to_minterm(vc, 0, 1), 1);
    assert_eq!(coords_to_minterm(vc, 1, 0), 2);
    assert_eq!(coords_to_minterm(vc, 1, 1), 3);

    for m in 0..4 {
        let (r, c) = minterm_to_coords(vc, m);
        assert_eq!(coords_to_minterm(vc, r, c), m);
    }
}

#[test]
fn test_3_var_kmap_mapping() {
    let vc = VarCount::Three;
    let dims = KMapDimensions::for_var_count(vc);
    assert_eq!(dims.rows, 2);
    assert_eq!(dims.cols, 4);

    // Row 0 (A=0): BC = 00 (m0), 01 (m1), 11 (m3), 10 (m2)
    assert_eq!(coords_to_minterm(vc, 0, 0), 0); // 000
    assert_eq!(coords_to_minterm(vc, 0, 1), 1); // 001
    assert_eq!(coords_to_minterm(vc, 0, 2), 3); // 011
    assert_eq!(coords_to_minterm(vc, 0, 3), 2); // 010

    // Row 1 (A=1): BC = 00 (m4), 01 (m5), 11 (m7), 10 (m6)
    assert_eq!(coords_to_minterm(vc, 1, 0), 4); // 100
    assert_eq!(coords_to_minterm(vc, 1, 1), 5); // 101
    assert_eq!(coords_to_minterm(vc, 1, 2), 7); // 111
    assert_eq!(coords_to_minterm(vc, 1, 3), 6); // 110

    for m in 0..8 {
        let (r, c) = minterm_to_coords(vc, m);
        assert_eq!(coords_to_minterm(vc, r, c), m);
    }
}

#[test]
fn test_4_var_kmap_mapping() {
    let vc = VarCount::Four;
    let dims = KMapDimensions::for_var_count(vc);
    assert_eq!(dims.rows, 4);
    assert_eq!(dims.cols, 4);

    // Standard 4-var K-map row/col minterms
    let expected = [
        [0, 1, 3, 2],     // AB = 00
        [4, 5, 7, 6],     // AB = 01
        [12, 13, 15, 14], // AB = 11
        [8, 9, 11, 10],   // AB = 10
    ];

    for (r, row) in expected.iter().enumerate() {
        for (c, &expected_m) in row.iter().enumerate() {
            assert_eq!(
                coords_to_minterm(vc, r, c),
                expected_m,
                "Mismatch at ({r}, {c})"
            );
            assert_eq!(minterm_to_coords(vc, expected_m), (r, c));
        }
    }
}

#[test]
fn test_gray_code_single_bit_change_torus_adjacency() {
    let vc = VarCount::Four;
    let dims = KMapDimensions::for_var_count(vc);

    for r in 0..dims.rows {
        for c in 0..dims.cols {
            let m = coords_to_minterm(vc, r, c);

            // Right neighbor (with torus wrap)
            let m_right = coords_to_minterm(vc, r, (c + 1) % dims.cols);
            let diff_right = (m ^ m_right).count_ones();
            assert_eq!(diff_right, 1, "Right neighbor must differ by 1 bit");

            // Down neighbor (with torus wrap)
            let m_down = coords_to_minterm(vc, (r + 1) % dims.rows, c);
            let diff_down = (m ^ m_down).count_ones();
            assert_eq!(diff_down, 1, "Down neighbor must differ by 1 bit");
        }
    }
}

#[test]
fn test_labels() {
    let row_labels_4 = get_row_labels(VarCount::Four);
    assert_eq!(
        row_labels_4,
        vec![("00", 0), ("01", 1), ("11", 2), ("10", 3)]
    );

    let col_labels_4 = get_col_labels(VarCount::Four);
    assert_eq!(
        col_labels_4,
        vec![("00", 0), ("01", 1), ("11", 2), ("10", 3)]
    );
}

#[test]
fn test_group_segmentation_torus() {
    let dims = KMapDimensions { rows: 4, cols: 4 };

    // 1. Simple planar group (no wrap): 2x2 at (1, 1)
    let segs = decompose_group_segments(1, 1, 2, 2, dims);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].r_start, 1);
    assert_eq!(segs[0].r_end, 2);
    assert_eq!(segs[0].c_start, 1);
    assert_eq!(segs[0].c_end, 2);
    assert!(!segs[0].open_top);
    assert!(!segs[0].open_bottom);
    assert!(!segs[0].open_left);
    assert!(!segs[0].open_right);

    // 2. Horizontal wrap group: cols 3 and 0 at rows 0..1
    let segs_h = decompose_group_segments(0, 3, 2, 2, dims);
    assert_eq!(segs_h.len(), 2);
    // Right part at col 3 (open right)
    assert!(segs_h.iter().any(|s| s.c_start == 3 && s.open_right));
    // Left part at col 0 (open left)
    assert!(segs_h.iter().any(|s| s.c_start == 0 && s.open_left));

    // 3. Four corners wrap (horizontal and vertical wrap): r0=3, c0=3, h=2, w=2
    let segs_corners = decompose_group_segments(3, 3, 2, 2, dims);
    assert_eq!(segs_corners.len(), 4);
    // Must contain segments touching rows 3 and 0, cols 3 and 0
    for s in &segs_corners {
        assert_eq!(s.r_start, s.r_end);
        assert_eq!(s.c_start, s.c_end);
    }
}

#[test]
fn test_full_span_and_full_map_groups_have_closed_outlines() {
    let dims4 = KMapDimensions { rows: 4, cols: 4 };

    // Full column in 4-variable map: r0=0, c0=1, h=4, w=1
    let full_col = decompose_group_segments(0, 1, 4, 1, dims4);
    assert_eq!(full_col.len(), 1);
    assert!(!full_col[0].open_top, "Full column top must be closed");
    assert!(
        !full_col[0].open_bottom,
        "Full column bottom must be closed"
    );
    assert!(!full_col[0].open_left, "Full column left must be closed");
    assert!(!full_col[0].open_right, "Full column right must be closed");

    // Full row in 4-variable map: r0=2, c0=0, h=1, w=4
    let full_row = decompose_group_segments(2, 0, 1, 4, dims4);
    assert_eq!(full_row.len(), 1);
    assert!(!full_row[0].open_top, "Full row top must be closed");
    assert!(!full_row[0].open_bottom, "Full row bottom must be closed");
    assert!(!full_row[0].open_left, "Full row left must be closed");
    assert!(!full_row[0].open_right, "Full row right must be closed");

    // Full map (16 cells): r0=0, c0=0, h=4, w=4
    let full_map = decompose_group_segments(0, 0, 4, 4, dims4);
    assert_eq!(full_map.len(), 1);
    assert!(!full_map[0].open_top, "Full map top must be closed");
    assert!(!full_map[0].open_bottom, "Full map bottom must be closed");
    assert!(!full_map[0].open_left, "Full map left must be closed");
    assert!(!full_map[0].open_right, "Full map right must be closed");

    // 2-variable map tests (2x2)
    let dims2 = KMapDimensions { rows: 2, cols: 2 };
    let full_col_2 = decompose_group_segments(0, 0, 2, 1, dims2);
    assert_eq!(full_col_2.len(), 1);
    assert!(!full_col_2[0].open_top);
    assert!(!full_col_2[0].open_bottom);

    let full_row_2 = decompose_group_segments(0, 0, 1, 2, dims2);
    assert_eq!(full_row_2.len(), 1);
    assert!(!full_row_2[0].open_left);
    assert!(!full_row_2[0].open_right);

    let full_map_2 = decompose_group_segments(0, 0, 2, 2, dims2);
    assert_eq!(full_map_2.len(), 1);
    assert!(!full_map_2[0].open_top && !full_map_2[0].open_bottom);
    assert!(!full_map_2[0].open_left && !full_map_2[0].open_right);
}
