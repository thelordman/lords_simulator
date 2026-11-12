use std::sync::Arc;
use std::time::Duration;

use lords_save::SaveRequest;
use lords_sim::{Name, Sex, State};

use super::*;
use crate::scenes::harness::Harness;

fn request(playthrough: SimulationId, kind: SaveKind, first: &str, label: &str) -> SaveRequest {
    let name = Name::new(first.to_owned(), Vec::new(), "Lord".to_owned());
    SaveRequest {
        playthrough,
        kind,
        label: label.to_owned(),
        speed: 1,
        state: Arc::new(State::new(name, Sex::Female)),
    }
}

/// Writes saves one after another, far enough apart to be told apart by age.
fn save_all(harness: &mut Harness, requests: impl IntoIterator<Item = SaveRequest>) {
    for request in requests {
        harness.saves.save(request);
        harness.wait_for_save();
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn time_ago_uses_the_largest_whole_unit() {
    let now = 10 * MS_PER_DAY;

    assert_eq!(time_ago(now, now), "just now");
    assert_eq!(time_ago(now, now - 59_999), "just now");
    assert_eq!(time_ago(now, now - MS_PER_MINUTE), "1 minute ago");
    assert_eq!(time_ago(now, now - 59 * MS_PER_MINUTE), "59 minutes ago");
    assert_eq!(time_ago(now, now - MS_PER_HOUR), "1 hour ago");
    assert_eq!(time_ago(now, now - 23 * MS_PER_HOUR), "23 hours ago");
    assert_eq!(time_ago(now, now - MS_PER_DAY), "1 day ago");
    assert_eq!(time_ago(now, 0), "10 days ago");
}

#[test]
fn time_ago_copes_with_a_clock_set_back() {
    assert_eq!(time_ago(1_000, 5_000), "just now");
}

#[test]
fn saves_are_grouped_by_playthrough_newest_first() {
    let mut harness = Harness::new();
    let ada = SimulationId::generate();
    let bob = SimulationId::generate();
    save_all(
        &mut harness,
        [
            request(ada, SaveKind::Manual, "Ada", "a1"),
            request(bob, SaveKind::Manual, "Bob", "b1"),
            request(ada, SaveKind::Manual, "Ada", "a2"),
            request(bob, SaveKind::Manual, "Bob", "b2"),
        ],
    );

    let playthroughs = group(harness.saves.list().saves);

    let labels = |playthrough: &Playthrough| -> Vec<String> {
        playthrough
            .saves
            .iter()
            .map(|save| save.header.label.clone())
            .collect()
    };
    assert_eq!(playthroughs.len(), 2);
    assert_eq!(playthroughs[0].character, "Bob Lord");
    assert_eq!(labels(&playthroughs[0]), ["b2", "b1"]);
    assert_eq!(playthroughs[1].character, "Ada Lord");
    assert_eq!(labels(&playthroughs[1]), ["a2", "a1"]);
}

#[test]
fn saves_are_titled_by_kind() {
    let mut harness = Harness::new();
    let playthrough = SimulationId::generate();
    save_all(
        &mut harness,
        [
            request(playthrough, SaveKind::Manual, "Ada", "Before the storm"),
            request(playthrough, SaveKind::Manual, "Ada", ""),
            request(playthrough, SaveKind::Auto, "Ada", ""),
            request(playthrough, SaveKind::Exit, "Ada", ""),
        ],
    );

    let titles: Vec<String> = harness.saves.list().saves.iter().map(title).collect();

    assert_eq!(
        titles,
        ["Exit save", "Autosave", "Unnamed save", "Before the storm"]
    );
}

#[test]
fn list_draws_with_saves_and_damaged_files() {
    for (width, height) in [(960.0, 540.0), (480.0, 300.0)] {
        let mut harness = Harness::new().sized(width, height);
        save_all(
            &mut harness,
            [
                request(SimulationId::generate(), SaveKind::Manual, "Ada", "one"),
                request(SimulationId::generate(), SaveKind::Exit, "Bob", ""),
            ],
        );
        let saved = harness.saves.list().saves;
        let directory = saved[0].path.parent().expect("saves are in a directory");
        std::fs::write(directory.join("damaged.lss"), b"nonsense").expect("the file is written");
        let mut scene = LoadGame::default();

        for _ in 0..3 {
            assert!(matches!(
                harness.frame(&mut scene, Vec::new()),
                SceneResponse::None
            ));
        }

        let list = scene.list.as_ref().expect("the saves were read");
        assert_eq!(list.playthroughs.len(), 2);
        assert_eq!(list.unreadable.len(), 1);
    }
}

#[test]
fn list_is_read_again_once_the_saves_change() {
    let shown = |scene: &LoadGame| scene.list.as_ref().map(|list| list.playthroughs.len());
    let mut harness = Harness::new();
    let mut scene = LoadGame::default();

    harness.frames(&mut scene, 2);
    assert_eq!(shown(&scene), Some(0));

    save_all(
        &mut harness,
        [request(
            SimulationId::generate(),
            SaveKind::Manual,
            "Ada",
            "x",
        )],
    );
    harness.frames(&mut scene, 1);
    assert_eq!(shown(&scene), Some(1));

    let path = harness.saves.list().saves[0].path.clone();
    harness.saves.delete(&path);
    harness.frames(&mut scene, 1);
    assert_eq!(shown(&scene), Some(0));
    assert!(harness.saves.error().is_none());
}

#[test]
fn escape_goes_back_to_the_main_menu() {
    let mut harness = Harness::new();
    let mut scene = LoadGame::default();
    harness.frames(&mut scene, 2);

    let response = harness.press(&mut scene, egui::Modifiers::NONE, egui::Key::Escape);

    assert!(matches!(response, SceneResponse::Transition(_)));
}

#[test]
fn escape_cancels_a_deletion_instead_of_going_back() {
    let mut harness = Harness::new();
    save_all(
        &mut harness,
        [request(
            SimulationId::generate(),
            SaveKind::Manual,
            "Ada",
            "x",
        )],
    );
    let path = harness.saves.list().saves[0].path.clone();
    let mut scene = LoadGame {
        deleting: Some(Deletion {
            path,
            description: "x".to_owned(),
        }),
        ..LoadGame::default()
    };
    harness.frames(&mut scene, 2);

    let response = harness.press(&mut scene, egui::Modifiers::NONE, egui::Key::Escape);

    assert!(matches!(response, SceneResponse::None));
    assert!(scene.deleting.is_none());
    assert_eq!(harness.saves.list().saves.len(), 1);
}
