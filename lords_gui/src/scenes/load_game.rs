use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use egui::RichText;
use lords_save::{SaveEntry, SaveKind, SimulationId, UnreadableSave};

use crate::datetime::DateTimeFormat;

use super::{MainMenu, MenuAction, Scene, SceneResponse, Shared, top_panel};

const LIST_MAX_WIDTH: f32 = 640.0;
const ROW_SPACING: egui::Vec2 = egui::vec2(16.0, 6.0);
const GROUP_SPACING: f32 = 12.0;
const CONFIRM_WIDTH: f32 = 300.0;

const MS_PER_MINUTE: u64 = 60 * 1000;
const MS_PER_HOUR: u64 = 60 * MS_PER_MINUTE;
const MS_PER_DAY: u64 = 24 * MS_PER_HOUR;

#[derive(Default)]
pub struct LoadGame {
    list: Option<List>,
    deleting: Option<Deletion>,
    exported: Option<PathBuf>,
}

struct List {
    generation: u64,
    playthroughs: Vec<Playthrough>,
    unreadable: Vec<UnreadableSave>,
}

struct Playthrough {
    id: SimulationId,
    character: String,
    /// Newest first.
    saves: Vec<SaveEntry>,
}

struct Deletion {
    path: PathBuf,
    description: String,
}

enum Request {
    Load(PathBuf),
    Delete(Deletion),
    Export(PathBuf),
}

impl Scene for LoadGame {
    fn update(&mut self, ui: &mut egui::Ui, shared: &mut Shared<'_>) -> SceneResponse {
        let menu = top_panel(ui, false);
        self.refresh(shared);

        let mut back = !shared.dialog_open
            && self.deleting.is_none()
            && ui.input(|input| input.key_pressed(egui::Key::Escape));
        let mut request = None;

        egui::CentralPanel::default().show(ui, |ui| {
            back |= ui.button("Back").clicked();
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Load Game").size(40.0).strong());
                ui.add_space(12.0);
            });

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    egui::Aligned::new(egui::Align2::CENTER_TOP).show(ui, |ui| {
                        ui.set_max_width(LIST_MAX_WIDTH.min(ui.available_width()));
                        ui.vertical(|ui| {
                            request = self.show_list(ui, &shared.settings.datetime);
                        });
                    });
                });
        });

        self.confirm_deletion(ui, shared);

        if let Some(MenuAction::App(action)) = menu {
            return SceneResponse::App(action);
        }
        if back {
            return SceneResponse::Transition(Box::new(MainMenu::default()));
        }

        match request {
            Some(Request::Load(path)) => {
                if let Some(save) = shared.saves.load(&path) {
                    return SceneResponse::LoadSim(Box::new(save));
                }
            }
            Some(Request::Delete(deletion)) => self.deleting = Some(deletion),
            Some(Request::Export(path)) => {
                if let Some(exported) = shared.saves.export(&path) {
                    self.exported = Some(exported);
                }
            }
            None => {}
        }
        SceneResponse::None
    }
}

impl LoadGame {
    fn refresh(&mut self, shared: &Shared<'_>) {
        let generation = shared.saves.generation();
        if self
            .list
            .as_ref()
            .is_some_and(|list| list.generation == generation)
        {
            return;
        }

        let found = shared.saves.list();
        self.list = Some(List {
            generation,
            playthroughs: group(found.saves),
            unreadable: found.unreadable,
        });
    }

    fn show_list(&self, ui: &mut egui::Ui, format: &DateTimeFormat) -> Option<Request> {
        let list = self.list.as_ref()?;
        let now_ms = now_ms();
        let mut request = None;

        if list.playthroughs.is_empty() && list.unreadable.is_empty() {
            ui.label(RichText::new("There are no saved games.").weak());
        }

        for playthrough in &list.playthroughs {
            ui.label(RichText::new(&playthrough.character).heading().strong());
            egui::Grid::new(("saves", playthrough.id))
                .num_columns(4)
                .spacing(ROW_SPACING)
                .striped(true)
                .show(ui, |ui| {
                    for save in &playthrough.saves {
                        request = save_row(ui, save, format, now_ms).or(request.take());
                        ui.end_row();
                    }
                });
            ui.add_space(GROUP_SPACING);
        }

        if !list.unreadable.is_empty() {
            ui.label(RichText::new("Unreadable").heading().strong());
            egui::Grid::new("unreadable_saves")
                .num_columns(3)
                .spacing(ROW_SPACING)
                .striped(true)
                .show(ui, |ui| {
                    for save in &list.unreadable {
                        let name = file_name(&save.path);
                        ui.label(&name);
                        ui.label(RichText::new(save.error.to_string()).weak());
                        if ui.button("Delete").clicked() {
                            request = Some(Request::Delete(Deletion {
                                path: save.path.clone(),
                                description: name,
                            }));
                        }
                        ui.end_row();
                    }
                });
            ui.add_space(GROUP_SPACING);
        }

        if let Some(exported) = &self.exported {
            ui.label(RichText::new("Exported to").weak().small());
            ui.add(egui::Label::new(RichText::new(exported.display().to_string()).small()).wrap());
        }

        request
    }

