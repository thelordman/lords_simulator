use egui::{Layout, Vec2};
use lords_runtime::Runtime;
use lords_sim::object::Living;

use crate::datetime::DateTime;

use super::{top_panel, Scene, SceneResponse};

const TIME_CONTROL_COMBO_BOX_ID: &str = "time_control_combo_box";

pub struct Overview {
    runtime: Runtime,
}

impl Overview {
    pub fn new(runtime: Runtime) -> Self {
        let _ = egui::ComboBox::new(TIME_CONTROL_COMBO_BOX_ID, "");

        Self { runtime }
    }
}

impl Scene for Overview {
    fn update(&mut self, ui: &mut egui::Ui) -> SceneResponse {
        let state = self.runtime.state.read().expect("state lock poisoned");

        top_panel(ui);

        let name = state.player.name().full();
        let sex = state.player.sex();
        let age = state.time.years();

        egui::Panel::left("left_panel").show(ui, |ui| {
            ui.label(format!("Name: {name}"));
            ui.label(format!("Sex: {sex}"));
            ui.label(format!("Age: {age}"));
        });

        let year = state.time.year();
        let datetime = DateTime::from(state.time).format_default();

        egui::CentralPanel::default().show(ui, |ui| {
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
