pub struct App {
    label: String,
    value: f32,
    show_fullscreen_prompt: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Welcome to Lord's Simulator".to_owned(),
            value: 2.7,
            show_fullscreen_prompt: true,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(2.0);
        Default::default()
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::warn_if_debug_build(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::github_link_file!(
                        "https://github.com/thelordman/lords_simulator/blob/main/",
                        "Source code"
                    ));
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Lord's Simulator");

                ui.label("I love text: ");
                ui.text_edit_singleline(&mut self.label);
                ui.add_sized(
                    [200.0, 20.0],
                    egui::Slider::new(&mut self.value, 0.0..=10.0).text("value")
                );
                if ui.button("Increment").clicked() {
                    self.value += 1.0;
                }
            });
        });

        if self.show_fullscreen_prompt {
            let mut open = self.show_fullscreen_prompt;
            egui::Window::new("Fullscreen?")
                .open(&mut open)
                .movable(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
                .show(ctx, |ui| {
                ui.label("Can later be configured through the options menu.");
                ui.horizontal(|ui| {
                    if ui.button("Enable").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        self.show_fullscreen_prompt = false;
                    }
                    if ui.button("Borderless windowed").clicked() {
                        if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::Pos2::ZERO));
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(monitor));
                            self.show_fullscreen_prompt = false;
                        } else {
                            log::warn!("Could not determine monitor size for borderless window, maximizing instead");
                            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                            self.show_fullscreen_prompt = false;
                        }
                    }
                    if ui.button("Keep windowed").clicked() {
                        self.show_fullscreen_prompt = false;
                    }
                })
            });
            if !open {
                self.show_fullscreen_prompt = false;
            }
        }
    }
}