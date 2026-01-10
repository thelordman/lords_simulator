use crate::App;

mod main_menu;
mod character_creation;

pub enum Scene {
    MainMenu,
    CharacterCreation,
}

pub fn draw_scene(app: &mut App, ctx: &egui::Context) {
    match app.scene {
        Scene::MainMenu => main_menu::ui(app, ctx),
        Scene::CharacterCreation => character_creation::ui(ctx),
    }
}

fn top_panel(ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            let is_web = cfg!(target_arch = "wasm32");
            if !is_web {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
            }

            egui::widgets::global_theme_preference_buttons(ui);
        });
    });
}