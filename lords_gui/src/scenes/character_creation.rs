use lords_sim::{Name, Sex};
use crate::icons;
use super::{top_panel, Scene, SceneResponse};

#[derive(Default)]
pub struct CharacterCreation {
    first_name: String,
    last_name: String,
    middle_names: Vec<String>,
    sex: Option<Sex>,
}

impl Scene for CharacterCreation {
    fn update(&mut self, ui: &mut egui::Ui) -> SceneResponse {
        top_panel(ui);

        self.sex_panel(ui);
        self.central_panel(ui)
    }
}

impl CharacterCreation {
    fn sex_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("sex_panel").show(ui, |ui| {
            ui.label(egui::RichText::new("Sex").size(25.0).strong());

            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.sex == Some(Sex::Male), "Male")
                    .clicked()
                {
                    self.sex = Some(Sex::Male);
                }
                if ui
                    .selectable_label(self.sex == Some(Sex::Female), "Female")
                    .clicked()
                {
                    self.sex = Some(Sex::Female);
                }
            });
        });
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) -> SceneResponse {
        let mut response = SceneResponse::None;

        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(25.0);
                ui.label(
                    egui::RichText::new("Character Creation")
                        .size(50.0)
                        .strong(),
                );
                ui.add_space(25.0);
            });

            self.name_fields(ui);

            ui.add_space(25.0);

            ui.vertical_centered(|ui| {
                if ui
                    .button(egui::RichText::new("Start").size(50.0))
                    .clicked()
                {
                    if let Some(sex) = self.sex {
                        let name = Name::new(
                            self.first_name.clone(),
                            self.middle_names.clone(),
                            self.last_name.clone(),
                        );
                        response = SceneResponse::StartSim { name, sex };
                    }
                }
            });
        });

        response
    }

    fn name_fields(&mut self, ui: &mut egui::Ui) {
        // TODO: Replace hard-coded layout with column layout (macro)
        const TEXT_BOX_WIDTH: f32 = 150.0;
        const TEXT_BOX_HEIGHT: f32 = 20.0;
        const ADD_BUTTON_WIDTH: f32 = 20.0;

        let text_box_count = self.middle_names.len() as f32 + 2.0;
        let spacing = ui.spacing().item_spacing.x;
        let total_width =
            TEXT_BOX_WIDTH * text_box_count + spacing * text_box_count + ADD_BUTTON_WIDTH;

        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            ui.add_space((available_width - total_width) / 2.0);

            Self::name_field(ui, TEXT_BOX_WIDTH, &mut self.first_name, "First Name");

            for (i, name) in self.middle_names.iter_mut().enumerate() {
                let hint = if i > 0 {
                    format!("Middle Name {}", i + 1)
                } else {
                    "Middle Name".to_string()
                };
                Self::name_field(ui, TEXT_BOX_WIDTH, name, &hint);
            }

            if ui
                .add_sized([ADD_BUTTON_WIDTH, TEXT_BOX_HEIGHT], egui::Button::new(egui::RichText::new(icons::PLUS)))
                .on_hover_text("Add Middle Name")
                .clicked()
            {
                if (available_width - total_width + 2 * TEXT_BOX_WIDTH) >= 0f32 {
                    self.middle_names.push(String::new());
                }
            }

            Self::name_field(ui, TEXT_BOX_WIDTH, &mut self.last_name, "Last Name");
        });
    }

    fn name_field(ui: &mut egui::Ui, width: f32, value: &mut String, hint: &str) {
        ui.add_sized(
            [width, 20.0],
            egui::TextEdit::singleline(value).hint_text(hint),
        );
    }
}
