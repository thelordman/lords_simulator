use egui::{Layout, Vec2};
use lords_sim::object::Living;
use crate::{datetime::DateTime, scenes, App};
use crate::datetime::{DateOrder, DateSeparator, TimeOrder, TimeSeparator};

pub fn ui(app: &mut App, ctx: &egui::Context) {
    scenes::top_panel(ctx);

    let name = app.state().player.name().full();
    let sex = app.state().player.sex();
    let age = app.state().time.years();
    
    let year = app.state().time.year();

    egui::SidePanel::left("left_panel").show(ctx, |ui| {

        ui.label(format!("Name: {name}"));
        ui.label(format!("Sex: {sex}"));
        ui.label(format!("Age: {age}"));
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

            // TODO: Extract column logic to a macro
            ui.columns(2, |columns| {
                columns[0].allocate_ui_with_layout(
                    Vec2::ZERO,
                    Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Pause").clicked() {
                            // TODO: Unwrap unnecessary after runtime is no longer Option
                            app.runtime.as_ref().unwrap().pause();
                        }
                    },
                );

                columns[1].allocate_ui_with_layout(
                    Vec2::ZERO,
                    Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if ui.button("Resume").clicked() {
                            app.runtime.as_ref().unwrap().resume();
                        }
                    },
                );
            });

        });
    });
}