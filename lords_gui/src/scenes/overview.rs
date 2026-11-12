use std::sync::Arc;
use std::time::Instant;

use egui::RichText;
use lords_runtime::{Runtime, Snapshot, Speed};
use lords_save::{SimulationId, SaveKind, SaveRequest};
use lords_sim::object::Living;
use lords_sim::time::gregorian::Date;
use lords_sim::{Human, Timestamp};

use crate::datetime::DateTimeFormat;
use crate::icons;

use super::{MainMenu, MenuAction, SAVE, Scene, SceneResponse, Shared, top_panel};

const TOGGLE_PAUSE: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Space);

const CONTROL_SIZE: f32 = 16.0;
const PAUSE_BUTTON_SIZE: egui::Vec2 = egui::Vec2::splat(24.0);

const SIDEBAR_WIDTH: f32 = 200.0;
const SIDEBAR_PADDING: f32 = 4.0;
const FACT_SPACING: egui::Vec2 = egui::vec2(12.0, 6.0);

const SAVE_DIALOG_WIDTH: f32 = 300.0;
const SAVE_LABEL_LIMIT: usize = 40;

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

pub struct Overview {
    runtime: Runtime,
    playthrough: SimulationId,
    /// Name of the save that saving again replaces, once the player has named one.
    save_label: Option<String>,
    /// The name being typed into the save dialog, while it is open.
    save_dialog: Option<String>,
    focus_save_dialog: bool,
    last_autosave: Instant,
    /// Simulation time of the latest save, so that a game that has not moved on since is
    /// not saved again by itself.
    last_saved_time: Option<Timestamp>,
}

impl Overview {
    /// Shows the game `runtime` is running, which is part of `playthrough` and, if it was
    /// loaded from a save the player named, was last saved as `save_label`.
    pub fn new(runtime: Runtime, playthrough: SimulationId, save_label: Option<String>) -> Self {
        Self {
            runtime,
            playthrough,
            save_label,
            save_dialog: None,
            focus_save_dialog: false,
            last_autosave: Instant::now(),
            last_saved_time: None,
        }
    }
}

impl Scene for Overview {
    fn update(&mut self, ui: &mut egui::Ui, shared: &mut Shared<'_>) -> SceneResponse {
        let snapshot = self.runtime.snapshot();
        let state = &snapshot.state;
        let keyboard_free = !shared.dialog_open && self.save_dialog.is_none();

        let mut action = top_panel(ui, true);
        if keyboard_free && ui.input_mut(|input| input.consume_shortcut(&SAVE)) {
            action = Some(MenuAction::Save);
        }

        self.time_bar(ui, &shared.settings.datetime, &snapshot, keyboard_free);
        let mut leave = !self.runtime.is_running() && stopped_banner(ui);
        character_panel(ui, &state.player, state.time, &shared.settings.datetime);
        content_area(ui);

        self.autosave(shared, &snapshot, Instant::now());
        self.show_save_dialog(ui, shared, &snapshot);

        match action {
            Some(MenuAction::Save) => match self.save_label.clone() {
                Some(label) => self.save(shared, &snapshot, SaveKind::Manual, label),
                None => self.open_save_dialog(shared, &snapshot),
            },
            Some(MenuAction::SaveAs) => self.open_save_dialog(shared, &snapshot),
            Some(MenuAction::MainMenu) => leave = true,
            Some(MenuAction::App(action)) => return SceneResponse::App(action),
            None => {}
        }

        if leave {
            SceneResponse::Transition(Box::new(MainMenu::default()))
        } else {
            SceneResponse::None
        }
    }

    fn on_exit(&mut self, shared: &mut Shared<'_>) {
        let snapshot = self.runtime.snapshot();
        self.save(shared, &snapshot, SaveKind::Exit, String::new());
    }
}

