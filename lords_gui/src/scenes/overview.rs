use egui::RichText;
use lords_runtime::Runtime;
use lords_sim::object::Living;
use lords_sim::time::gregorian::Date;
use lords_sim::{Human, Timestamp};

use crate::datetime::DateTime;
use crate::icons;

use super::{top_panel, Scene, SceneResponse};

const TOGGLE_PAUSE: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Space);

const CONTROL_SIZE: f32 = 16.0;
const PAUSE_BUTTON_SIZE: egui::Vec2 = egui::Vec2::splat(24.0);

const SIDEBAR_WIDTH: f32 = 200.0;
const SIDEBAR_PADDING: f32 = 4.0;
const FACT_SPACING: egui::Vec2 = egui::vec2(12.0, 6.0);

pub struct Overview {
    runtime: Runtime,
}

impl Overview {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl Scene for Overview {
    fn update(&mut self, ui: &mut egui::Ui) -> SceneResponse {
        let state = self.runtime.state();

        top_panel(ui);
        self.time_bar(ui, state.time);
        character_panel(ui, &state.player, state.time);
        content_area(ui);

        SceneResponse::None
    }
}

impl Overview {
    fn time_bar(&self, ui: &mut egui::Ui, time: Timestamp) {
        let paused = self.runtime.is_paused();

        egui::Panel::top("time_bar").show(ui, |ui| {
            egui::Aligned::new(egui::Align2::CENTER_TOP).show(ui, |ui| {
                ui.horizontal(|ui| {
                    self.pause_button(ui, paused);
                    clock(ui, time, paused);
                });
            });
        });
    }

    fn pause_button(&self, ui: &mut egui::Ui, paused: bool) {
        let (icon, action) = if paused {
            (icons::PLAY, "Resume")
        } else {
            (icons::PAUSE, "Pause")
        };

        let shortcut_pressed = toggle_pause_pressed(ui);

        let shortcut = ui.ctx().format_shortcut(&TOGGLE_PAUSE);
        let button =
            egui::Button::new(RichText::new(icon).size(CONTROL_SIZE)).min_size(PAUSE_BUTTON_SIZE);
        let clicked = ui
            .add(button)
            .on_hover_text(format!("{action} ({shortcut})"))
            .clicked();

        if clicked || shortcut_pressed {
            self.set_paused(!paused);
            ui.ctx().request_repaint();
        }
    }

    fn set_paused(&self, paused: bool) {
        if paused {
            self.runtime.pause();
        } else {
            self.runtime.resume();
        }
    }
}

fn clock(ui: &mut egui::Ui, time: Timestamp, paused: bool) {
    let now = RichText::new(DateTime::from(time).format_default()).size(CONTROL_SIZE);

    ui.label(if paused { now.weak() } else { now.strong() });
}

fn character_panel(ui: &mut egui::Ui, player: &Human, time: Timestamp) {
    let sex = player.sex();
    let age = Date::from(time).years_since(Date::from(player.birth()));

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
                    fact(ui, "Age", format_age(age));
                });
        });
}

fn fact(ui: &mut egui::Ui, key: &str, value: String) {
    ui.label(RichText::new(key).weak());
    ui.label(value);
    ui.end_row();
}

fn format_age(years: u64) -> String {
    match years {
        1 => "1 year".to_owned(),
        years => format!("{years} years"),
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
