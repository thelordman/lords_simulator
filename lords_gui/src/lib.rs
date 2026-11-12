mod app;
mod datetime;
mod icons;
mod options;
mod saves;
mod scenes;
mod settings;

pub use app::App;

/// Names the game to the system, and with it where the game keeps its settings and saves.
pub const APP_ID: &str = "lords_simulator";
