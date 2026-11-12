use crate::datetime::{self, DateOrder, DateSeparator, TimeSeparator};
use crate::settings::{self, Settings};

const DIALOG_WIDTH: f32 = 340.0;
const CHOICE_WIDTH: f32 = 170.0;
const ROW_SPACING: egui::Vec2 = egui::vec2(16.0, 8.0);

pub fn show(ctx: &egui::Context, settings: &mut Settings) -> bool {
    let mut close = false;

    let modal = egui::Modal::new(egui::Id::unique("options")).show(ctx, |ui| {
        ui.set_width(DIALOG_WIDTH);
        ui.heading("Options");
        ui.separator();

        section(ui, "Display");
        egui::Grid::new("display_options")
            .num_columns(2)
            .spacing(ROW_SPACING)
            .show(ui, |ui| {
                ui.label("Interface size");
                ui_scale(ui, settings);
                ui.end_row();

                ui.label("Theme");
                theme(ui, settings);
                ui.end_row();

                ui.label("Fullscreen");
                fullscreen(ui);
                ui.end_row();
            });

        section(ui, "Date and time");
        egui::Grid::new("datetime_options")
            .num_columns(2)
            .spacing(ROW_SPACING)
            .show(ui, |ui| {
                let format = &mut settings.datetime;

                ui.label("Date order");
                choice(
                    ui,
                    "date_order",
                    &mut format.date_order,
                    DateOrder::ALL,
                    |v| v.label().to_owned(),
                );
                ui.end_row();

                ui.label("Date separator");
                choice(
                    ui,
                    "date_separator",
                    &mut format.date_separator,
                    DateSeparator::ALL,
                    |v| v.label().to_owned(),
                );
                ui.end_row();

                ui.label("Time separator");
                choice(
                    ui,
                    "time_separator",
                    &mut format.time_separator,
                    TimeSeparator::ALL,
                    |v| v.label().to_owned(),
                );
                ui.end_row();

                ui.label("Looks like");
                ui.label(
                    egui::RichText::new(format.date_time(datetime::SAMPLE))
                        .monospace()
                        .strong(),
                );
                ui.end_row();
            });

        section(ui, "Saving");
        egui::Grid::new("saving_options")
            .num_columns(2)
            .spacing(ROW_SPACING)
            .show(ui, |ui| {
                ui.label("Autosave");
                choice(
                    ui,
                    "autosave",
                    &mut settings.autosave_minutes,
                    Settings::AUTOSAVE_CHOICES,
                    autosave_label,
                );
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.separator();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            close = ui.button("Close").clicked();
        });
    });

    !(close || modal.should_close())
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(title).strong());
    ui.add_space(2.0);
}

fn ui_scale(ui: &mut egui::Ui, settings: &mut Settings) {
    let mut scale = settings.ui_scale.unwrap_or_else(|| ui.ctx().zoom_factor());

    if choice(
        ui,
        "ui_scale",
        &mut scale,
        settings::ui_scale_choices(),
        |v| format!("{:.0} %", v * 100.0),
    ) {
        settings.ui_scale = Some(scale);
        ui.ctx().set_zoom_factor(scale);
    }
}

fn theme(ui: &mut egui::Ui, settings: &mut Settings) {
    use egui::ThemePreference::{Dark, Light, System};

    if choice(
        ui,
        "theme",
        &mut settings.theme,
        [System, Light, Dark],
        |v| {
            match v {
                System => "Same as system",
                Light => "Light",
                Dark => "Dark",
            }
            .to_owned()
        },
    ) {
        ui.ctx().set_theme(settings.theme);
    }
}

fn fullscreen(ui: &mut egui::Ui) {
    let mut fullscreen = is_fullscreen(ui.ctx());

    if ui.checkbox(&mut fullscreen, "").changed() {
        ui.send_viewport_cmd(egui::ViewportCommand::Fullscreen(fullscreen));
    }
}

pub fn is_fullscreen(ctx: &egui::Context) -> bool {
    ctx.input(|input| input.viewport().fullscreen)
        .unwrap_or(false)
}

fn autosave_label(minutes: u32) -> String {
    match minutes {
        0 => "Off".to_owned(),
        1 => "Every minute".to_owned(),
        minutes => format!("Every {minutes} minutes"),
    }
}

fn choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: &str,
    current: &mut T,
    choices: impl IntoIterator<Item = T>,
    label: impl Fn(T) -> String,
) -> bool {
    let before = *current;

    egui::ComboBox::from_id_salt(id)
        .width(CHOICE_WIDTH)
        .selected_text(label(*current))
        .show_ui(ui, |ui| {
            for choice in choices {
                ui.selectable_value(current, choice, label(choice));
            }
        });

    *current != before
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shows the dialog for one frame with `events` as input, and returns whether it wants
    /// to stay open.
    fn frame(ctx: &egui::Context, settings: &mut Settings, events: Vec<egui::Event>) -> bool {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(960.0, 540.0),
            )),
            focused: true,
            events,
            ..Default::default()
        };

        let mut open = true;
        ctx.run_ui(input, |ui| open &= show(ui.ctx(), settings))
            .drop_without_applying_deltas();
        open
    }

    #[test]
    fn dialog_stays_open_until_escape_and_changes_nothing_by_itself() {
        let ctx = egui::Context::default();
        let mut settings = Settings::default();

        for _ in 0..3 {
            assert!(frame(&ctx, &mut settings, Vec::new()));
        }
        assert_eq!(settings, Settings::default());

        let escape = |pressed| egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        assert!(!frame(
            &ctx,
            &mut settings,
            vec![escape(true), escape(false)]
        ));
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn every_autosave_choice_has_a_label() {
        let labels: Vec<String> = Settings::AUTOSAVE_CHOICES
            .into_iter()
            .map(autosave_label)
            .collect();

        assert_eq!(
            labels,
            ["Off", "Every minute", "Every 5 minutes", "Every 15 minutes"]
        );
    }
}
