use lords_runtime::Runtime;

use crate::scenes::{MainMenu, Overview, Scene, SceneResponse};

pub struct App {
    ctx: egui::Context,
    scene: Box<dyn Scene>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(2.0);

        Self {
            ctx: cc.egui_ctx.clone(),
            scene: Box::new(MainMenu::default()),
        }
    }

    fn start_sim(&mut self, name: lords_sim::Name, sex: lords_sim::Sex) {
        let (runtime, tick_rx) = Runtime::new(lords_sim::Simulation::new(name, sex));

        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            while tick_rx.recv_is_ok() {
                ctx.request_repaint();
            }
        });

        self.scene = Box::new(Overview::new(runtime));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.scene.update(ctx) {
            SceneResponse::None => {}
            SceneResponse::Transition(next) => self.scene = next,
            SceneResponse::StartSim { name, sex } => self.start_sim(name, sex),
        }
    }
}