    fn confirm_deletion(&mut self, ui: &mut egui::Ui, shared: &mut Shared<'_>) {
        let Some(deletion) = &self.deleting else {
            return;
        };
        let mut delete = false;
        let mut cancel = false;

        let modal = egui::Modal::new(egui::Id::unique("confirm_deletion")).show(ui.ctx(), |ui| {
            ui.set_width(CONFIRM_WIDTH);
            ui.heading("Delete this save?");
            ui.add_space(4.0);
            ui.label(&deletion.description);
            ui.label(RichText::new("This cannot be undone.").weak());
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                delete = ui.button("Delete").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        });

        if delete {
            shared.saves.delete(&deletion.path);
        }
        if delete || cancel || modal.should_close() {
            self.deleting = None;
        }
    }
}

fn save_row(
    ui: &mut egui::Ui,
    save: &SaveEntry,
    format: &DateTimeFormat,
    now_ms: u64,
) -> Option<Request> {
    let header = &save.header;
    let title = title(save);
    let mut request = None;

    ui.label(&title);
    ui.label(RichText::new(format.date_time(header.game_time)).monospace());
    ui.label(RichText::new(time_ago(now_ms, header.saved_at_ms)).weak());

    ui.horizontal(|ui| {
        let problem = save.problem();
        let load = ui.add_enabled(problem.is_none(), egui::Button::new("Load"));
        let load = match &problem {
            Some(problem) => load.on_disabled_hover_text(problem.to_string()),
            None => load,
        };
        if load.clicked() {
            request = Some(Request::Load(save.path.clone()));
        }

        if ui
            .button("Export")
            .on_hover_text("Write the save out as text, to read it by hand")
            .clicked()
        {
            request = Some(Request::Export(save.path.clone()));
        }

        if ui.button("Delete").clicked() {
            request = Some(Request::Delete(Deletion {
                path: save.path.clone(),
                description: format!(
                    "{title} of {}, at {}",
                    header.character,
                    format.date_time(header.game_time)
                ),
            }));
        }
    });

    request
}

fn title(save: &SaveEntry) -> String {
    match save.header.kind {
        SaveKind::Manual if save.header.label.is_empty() => "Unnamed save".to_owned(),
        SaveKind::Manual => save.header.label.clone(),
        SaveKind::Auto => "Autosave".to_owned(),
        SaveKind::Exit => "Exit save".to_owned(),
    }
}

/// Groups `saves`, given newest first, by playthrough, keeping that order both between and
/// within the groups.
fn group(saves: Vec<SaveEntry>) -> Vec<Playthrough> {
    let mut playthroughs: Vec<Playthrough> = Vec::new();

    for save in saves {
        let id = save.header.playthrough;
        match playthroughs
            .iter_mut()
            .find(|playthrough| playthrough.id == id)
        {
            Some(playthrough) => playthrough.saves.push(save),
            None => playthroughs.push(Playthrough {
                id,
                // The newest save names the group.
                character: save.header.character.clone(),
                saves: vec![save],
            }),
        }
    }

    playthroughs
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// Describes how long before `now_ms` the moment `then_ms` was, both in milliseconds since
/// the Unix epoch.
fn time_ago(now_ms: u64, then_ms: u64) -> String {
    let elapsed = now_ms.saturating_sub(then_ms);

    let (number, unit) = if elapsed < MS_PER_MINUTE {
        return "just now".to_owned();
    } else if elapsed < MS_PER_HOUR {
        (elapsed / MS_PER_MINUTE, "minute")
    } else if elapsed < MS_PER_DAY {
        (elapsed / MS_PER_HOUR, "hour")
    } else {
        (elapsed / MS_PER_DAY, "day")
    };

    if number == 1 {
        format!("1 {unit} ago")
    } else {
        format!("{number} {unit}s ago")
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since_epoch| {
            u64::try_from(since_epoch.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests;
