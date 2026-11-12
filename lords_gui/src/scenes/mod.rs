mod character_creation;
#[cfg(test)]
mod harness;
mod load_game;
mod main_menu;
mod overview;

pub use character_creation::CharacterCreation;
pub use load_game::LoadGame;
pub use main_menu::MainMenu;
pub use overview::Overview;

use lords_save::LoadedSave;
use lords_sim::{Name, Sex};

use crate::options;
use crate::saves::Saves;
use crate::settings::Settings;

pub const SAVE: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);

pub const TOGGLE_FULLSCREEN: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F11);

pub trait Scene {
    /// Draw the scene UI and return any requested action.
    fn update(&mut self, ui: &mut egui::Ui, shared: &mut Shared<'_>) -> SceneResponse;

    /// Called once when the scene is left for another, or the game is closed.
    fn on_exit(&mut self, _shared: &mut Shared<'_>) {}
}

/// What the app lends to whichever scene is showing.
pub struct Shared<'a> {
    pub settings: &'a mut Settings,
    pub saves: &'a mut Saves,
    /// Whether a dialog of the app's is covering the scene, which should then leave the
    /// keyboard to it.
    pub dialog_open: bool,
}

/// Response from a scene update, indicating what action to take.
pub enum SceneResponse {
    None,
    Transition(Box<dyn Scene>),
    StartSim { name: Name, sex: Sex },
    LoadSim(Box<LoadedSave>),
    App(AppAction),
}

/// Something every scene can ask for and none of them does itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    OpenOptions,
    ToggleFullscreen,
    Quit,
}

/// What the player picked from the menu bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    App(AppAction),
    Save,
    SaveAs,
    MainMenu,
}

/// Renders the standard top panel with menu bar, with the entries that only make sense
/// while a game is running if `in_game`.
pub fn top_panel(ui: &mut egui::Ui, in_game: bool) -> Option<MenuAction> {
    let mut action = None;

    egui::Panel::top("top_panel").show(ui, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if in_game {
                    if menu_item(ui, "Save", Some(&SAVE)) {
                        action = Some(MenuAction::Save);
                    }
                    if menu_item(ui, "Save As…", None) {
                        action = Some(MenuAction::SaveAs);
                    }
                    ui.separator();
                    if menu_item(ui, "Main Menu", None) {
                        action = Some(MenuAction::MainMenu);
                    }
                    ui.separator();
                }
                if menu_item(ui, "Options…", None) {
                    action = Some(MenuAction::App(AppAction::OpenOptions));
                }
                if menu_item(ui, "Quit", None) {
                    action = Some(MenuAction::App(AppAction::Quit));
                }
            });

            ui.menu_button("View", |ui| {
                let label = if options::is_fullscreen(ui.ctx()) {
                    "Leave Fullscreen"
                } else {
                    "Fullscreen"
                };
                if menu_item(ui, label, Some(&TOGGLE_FULLSCREEN)) {
                    action = Some(MenuAction::App(AppAction::ToggleFullscreen));
                }
            });
        });
    });

    action
}

fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: Option<&egui::KeyboardShortcut>) -> bool {
    let mut button = egui::Button::new(label);
    if let Some(shortcut) = shortcut {
        button = button.shortcut_text(ui.ctx().format_shortcut(shortcut));
    }
    ui.add(button).clicked()
}
