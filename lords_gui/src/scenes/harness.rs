use std::time::{Duration, Instant};

use tempfile::TempDir;

use crate::saves::Saves;
use crate::settings::Settings;

use super::{Scene, SceneResponse, Shared};

const TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) struct Harness {
    pub(crate) ctx: egui::Context,
    pub(crate) settings: Settings,
    pub(crate) saves: Saves,
    pub(crate) dialog_open: bool,
    size: egui::Vec2,
    time: f64,
    _directory: TempDir,
}

impl Harness {
    /// Returns a harness with default settings and an empty directory of saves.
    pub(crate) fn new() -> Self {
        let directory = tempfile::tempdir().expect("a temporary directory can be made");
        let ctx = egui::Context::default();

        Self {
            saves: Saves::new(Some(directory.path().join("saves")), &ctx),
            ctx,
            settings: Settings::default(),
            dialog_open: false,
            size: egui::vec2(960.0, 540.0),
            time: 0.0,
            _directory: directory,
        }
    }

    pub(crate) fn sized(mut self, width: f32, height: f32) -> Self {
        self.size = egui::vec2(width, height);
        self
    }

    pub(crate) fn frame(
        &mut self,
        scene: &mut dyn Scene,
        events: Vec<egui::Event>,
    ) -> SceneResponse {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.size)),
            time: Some(self.time),
            focused: true,
            events,
            ..Default::default()
        };
        self.time += 1.0 / 60.0;
        self.saves.poll();

        let mut response = SceneResponse::None;
        self.ctx
            .run_ui(input, |ui| {
                let mut shared = Shared {
                    settings: &mut self.settings,
                    saves: &mut self.saves,
                    dialog_open: self.dialog_open,
                };
                let pass_response = scene.update(ui, &mut shared);
                if matches!(response, SceneResponse::None) {
                    response = pass_response;
                }
            })
            .drop_without_applying_deltas();
        response
    }

    pub(crate) fn show(&self, add_contents: impl FnMut(&mut egui::Ui)) {
        self.ctx
            .run_ui(egui::RawInput::default(), add_contents)
            .drop_without_applying_deltas();
    }

    pub(crate) fn frames(&mut self, scene: &mut dyn Scene, count: usize) {
        for _ in 0..count {
            self.frame(scene, Vec::new());
        }
    }

    pub(crate) fn press(
        &mut self,
        scene: &mut dyn Scene,
        modifiers: egui::Modifiers,
        key: egui::Key,
    ) -> SceneResponse {
        let event = |pressed| egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        };
        self.frame(scene, vec![event(true), event(false)])
    }

    pub(crate) fn shared(&mut self) -> Shared<'_> {
        Shared {
            settings: &mut self.settings,
            saves: &mut self.saves,
            dialog_open: self.dialog_open,
        }
    }

    pub(crate) fn exit(&mut self, scene: &mut dyn Scene) {
        scene.on_exit(&mut self.shared());
    }

    pub(crate) fn wait_for_save(&mut self) {
        let generation = self.saves.generation();
        wait_until(|| {
            self.saves.poll();
            self.saves.generation() != generation
        });
    }
}

pub(crate) fn wait_until(mut condition: impl FnMut() -> bool) {
    let start = Instant::now();
    while !condition() {
        assert!(start.elapsed() < TIMEOUT, "waited too long");
        std::thread::sleep(Duration::from_millis(1));
    }
}
