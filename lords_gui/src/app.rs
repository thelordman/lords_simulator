use eframe::Frame;
use egui::Ui;
use lords_runtime::{Runtime, Speed};
use lords_save::{LoadedSave, SaveKind, SimulationId};
use lords_sim::Simulation;

use crate::saves::Saves;
use crate::scenes::{
    AppAction, MainMenu, Overview, Scene, SceneResponse, Shared, TOGGLE_FULLSCREEN,
};
use crate::settings::{self, Settings};
use crate::{icons, options};

const ERROR_WIDTH: f32 = 320.0;

/// Where the note that the game was saved sits, from the top right corner of the window.
const NOTICE_OFFSET: egui::Vec2 = egui::vec2(-12.0, 64.0);

pub struct App {
    ctx: egui::Context,
    scene: Box<dyn Scene>,
    settings: Settings,
    saves: Saves,
    options_open: bool,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        let settings = cc.storage.map(Settings::load).unwrap_or_default();

        icons::install(ctx);
        // The interface scale is a setting, which the built-in shortcuts would bypass.
        ctx.options_mut(|options| options.zoom_with_keyboard = false);
        ctx.set_zoom_factor(settings.ui_scale.unwrap_or(settings::PROVISIONAL_UI_SCALE));
        ctx.set_theme(settings.theme);

        let save_directory =
            eframe::storage_dir(crate::APP_ID).map(|directory| directory.join("saves"));

        Self {
            ctx: ctx.clone(),
            scene: Box::new(MainMenu::default()),
            saves: Saves::new(save_directory, ctx),
            settings,
            options_open: false,
        }
    }

    /// Picks an interface scale for the display the window is on, unless there is one.
    fn pick_ui_scale(&mut self, ctx: &egui::Context) {
        if self.settings.ui_scale.is_some() {
            return;
        }

        // Sizes come in interface points, which the current scale has already shrunk.
        let zoom = ctx.zoom_factor();
        let display = ctx
            .input(|input| input.viewport().monitor_size)
            .filter(|size| size.x >= 1.0 && size.y >= 1.0)
            .unwrap_or_else(|| ctx.content_rect().size());

        let scale = settings::pick_ui_scale(display * zoom);
        self.settings.ui_scale = Some(scale);
        ctx.set_zoom_factor(scale);
    }

    fn update_scene(&mut self, ui: &mut Ui) {
        let mut shared = Shared {
            settings: &mut self.settings,
            saves: &mut self.saves,
            dialog_open: self.options_open,
        };
        shared.dialog_open |= shared.saves.error().is_some();

        match self.scene.update(ui, &mut shared) {
            SceneResponse::None => {}
            SceneResponse::Transition(next) => self.enter(next),
            SceneResponse::StartSim { name, sex } => {
                let overview = self.overview(
                    Simulation::new(name, sex),
                    Speed::BASE,
                    false,
                    SimulationId::generate(),
                    None,
                );
                self.enter(overview);
            }
            SceneResponse::LoadSim(save) => {
                let LoadedSave { header, state } = *save;
                let label = (header.kind == SaveKind::Manual).then_some(header.label);
                // Loaded games wait for the player rather than running on at once.
                let overview = self.overview(
                    Simulation::from_state(state),
                    Speed::new(header.speed),
                    true,
                    header.playthrough,
                    label,
                );
                self.enter(overview);
            }
            SceneResponse::App(action) => self.perform(action, ui.ctx()),
        }
    }

    fn overview(
        &self,
        sim: Simulation,
        speed: Speed,
        paused: bool,
        playthrough: SimulationId,
        save_label: Option<String>,
    ) -> Box<dyn Scene> {
        let ctx = self.ctx.clone();
        let runtime = Runtime::spawn_with(sim, speed, paused, move || {
            ctx.request_repaint();
        });

        Box::new(Overview::new(runtime, playthrough, save_label))
    }

    /// Leaves the current scene for `next`.
    fn enter(&mut self, next: Box<dyn Scene>) {
        self.exit_scene();
        self.scene = next;
    }

    fn exit_scene(&mut self) {
        self.scene.on_exit(&mut Shared {
            settings: &mut self.settings,
            saves: &mut self.saves,
            dialog_open: false,
        });
    }

    fn perform(&mut self, action: AppAction, ctx: &egui::Context) {
        match action {
            AppAction::OpenOptions => self.options_open = true,
            AppAction::ToggleFullscreen => {
                let fullscreen = !options::is_fullscreen(ctx);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(fullscreen));
            }
            AppAction::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    fn error_dialog(&mut self, ctx: &egui::Context) {
        let Some(error) = self.saves.error() else {
            return;
        };

        let mut dismissed = false;
        let modal = egui::Modal::new(egui::Id::unique("error")).show(ctx, |ui| {
            ui.set_width(ERROR_WIDTH);
            ui.heading("Something went wrong");
            ui.add_space(4.0);
            ui.label(error);
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                dismissed = ui.button("OK").clicked();
            });
        });

        if dismissed || modal.should_close() {
            self.saves.dismiss_error();
        }
    }

    fn saved_notice(&self, ctx: &egui::Context) {
        let Some(remaining) = self.saves.saved_notice() else {
            return;
        };

        egui::Area::new(egui::Id::unique("saved_notice"))
            .anchor(egui::Align2::RIGHT_TOP, NOTICE_OFFSET)
            .order(egui::Order::Tooltip)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label("Game saved");
                });
            });
        ctx.request_repaint_after(remaining);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();

        self.saves.poll();
        self.pick_ui_scale(&ctx);
        if ctx.input_mut(|input| input.consume_shortcut(&TOGGLE_FULLSCREEN)) {
            self.perform(AppAction::ToggleFullscreen, &ctx);
        }

        self.update_scene(ui);

        if self.options_open {
            self.options_open = options::show(&ctx, &mut self.settings);
        }
        self.error_dialog(&ctx);
        self.saved_notice(&ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.settings.save(storage);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.exit_scene();
        self.saves.finish();
    }
}
