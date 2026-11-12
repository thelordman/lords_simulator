use egui::RichText;
use lords_save::SaveEntry;

use super::{
    AppAction, CharacterCreation, LoadGame, MenuAction, Scene, SceneResponse, Shared, top_panel,
};

const TITLE_SIZE: f32 = 100.0;
const MIN_TITLE_SIZE: f32 = 32.0;
/// How many times its own font size the title is wide, roughly.
const TITLE_ASPECT: f32 = 8.5;

const BUTTON_SIZE: egui::Vec2 = egui::vec2(260.0, 44.0);
const BUTTON_TEXT_SIZE: f32 = 24.0;
const BUTTON_SPACING: f32 = 8.0;

#[derive(Default)]
pub struct MainMenu {
    saves: Option<SaveSummary>,
}

/// What the menu needs to know about the saves on disk.
struct SaveSummary {
    /// The [`Saves::generation`](crate::saves::Saves::generation) this was read at.
    generation: u64,
    /// The save to continue from: the newest one that can be loaded.
    newest: Option<SaveEntry>,
    /// Whether there is anything at all to show on the load screen.
    any: bool,
}

impl Scene for MainMenu {
    fn update(&mut self, ui: &mut egui::Ui, shared: &mut Shared<'_>) -> SceneResponse {
        let menu = top_panel(ui, false);
        self.refresh_saves(shared);

        self.bottom_panel(ui);
        let mut response = self.central_panel(ui, shared);

        if !shared.settings.fullscreen_prompt_seen {
            fullscreen_prompt(ui, shared);
        }

        if let Some(MenuAction::App(action)) = menu {
            response = SceneResponse::App(action);
        }
        response
    }
}

impl MainMenu {
    fn refresh_saves(&mut self, shared: &Shared<'_>) {
        let generation = shared.saves.generation();
        if self
            .saves
            .as_ref()
            .is_some_and(|saves| saves.generation == generation)
        {
            return;
        }

        let list = shared.saves.list();
        self.saves = Some(SaveSummary {
            generation,
            any: !list.saves.is_empty() || !list.unreadable.is_empty(),
            newest: list.saves.into_iter().find(|save| save.problem().is_none()),
        });
    }

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

    fn central_panel(&self, ui: &mut egui::Ui, shared: &mut Shared<'_>) -> SceneResponse {
        let mut response = SceneResponse::None;
        let newest = self.saves.as_ref().and_then(|saves| saves.newest.as_ref());
        let any_saves = self.saves.as_ref().is_some_and(|saves| saves.any);

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        let title_size =
                            (ui.available_width() / TITLE_ASPECT).clamp(MIN_TITLE_SIZE, TITLE_SIZE);

                        ui.add_space(24.0);
                        ui.label(RichText::new("Lord's Simulator").size(title_size).strong());
                        ui.add_space(24.0);
                        ui.spacing_mut().item_spacing.y = BUTTON_SPACING;

                        if let Some(save) = newest {
                            let continued =
                                menu_button(ui, "Continue", true).on_hover_text(format!(
                                    "{}, {}",
                                    save.header.character,
                                    shared.settings.datetime.date(save.header.game_time)
                                ));
                            if continued.clicked()
                                && let Some(loaded) = shared.saves.load(&save.path)
                            {
                                response = SceneResponse::LoadSim(Box::new(loaded));
                            }
                        }

                        if menu_button(ui, "New Game", true).clicked() {
                            response =
                                SceneResponse::Transition(Box::new(CharacterCreation::default()));
                        }

                        if menu_button(ui, "Load Game", any_saves)
                            .on_disabled_hover_text("There are no saved games yet")
                            .clicked()
                        {
                            response = SceneResponse::Transition(Box::new(LoadGame::default()));
                        }

                        if menu_button(ui, "Options", true).clicked() {
                            response = SceneResponse::App(AppAction::OpenOptions);
                        }

                        if menu_button(ui, "Quit", true).clicked() {
                            response = SceneResponse::App(AppAction::Quit);
                        }
                    });
                });
        });

        response
    }
}

fn menu_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(text).size(BUTTON_TEXT_SIZE)).min_size(BUTTON_SIZE),
    )
}

/// Offers to go fullscreen, once.
fn fullscreen_prompt(ui: &mut egui::Ui, shared: &mut Shared<'_>) {
    let mut open = true;
    let mut answered = false;

    egui::Window::new("Play in fullscreen?")
        .open(&mut open)
        .collapsible(false)
        .movable(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -40.0])
        .show(ui, |ui| {
            ui.label("You can change this later in the options, or with F11.");
            ui.horizontal(|ui| {
                if ui.button("Fullscreen").clicked() {
                    ui.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    answered = true;
                }
                if ui.button("Keep windowed").clicked() {
                    answered = true;
                }
            });
        });

    if answered || !open {
        shared.settings.fullscreen_prompt_seen = true;
    }
}

#[cfg(test)]
mod tests;
