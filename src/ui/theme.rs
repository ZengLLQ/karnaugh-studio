use egui::{Color32, Context, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub bg_color: Color32,
    pub surface_color: Color32,
    pub surface_subtle: Color32,
    pub surface_hover: Color32,
    pub primary_text: Color32,
    pub muted_text: Color32,
    pub border_color: Color32,
    pub border_subtle: Color32,
    pub primary_indigo: Color32,
    pub accent_teal: Color32,
    pub error_color: Color32,
    pub error_bg: Color32,
    pub success_color: Color32,
    pub success_bg: Color32,
    pub group_colors: Vec<Color32>,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            bg_color: Color32::from_rgb(0xF7, 0xF8, 0xFC),
            surface_color: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            surface_subtle: Color32::from_rgb(0xF1, 0xF5, 0xF9),
            surface_hover: Color32::from_rgb(0xE2, 0xE8, 0xF0),
            primary_text: Color32::from_rgb(0x18, 0x23, 0x3B),
            muted_text: Color32::from_rgb(0x64, 0x74, 0x8B),
            border_color: Color32::from_rgb(0xCB, 0xD5, 0xE1),
            border_subtle: Color32::from_rgb(0xE2, 0xE8, 0xF0),
            primary_indigo: Color32::from_rgb(0x4F, 0x46, 0xE5),
            accent_teal: Color32::from_rgb(0x0F, 0xAF, 0x9A),
            error_color: Color32::from_rgb(0xD9, 0x2D, 0x20),
            error_bg: Color32::from_rgb(0xFE, 0xF2, 0xF2),
            success_color: Color32::from_rgb(0x16, 0xA3, 0x4A),
            success_bg: Color32::from_rgb(0xF0, 0xFD, 0xF4),
            group_colors: vec![
                Color32::from_rgb(0x8B, 0x5C, 0xF6), // Violet
                Color32::from_rgb(0x3B, 0x82, 0xF6), // Blue
                Color32::from_rgb(0xD9, 0x77, 0x06), // Amber
                Color32::from_rgb(0xEA, 0x58, 0x0C), // Orange / Coral
                Color32::from_rgb(0x05, 0x96, 0x69), // Emerald
                Color32::from_rgb(0xDB, 0x27, 0x77), // Rose
                Color32::from_rgb(0x08, 0x91, 0xB2), // Cyan
                Color32::from_rgb(0x4F, 0x46, 0xE5), // Indigo
            ],
        }
    }

    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            bg_color: Color32::from_rgb(0x0F, 0x17, 0x2A), // Slate 900
            surface_color: Color32::from_rgb(0x1E, 0x29, 0x3B), // Slate 800
            surface_subtle: Color32::from_rgb(0x33, 0x41, 0x55), // Slate 700
            surface_hover: Color32::from_rgb(0x47, 0x55, 0x69),
            primary_text: Color32::from_rgb(0xF8, 0xFA, 0xFC),
            muted_text: Color32::from_rgb(0x94, 0xA3, 0xB8),
            border_color: Color32::from_rgb(0x47, 0x55, 0x69),
            border_subtle: Color32::from_rgb(0x33, 0x41, 0x55),
            primary_indigo: Color32::from_rgb(0x63, 0x66, 0xF1),
            accent_teal: Color32::from_rgb(0x2D, 0xD4, 0xBF),
            error_color: Color32::from_rgb(0xF8, 0x71, 0x71),
            error_bg: Color32::from_rgb(0x45, 0x1A, 0x1A),
            success_color: Color32::from_rgb(0x4A, 0xDE, 0x80),
            success_bg: Color32::from_rgb(0x14, 0x53, 0x2D),
            group_colors: vec![
                Color32::from_rgb(0xA7, 0x8B, 0xFA), // Violet
                Color32::from_rgb(0x60, 0xA5, 0xFA), // Blue
                Color32::from_rgb(0xFB, 0xBF, 0x24), // Amber
                Color32::from_rgb(0xFB, 0x92, 0x3C), // Coral
                Color32::from_rgb(0x34, 0xD3, 0x99), // Emerald
                Color32::from_rgb(0xF4, 0x72, 0xB6), // Rose
                Color32::from_rgb(0x22, 0xD3, 0xEE), // Cyan
                Color32::from_rgb(0x81, 0x8C, 0xF8), // Indigo
            ],
        }
    }

    pub fn get_group_color(&self, index: usize) -> Color32 {
        self.group_colors[index % self.group_colors.len()]
    }

    pub fn apply_to_ctx(&self, ctx: &Context) {
        let mut visuals = match self.mode {
            ThemeMode::Light => Visuals::light(),
            ThemeMode::Dark => Visuals::dark(),
        };

        visuals.override_text_color = Some(self.primary_text);
        visuals.window_fill = self.surface_color;
        visuals.panel_fill = self.bg_color;
        visuals.window_corner_radius = CornerRadius::same(10);
        visuals.window_stroke = Stroke::new(1.0_f32, self.border_color);

        visuals.widgets.noninteractive.bg_fill = self.surface_color;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.border_subtle);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.primary_text);

        visuals.widgets.inactive.bg_fill = self.surface_subtle;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, self.border_color);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.primary_text);

        visuals.widgets.hovered.bg_fill = self.surface_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, self.primary_indigo);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

        visuals.widgets.active.bg_fill = self.primary_indigo;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, self.primary_indigo);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
        visuals.widgets.active.corner_radius = CornerRadius::same(6);

        visuals.selection.bg_fill = self.primary_indigo;
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);

        ctx.set_visuals(visuals);

        // Configure typography
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(18.0, FontFamily::Proportional),
        );
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        );
        ctx.set_style(style);
    }
}
