use crate::domain::model::{CellValue, TruthTable};
use crate::domain::validation::TableValidation;
use crate::ui::theme::Theme;
use egui::{Align, CornerRadius, Layout, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TruthTableAction {
    None,
    Edited,
    RequestCanonical,
    SetAllOutputs(CellValue),
    RowSelected(Option<usize>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthTableResponse {
    pub action: TruthTableAction,
    pub hovered_row: Option<usize>,
}

pub struct TruthTableView;

impl TruthTableView {
    pub fn show(
        ui: &mut Ui,
        theme: &Theme,
        table: &mut TruthTable,
        validation: &TableValidation,
        hovered_row: Option<usize>,
        selected_row: Option<usize>,
        hovered_minterm: Option<usize>,
    ) -> TruthTableResponse {
        let mut action = TruthTableAction::None;
        let var_names = table.var_count.var_names();
        let n_vars = table.var_count.as_usize();

        // 1. Header and Action Toolbar
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("TRUTH TABLE")
                    .size(15.0)
                    .color(theme.primary_text)
                    .strong(),
            );
            ui.label(
                RichText::new(format!("({} rows)", table.rows.len()))
                    .size(12.0)
                    .color(theme.muted_text),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button(RichText::new("All X").size(11.0))
                    .on_hover_text("Set all outputs to Don't-Care (X)")
                    .clicked()
                {
                    action = TruthTableAction::SetAllOutputs(CellValue::DontCare);
                }
                if ui
                    .button(RichText::new("All 1").size(11.0))
                    .on_hover_text("Set all outputs to 1")
                    .clicked()
                {
                    action = TruthTableAction::SetAllOutputs(CellValue::One);
                }
                if ui
                    .button(RichText::new("All 0").size(11.0))
                    .on_hover_text("Set all outputs to 0")
                    .clicked()
                {
                    action = TruthTableAction::SetAllOutputs(CellValue::Zero);
                }
                if ui
                    .button(RichText::new("Canonical Rows").size(11.0))
                    .on_hover_text("Fill standard binary order 0..2^n-1")
                    .clicked()
                {
                    action = TruthTableAction::RequestCanonical;
                }
            });
        });

        ui.add_space(4.0);

        // 2. Table Column Headers
        egui::Frame::new()
            .fill(theme.surface_subtle)
            .corner_radius(CornerRadius::same(4))
            .stroke(Stroke::new(1.0_f32, theme.border_subtle))
            .inner_margin(Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [28.0, 18.0],
                        egui::Label::new(RichText::new("#").size(11.0).color(theme.muted_text)),
                    );
                    for name in &var_names {
                        ui.add_sized(
                            [32.0, 18.0],
                            egui::Label::new(
                                RichText::new(*name)
                                    .size(13.0)
                                    .color(theme.primary_text)
                                    .strong(),
                            ),
                        );
                    }
                    ui.label(RichText::new("│").color(theme.border_color));
                    ui.add_sized(
                        [38.0, 18.0],
                        egui::Label::new(
                            RichText::new("F")
                                .size(13.0)
                                .color(theme.primary_indigo)
                                .strong(),
                        ),
                    );
                    ui.label(
                        RichText::new("Minterm / Status")
                            .size(11.0)
                            .color(theme.muted_text),
                    );
                });
            });

        ui.add_space(2.0);

        // 3. Scrollable Table Rows
        let row_height = 28.0;
        let mut row_now_hovered = None;

        ScrollArea::vertical()
            .max_height(260.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (row_idx, row) in table.rows.iter_mut().enumerate() {
                    let has_errors = validation.get_row_errors(row_idx).is_some();
                    let row_minterm = validation.row_minterm(row_idx);

                    let is_active_hover = hovered_row == Some(row_idx)
                        || (row_minterm.is_some() && row_minterm == hovered_minterm);
                    let is_selected = selected_row == Some(row_idx);

                    let bg_color = if has_errors {
                        theme.error_bg
                    } else if is_selected {
                        theme.surface_hover
                    } else if is_active_hover {
                        theme.surface_subtle
                    } else if row_idx % 2 == 1 {
                        theme.surface_subtle.gamma_multiply(0.4)
                    } else {
                        theme.surface_color
                    };

                    let stroke = if has_errors {
                        Stroke::new(1.0_f32, theme.error_color)
                    } else if is_active_hover || is_selected {
                        Stroke::new(1.0_f32, theme.primary_indigo)
                    } else {
                        Stroke::new(0.5_f32, theme.border_subtle)
                    };

                    let response = egui::Frame::new()
                        .fill(bg_color)
                        .stroke(stroke)
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(8, 2))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Row number
                                ui.add_sized(
                                    [28.0, row_height - 6.0],
                                    egui::Label::new(
                                        RichText::new(format!("{}", row_idx + 1))
                                            .size(11.0)
                                            .color(theme.muted_text),
                                    ),
                                );

                                // Input columns
                                while row.inputs.len() < n_vars {
                                    row.inputs.push("0".to_string());
                                }
                                for col in 0..n_vars {
                                    let mut input_text = row.inputs[col].clone();
                                    let edit = ui.add_sized(
                                        [32.0, row_height - 6.0],
                                        TextEdit::singleline(&mut input_text)
                                            .char_limit(1)
                                            .horizontal_align(Align::Center)
                                            .font(egui::TextStyle::Monospace),
                                    );
                                    if edit.changed() {
                                        row.inputs[col] = input_text;
                                        action = TruthTableAction::Edited;
                                    }
                                    if edit.secondary_clicked() {
                                        // Quick toggle 0 <-> 1 on right click
                                        let current = row.inputs[col].trim();
                                        row.inputs[col] = if current == "1" {
                                            "0".to_string()
                                        } else {
                                            "1".to_string()
                                        };
                                        action = TruthTableAction::Edited;
                                    }
                                    if edit.clicked() {
                                        action = TruthTableAction::RowSelected(Some(row_idx));
                                    }
                                }

                                ui.label(RichText::new("│").color(theme.border_color));

                                // Output column F
                                let mut out_text = row.output.clone();
                                let out_color = if out_text == "1" {
                                    theme.primary_indigo
                                } else if out_text.eq_ignore_ascii_case("x") {
                                    theme.accent_teal
                                } else {
                                    theme.primary_text
                                };
                                let out_edit = ui.add_sized(
                                    [38.0, row_height - 6.0],
                                    TextEdit::singleline(&mut out_text)
                                        .char_limit(1)
                                        .horizontal_align(Align::Center)
                                        .font(egui::TextStyle::Monospace)
                                        .text_color(out_color),
                                );
                                if out_edit.changed() {
                                    row.output = out_text.to_uppercase();
                                    action = TruthTableAction::Edited;
                                }

                                // Quick cycle button for F: [0 -> 1 -> X -> 0]
                                let cycle_btn =
                                    ui.small_button("↻").on_hover_text("Toggle 0 → 1 → X");
                                if cycle_btn.clicked() {
                                    let next_val = match CellValue::parse(&row.output) {
                                        Some(CellValue::Zero) => "1",
                                        Some(CellValue::One) => "X",
                                        Some(CellValue::DontCare) | None => "0",
                                    };
                                    row.output = next_val.to_string();
                                    action = TruthTableAction::Edited;
                                }

                                // Minterm / Validation status
                                if let Some(errors) = validation.get_row_errors(row_idx) {
                                    let err_msg = errors
                                        .iter()
                                        .map(|e| e.message(&var_names))
                                        .collect::<Vec<_>>()
                                        .join("; ");
                                    ui.label(
                                        RichText::new(format!("⚠ {err_msg}"))
                                            .size(11.0)
                                            .color(theme.error_color),
                                    );
                                } else if let Some(m) = row_minterm {
                                    ui.label(
                                        RichText::new(format!("m{m}"))
                                            .size(11.0)
                                            .color(theme.muted_text)
                                            .monospace(),
                                    );
                                }
                            });
                        })
                        .response;

                    if response.hovered() {
                        row_now_hovered = Some(row_idx);
                    }
                    if response.clicked() {
                        action = TruthTableAction::RowSelected(Some(row_idx));
                    }
                }
            });

        ui.add_space(4.0);

        // 4. Validation Summary Bar
        egui::Frame::new()
            .fill(if validation.is_valid {
                theme.success_bg
            } else {
                theme.error_bg
            })
            .stroke(Stroke::new(
                1.0_f32,
                if validation.is_valid {
                    theme.success_color
                } else {
                    theme.error_color
                },
            ))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if validation.is_valid {
                        ui.label(
                            RichText::new("✓ Valid Truth Table")
                                .size(12.0)
                                .color(theme.success_color)
                                .strong(),
                        );
                        ui.label(
                            RichText::new("· Complete 2^n space · Minimization active")
                                .size(11.0)
                                .color(theme.muted_text),
                        );
                    } else {
                        ui.label(
                            RichText::new("⚠ Table Invalid")
                                .size(12.0)
                                .color(theme.error_color)
                                .strong(),
                        );
                        let summary = validation.summary_errors.join(" | ");
                        ui.label(
                            RichText::new(format!("· {summary} · Minimization disabled"))
                                .size(11.0)
                                .color(theme.error_color),
                        );
                    }
                });
            });

        TruthTableResponse {
            action,
            hovered_row: row_now_hovered,
        }
    }
}
