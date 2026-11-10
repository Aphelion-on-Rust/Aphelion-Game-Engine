use crate::theme::{Color, ThemeData};
use eframe::egui;

pub fn to_color32(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

/// Same color, different transparency (for highlights).
pub fn with_alpha(color: Color, alpha: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, alpha)
}

fn is_dark(color: Color) -> bool {
    let brightness = (color.r as u32 * 299 + color.g as u32 * 587 + color.b as u32 * 114) / 1000;
    brightness < 128
}

/// Tells egui to use the theme's colors for every widget.
pub fn apply(ctx: &egui::Context, theme: &ThemeData) {
    let colors = &theme.colors;
    let mut visuals = if is_dark(colors.background) { egui::Visuals::dark() } else { egui::Visuals::light() };

    visuals.panel_fill = to_color32(colors.background);
    visuals.window_fill = to_color32(colors.panel);
    visuals.extreme_bg_color = to_color32(colors.background);
    visuals.faint_bg_color = to_color32(colors.panel_header);
    visuals.override_text_color = Some(to_color32(colors.text));
    visuals.hyperlink_color = to_color32(colors.accent);
    visuals.error_fg_color = to_color32(colors.error);
    visuals.warn_fg_color = to_color32(colors.warning);
    visuals.selection.bg_fill = to_color32(colors.selection);

    visuals.widgets.noninteractive.bg_fill = to_color32(colors.panel);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, to_color32(colors.border));
    visuals.widgets.inactive.bg_fill = to_color32(colors.panel_header);
    visuals.widgets.inactive.weak_bg_fill = to_color32(colors.panel_header);
    visuals.widgets.hovered.bg_fill = to_color32(colors.accent_hover);
    visuals.widgets.hovered.weak_bg_fill = to_color32(colors.selection);
    visuals.widgets.active.bg_fill = to_color32(colors.accent);
    visuals.widgets.active.weak_bg_fill = to_color32(colors.accent);

    ctx.set_visuals(visuals);
}
