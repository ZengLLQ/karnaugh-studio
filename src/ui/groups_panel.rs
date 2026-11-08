use crate::domain::implicant::Implicant;
use crate::ui::theme::Theme;
use egui::{CornerRadius, Margin, RichText, ScrollArea, Stroke, Ui};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupsResponse {
    pub selected_group: Option<usize>,
    pub hovered_group: Option<usize>,
}

pub struct GroupsPanel;

impl GroupsPanel {
    pub fn show(
        ui: &mut Ui,
        theme: &Theme,
        groups: &[Implicant],
        is_sop: bool,
        hovered_group: Option<usize>,
        selected_group: Option<usize>,
    ) -> GroupsResponse {
        let mut selected_group_action = None;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("GROUPS & IMPLICANTS")
                    .size(15.0)
                    .color(theme.primary_text)
                    .strong(),
            );
            ui.label(
                RichText::new(format!("({} loops)", groups.len()))
                    .size(12.0)
                    .color(theme.muted_text),
            );
        });

        ui.add_space(4.0);

        if groups.is_empty() {
            egui::Frame::new()
                .fill(theme.surface_subtle)
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("No groups formed (function is constant or table invalid).")
                            .size(12.0)
                            .color(theme.muted_text)
                            .italics(),
                    );
                });
            return GroupsResponse {
                selected_group: None,
                hovered_group: None,
            };
        }

        let mut current_hover = None;

        ScrollArea::vertical()
            .max_height(240.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (idx, group) in groups.iter().enumerate() {
                    let color = theme.get_group_color(idx);
                    let is_active = hovered_group == Some(idx) || selected_group == Some(idx);

                    let term_str = if is_sop {
                        group.to_sop_term()
                    } else {
                        group.to_pos_term()
                    };

                    let bg_color = if is_active {
                        theme.surface_hover
                    } else if idx % 2 == 1 {
                        theme.surface_subtle.gamma_multiply(0.5)
                    } else {
                        theme.surface_color
                    };

                    let stroke = if is_active {
                        Stroke::new(1.5_f32, color)
                    } else {
                        Stroke::new(1.0_f32, theme.border_subtle)
                    };

                    let response = egui::Frame::new()
                        .fill(bg_color)
                        .stroke(stroke)
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            // Row 1: Badge, Title, Result Term, and Essential Pill
                            ui.horizontal(|ui| {
                                // Color Dot
                                ui.label(RichText::new("●").size(16.0).color(color));

                                // Group Title
                                ui.label(
                                    RichText::new(format!("Group {}", idx + 1))
                                        .size(13.0)
                                        .color(theme.primary_text)
                                        .strong(),
                                );

                                ui.label(
                                    RichText::new(format!(
                                        "· {}",
                                        group.minterm_list_string(is_sop)
                                    ))
                                    .size(12.0)
                                    .color(theme.muted_text)
                                    .monospace(),
                                );

                                if group.is_essential {
                                    ui.add_space(4.0);
                                    egui::Frame::new()
                                        .fill(theme.primary_indigo.gamma_multiply(0.15))
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new("Essential")
                                                    .size(10.0)
                                                    .color(theme.primary_indigo)
                                                    .strong(),
                                            );
                                        });
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(&term_str)
                                                .size(14.0)
                                                .color(color)
                                                .strong()
                                                .monospace(),
                                        );
                                        ui.label(
                                            RichText::new("Result:")
                                                .size(11.0)
                                                .color(theme.muted_text),
                                        );
                                    },
                                );
                            });

                            ui.add_space(2.0);

                            // Row 2: Educational details
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("Size: {}", group.size()))
                                        .size(11.0)
                                        .color(theme.muted_text),
                                );
                                ui.label(RichText::new("•").size(9.0).color(theme.border_color));
                                ui.label(
                                    RichText::new(if group.is_wrap {
                                        "Wraps around torus"
                                    } else {
                                        "Planar (no wrap)"
                                    })
                                    .size(11.0)
                                    .color(if group.is_wrap {
                                        theme.accent_teal
                                    } else {
                                        theme.muted_text
                                    }),
                                );
                                ui.label(RichText::new("•").size(9.0).color(theme.border_color));
                                ui.label(
                                    RichText::new(format!(
                                        "Eliminates: {}",
                                        group.eliminated_vars_string()
                                    ))
                                    .size(11.0)
                                    .color(theme.muted_text),
                                );
                            });
                        })
                        .response;

                    if response.hovered() {
                        current_hover = Some(idx);
                    }
                    if response.clicked() {
                        selected_group_action = Some(idx);
                    }
                    ui.add_space(2.0);
                }
            });

        GroupsResponse {
            selected_group: selected_group_action,
            hovered_group: current_hover,
        }
    }
}
