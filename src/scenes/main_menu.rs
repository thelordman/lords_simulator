use crate::{scenes, App};

pub fn ui(app: &mut App, ctx: &egui::Context) {
    scenes::top_panel(ctx);

    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
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

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(
                egui::RichText::new("Lord's Simulator")
                    .size(100.0)
                    .strong()
            );
            ui.add_space(50.0);

            if ui.button(egui::RichText::new("New Game").size(50.0)).clicked() {
                app.scene = scenes::Scene::CharacterCreation;
            }
        });
    });

    if app.main_menu_data.show_fullscreen_prompt {
        app.main_menu_data.show_fullscreen_prompt = fullscreen_prompt(ctx, app.main_menu_data.show_fullscreen_prompt);
    }
}

fn fullscreen_prompt(ctx: &egui::Context, mut show_fullscreen_prompt: bool) -> bool {
    let mut open = show_fullscreen_prompt;
    egui::Window::new("Fullscreen?")
        .open(&mut open)
        .movable(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
        .show(ctx, |ui| {
            ui.label("Can later be configured through the options menu.");
            ui.horizontal(|ui| {
                if ui.button("Enable").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    show_fullscreen_prompt = false;
                }
                if ui.button("Borderless Windowed").clicked() {
                    if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::Pos2::ZERO));
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(monitor));
                    } else {
                        log::warn!("Could not determine monitor size for borderless window, maximizing instead");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                    }
                    show_fullscreen_prompt = false;
                }
                if ui.button("Keep Windowed").clicked() {
                    show_fullscreen_prompt = false;
                }
            })
        });
    if !open {
        show_fullscreen_prompt = false;
    }
    show_fullscreen_prompt
}