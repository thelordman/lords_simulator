use crate::{scenes, App};

pub fn ui(app: &mut App, ctx: &egui::Context) {
    scenes::top_panel(ctx);

    egui::SidePanel::left("left_panel").show(ctx, |ui| {
        ui.label("Name: ");
        ui.label("Sex: ");
        ui.label("Age: ");
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("Year 1")
                    .size(50.0)
                    .strong()
            );
        });
    });
}