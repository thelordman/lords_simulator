use egui::{RichText, Sense};

const SIZE: egui::Vec2 = egui::vec2(80.0, 6.0);

pub(super) fn show(ui: &mut egui::Ui, percent: u8) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(SIZE, Sense::hover());
        let filled = rect.with_max_x(rect.left() + rect.width() * f32::from(percent) / 100.0);
        let visuals = ui.visuals();
        let radius = SIZE.y / 2.0;
        ui.painter()
            .rect_filled(rect, radius, visuals.extreme_bg_color);
        ui.painter()
            .rect_filled(filled, radius, visuals.selection.bg_fill);
        ui.label(RichText::new(format!("{percent}%")).weak());
    });
}
