use super::*;
use crate::scenes::harness::Harness;

fn filled_in() -> CharacterCreation {
    CharacterCreation {
        first_name: "  Ada ".to_owned(),
        middle_names: vec!["Maria".to_owned(), "   ".to_owned(), " Mäki".to_owned()],
        last_name: "Lord".to_owned(),
        sex: Some(Sex::Female),
        focus_middle_name: None,
    }
}

#[test]
fn what_is_missing_is_asked_for_in_order() {
    let mut creation = CharacterCreation::default();
    assert_eq!(creation.missing_requirement(), Some("Enter a first name"));

    creation.first_name = "Ada".to_owned();
    assert_eq!(creation.missing_requirement(), Some("Enter a last name"));

    creation.last_name = "   ".to_owned();
    assert_eq!(creation.missing_requirement(), Some("Enter a last name"));

    creation.last_name = "Lord".to_owned();
    assert_eq!(creation.missing_requirement(), Some("Choose a sex"));

    creation.sex = Some(Sex::Male);
    assert_eq!(creation.missing_requirement(), None);
}

#[test]
fn name_is_trimmed_and_blank_middle_names_dropped() {
    let name = filled_in().build_name();

    assert_eq!(name.first(), "Ada");
    assert_eq!(name.middle(), ["Maria", "Mäki"]);
    assert_eq!(name.last(), "Lord");
    assert_eq!(name.full(), "Ada Maria Mäki Lord");
}

#[test]
fn creation_draws_in_any_window() {
    for (width, height) in [(960.0, 540.0), (480.0, 300.0)] {
        let mut harness = Harness::new().sized(width, height);

        for mut creation in [CharacterCreation::default(), filled_in()] {
            for _ in 0..3 {
                assert!(matches!(
                    harness.frame(&mut creation, Vec::new()),
                    SceneResponse::None
                ));
            }
        }
    }
}

#[test]
fn escape_goes_back_to_the_main_menu() {
    let mut harness = Harness::new();
    let mut creation = CharacterCreation::default();
    harness.frames(&mut creation, 2);

    let response = harness.press(&mut creation, egui::Modifiers::NONE, egui::Key::Escape);

    assert!(matches!(response, SceneResponse::Transition(_)));
}

#[test]
fn escape_is_left_to_an_open_dialog() {
    let mut harness = Harness::new();
    harness.dialog_open = true;
    let mut creation = CharacterCreation::default();
    harness.frames(&mut creation, 2);

    let response = harness.press(&mut creation, egui::Modifiers::NONE, egui::Key::Escape);

    assert!(matches!(response, SceneResponse::None));
}
