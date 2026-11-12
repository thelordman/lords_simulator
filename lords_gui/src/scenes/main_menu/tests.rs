use std::sync::Arc;
use std::time::Duration;

use lords_save::{SaveKind, SaveRequest, SimulationId};
use lords_sim::{Name, Sex, State};

use super::*;
use crate::scenes::harness::Harness;

fn save(harness: &mut Harness, label: &str) {
    harness.saves.save(SaveRequest {
        playthrough: SimulationId::generate(),
        kind: SaveKind::Manual,
        label: label.to_owned(),
        speed: 1,
        state: Arc::new(State::new(Name::default(), Sex::Male)),
    });
    harness.wait_for_save();
    // Far enough apart for the saves to be told apart by age.
    std::thread::sleep(Duration::from_millis(5));
}

#[test]
fn menu_draws_in_any_window() {
    for (width, height) in [(960.0, 540.0), (480.0, 300.0), (1920.0, 1080.0)] {
        let mut harness = Harness::new().sized(width, height);
        save(&mut harness, "there to continue from");
        let mut menu = MainMenu::default();

        for _ in 0..3 {
            assert!(matches!(
                harness.frame(&mut menu, Vec::new()),
                SceneResponse::None
            ));
        }
    }
}

#[test]
fn menu_offers_to_continue_from_the_newest_save() {
    let mut harness = Harness::new();
    let mut menu = MainMenu::default();

    harness.frames(&mut menu, 1);
    let saves = menu.saves.as_ref().expect("the saves were looked at");
    assert!(saves.newest.is_none());
    assert!(!saves.any);

    save(&mut harness, "older");
    save(&mut harness, "newer");
    harness.frames(&mut menu, 1);

    let saves = menu.saves.as_ref().expect("the saves were looked at");
    assert!(saves.any);
    assert_eq!(
        saves.newest.as_ref().map(|save| save.header.label.as_str()),
        Some("newer")
    );
}

#[test]
fn fullscreen_offer_is_not_answered_by_itself() {
    let mut harness = Harness::new();
    let mut menu = MainMenu::default();

    harness.frames(&mut menu, 3);

    assert!(!harness.settings.fullscreen_prompt_seen);
}
