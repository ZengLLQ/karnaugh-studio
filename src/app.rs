use crate::domain::minimizer::{MinimizationResult, Minimizer};
use crate::domain::model::{CellValue, TruthTable, VarCount};
use crate::domain::validation::TableValidation;
use crate::ui::function_panel::{FunctionAction, FunctionPanel};
use crate::ui::groups_panel::GroupsPanel;
use crate::ui::kmap_view::KMapView;
use crate::ui::theme::{Theme, ThemeMode};
use crate::ui::toolbar::{Toolbar, ToolbarAction};
use crate::ui::truth_table::{TruthTableAction, TruthTableView};
use eframe::App;
use egui::{
    Align, CentralPanel, Color32, Context, CornerRadius, Frame, Layout, Margin, RichText, Stroke,
    TopBottomPanel, Window, vec2,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    GenerateCanonical,
    Reset,
    ChangeVarCount(VarCount),
}

pub struct KarnaughStudioApp {
    pub table: TruthTable,
    pub validation: TableValidation,
    pub sop_result: MinimizationResult,
    pub pos_result: MinimizationResult,
    pub theme: Theme,
    pub is_showing_sop: bool,

    // Hover & Selection Cross-linking
    pub hovered_row: Option<usize>,
    pub selected_row: Option<usize>,
    pub hovered_minterm: Option<usize>,
    pub selected_minterm: Option<usize>,
    pub hovered_group: Option<usize>,
    pub selected_group: Option<usize>,

    // Clipboard notification
    pub toast_message: Option<(String, Instant)>,

    // Confirmation dialog
    pub pending_confirm: Option<ConfirmAction>,
}

impl Default for KarnaughStudioApp {
    fn default() -> Self {
        let mut app = Self {
            table: TruthTable::new(VarCount::Three),
            validation: TableValidation {
                is_valid: false,
                row_errors: HashMap::new(),
                missing_minterms: Vec::new(),
                duplicate_minterm_count: 0,
                minterm_map: HashMap::new(),
                row_to_minterm: HashMap::new(),
                summary_errors: Vec::new(),
            },
            sop_result: MinimizationResult::empty(true),
            pos_result: MinimizationResult::empty(false),
            theme: Theme::light(),
            is_showing_sop: true,
            hovered_row: None,
            selected_row: None,
            hovered_minterm: None,
            selected_minterm: None,
            hovered_group: None,
            selected_group: None,
            toast_message: None,
            pending_confirm: None,
        };
        app.recalculate();
        app
    }
}

impl KarnaughStudioApp {
    pub fn recalculate(&mut self) {
        self.validation = TableValidation::validate(&self.table);

        if self.validation.is_valid {
            let minterms: HashMap<usize, CellValue> = self
                .validation
                .minterm_map
                .iter()
                .map(|(&m, &(_, cv))| (m, cv))
                .collect();

            self.sop_result = Minimizer::minimize_sop(self.table.var_count, &minterms);
            self.pos_result = Minimizer::minimize_pos(self.table.var_count, &minterms);
        } else {
            self.sop_result = MinimizationResult::empty(true);
            self.pos_result = MinimizationResult::empty(false);
        }
    }

    pub fn set_toast(&mut self, msg: &str) {
        self.toast_message = Some((msg.to_string(), Instant::now()));
    }
}

