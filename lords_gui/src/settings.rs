use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::datetime::DateTimeFormat;

const STORAGE_KEY: &str = "settings";

/// The size of window, in interface points, that the interface is laid out for.
const DESIGN_SIZE: egui::Vec2 = egui::vec2(960.0, 540.0);

const UI_SCALE_STEP: f32 = 0.25;
const MIN_UI_SCALE: f32 = 1.0;
const MAX_UI_SCALE: f32 = 3.0;

/// The interface scale used until one has been picked for the display.
pub const PROVISIONAL_UI_SCALE: f32 = 1.5;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub ui_scale: Option<f32>,
    pub fullscreen_prompt_seen: bool,
    pub theme: egui::ThemePreference,
    pub datetime: DateTimeFormat,
    /// Real minutes between autosaves while the game runs. Zero turns them off.
    pub autosave_minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_scale: None,
            fullscreen_prompt_seen: false,
            theme: egui::ThemePreference::default(),
            datetime: DateTimeFormat::default(),
            autosave_minutes: 5,
        }
    }
}

impl Settings {
    pub const AUTOSAVE_CHOICES: [u32; 4] = [0, 1, 5, 15];

    pub fn load(storage: &dyn eframe::Storage) -> Self {
        eframe::get_value(storage, STORAGE_KEY).unwrap_or_default()
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, self);
    }

    pub fn autosave_interval(&self) -> Option<Duration> {
        (self.autosave_minutes > 0)
            .then(|| Duration::from_secs(u64::from(self.autosave_minutes) * 60))
    }
}

/// Returns the interface scales on offer, smallest first.
pub fn ui_scale_choices() -> impl Iterator<Item = f32> {
    let steps = ((MAX_UI_SCALE - MIN_UI_SCALE) / UI_SCALE_STEP).round() as u16;
    (0..=steps).map(|step| MIN_UI_SCALE + f32::from(step) * UI_SCALE_STEP)
}

pub fn pick_ui_scale(native_size: egui::Vec2) -> f32 {
    let fitting = (native_size.x / DESIGN_SIZE.x).min(native_size.y / DESIGN_SIZE.y);
    let stepped = (fitting / UI_SCALE_STEP).floor() * UI_SCALE_STEP;

    if stepped.is_finite() {
        stepped.clamp(MIN_UI_SCALE, MAX_UI_SCALE)
    } else {
        MIN_UI_SCALE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_fits_the_design_size_on_the_display() {
        // Full HD at 100 % keeps the scale the game used to hard-code.
        assert_eq!(pick_ui_scale(egui::vec2(1920.0, 1080.0)), 2.0);
        // A 13-inch MacBook Air.
        assert_eq!(pick_ui_scale(egui::vec2(1470.0, 956.0)), 1.5);
        assert_eq!(pick_ui_scale(egui::vec2(2560.0, 1440.0)), 2.5);
        assert_eq!(pick_ui_scale(egui::vec2(1366.0, 768.0)), 1.25);
        // The shorter side decides.
        assert_eq!(pick_ui_scale(egui::vec2(3840.0, 1080.0)), 2.0);
    }

    #[test]
    fn scale_stays_within_what_is_on_offer() {
        assert_eq!(pick_ui_scale(egui::vec2(800.0, 600.0)), MIN_UI_SCALE);
        assert_eq!(pick_ui_scale(egui::vec2(7680.0, 4320.0)), MAX_UI_SCALE);
        assert_eq!(pick_ui_scale(egui::Vec2::ZERO), MIN_UI_SCALE);
        assert_eq!(pick_ui_scale(egui::Vec2::NAN), MIN_UI_SCALE);

        let choices: Vec<f32> = ui_scale_choices().collect();
        assert_eq!(choices.first(), Some(&MIN_UI_SCALE));
        assert_eq!(choices.last(), Some(&MAX_UI_SCALE));
        assert!(choices.contains(&pick_ui_scale(egui::vec2(1470.0, 956.0))));
        assert!(choices.contains(&PROVISIONAL_UI_SCALE));
    }

    #[test]
    fn autosave_can_be_turned_off() {
        let mut settings = Settings::default();
        assert_eq!(
            settings.autosave_interval(),
            Some(Duration::from_secs(5 * 60))
        );
        assert!(Settings::AUTOSAVE_CHOICES.contains(&settings.autosave_minutes));

        settings.autosave_minutes = 0;
        assert_eq!(settings.autosave_interval(), None);
    }
}
