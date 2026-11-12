mod character_creation;
mod main_menu;
mod overview;

pub use character_creation::CharacterCreation;
pub use main_menu::MainMenu;
pub use overview::Overview;

use lords_sim::{Name, Sex};

pub trait Scene {
    /// Draw the scene UI and return any requested action.
    fn update(&mut self, ui: &mut egui::Ui) -> SceneResponse;
}

/// Response from a scene update, indicating what action to take.
pub enum SceneResponse {
    None,
    Transition(Box<dyn Scene>),
    StartSim { name: Name, sex: Sex },
}

/// Renders the standard top panel with menu bar.
pub fn top_panel(ui: &mut egui::Ui) {
    egui::Panel::top("top_panel").show(ui, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.add_space(16.0);
        });
    });
}
