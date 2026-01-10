use eframe::CreationContext;
use crate::scenes;

pub struct App {
    pub scene: scenes::Scene,

    pub ui_scale: f32,

    pub show_fullscreen_prompt: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            scene: scenes::Scene::MainMenu,
            ui_scale: 2.0,
            show_fullscreen_prompt: true,
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