impl App for KarnaughStudioApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.theme.apply_to_ctx(ctx);

        // 1. Top Toolbar
        TopBottomPanel::top("top_panel")
            .frame(
                Frame::new()
                    .fill(self.theme.surface_color)
                    .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                    .inner_margin(Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                let action = Toolbar::show(ui, &self.theme, self.table.var_count);
                match action {
                    ToolbarAction::None => {}
                    ToolbarAction::ChangeVarCount(new_vc) => {
                        if self.table.is_dirty() {
                            self.pending_confirm = Some(ConfirmAction::ChangeVarCount(new_vc));
                        } else {
                            self.table.set_var_count(new_vc);
                            self.recalculate();
                        }
                    }
                    ToolbarAction::ToggleTheme => {
                        self.theme = match self.theme.mode {
                            ThemeMode::Light => Theme::dark(),
                            ThemeMode::Dark => Theme::light(),
                        };
                    }
                    ToolbarAction::RequestReset => {
                        if self.table.is_dirty() {
                            self.pending_confirm = Some(ConfirmAction::Reset);
                        } else {
                            self.table.reset();
                            self.recalculate();
                        }
                    }
                }
            });

        // 2. Central Panel with 2x2 Responsive Academic Layout
        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(self.theme.bg_color)
                    .inner_margin(Margin::same(16)),
            )
            .show(ctx, |ui| {
                let available_w = ui.available_width();
                let left_w = (available_w * 0.44).clamp(360.0, 520.0);
                let right_w = (available_w - left_w - 16.0).max(360.0);

                ui.horizontal(|ui| {
                    // Left Column: Truth Table & Groups
                    ui.allocate_ui_with_layout(
                        vec2(left_w, ui.available_height()),
                        Layout::top_down(Align::LEFT),
                        |ui| {
                            // Top Left: Truth Table Panel
                            Frame::new()
                                .fill(self.theme.surface_color)
                                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(ui, |ui| {
                                    let tt_res = TruthTableView::show(
                                        ui,
                                        &self.theme,
                                        &mut self.table,
                                        &self.validation,
                                        self.hovered_row,
                                        self.selected_row,
                                        self.hovered_minterm,
                                    );
                                    if tt_res.hovered_row != self.hovered_row {
                                        self.hovered_row = tt_res.hovered_row;
                                        if let Some(row_idx) = tt_res.hovered_row {
                                            self.hovered_minterm =
                                                self.validation.row_minterm(row_idx);
                                        } else {
                                            self.hovered_minterm = None;
                                        }
                                    }
                                    match tt_res.action {
                                        TruthTableAction::None => {}
                                        TruthTableAction::Edited => {
                                            self.recalculate();
                                        }
                                        TruthTableAction::RequestCanonical => {
                                            if self.table.is_dirty() {
                                                self.pending_confirm =
                                                    Some(ConfirmAction::GenerateCanonical);
                                            } else {
                                                self.table.populate_canonical(CellValue::Zero);
                                                self.recalculate();
                                            }
                                        }
                                        TruthTableAction::SetAllOutputs(cv) => {
                                            self.table.set_all_outputs(cv);
                                            self.recalculate();
                                        }
                                        TruthTableAction::RowSelected(r) => {
                                            self.selected_row = r;
                                            if let Some(row_idx) = r {
                                                self.selected_minterm =
                                                    self.validation.row_minterm(row_idx);
                                            } else {
                                                self.selected_minterm = None;
                                            }
                                        }
                                    }
                                });

                            ui.add_space(12.0);

                            // Bottom Left: Groups & Implicants Panel
                            Frame::new()
                                .fill(self.theme.surface_color)
                                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(ui, |ui| {
                                    let active_groups = if self.is_showing_sop {
                                        &self.sop_result.groups
                                    } else {
                                        &self.pos_result.groups
                                    };

                                    let g_res = GroupsPanel::show(
                                        ui,
                                        &self.theme,
                                        active_groups,
                                        self.is_showing_sop,
                                        self.hovered_group,
                                        self.selected_group,
                                    );
                                    if g_res.hovered_group != self.hovered_group {
                                        self.hovered_group = g_res.hovered_group;
                                    }
                                    if let Some(g) = g_res.selected_group {
                                        self.selected_group = if self.selected_group == Some(g) {
                                            None
                                        } else {
                                            Some(g)
                                        };
                                    }
                                });
                        },
                    );

                    ui.add_space(16.0);

                    // Right Column: Karnaugh Map & Minimized Function
                    ui.allocate_ui_with_layout(
                        vec2(right_w, ui.available_height()),
                        Layout::top_down(Align::LEFT),
                        |ui| {
                            // Top Right: Karnaugh Map Panel
                            Frame::new()
                                .fill(self.theme.surface_color)
                                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(ui, |ui| {
                                    let active_groups = if self.is_showing_sop {
                                        &self.sop_result.groups
                                    } else {
                                        &self.pos_result.groups
                                    };

                                    let minterm_values: HashMap<usize, CellValue> = self
                                        .validation
                                        .minterm_map
                                        .iter()
                                        .map(|(&m, &(_, cv))| (m, cv))
                                        .collect();

                                    let km_res = KMapView::show(
                                        ui,
                                        &self.theme,
                                        self.table.var_count,
                                        &minterm_values,
                                        active_groups,
                                        self.hovered_minterm,
                                        self.selected_minterm,
                                        self.hovered_group,
                                    );
                                    if km_res.hovered_minterm != self.hovered_minterm {
                                        self.hovered_minterm = km_res.hovered_minterm;
                                        if let Some(minterm) = km_res.hovered_minterm {
                                            if let Some(&(row_idx, _)) =
                                                self.validation.minterm_map.get(&minterm)
                                            {
                                                self.hovered_row = Some(row_idx);
                                            }
                                        } else {
                                            self.hovered_row = None;
                                        }
                                    }
                                    if km_res.hovered_group != self.hovered_group {
                                        self.hovered_group = km_res.hovered_group;
                                    }
                                    if let Some(m) = km_res.clicked_minterm {
                                        self.selected_minterm = Some(m);
                                        if let Some(&(row_idx, current_cv)) =
                                            self.validation.minterm_map.get(&m)
                                        {
                                            self.selected_row = Some(row_idx);
                                            let next_val = match current_cv {
                                                CellValue::Zero => "1",
                                                CellValue::One => "X",
                                                CellValue::DontCare => "0",
                                            };
                                            self.table.rows[row_idx].output = next_val.to_string();
                                            self.recalculate();
                                        }
                                    }
                                });

                            ui.add_space(12.0);

                            // Bottom Right: Minimized Function Panel
                            Frame::new()
                                .fill(self.theme.surface_color)
                                .stroke(Stroke::new(1.0_f32, self.theme.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(ui, |ui| {
                                    let fn_res = FunctionPanel::show(
                                        ui,
                                        &self.theme,
                                        &self.sop_result.expression,
                                        &self.sop_result.groups,
                                        &self.pos_result.expression,
                                        &self.pos_result.groups,
                                        self.is_showing_sop,
                                        self.hovered_group,
                                        self.selected_group,
                                        self.validation.is_valid,
                                    );
                                    if fn_res.hovered_group != self.hovered_group {
                                        self.hovered_group = fn_res.hovered_group;
                                    }
                                    if let Some(g) = fn_res.selected_group {
                                        self.selected_group = if self.selected_group == Some(g) {
                                            None
                                        } else {
                                            Some(g)
                                        };
                                    }
                                    match fn_res.action {
                                        FunctionAction::None => {}
                                        FunctionAction::CopySop(expr) => {
                                            ctx.copy_text(expr.clone());
                                            self.set_toast(&format!("Copied SoP: {expr}"));
                                        }
                                        FunctionAction::CopyPos(expr) => {
                                            ctx.copy_text(expr.clone());
                                            self.set_toast(&format!("Copied PoS: {expr}"));
                                        }
                                        FunctionAction::ToggleTarget(is_sop) => {
                                            self.is_showing_sop = is_sop;
                                            self.hovered_group = None;
                                            self.selected_group = None;
                                        }
                                    }
                                });
                        },
                    );
                });
            });

        // 3. Toast Notification Overlay
        if let Some((msg, time)) = &self.toast_message {
            if time.elapsed() < Duration::from_secs(3) {
                Window::new("Notification")
                    .title_bar(false)
                    .resizable(false)
                    .anchor(egui::Align2::RIGHT_BOTTOM, vec2(-24.0, -24.0))
                    .frame(
                        Frame::new()
                            .fill(self.theme.primary_indigo)
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(14, 10)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("✓").color(Color32::WHITE).strong());
                            ui.label(RichText::new(msg).color(Color32::WHITE).strong());
                        });
                    });
            } else {
                self.toast_message = None;
            }
        }

        // 4. Confirmation Dialog
        if let Some(action) = self.pending_confirm {
            let mut close_dialog = false;
            let mut proceed = false;

            Window::new("Confirm Overwrite")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .frame(
                    Frame::new()
                        .fill(self.theme.surface_color)
                        .stroke(Stroke::new(1.5_f32, self.theme.primary_indigo))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(20)),
                )
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("⚠ Unsaved Table Edits")
                                .size(16.0)
                                .color(self.theme.error_color)
                                .strong(),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "You have edited truth-table entries. Proceeding will overwrite your custom values.",
                            )
                            .size(13.0)
                            .color(self.theme.primary_text),
                        );
                        ui.add_space(16.0);

                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Cancel").size(13.0)).clicked() {
                                close_dialog = true;
                            }
                            ui.add_space(12.0);
                            if ui
                                .button(
                                    RichText::new("Overwrite")
                                        .size(13.0)
                                        .color(self.theme.error_color)
                                        .strong(),
                                )
                                .clicked()
                            {
                                proceed = true;
                                close_dialog = true;
                            }
                        });
                    });
                });

            if proceed {
                match action {
                    ConfirmAction::GenerateCanonical => {
                        self.table.populate_canonical(CellValue::Zero);
                        self.recalculate();
                    }
                    ConfirmAction::Reset => {
                        self.table.reset();
                        self.recalculate();
                    }
                    ConfirmAction::ChangeVarCount(vc) => {
                        self.table.set_var_count(vc);
                        self.recalculate();
                    }
                }
            }

            if close_dialog {
                self.pending_confirm = None;
            }
        }
    }
}
