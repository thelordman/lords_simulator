use egui::panel::Side;
use crate::{scenes, App};

pub struct CharacterCreationData {
    pub first_name: String,
    pub last_name: String,
    pub middle_names: Vec<String>,
    
    pub sex: Option<Sex>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

pub fn ui(app: &mut App, ctx: &egui::Context) {
    scenes::top_panel(ctx);

    egui::SidePanel::new(Side::Left, "left_panel").show(ctx, |ui| {
        ui.label(
            egui::RichText::new("Sex")
                .size(25.0)
                .strong()
        );

        let mut sex = app.character_creation_data.sex;
        
        ui.horizontal(|ui| {
            if ui.selectable_label(sex.is_some_and(|sex| sex == Sex::Male), "Male").clicked() {
                sex = Some(Sex::Male);
            }
            if ui.selectable_label(sex.is_some_and(|sex| sex == Sex::Female), "Female").clicked() {
                sex = Some(Sex::Female);
            }
        });

        app.character_creation_data.sex = sex;
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(25.0);
            ui.label(
                egui::RichText::new("Character Creation")
                    .size(50.0)
                    .strong()
            );
            ui.add_space(25.0);
        });

        let text_box_width = 150.0;
        let add_button_width = 20.0;
        let text_box_count = app.character_creation_data.middle_names.len() as f32 + 2.0;
        let spacing = ui.spacing().item_spacing.x;
        let total_width = text_box_width * text_box_count + spacing * text_box_count + add_button_width;

        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            ui.add_space((available_width - total_width) / 2.0);

            add_name_field(ui, text_box_width, &mut app.character_creation_data.first_name, "First Name");

            for (i, name) in app.character_creation_data.middle_names.iter_mut().enumerate() {
                add_name_field(
                    ui,
                    text_box_width,
                    name,
                    &format!("Middle Name {}", if i > 0 {(i + 1).to_string()} else {"".to_string()})
                );
            }

            if ui.add_sized(
                [add_button_width, 20.0],
                egui::Button::new("+")
            ).on_hover_text("Add Middle Name").clicked() {
                app.character_creation_data.middle_names.push(String::new());
            }

            add_name_field(ui, text_box_width, &mut app.character_creation_data.last_name, "Last Name");
        });

        ui.add_space(25.0);
        
        ui.vertical_centered(|ui| {
            if ui.button(egui::RichText::new("Start").size(50.0)).clicked() {
                app.scene = scenes::Scene::Overview;
            }
        });
    });
}

fn add_name_field(ui: &mut egui::Ui, width: f32, name: &mut String, hint_text: &str) {
    ui.add_sized(
        [width, 20.0],
        egui::TextEdit::singleline(name)
            .hint_text(hint_text)
    );
}