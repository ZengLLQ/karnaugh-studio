use crate::domain::implicant::Implicant;
use crate::domain::kmap::{KMapDimensions, group_minterms};
use crate::domain::model::{CellValue, VarCount};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimizationResult {
    pub is_sop: bool,
    pub expression: String,
    pub groups: Vec<Implicant>,
}

impl MinimizationResult {
    pub fn empty(is_sop: bool) -> Self {
        Self {
            is_sop,
            expression: if is_sop {
                "0".to_string()
            } else {
                "1".to_string()
            },
            groups: Vec::new(),
        }
    }
}

pub struct Minimizer;

impl Minimizer {
    /// Minimize boolean function for Sum of Products (SoP).
    /// Target is 1s, allowed are 1s and Xs, forbidden are 0s.
    pub fn minimize_sop(
        var_count: VarCount,
        minterm_values: &HashMap<usize, CellValue>,
    ) -> MinimizationResult {
        let total_minterms = var_count.num_rows();
        let mut on_set = BTreeSet::new();
        let mut dc_set = BTreeSet::new();

        for m in 0..total_minterms {
            match minterm_values.get(&m).copied().unwrap_or(CellValue::Zero) {
                CellValue::One => {
                    on_set.insert(m);
                }
                CellValue::DontCare => {
                    dc_set.insert(m);
                }
                CellValue::Zero => {}
            }
        }

        // Special case: empty on-set
        if on_set.is_empty() {
            return MinimizationResult {
                is_sop: true,
                expression: "0".to_string(),
                groups: Vec::new(),
            };
        }

        // Special case: all minterms are 1 (or 1 + X covering all 2^n)
        if on_set.len() + dc_set.len() == total_minterms {
            let mut full_group = Implicant::new(
                var_count,
                (0..total_minterms).collect(),
                0,
                0,
                KMapDimensions::for_var_count(var_count).rows,
                KMapDimensions::for_var_count(var_count).cols,
            );
            full_group.is_essential = true;
            return MinimizationResult {
                is_sop: true,
                expression: "1".to_string(),
                groups: vec![full_group],
            };
        }

        let selected = Self::solve_exact_cover(var_count, &on_set, &dc_set);
        let expression = if selected.is_empty() {
            "0".to_string()
        } else {
            selected
                .iter()
                .map(|g| g.to_sop_term())
                .collect::<Vec<_>>()
                .join(" + ")
        };

        MinimizationResult {
            is_sop: true,
            expression,
            groups: selected,
        }
    }

    /// Minimize boolean function for Product of Sums (PoS).
    /// Target is 0s, allowed are 0s and Xs, forbidden are 1s.
    pub fn minimize_pos(
        var_count: VarCount,
        minterm_values: &HashMap<usize, CellValue>,
    ) -> MinimizationResult {
        let total_minterms = var_count.num_rows();
        let mut off_set = BTreeSet::new();
        let mut dc_set = BTreeSet::new();

        for m in 0..total_minterms {
            match minterm_values.get(&m).copied().unwrap_or(CellValue::Zero) {
                CellValue::Zero => {
                    off_set.insert(m);
                }
                CellValue::DontCare => {
                    dc_set.insert(m);
                }
                CellValue::One => {}
            }
        }

        // Special case: empty off-set (function is identically 1)
        if off_set.is_empty() {
            return MinimizationResult {
                is_sop: false,
                expression: "1".to_string(),
                groups: Vec::new(),
            };
        }

        // Special case: all minterms are 0 (or 0 + X covering all 2^n)
        if off_set.len() + dc_set.len() == total_minterms {
            let mut full_group = Implicant::new(
                var_count,
                (0..total_minterms).collect(),
                0,
                0,
                KMapDimensions::for_var_count(var_count).rows,
                KMapDimensions::for_var_count(var_count).cols,
            );
            full_group.is_essential = true;
            return MinimizationResult {
                is_sop: false,
                expression: "0".to_string(),
                groups: vec![full_group],
            };
        }

        let selected = Self::solve_exact_cover(var_count, &off_set, &dc_set);
        let expression = if selected.is_empty() {
            "1".to_string()
        } else {
            selected
                .iter()
                .map(|g| g.to_pos_term())
                .collect::<Vec<_>>()
                .join("")
        };

        MinimizationResult {
            is_sop: false,
            expression,
            groups: selected,
        }
    }

