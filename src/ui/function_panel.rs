use crate::domain::implicant::Implicant;
use crate::ui::theme::Theme;
use egui::{Align, CornerRadius, Layout, Margin, RichText, Stroke, Ui};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionAction {
    None,
    CopySop(String),
    CopyPos(String),
    ToggleTarget(bool), // true = SoP, false = PoS
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionResponse {
    pub action: FunctionAction,
    pub hovered_group: Option<usize>,
    pub selected_group: Option<usize>,
}

pub struct FunctionPanel;

impl FunctionPanel {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &Theme,
        sop_expr: &str,
        sop_groups: &[Implicant],
        pos_expr: &str,
        pos_groups: &[Implicant],
        is_showing_sop: bool,
        hovered_group: Option<usize>,
        selected_group: Option<usize>,
        is_valid: bool,
    ) -> FunctionResponse {
        let mut action = FunctionAction::None;
        let mut hovered_g = None;
        let mut selected_g = None;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("MINIMIZED FUNCTION")
                    .size(15.0)
                    .color(theme.primary_text)
                    .strong(),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let pos_btn =
                    ui.selectable_label(!is_showing_sop, RichText::new("PoS View").size(11.0));
                if pos_btn.clicked() && is_showing_sop {
                    action = FunctionAction::ToggleTarget(false);
                }
                let sop_btn =
                    ui.selectable_label(is_showing_sop, RichText::new("SoP View").size(11.0));
                if sop_btn.clicked() && !is_showing_sop {
                    action = FunctionAction::ToggleTarget(true);
                }
                ui.label(
                    RichText::new("K-Map Loop Target:")
                        .size(11.0)
                        .color(theme.muted_text),
                );
            });
        });

        ui.add_space(4.0);

        if !is_valid {
            egui::Frame::new()
                .fill(theme.error_bg)
                .stroke(Stroke::new(1.0_f32, theme.error_color))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("⚠ Minimization is disabled while the truth table is incomplete or contains errors.")
                            .size(12.0)
                            .color(theme.error_color),
                    );
                });
            return FunctionResponse {
                action: FunctionAction::None,
                hovered_group: None,
                selected_group: None,
            };
        }

        // 1. SoP Card
        let is_sop_active = is_showing_sop;
        let sop_border = if is_sop_active {
            Stroke::new(1.5_f32, theme.primary_indigo)
        } else {
            Stroke::new(1.0_f32, theme.border_subtle)
        };

        egui::Frame::new()
            .fill(theme.surface_color)
            .stroke(sop_border)
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("SoP (Sum of Products):")
                            .size(12.0)
                            .color(theme.muted_text)
                            .strong(),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(RichText::new("📋 Copy SoP").size(11.0)).clicked() {
                            action = FunctionAction::CopySop(sop_expr.to_string());
                        }
                    });
                });

                ui.add_space(4.0);

                // Render colored terms
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("F = ")
                            .size(16.0)
                            .color(theme.primary_text)
                            .strong()
                            .monospace(),
                    );

                    if sop_groups.is_empty() {
                        ui.label(
                            RichText::new(sop_expr)
                                .size(16.0)
                                .color(theme.primary_text)
                                .strong()
                                .monospace(),
                        );
                    } else {
                        for (idx, group) in sop_groups.iter().enumerate() {
                            if idx > 0 {
                                ui.label(
                                    RichText::new(" + ")
                                        .size(16.0)
                                        .color(theme.muted_text)
                                        .monospace(),
                                );
                            }

                            let color = theme.get_group_color(idx);
                            let is_highlighted = is_showing_sop
                                && (hovered_group == Some(idx) || selected_group == Some(idx));

                            let term_text = group.to_sop_term();
                            let term_resp = ui.selectable_label(
                                is_highlighted,
                                RichText::new(term_text)
                                    .size(16.0)
                                    .color(color)
                                    .strong()
                                    .monospace(),
                            );

                            if is_showing_sop {
                                if term_resp.hovered() {
                                    hovered_g = Some(idx);
                                }
                                if term_resp.clicked() {
                                    selected_g = Some(idx);
                                }
                            }
                        }
                    }
                });
            });

        ui.add_space(6.0);

        // 2. PoS Card
        let is_pos_active = !is_showing_sop;
        let pos_border = if is_pos_active {
            Stroke::new(1.5_f32, theme.accent_teal)
        } else {
            Stroke::new(1.0_f32, theme.border_subtle)
        };

        egui::Frame::new()
            .fill(theme.surface_color)
            .stroke(pos_border)
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("PoS (Product of Sums):")
                            .size(12.0)
                            .color(theme.muted_text)
                            .strong(),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(RichText::new("📋 Copy PoS").size(11.0)).clicked() {
                            action = FunctionAction::CopyPos(pos_expr.to_string());
                        }
                    });
                });

                ui.add_space(4.0);

                // Render colored maxterms
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("F = ")
                            .size(16.0)
                            .color(theme.primary_text)
                            .strong()
                            .monospace(),
                    );

                    if pos_groups.is_empty() {
                        ui.label(
                            RichText::new(pos_expr)
                                .size(16.0)
                                .color(theme.primary_text)
                                .strong()
                                .monospace(),
                        );
                    } else {
                        for (idx, group) in pos_groups.iter().enumerate() {
                            let color = theme.get_group_color(idx);
                            let is_highlighted = !is_showing_sop
                                && (hovered_group == Some(idx) || selected_group == Some(idx));

                            let factor_text = group.to_pos_term();
                            let term_resp = ui.selectable_label(
                                is_highlighted,
                                RichText::new(factor_text)
                                    .size(16.0)
                                    .color(color)
                                    .strong()
                                    .monospace(),
                            );

                            if !is_showing_sop {
                                if term_resp.hovered() {
                                    hovered_g = Some(idx);
                                }
                                if term_resp.clicked() {
                                    selected_g = Some(idx);
                                }
                            }
                        }
                    }
                });
            });

        ui.add_space(4.0);

        // Verification & Minimization Metric Indicator
        ui.horizontal(|ui| {
            let active_groups = if is_showing_sop {
                sop_groups
            } else {
                pos_groups
            };
            let total_literals: usize = active_groups.iter().map(|g| g.literal_count()).sum();

            ui.label(
                RichText::new(format!(
                    "Optimal Cover: {} implicants · {} literals",
                    active_groups.len(),
                    total_literals
                ))
                .size(11.0)
                .color(theme.muted_text),
            );
        });

        FunctionResponse {
            action,
            hovered_group: hovered_g,
            selected_group: selected_g,
        }
    }
}