impl Overview {
    fn time_bar(
        &self,
        ui: &mut egui::Ui,
        format: &DateTimeFormat,
        snapshot: &Snapshot,
        keyboard_free: bool,
    ) {
        egui::Panel::top("time_bar").show(ui, |ui| {
            egui::Aligned::new(egui::Align2::CENTER_TOP).show(ui, |ui| {
                ui.horizontal(|ui| {
                    self.pause_button(ui, snapshot.paused, keyboard_free);
                    clock(ui, format, snapshot.state.time, snapshot.paused);
                    self.speed_slider(ui, snapshot.speed);
                });
            });
        });
    }

    fn pause_button(&self, ui: &mut egui::Ui, paused: bool, keyboard_free: bool) {
        let (icon, action) = if paused {
            (icons::PLAY, "Resume")
        } else {
            (icons::PAUSE, "Pause")
        };

        let shortcut_pressed = keyboard_free && toggle_pause_pressed(ui);

        let shortcut = ui.ctx().format_shortcut(&TOGGLE_PAUSE);
        let button =
            egui::Button::new(RichText::new(icon).size(CONTROL_SIZE)).min_size(PAUSE_BUTTON_SIZE);
        let clicked = ui
            .add(button)
            .on_hover_text(format!("{action} ({shortcut})"))
            .clicked();

        if clicked || shortcut_pressed {
            self.set_paused(!paused);
        }
    }

    fn set_paused(&self, paused: bool) {
        if paused {
            self.runtime.pause();
        } else {
            self.runtime.resume();
        }
    }

    fn speed_slider(&self, ui: &mut egui::Ui, speed: Speed) {
        let mut multiplier = speed.multiplier();
        let slider = egui::Slider::new(
            &mut multiplier,
            Speed::BASE.multiplier()..=Speed::MAX.multiplier(),
        )
        .logarithmic(true)
        .suffix("×");

        let changed = ui.add(slider).on_hover_text("Simulation speed").changed();

        if changed {
            self.runtime.set_speed(Speed::new(multiplier));
        }
    }

    /// Asks for the game as it is in `snapshot` to be saved.
    fn save(
        &mut self,
        shared: &mut Shared<'_>,
        snapshot: &Snapshot,
        kind: SaveKind,
        label: String,
    ) {
        self.last_saved_time = Some(snapshot.state.time);
        if kind == SaveKind::Manual {
            self.save_label = Some(label.clone());
        }

        shared.saves.save(SaveRequest {
            playthrough: self.playthrough,
            kind,
            label,
            speed: snapshot.speed.multiplier(),
            state: Arc::clone(&snapshot.state),
        });
    }

    /// Saves the game by itself if, at `now`, it has run for long enough since the last time.
    fn autosave(&mut self, shared: &mut Shared<'_>, snapshot: &Snapshot, now: Instant) {
        let Some(interval) = shared.settings.autosave_interval() else {
            return;
        };
        if snapshot.paused || now.saturating_duration_since(self.last_autosave) < interval {
            return;
        }

        self.last_autosave = now;
        if self.last_saved_time != Some(snapshot.state.time) {
            self.save(shared, snapshot, SaveKind::Auto, String::new());
        }
    }

    fn open_save_dialog(&mut self, shared: &Shared<'_>, snapshot: &Snapshot) {
        let suggestion = self
            .save_label
            .clone()
            .unwrap_or_else(|| shared.settings.datetime.date(snapshot.state.time));

        self.save_dialog = Some(suggestion);
        self.focus_save_dialog = true;
    }

