use eframe::egui;

pub fn apply_style(ctx: &egui::Context, accent_color: [u8; 3]) {
    let mut visuals = egui::Visuals::dark();

    // Die gewählte Akzentfarbe
    let accent = egui::Color32::from_rgb(accent_color[0], accent_color[1], accent_color[2]);
    let dark_gray = egui::Color32::from_rgb(30, 30, 30);

    // Hintergrund für inaktive Elemente
    visuals.widgets.noninteractive.bg_fill = dark_gray;

    // Akzentfarbe anwenden (für Auswahl und Rahmen)
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = egui::Stroke::new(1.0, accent);

    // Moderne Rundungen
    visuals.window_rounding = egui::Rounding::same(8.0);
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
    visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
    visuals.widgets.active.rounding = egui::Rounding::same(4.0);
    visuals.widgets.open.rounding = egui::Rounding::same(4.0);

    ctx.set_visuals(visuals);
}