    /// Core exact cover algorithm:
    /// Generates candidate groups on K-map torus, identifies primes, finds essentials, solves remaining with exact cover.
    fn solve_exact_cover(
        var_count: VarCount,
        target_set: &BTreeSet<usize>,
        dc_set: &BTreeSet<usize>,
    ) -> Vec<Implicant> {
        let dims = KMapDimensions::for_var_count(var_count);
        let allowed_minterms: BTreeSet<usize> = target_set.union(dc_set).copied().collect();

        // 1. Enumerate all valid rectangular toroidal candidate groups of power-of-two size
        let mut candidates_map: BTreeMap<BTreeSet<usize>, (usize, usize, usize, usize)> =
            BTreeMap::new();

        let possible_h: Vec<usize> = [1, 2, 4].into_iter().filter(|&h| h <= dims.rows).collect();
        let possible_w: Vec<usize> = [1, 2, 4].into_iter().filter(|&w| w <= dims.cols).collect();

        for &h in &possible_h {
            for &w in &possible_w {
                let size = h * w;
                if !size.is_power_of_two() {
                    continue;
                }

                for r0 in 0..dims.rows {
                    for c0 in 0..dims.cols {
                        let cell_set = group_minterms(var_count, r0, c0, h, w);

                        // Must contain only allowed cells
                        if !cell_set.iter().all(|m| allowed_minterms.contains(m)) {
                            continue;
                        }

                        // Must contain at least one target cell (not pure don't-care)
                        if !cell_set.iter().any(|m| target_set.contains(m)) {
                            continue;
                        }

                        // Deduplicate identical sets of minterms; keep deterministic smallest coordinate
                        if let Some(&(prev_r, prev_c, prev_h, prev_w)) =
                            candidates_map.get(&cell_set)
                        {
                            if (r0, c0, h, w) < (prev_r, prev_c, prev_h, prev_w) {
                                candidates_map.insert(cell_set, (r0, c0, h, w));
                            }
                        } else {
                            candidates_map.insert(cell_set, (r0, c0, h, w));
                        }
                    }
                }
            }
        }

        // Build list of candidate implicants
        let all_candidates: Vec<Implicant> = candidates_map
            .into_iter()
            .map(|(minterms, (r0, c0, h, w))| Implicant::new(var_count, minterms, r0, c0, h, w))
            .collect();

        // 2. Filter to Prime Implicants
        // A candidate is prime if it is not a strict subset of any other candidate
        let mut prime_implicants: Vec<Implicant> = Vec::new();
        for i in 0..all_candidates.len() {
            let mut is_prime = true;
            for j in 0..all_candidates.len() {
                if i != j
                    && all_candidates[i]
                        .minterms
                        .is_subset(&all_candidates[j].minterms)
                    && all_candidates[i].minterms.len() < all_candidates[j].minterms.len()
                {
                    is_prime = false;
                    break;
                }
            }
            if is_prime {
                prime_implicants.push(all_candidates[i].clone());
            }
        }

        // 3. Identify Essential Prime Implicants
        let mut essential_indices = BTreeSet::new();
        for &m in target_set {
            let covering_primes: Vec<usize> = prime_implicants
                .iter()
                .enumerate()
                .filter(|(_, p)| p.minterms.contains(&m))
                .map(|(idx, _)| idx)
                .collect();

            if covering_primes.len() == 1 {
                essential_indices.insert(covering_primes[0]);
            }
        }

        for &idx in &essential_indices {
            prime_implicants[idx].is_essential = true;
        }

        // 4. Determine uncovered target minterms
        let mut covered_by_essential = BTreeSet::new();
        for &idx in &essential_indices {
            for &m in &prime_implicants[idx].minterms {
                if target_set.contains(&m) {
                    covered_by_essential.insert(m);
                }
            }
        }

        let remaining_targets: Vec<usize> = target_set
            .difference(&covered_by_essential)
            .copied()
            .collect();

        let mut final_selected_primes: Vec<Implicant> = Vec::new();

        // If all target minterms are covered by essential prime implicants, we are done
        if remaining_targets.is_empty() {
            for &idx in &essential_indices {
                final_selected_primes.push(prime_implicants[idx].clone());
            }
        } else {
            // 5. Solve remaining coverage using Branch & Bound Exact Cover
            // Collect non-essential primes that cover at least one remaining target
            let remaining_prime_candidates: Vec<(usize, Implicant)> = prime_implicants
                .iter()
                .enumerate()
                .filter(|(idx, p)| {
                    !essential_indices.contains(idx)
                        && remaining_targets.iter().any(|m| p.minterms.contains(m))
                })
                .map(|(idx, p)| (idx, p.clone()))
                .collect();

            let best_cover_indices =
                Self::find_best_cover(&remaining_targets, &remaining_prime_candidates);

            for &idx in &essential_indices {
                final_selected_primes.push(prime_implicants[idx].clone());
            }
            for idx in best_cover_indices {
                final_selected_primes.push(remaining_prime_candidates[idx].1.clone());
            }
        }

        // 6. Sort selected implicants deterministically:
        // Essential first, then larger group size, then lowest minterm, then coordinates
        final_selected_primes.sort_by(|a, b| {
            b.is_essential
                .cmp(&a.is_essential)
                .then_with(|| b.size().cmp(&a.size()))
                .then_with(|| {
                    let min_a = a.minterms.iter().next().unwrap_or(&0);
                    let min_b = b.minterms.iter().next().unwrap_or(&0);
                    min_a.cmp(min_b)
                })
                .then_with(|| (a.r0, a.c0, a.h, a.w).cmp(&(b.r0, b.c0, b.h, b.w)))
        });

        final_selected_primes
    }