    fn show_save_dialog(
        &mut self,
        ui: &mut egui::Ui,
        shared: &mut Shared<'_>,
        snapshot: &Snapshot,
    ) {
        let Some(mut label) = self.save_dialog.take() else {
            return;
        };
        let mut save = false;
        let mut cancel = false;

        let modal = egui::Modal::new(egui::Id::unique("save_dialog")).show(ui.ctx(), |ui| {
            ui.set_width(SAVE_DIALOG_WIDTH);
            ui.heading("Save game");
            ui.add_space(4.0);

            let field = ui.add(
                egui::TextEdit::singleline(&mut label)
                    .hint_text("Name of the save")
                    .char_limit(SAVE_LABEL_LIMIT)
                    .desired_width(f32::INFINITY),
            );
            if self.focus_save_dialog {
                field.request_focus();
                self.focus_save_dialog = false;
            }
            let entered =
                field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

            ui.label(
                RichText::new("A save with the same name is replaced.")
                    .weak()
                    .small(),
            );
            ui.add_space(8.0);

            let named = !label.trim().is_empty();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                save = ui.add_enabled(named, egui::Button::new("Save")).clicked()
                    || (entered && named);
                cancel = ui.button("Cancel").clicked();
            });
        });

        if save {
            self.save(shared, snapshot, SaveKind::Manual, label.trim().to_owned());
        } else if !(cancel || modal.should_close()) {
            self.save_dialog = Some(label);
        }
    }
}

fn clock(ui: &mut egui::Ui, format: &DateTimeFormat, time: Timestamp, paused: bool) {
    // Monospace, and padded by the format, so that the controls beside the clock stay put
    // as time passes.
    let now = RichText::new(format.date_time(time))
        .monospace()
        .size(CONTROL_SIZE);

    ui.label(if paused { now.weak() } else { now.strong() });
}

/// Tells the player that the simulation thread is gone, and returns whether they chose to
/// leave the game.
fn stopped_banner(ui: &mut egui::Ui) -> bool {
    let mut leave = false;

    egui::Panel::top("stopped_banner").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.colored_label(
                ui.visuals().error_fg_color,
                "The simulation has stopped unexpectedly. Time no longer passes, but the game \
                 can still be saved.",
            );
            leave = ui.button("Main Menu").clicked();
        });
    });

    leave
}

fn character_panel(ui: &mut egui::Ui, player: &Human, time: Timestamp, format: &DateTimeFormat) {
    let sex = player.sex();
    let birth = player.birth();

    egui::Panel::left("character_panel")
        .resizable(false)
        .exact_size(SIDEBAR_WIDTH)
        .show(ui, |ui| {
            ui.add_space(SIDEBAR_PADDING);
            ui.add(
                egui::Label::new(RichText::new(player.name().to_string()).heading().strong())
                    .wrap(),
            );
            ui.separator();

            egui::Grid::new("character_facts")
                .num_columns(2)
                .spacing(FACT_SPACING)
                .show(ui, |ui| {
                    fact(ui, "Sex", format!("{} {sex}", icons::sex(sex)));
                    fact(ui, "Age", format_age(time, birth));
                    fact(ui, "Born", format.date(birth));
                });
        });
}

fn fact(ui: &mut egui::Ui, key: &str, value: String) {
    ui.label(RichText::new(key).weak());
    ui.label(value);
    ui.end_row();
}

/// Formats the age at `now` of someone born at `birth`, in the largest unit they have
/// completed at least one of.
fn format_age(now: Timestamp, birth: Timestamp) -> String {
    let today = Date::from(now);
    let birthday = Date::from(birth);

    let years = today.years_since(birthday);
    let months = today.months_since(birthday);
    if years > 0 {
        count(years, "year")
    } else if months > 0 {
        count(months, "month")
    } else {
        count(now.seconds_since(birth) / SECONDS_PER_DAY, "day")
    }
}

fn count(number: u64, unit: &str) -> String {
    if number == 1 {
        format!("1 {unit}")
    } else {
        format!("{number} {unit}s")
    }
}

fn content_area(ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |_ui| {});
}

