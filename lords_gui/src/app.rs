use lords_runtime::SimRunner;
use lords_sim::State;
use crate::scenes;

pub struct App {
    pub sim_runner: Option<SimRunner>,

    pub scene: scenes::Scene,

    pub main_menu_data: scenes::main_menu::MainMenuData,
    pub character_creation_data: scenes::character_creation::CharacterCreationData,

    pub ui_scale: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            sim_runner: None,

            scene: scenes::Scene::MainMenu,

            main_menu_data: scenes::main_menu::MainMenuData { show_fullscreen_prompt: true },
            character_creation_data: scenes::character_creation::CharacterCreationData {
                first_name: String::new(),
                last_name: String::new(),
                middle_names: Vec::new(),
                sex: None,
            },

            ui_scale: 2.0,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(2.0);
        Default::default()
    }

    pub fn state(&self) -> State {
        self.sim_runner.as_ref().unwrap().state.read().unwrap().clone()
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        scenes::draw_scene(self, ctx);
    }
}