    /// Finds the optimal subset of candidate implicants covering all targets.
    /// Optimization order:
    /// 1. Fewest implicants
    /// 2. Fewest total literals
    /// 3. Deterministic coordinate/minterm tie-break
    fn find_best_cover(targets: &[usize], candidates: &[(usize, Implicant)]) -> Vec<usize> {
        let n_cands = candidates.len();
        if n_cands == 0 {
            return Vec::new();
        }

        let mut best_solution: Option<Vec<usize>> = None;
        let mut best_implicant_count = usize::MAX;
        let mut best_literal_count = usize::MAX;

        // Recursive branch and bound search
        #[allow(clippy::too_many_arguments)]
        fn search(
            cand_idx: usize,
            current_selection: &mut Vec<usize>,
            covered: &mut BTreeSet<usize>,
            targets: &[usize],
            candidates: &[(usize, Implicant)],
            best_solution: &mut Option<Vec<usize>>,
            best_implicant_count: &mut usize,
            best_literal_count: &mut usize,
        ) {
            // Prune if current selection already has more implicants than best
            if current_selection.len() > *best_implicant_count {
                return;
            }

            // Check if all targets are covered
            let all_covered = targets.iter().all(|t| covered.contains(t));
            if all_covered {
                let imp_count = current_selection.len();
                let lit_count: usize = current_selection
                    .iter()
                    .map(|&i| candidates[i].1.literal_count())
                    .sum();

                let is_better = if imp_count < *best_implicant_count {
                    true
                } else if imp_count == *best_implicant_count {
                    if lit_count < *best_literal_count {
                        true
                    } else if lit_count == *best_literal_count {
                        // Deterministic tie-break: compare sorted implicant minterms / coordinates
                        if let Some(cur_best) = best_solution.as_ref() {
                            let cur_keys: Vec<_> = current_selection
                                .iter()
                                .map(|&i| {
                                    (
                                        candidates[i].1.r0,
                                        candidates[i].1.c0,
                                        candidates[i].1.h,
                                        candidates[i].1.w,
                                    )
                                })
                                .collect();
                            let best_keys: Vec<_> = cur_best
                                .iter()
                                .map(|&i| {
                                    (
                                        candidates[i].1.r0,
                                        candidates[i].1.c0,
                                        candidates[i].1.h,
                                        candidates[i].1.w,
                                    )
                                })
                                .collect();
                            cur_keys < best_keys
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_better {
                    *best_solution = Some(current_selection.clone());
                    *best_implicant_count = imp_count;
                    *best_literal_count = lit_count;
                }
                return;
            }

            if cand_idx >= candidates.len() {
                return;
            }

            // Option 1: Include candidates[cand_idx]
            let newly_covered: Vec<usize> = candidates[cand_idx]
                .1
                .minterms
                .iter()
                .filter(|&&m| targets.contains(&m) && !covered.contains(&m))
                .copied()
                .collect();

            if !newly_covered.is_empty() {
                current_selection.push(cand_idx);
                for &m in &newly_covered {
                    covered.insert(m);
                }

                search(
                    cand_idx + 1,
                    current_selection,
                    covered,
                    targets,
                    candidates,
                    best_solution,
                    best_implicant_count,
                    best_literal_count,
                );

                for &m in &newly_covered {
                    covered.remove(&m);
                }
                current_selection.pop();
            }

            // Option 2: Exclude candidates[cand_idx]
            search(
                cand_idx + 1,
                current_selection,
                covered,
                targets,
                candidates,
                best_solution,
                best_implicant_count,
                best_literal_count,
            );
        }

        let mut current_selection = Vec::new();
        let mut covered = BTreeSet::new();

        search(
            0,
            &mut current_selection,
            &mut covered,
            targets,
            candidates,
            &mut best_solution,
            &mut best_implicant_count,
            &mut best_literal_count,
        );

        best_solution.unwrap_or_default()
    }
}