fn toggle_pause_pressed(ui: &egui::Ui) -> bool {
    if ui.ctx().text_edit_focused() {
        return false;
    }

    let egui::KeyboardShortcut {
        modifiers,
        logical_key,
    } = TOGGLE_PAUSE;

    ui.input_mut(|input| {
        let pressed = input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: held,
                    ..
                } if *key == logical_key && held.matches_logically(modifiers)
            )
        });
        input.consume_key(modifiers, logical_key);
        pressed
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use lords_sim::{Name, Sex, Simulation};

    use super::*;
    use crate::scenes::harness::{Harness, wait_until};

    const fn days(days: u64) -> Timestamp {
        Timestamp::from_seconds(days * SECONDS_PER_DAY)
    }

    #[test]
    fn age_is_given_in_the_largest_completed_unit() {
        let birth = Timestamp::EPOCH;

        assert_eq!(format_age(birth, birth), "0 days");
        assert_eq!(format_age(Timestamp::from_seconds(86_399), birth), "0 days");
        assert_eq!(format_age(days(1), birth), "1 day");
        assert_eq!(format_age(days(30), birth), "30 days");
        // 1 February of the first year.
        assert_eq!(format_age(days(31), birth), "1 month");
        assert_eq!(format_age(days(364), birth), "11 months");
        assert_eq!(format_age(days(365), birth), "1 year");
        assert_eq!(format_age(days(2 * 365), birth), "2 years");
    }

    #[test]
    fn age_never_counts_backwards() {
        assert_eq!(format_age(Timestamp::EPOCH, days(400)), "0 days");
    }

    fn overview(paused: bool) -> Overview {
        let name = Name::new("Ada".to_owned(), Vec::new(), "Lord".to_owned());
        let runtime = Runtime::spawn_with(
            Simulation::new(name, Sex::Female),
            Speed::new(10),
            paused,
            || {},
        );

        Overview::new(runtime, SimulationId::generate(), None)
    }

    #[test]
    fn clock_keeps_its_width_as_time_passes() {
        let harness = Harness::new();
        let format = DateTimeFormat::default();
        let mut widths = Vec::new();

        harness.show(|ui| {
            let width = |time| {
                ui.painter()
                    .layout_no_wrap(
                        format.date_time(time),
                        egui::FontId::monospace(CONTROL_SIZE),
                        egui::Color32::WHITE,
                    )
                    .size()
                    .x
            };

            // A stride of about 289 days and a bit, to run through every digit in every place.
            widths = (0..500)
                .map(|step| width(Timestamp::from_seconds(step * 25_000_007)))
                .collect();
        });

        assert!(widths[0] > 0.0);
        assert!(widths.iter().all(|width| *width == widths[0]), "{widths:?}");
    }

    #[test]
    fn save_shortcut_asks_for_a_name_once_and_then_replaces_that_save() {
        let mut harness = Harness::new();
        let mut overview = overview(true);
        harness.frames(&mut overview, 2);

        harness.press(&mut overview, egui::Modifiers::COMMAND, egui::Key::S);
        assert_eq!(overview.save_dialog.as_deref(), Some("01.01.0001"));
        assert!(harness.saves.list().saves.is_empty());

        harness.frames(&mut overview, 3);
        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Enter);
        assert_eq!(overview.save_dialog, None);
        assert_eq!(overview.save_label.as_deref(), Some("01.01.0001"));
        harness.wait_for_save();

        let saves = harness.saves.list().saves;
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].header.kind, SaveKind::Manual);
        assert_eq!(saves[0].header.label, "01.01.0001");
        assert_eq!(saves[0].header.character, "Ada Lord");
        assert_eq!(saves[0].header.speed, 10);
        assert_eq!(saves[0].header.playthrough, overview.playthrough);

        harness.press(&mut overview, egui::Modifiers::COMMAND, egui::Key::S);
        assert_eq!(overview.save_dialog, None);
        harness.wait_for_save();
        assert_eq!(harness.saves.list().saves.len(), 1);
        assert!(harness.saves.error().is_none());
        assert!(harness.saves.saved_notice().is_some());
    }

    #[test]
    fn escape_closes_the_save_dialog_without_saving() {
        let mut harness = Harness::new();
        let mut overview = overview(true);

        harness.press(&mut overview, egui::Modifiers::COMMAND, egui::Key::S);
        harness.frames(&mut overview, 3);
        assert!(overview.save_dialog.is_some());

        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Escape);
        // Once to let go of the name field, once more to close the dialog.
        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Escape);

        assert_eq!(overview.save_dialog, None);
        assert_eq!(overview.save_label, None);
        harness.saves.finish();
        assert!(harness.saves.list().saves.is_empty());
    }

    #[test]
    fn leaving_writes_an_exit_save_that_resumes_the_game() {
        let mut harness = Harness::new();
        let mut overview = overview(true);
        harness.frames(&mut overview, 2);

        harness.exit(&mut overview);
        harness.saves.finish();

        let saves = harness.saves.list().saves;
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].header.kind, SaveKind::Exit);

        let loaded = harness
            .saves
            .load(&saves[0].path)
            .expect("the exit save loads");
        assert_eq!(loaded.state, *overview.runtime.snapshot().state);
        assert_eq!(loaded.header.speed, 10);
    }

    #[test]
    fn game_is_autosaved_once_it_has_run_for_the_interval() {
        let mut harness = Harness::new();
        harness.settings.autosave_minutes = 1;
        let mut overview = overview(false);
        wait_until(|| overview.runtime.snapshot().state.time > Timestamp::EPOCH);
        let snapshot = overview.runtime.snapshot();
        let started = overview.last_autosave;
        let after = |seconds| started + Duration::from_secs(seconds);

        overview.autosave(&mut harness.shared(), &snapshot, after(59));
        assert_eq!(overview.last_saved_time, None);

        overview.autosave(&mut harness.shared(), &snapshot, after(60));
        assert_eq!(overview.last_saved_time, Some(snapshot.state.time));
        harness.wait_for_save();

        // Neither before another interval has passed, nor while the game has not moved on.
        overview.autosave(&mut harness.shared(), &snapshot, after(119));
        overview.autosave(&mut harness.shared(), &snapshot, after(120));
        harness.saves.finish();

        let saves = harness.saves.list().saves;
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].header.kind, SaveKind::Auto);
    }

    #[test]
    fn game_is_not_autosaved_while_paused_or_when_turned_off() {
        let mut harness = Harness::new();
        let much_later = Instant::now() + Duration::from_secs(3_600);

        harness.settings.autosave_minutes = 1;
        let mut paused = overview(true);
        let snapshot = paused.runtime.snapshot();
        paused.autosave(&mut harness.shared(), &snapshot, much_later);
        assert_eq!(paused.last_saved_time, None);

        harness.settings.autosave_minutes = 0;
        let mut running = overview(false);
        wait_until(|| running.runtime.snapshot().state.time > Timestamp::EPOCH);
        let snapshot = running.runtime.snapshot();
        running.autosave(&mut harness.shared(), &snapshot, much_later);
        assert_eq!(running.last_saved_time, None);

        harness.saves.finish();
        assert!(harness.saves.list().saves.is_empty());
    }

    #[test]
    fn space_pauses_and_resumes() {
        let mut harness = Harness::new();
        let mut overview = overview(false);
        harness.frames(&mut overview, 2);

        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Space);
        wait_until(|| overview.runtime.snapshot().paused);

        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Space);
        wait_until(|| !overview.runtime.snapshot().paused);
    }

    #[test]
    fn shortcuts_are_left_alone_while_a_dialog_is_open() {
        let mut harness = Harness::new();
        harness.dialog_open = true;
        let mut overview = overview(false);
        harness.frames(&mut overview, 2);

        harness.press(&mut overview, egui::Modifiers::NONE, egui::Key::Space);
        harness.press(&mut overview, egui::Modifiers::COMMAND, egui::Key::S);
        std::thread::sleep(Duration::from_millis(50));

        assert!(!overview.runtime.snapshot().paused);
        assert_eq!(overview.save_dialog, None);
    }

    #[test]
    fn overview_draws_in_a_small_window_and_stays() {
        let mut harness = Harness::new().sized(480.0, 300.0);
        let mut overview = overview(true);

        // Nothing should ask to leave the game by itself, however cramped it is.
        for _ in 0..3 {
            assert!(matches!(
                harness.frame(&mut overview, Vec::new()),
                SceneResponse::None
            ));
        }
        assert!(overview.runtime.is_running());
    }
}
