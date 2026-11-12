use super::{CharacterCreation, Scene, SceneResponse, top_panel};

pub struct MainMenu {
    show_fullscreen_prompt: bool,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            show_fullscreen_prompt: true,
        }
    }
}

impl Scene for MainMenu {
    fn update(&mut self, ui: &mut egui::Ui) -> SceneResponse {
        top_panel(ui);

        self.bottom_panel(ui);
        let response = self.central_panel(ui);

        if self.show_fullscreen_prompt {
            self.fullscreen_prompt(ui);
        }

        response
    }
}

impl MainMenu {
    fn bottom_panel(&self, ui: &mut egui::Ui) {
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::warn_if_debug_build(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::github_link_file!(
                        "https://github.com/thelordman/lords_simulator/blob/main/",
                        "Source Code"
                    ));
                });
            });
        });
    }

    fn central_panel(&self, ui: &mut egui::Ui) -> SceneResponse {
        let mut response = SceneResponse::None;

        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(egui::RichText::new("Lord's Simulator").size(100.0).strong());
                ui.add_space(50.0);

                if ui
                    .button(egui::RichText::new("New Game").size(50.0))
                    .clicked()
                {
                    response = SceneResponse::Transition(Box::new(CharacterCreation::default()));
                }
            });
        });

        response
    }

    fn fullscreen_prompt(&mut self, ui: &mut egui::Ui) {
        let mut open = true;

        egui::Window::new("Fullscreen?")
            .open(&mut open)
            .movable(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
            .show(ui, |ui| {
                ui.label("Can later be configured through the options menu.");
                ui.horizontal(|ui| {
                    if ui.button("Enable").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        self.show_fullscreen_prompt = false;
                    }
                    if ui.button("Borderless Windowed").clicked() {
                        self.set_borderless_windowed(ui);
                        self.show_fullscreen_prompt = false;
                    }
                    if ui.button("Keep Windowed").clicked() {
                        self.show_fullscreen_prompt = false;
                    }
                });
            });

        if !open {
            self.show_fullscreen_prompt = false;
        }
    }

    fn set_borderless_windowed(&self, ui: &mut egui::Ui) {
        if let Some(monitor) = ui.input(|i| i.viewport().monitor_size) {
            ui.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
            ui.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::Pos2::ZERO));
            ui.send_viewport_cmd(egui::ViewportCommand::InnerSize(monitor));
        } else {
            log::warn!(
                "Could not determine monitor size for borderless window, maximizing instead"
            );
            ui.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
            ui.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
    }
}
