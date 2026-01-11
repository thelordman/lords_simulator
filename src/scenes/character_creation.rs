use crate::scenes;

pub fn ui(ctx: &egui::Context) {
    scenes::top_panel(ctx);

    let mut first_name = String::new();
    let mut last_name = String::new();

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
        let spacing = ui.spacing().item_spacing.x;
        let total_width = text_box_width * 2.0 + 2.0 * spacing + add_button_width;

        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            ui.add_space((available_width - total_width) / 2.0);

            ui.add_sized(
                [text_box_width, 20.0],
                egui::TextEdit::singleline(&mut first_name)
                    .hint_text("First Name")
            );
            if ui.add_sized(
                [add_button_width, 20.0],
                egui::Button::new("+")
            ).on_hover_text("Add Middle Name").clicked() {

            }
            ui.add_sized(
                [text_box_width, 20.0],
                egui::TextEdit::singleline(&mut last_name)
                    .hint_text("Last Name")
            );
        });
    });
}