use crate::domain::model::VarCount;
use crate::ui::theme::{Theme, ThemeMode};
use egui::{Align, Layout, RichText, Ui};

pub enum ToolbarAction {
    None,
    ChangeVarCount(VarCount),
    ToggleTheme,
    RequestReset,
}

pub struct Toolbar;

impl Toolbar {
    pub fn show(ui: &mut Ui, theme: &Theme, current_vars: VarCount) -> ToolbarAction {
        let mut action = ToolbarAction::None;

        ui.horizontal(|ui| {
            // App Logo & Title
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("◫")
                        .size(24.0)
                        .color(theme.accent_teal)
                        .strong(),
                );
                ui.label(
                    RichText::new("Karnaugh Studio")
                        .size(19.0)
                        .color(theme.primary_text)
                        .strong(),
                );
                ui.label(RichText::new("v1.0").size(11.0).color(theme.muted_text));
            });

            ui.add_space(24.0);

            // Variable Count Selector
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Variables:")
                        .size(13.0)
                        .color(theme.muted_text),
                );

                for &vc in &[VarCount::Two, VarCount::Three, VarCount::Four] {
                    let is_selected = current_vars == vc;
                    let label = format!("{} Vars", vc.as_usize());
                    let btn = ui.selectable_label(is_selected, RichText::new(label).strong());
                    if btn.clicked() && !is_selected {
                        action = ToolbarAction::ChangeVarCount(vc);
                    }
                }
            });

            // Right-aligned controls
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Reset Button
                let reset_btn = ui.button(RichText::new("↺ Reset").color(theme.error_color));
                if reset_btn.clicked() {
                    action = ToolbarAction::RequestReset;
                }

                ui.add_space(8.0);

                // Theme Toggle
                let theme_label = match theme.mode {
                    ThemeMode::Light => "🌙 Dark",
                    ThemeMode::Dark => "☀ Light",
                };
                let theme_btn = ui.button(RichText::new(theme_label));
                if theme_btn.clicked() {
                    action = ToolbarAction::ToggleTheme;
                }
            });
        });

        action
    }
}
