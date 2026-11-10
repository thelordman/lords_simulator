//! Icons from the [Phosphor](https://phosphoricons.com/) icon font.

use egui::{FontData, FontDefinitions, FontFamily};

const FONT_NAME: &str = "phosphor";

pub const GENDER_FEMALE: &str = "\u{E6E0}";
pub const GENDER_MALE: &str = "\u{E6E2}";
pub const PAUSE: &str = "\u{E39E}";
pub const PLAY: &str = "\u{E3D0}";
pub const PLUS: &str = "\u{E3D4}";
pub const X: &str = "\u{E4F6}";

/// The icon representing the given sex.
pub const fn sex(sex: lords_sim::Sex) -> &'static str {
    match sex {
        lords_sim::Sex::Male => GENDER_MALE,
        lords_sim::Sex::Female => GENDER_FEMALE,
    }
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FONT_NAME.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Phosphor.ttf")).into(),
    );

    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.insert(proportional.len().min(1), FONT_NAME.to_owned());

    ctx.set_fonts(fonts);
}
