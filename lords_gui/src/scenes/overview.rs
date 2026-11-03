use egui::{Layout, Vec2};
use lords_runtime::Runtime;
use lords_sim::object::Living;

use crate::datetime::DateTime;

use super::{top_panel, Scene, SceneResponse};

pub struct Overview {
    runtime: Runtime,
}

impl Overview {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl Scene for Overview {
    fn update(&mut self, ctx: &egui::Context) -> SceneResponse {
        let state = self.runtime.state.read().expect("state lock poisoned");

        top_panel(ctx);

        let name = state.player.name().full();
        let sex = state.player.sex();
        let age = state.time.years();

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.label(format!("Name: {name}"));
            ui.label(format!("Sex: {sex}"));
            ui.label(format!("Age: {age}"));
        });

        let year = state.time.year();
        let datetime = DateTime::from(state.time).format_default();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(format!("Year {year}")).size(50.0).strong());
                ui.add_space(25.0);
                ui.label(datetime);

                self.playback_controls(ui);
            });
        });

        SceneResponse::None
    }
}

impl Overview {
    fn playback_controls(&self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            columns[0].allocate_ui_with_layout(
                Vec2::ZERO,
                Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if ui.button("Pause").clicked() {
                        self.runtime.pause();
                    }
                },
            );

            columns[1].allocate_ui_with_layout(
                Vec2::ZERO,
                Layout::left_to_right(egui::Align::Center),
                |ui| {
                    if ui.button("Resume").clicked() {
                        self.runtime.resume();
                    }
                },
            );
        });
    }
}
