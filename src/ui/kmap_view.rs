use crate::domain::implicant::Implicant;
use crate::domain::kmap::{
    KMapDimensions, col_var_header, coords_to_minterm, get_col_labels, get_row_labels,
    row_var_header,
};
use crate::domain::model::{CellValue, VarCount};
use crate::ui::theme::Theme;
use egui::{
    Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, pos2,
    vec2,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KMapResponse {
    pub clicked_minterm: Option<usize>,
    pub hovered_minterm: Option<usize>,
    pub hovered_group: Option<usize>,
}

pub struct KMapView;

impl KMapView {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        theme: &Theme,
        var_count: VarCount,
        minterm_values: &HashMap<usize, CellValue>,
        groups: &[Implicant],
        hovered_minterm: Option<usize>,
        selected_minterm: Option<usize>,
        hovered_group: Option<usize>,
    ) -> KMapResponse {
        let mut clicked_minterm = None;
        let mut hovered_g = None;
        let dims = KMapDimensions::for_var_count(var_count);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("KARNAUGH MAP")
                    .size(15.0)
                    .color(theme.primary_text)
                    .strong(),
            );
            ui.label(
                egui::RichText::new("(Toroidal Gray Code Grid)")
                    .size(12.0)
                    .color(theme.muted_text),
            );
        });

        ui.add_space(4.0);

        // Header variable indicators: RowVar \ ColVar
        let row_var = row_var_header(var_count);
        let col_var = col_var_header(var_count);

        // Calculate responsive cell size
        let available_size = ui.available_size();
        let header_w = 48.0;
        let header_h = 36.0;

        let max_grid_w = (available_size.x - header_w - 20.0).max(200.0);
        let max_grid_h = 240.0;

        let cell_w = (max_grid_w / dims.cols as f32).clamp(48.0, 72.0);
        let cell_h = (max_grid_h / dims.rows as f32).clamp(42.0, 56.0);

        let grid_total_w = header_w + dims.cols as f32 * cell_w;
        let grid_total_h = header_h + dims.rows as f32 * cell_h;

        let (response, painter) =
            ui.allocate_painter(vec2(grid_total_w, grid_total_h), Sense::click_and_drag());
        let origin = response.rect.min;

        let grid_origin = pos2(origin.x + header_w, origin.y + header_h);

        // 1. Draw Axis Variable Split Header in upper-left corner
        painter.text(
            pos2(origin.x + 8.0, origin.y + header_h - 10.0),
            egui::Align2::LEFT_BOTTOM,
            row_var,
            FontId::new(13.0, FontFamily::Proportional),
            theme.primary_indigo,
        );
        painter.text(
            pos2(origin.x + header_w - 6.0, origin.y + 10.0),
            egui::Align2::RIGHT_TOP,
            col_var,
            FontId::new(13.0, FontFamily::Proportional),
            theme.accent_teal,
        );
        painter.line_segment(
            [
                pos2(origin.x + 4.0, origin.y + 6.0),
                pos2(origin.x + header_w - 4.0, origin.y + header_h - 4.0),
            ],
            Stroke::new(1.0_f32, theme.border_color),
        );

        // 2. Draw Column Headers (Gray Code)
        let col_labels = get_col_labels(var_count);
        for (c, (label, _)) in col_labels.iter().enumerate() {
            let cx = grid_origin.x + (c as f32 + 0.5) * cell_w;
            let cy = origin.y + header_h * 0.5;
            painter.text(
                pos2(cx, cy),
                egui::Align2::CENTER_CENTER,
                *label,
                FontId::new(14.0, FontFamily::Monospace),
                theme.primary_text,
            );
        }

        // 3. Draw Row Headers (Gray Code)
        let row_labels = get_row_labels(var_count);
        for (r, (label, _)) in row_labels.iter().enumerate() {
            let cx = origin.x + header_w * 0.5;
            let cy = grid_origin.y + (r as f32 + 0.5) * cell_h;
            painter.text(
                pos2(cx, cy),
                egui::Align2::CENTER_CENTER,
                *label,
                FontId::new(14.0, FontFamily::Monospace),
                theme.primary_text,
            );
        }

        // 4. Draw Cells & Cell Values
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let mut hovered_m = None;

        for r in 0..dims.rows {
            for c in 0..dims.cols {
                let m = coords_to_minterm(var_count, r, c);
                let cell_rect = Rect::from_min_size(
                    pos2(
                        grid_origin.x + c as f32 * cell_w,
                        grid_origin.y + r as f32 * cell_h,
                    ),
                    vec2(cell_w, cell_h),
                );

                let is_hovered = pointer_pos.is_some_and(|p| cell_rect.contains(p));
                let is_minterm_matched = hovered_minterm == Some(m) || is_hovered;
                let is_selected = selected_minterm == Some(m);

                if is_hovered {
                    hovered_m = Some(m);
                    if ui.input(|i| i.pointer.primary_clicked()) {
                        clicked_minterm = Some(m);
                    }
                }

                // Cell Background
                let fill_color = if is_selected {
                    theme.primary_indigo.gamma_multiply(0.2)
                } else if is_minterm_matched {
                    theme.surface_hover
                } else {
                    theme.surface_color
                };

                let cell_stroke = if is_selected {
                    Stroke::new(1.5_f32, theme.primary_indigo)
                } else if is_minterm_matched {
                    Stroke::new(1.0_f32, theme.accent_teal)
                } else {
                    Stroke::new(0.5_f32, theme.border_subtle)
                };

                painter.rect(
                    cell_rect,
                    CornerRadius::same(2),
                    fill_color,
                    cell_stroke,
                    StrokeKind::Inside,
                );

                // Subtle minterm number
                painter.text(
                    pos2(cell_rect.min.x + 4.0, cell_rect.min.y + 3.0),
                    egui::Align2::LEFT_TOP,
                    format!("m{m}"),
                    FontId::new(10.0, FontFamily::Monospace),
                    theme.muted_text,
                );

                // Center cell value
                if let Some(val) = minterm_values.get(&m) {
                    let val_str = val.as_str();
                    let (val_color, val_size) = match val {
                        CellValue::One => (theme.primary_indigo, 20.0),
                        CellValue::Zero => (theme.muted_text, 18.0),
                        CellValue::DontCare => (theme.accent_teal, 19.0),
                    };

                    painter.text(
                        pos2(cell_rect.center().x, cell_rect.center().y + 2.0),
                        egui::Align2::CENTER_CENTER,
                        val_str,
                        FontId::new(val_size, FontFamily::Proportional),
                        val_color,
                    );
                }
            }
        }

        // 5. Draw Group Overlays (Toroidal Loops)
        let base_padding = 4.0;

        for (g_idx, group) in groups.iter().enumerate() {
            let color = theme.get_group_color(g_idx);
            let is_this_hovered = hovered_group == Some(g_idx);

            let stroke_width = if is_this_hovered { 3.5_f32 } else { 2.2_f32 };
            let fill_alpha = if is_this_hovered { 45 } else { 20 };
            let fill_color =
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), fill_alpha);
            let stroke_color = color;

            let segments = group.visual_segments();
            let inset = (g_idx as f32 * 2.0) % 6.0;

            for seg in segments {
                let seg_min_x = grid_origin.x + seg.c_start as f32 * cell_w + base_padding + inset;
                let seg_max_x =
                    grid_origin.x + (seg.c_end + 1) as f32 * cell_w - base_padding - inset;
                let seg_min_y = grid_origin.y + seg.r_start as f32 * cell_h + base_padding + inset;
                let seg_max_y =
                    grid_origin.y + (seg.r_end + 1) as f32 * cell_h - base_padding - inset;

                // Adjust open edges to flush with cell boundary so wrap-around is clear
                let left_x = if seg.open_left {
                    grid_origin.x + seg.c_start as f32 * cell_w
                } else {
                    seg_min_x
                };
                let right_x = if seg.open_right {
                    grid_origin.x + (seg.c_end + 1) as f32 * cell_w
                } else {
                    seg_max_x
                };
                let top_y = if seg.open_top {
                    grid_origin.y + seg.r_start as f32 * cell_h
                } else {
                    seg_min_y
                };
                let bottom_y = if seg.open_bottom {
                    grid_origin.y + (seg.r_end + 1) as f32 * cell_h
                } else {
                    seg_max_y
                };

                let seg_rect = Rect::from_min_max(pos2(left_x, top_y), pos2(right_x, bottom_y));

                // Check mouse hover on this group overlay
                if pointer_pos.is_some_and(|p| seg_rect.contains(p)) {
                    hovered_g = Some(g_idx);
                }

                let radius = 10u8;
                let radius_f = 10.0f32;
                let rounding = CornerRadius {
                    nw: if seg.open_top || seg.open_left {
                        0
                    } else {
                        radius
                    },
                    ne: if seg.open_top || seg.open_right {
                        0
                    } else {
                        radius
                    },
                    sw: if seg.open_bottom || seg.open_left {
                        0
                    } else {
                        radius
                    },
                    se: if seg.open_bottom || seg.open_right {
                        0
                    } else {
                        radius
                    },
                };

                // Draw filled rounded segment
                painter.rect_filled(seg_rect, rounding, fill_color);

                // Draw outline
                // If any side is open, we can draw the custom strokes for closed edges
                if !seg.open_top && !seg.open_bottom && !seg.open_left && !seg.open_right {
                    // Fully closed planar group
                    painter.rect_stroke(
                        seg_rect,
                        rounding,
                        Stroke::new(stroke_width, stroke_color),
                        StrokeKind::Inside,
                    );
                } else {
                    // Split wrap-around group outline: stroke with custom open sides
                    let nw_f = rounding.nw as f32;
                    let ne_f = rounding.ne as f32;
                    let sw_f = rounding.sw as f32;
                    let se_f = rounding.se as f32;

                    // Top edge
                    if !seg.open_top {
                        painter.line_segment(
                            [pos2(left_x + nw_f, top_y), pos2(right_x - ne_f, top_y)],
                            Stroke::new(stroke_width, stroke_color),
                        );
                    }
                    // Bottom edge
                    if !seg.open_bottom {
                        painter.line_segment(
                            [
                                pos2(left_x + sw_f, bottom_y),
                                pos2(right_x - se_f, bottom_y),
                            ],
                            Stroke::new(stroke_width, stroke_color),
                        );
                    }
                    // Left edge
                    if !seg.open_left {
                        painter.line_segment(
                            [pos2(left_x, top_y + nw_f), pos2(left_x, bottom_y - sw_f)],
                            Stroke::new(stroke_width, stroke_color),
                        );
                    }
                    // Right edge
                    if !seg.open_right {
                        painter.line_segment(
                            [pos2(right_x, top_y + ne_f), pos2(right_x, bottom_y - se_f)],
                            Stroke::new(stroke_width, stroke_color),
                        );
                    }

                    // Draw rounded corner arcs for closed corners
                    if rounding.nw > 0 {
                        draw_corner_arc(
                            &painter,
                            pos2(left_x + radius_f, top_y + radius_f),
                            radius_f,
                            std::f32::consts::PI,
                            1.5 * std::f32::consts::PI,
                            Stroke::new(stroke_width, stroke_color),
                        );
                    }
                    if rounding.ne > 0 {
                        draw_corner_arc(
                            &painter,
                            pos2(right_x - radius_f, top_y + radius_f),
                            radius_f,
                            1.5 * std::f32::consts::PI,
                            2.0 * std::f32::consts::PI,
                            Stroke::new(stroke_width, stroke_color),
                        );
                    }
                    if rounding.se > 0 {
                        draw_corner_arc(
                            &painter,
                            pos2(right_x - radius_f, bottom_y - radius_f),
                            radius_f,
                            0.0,
                            0.5 * std::f32::consts::PI,
                            Stroke::new(stroke_width, stroke_color),
                        );
                    }
                    if rounding.sw > 0 {
                        draw_corner_arc(
                            &painter,
                            pos2(left_x + radius_f, bottom_y - radius_f),
                            radius_f,
                            0.5 * std::f32::consts::PI,
                            std::f32::consts::PI,
                            Stroke::new(stroke_width, stroke_color),
                        );
                    }
                }
            }
        }

        KMapResponse {
            clicked_minterm,
            hovered_minterm: hovered_m,
            hovered_group: hovered_g,
        }
    }
}

fn draw_corner_arc(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    stroke: Stroke,
) {
    let steps = 6;
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = start_angle + t * (end_angle - start_angle);
        points.push(pos2(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        ));
    }
    for i in 0..steps {
        painter.line_segment([points[i], points[i + 1]], stroke);
    }
}
