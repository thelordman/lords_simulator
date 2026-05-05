use lords_runtime::Runtime;
use lords_sim::{Name, Sex, State};
use crate::scenes;

pub struct App {
    ctx: egui::Context,

    pub runtime: Option<Runtime>,

    pub scene: scenes::Scene,

    pub main_menu_data: scenes::main_menu::MainMenuData,
    pub character_creation_data: scenes::character_creation::CharacterCreationData,

    pub ui_scale: f32,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(2.0);

        Self {
            ctx: cc.egui_ctx.clone(),

            runtime: None,

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

    pub fn state(&self) -> State {
        self.runtime.as_ref().unwrap().state.read().unwrap().clone()
    }

    pub fn start_runtime(&mut self, name: Name, sex: Sex) {
        self.scene = scenes::Scene::Overview;

        let (runtime, tick_rx) = Runtime::new(lords_sim::Simulation::new(name, sex));

        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            while tick_rx.recv().is_ok() {
                ctx.request_repaint();
            }
        });

        self.runtime = Some(runtime);
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        scenes::draw_scene(self, ctx);
    }
}