use lords_sim::object::Living;
use crate::{datetime::DateTime, scenes, App};
use crate::datetime::{DateOrder, DateSeparator, TimeOrder, TimeSeparator};

pub fn ui(app: &mut App, ctx: &egui::Context) {
    scenes::top_panel(ctx);

    let name = app.state().player.name().full();
    let sex = app.state().player.sex();
    let year = app.state().time.years() + 1;

    egui::SidePanel::left("left_panel").show(ctx, |ui| {

        ui.label(format!("Name: {name}"));
        ui.label(format!("Sex: {sex}"));
        ui.label(format!("Age: {year}"));
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(format!("Year {year}"))
                    .size(50.0)
                    .strong()
            );
            ui.add_space(25.0);
            
            let datetime = DateTime::from(app.state().time)
                .format(DateOrder::DMY, DateSeparator::Period, TimeOrder::HMS, TimeSeparator::Colon);
            ui.label(datetime);
        });
    });
}