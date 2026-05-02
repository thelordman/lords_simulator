use crate::{scenes, App};

pub fn ui(app: &mut App, ctx: &egui::Context) {
    scenes::top_panel(ctx);

    egui::SidePanel::left("left_panel").show(ctx, |ui| {
        let name = app.state().name.full_name();
        let sex = app.state().sex;
        let year = app.state().time.years();

        ui.label(format!("Name: {name}"));
        ui.label(format!("Sex: {sex}"));
        ui.label(format!("Age: {year}"));
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("Year 1")
                    .size(50.0)
                    .strong()
            );
            ui.add_space(25.0);
            ui.label(format!("Time: {} s", app.state().time.seconds()));
        });
    });
}