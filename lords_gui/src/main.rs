#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(lords_gui::APP_ID)
            .with_maximized(true)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../../assets/icon-256.png")[..])
                    .expect(
                        "Failed to decode icon from binary, which means the embedded PNG data is malformed."
                    ),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Lord's Simulator",
        native_options,
        Box::new(|cc| Ok(Box::new(lords_gui::App::new(cc)))),
    )
}
