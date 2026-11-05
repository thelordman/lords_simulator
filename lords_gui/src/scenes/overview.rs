use lords_runtime::Runtime;
use lords_sim::object::Living;

use crate::datetime::DateTime;
use crate::icons;

use super::{top_panel, Scene, SceneResponse};

const TIME_CONTROL_COMBO_BOX_ID: &str = "time_control_combo_box";
const PLAYBACK_ICON_SIZE: f32 = 20.0;

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
        let state = self.runtime.state();

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
        let paused = self.runtime.is_paused();
        let (icon, label) = if paused {
            (icons::PLAY, "Resume")
        } else {
            (icons::PAUSE, "Pause")
        };

        let space_pressed = take_space_press(ui);

        let button = ui
            .button(egui::RichText::new(icon).size(PLAYBACK_ICON_SIZE))
            .on_hover_text(label);

        if button.clicked() || space_pressed {
            if paused {
                self.runtime.resume();
            } else {
                self.runtime.pause();
            }
        }
    }
}

// Helper to ignore key-repeats when holding down the spacebar.
fn take_space_press(ui: &mut egui::Ui) -> bool {
    if ui.ctx().text_edit_focused() {
        return false;
    }

    ui.input_mut(|input| {
        let pressed = input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    key: egui::Key::Space,
                    pressed: true,
                    repeat: false,
                    ..
                }
            )
        });
        input.consume_key(egui::Modifiers::NONE, egui::Key::Space);
        pressed
    })
}
