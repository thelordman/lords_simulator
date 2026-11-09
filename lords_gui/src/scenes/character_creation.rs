use egui::RichText;
use lords_sim::{Name, Sex};

use crate::icons;

use super::{top_panel, Scene, SceneResponse};

const MAX_MIDDLE_NAMES: usize = 3;
const NAME_CHAR_LIMIT: usize = 24;

const CARD_MAX_WIDTH: f32 = 380.0;
const CARD_MARGIN: i8 = 20;

const FIELD_MARGIN_X: i8 = 8;
const FIELD_MARGIN_Y: i8 = 5;
const FIELD_SPACING: f32 = 6.0;
const SECTION_SPACING: f32 = 16.0;

#[derive(Default)]
pub struct CharacterCreation {
    first_name: String,
    middle_names: Vec<String>,
    last_name: String,
    sex: Option<Sex>,
    focus_middle_name: Option<usize>,
}

impl Scene for CharacterCreation {
    fn update(&mut self, ui: &mut egui::Ui) -> SceneResponse {
        top_panel(ui);

        let mut start = false;

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(24.0);
                        ui.label(RichText::new("Character Creation").size(40.0).strong());
                        ui.add_space(20.0);
                    });

                    egui::Aligned::new(egui::Align2::CENTER_TOP).show(ui, |ui| {
                        start = self.card(ui);
                    });

                    ui.add_space(24.0);
                });
        });

        match self.sex {
            Some(sex) if start && self.missing_requirement().is_none() => SceneResponse::StartSim {
                name: self.build_name(),
                sex,
            },
            _ => SceneResponse::None,
        }
    }
}

impl CharacterCreation {
    fn card(&mut self, ui: &mut egui::Ui) -> bool {
        let frame = egui::Frame::window(ui.style()).inner_margin(CARD_MARGIN);
        let content_width =
            (ui.available_width().min(CARD_MAX_WIDTH) - frame.total_margin().sum().x).max(0.0);

        frame
            .show(ui, |ui| {
                ui.set_width(content_width);

                section_heading(ui, "Name");
                let submitted = self.name_fields(ui);

                ui.add_space(SECTION_SPACING);
                section_heading(ui, "Sex");
                self.sex_buttons(ui);

                ui.add_space(SECTION_SPACING);
                ui.separator();
                ui.add_space(8.0);
                self.name_preview(ui);
                ui.add_space(12.0);

                let clicked = self.start_button(ui);
                clicked || submitted
            })
            .inner
    }

    fn name_fields(&mut self, ui: &mut egui::Ui) -> bool {
        let mut submitted = false;

        field_label(ui, "First name");
        let width = ui.available_width();
        let field = name_field(ui, &mut self.first_name, "First name".to_owned(), width);
        submitted |= enter_pressed(ui, &field);
        ui.add_space(FIELD_SPACING);

        self.middle_name_fields(ui, &mut submitted);
        ui.add_space(FIELD_SPACING);

        field_label(ui, "Last name");
        let width = ui.available_width();
        let field = name_field(ui, &mut self.last_name, "Last name".to_owned(), width);
        submitted |= enter_pressed(ui, &field);

        submitted
    }

    fn middle_name_fields(&mut self, ui: &mut egui::Ui, submitted: &mut bool) {
        let count = self.middle_names.len();

        ui.horizontal(|ui| {
            field_label(ui, "Middle names");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{count} / {MAX_MIDDLE_NAMES}"))
                        .weak()
                        .small(),
                );
            });
        });

        let focus = self.focus_middle_name.take();
        let mut remove = None;

        for (i, name) in self.middle_names.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let button_width =
                    ui.text_style_height(&egui::TextStyle::Body) + 2.0 * f32::from(FIELD_MARGIN_Y);
                let field_width =
                    (ui.available_width() - button_width - ui.spacing().item_spacing.x).max(0.0);
                let hint = if i == 0 {
                    "Middle name".to_owned()
                } else {
                    format!("Middle name {}", i + 1)
                };

                let field = name_field(ui, name, hint, field_width);
                if focus == Some(i) {
                    field.request_focus();
                }
                *submitted |= enter_pressed(ui, &field);

                if ui
                    .add_sized(
                        [button_width, field.rect.height()],
                        egui::Button::new(icons::X),
                    )
                    .on_hover_text("Remove middle name")
                    .clicked()
                {
                    remove = Some(i);
                }
            });
        }

        if let Some(i) = remove {
            self.middle_names.remove(i);
        }

        let can_add = count < MAX_MIDDLE_NAMES;
        let add = ui
            .add_enabled(can_add, egui::Button::new((icons::PLUS, "Add middle name")))
            .on_disabled_hover_text(format!("Up to {MAX_MIDDLE_NAMES} middle names"));
        if add.clicked() && can_add {
            self.middle_names.push(String::new());
            self.focus_middle_name = Some(self.middle_names.len() - 1);
        }
    }

    fn sex_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let width = ((ui.available_width() - ui.spacing().item_spacing.x) / 2.0).max(0.0);
            let height = ui.spacing().interact_size.y * 1.6;

            for (sex, icon, label) in [
                (Sex::Male, icons::GENDER_MALE, "Male"),
                (Sex::Female, icons::GENDER_FEMALE, "Female"),
            ] {
                let button = egui::Button::new((icon, label))
                    .selected(self.sex == Some(sex))
                    .min_size(egui::vec2(width, height));
                if ui.add(button).clicked() {
                    self.sex = Some(sex);
                }
            }
        });
    }

    fn name_preview(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            let full = self.build_name().full();
            let full = full.trim();
            if full.is_empty() {
                ui.label(RichText::new("…").weak().size(20.0));
            } else {
                ui.label(RichText::new(full).strong().size(20.0));
            }
        });
    }

    fn start_button(&self, ui: &mut egui::Ui) -> bool {
        let missing = self.missing_requirement();

        let button = egui::Button::new((
            RichText::new(icons::PLAY).size(20.0),
            RichText::new("Start").size(20.0),
        ))
        .min_size(egui::vec2(ui.available_width(), 40.0));

        let mut response = ui.add_enabled(missing.is_none(), button);
        if let Some(missing) = missing {
            response = response.on_disabled_hover_text(missing);
        }
        response.clicked()
    }

    fn missing_requirement(&self) -> Option<&'static str> {
        if self.first_name.trim().is_empty() {
            Some("Enter a first name")
        } else if self.last_name.trim().is_empty() {
            Some("Enter a last name")
        } else if self.sex.is_none() {
            Some("Choose a sex")
        } else {
            None
        }
    }

    fn build_name(&self) -> Name {
        let middle = self
            .middle_names
            .iter()
            .map(|name| name.trim())
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .collect();

        Name::new(
            self.first_name.trim().to_owned(),
            middle,
            self.last_name.trim().to_owned(),
        )
    }
}

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(18.0).strong());
    ui.add_space(4.0);
}

fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).weak());
}

fn name_field(ui: &mut egui::Ui, value: &mut String, hint: String, width: f32) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            .char_limit(NAME_CHAR_LIMIT)
            .desired_width(width)
            .margin(egui::Margin::symmetric(FIELD_MARGIN_X, FIELD_MARGIN_Y)),
    )
}

fn enter_pressed(ui: &egui::Ui, field: &egui::Response) -> bool {
    field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
}
