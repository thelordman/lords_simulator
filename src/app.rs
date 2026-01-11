use eframe::CreationContext;
use crate::scenes;

pub struct App {
    pub scene: scenes::Scene,

    pub main_menu_data: scenes::MainMenuData,

    pub ui_scale: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            scene: scenes::Scene::MainMenu,
            main_menu_data: scenes::MainMenuData { show_fullscreen_prompt: true },
            ui_scale: 2.0,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(2.0);
        Default::default()
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        scenes::draw_scene(self, ctx);
    }
}