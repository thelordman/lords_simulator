use egui::RichText;
use lords_sim::object::Living;
use lords_sim::time::gregorian::{Date, SECONDS_PER_DAY};
use lords_sim::{Command, Human, Item, Timestamp};

use crate::datetime::DateTimeFormat;
use crate::icons;
use crate::scenes::count;

use super::meter;
use super::words::{about, at_most, item_name};

const PANEL_WIDTH: f32 = 240.0;
const PANEL_PADDING: f32 = 4.0;
const FACT_SPACING: egui::Vec2 = egui::vec2(12.0, 6.0);
const ITEM_SPACING: f32 = 4.0;
const AMOUNT_WIDTH: f32 = 40.0;
const DROP_WIDTH: f32 = 44.0;

/// How many points of dragging it takes to change the amount to drop by all that is carried.
const AMOUNT_DRAG: f64 = 200.0;

pub fn show(
    ui: &mut egui::Ui,
    player: &Human,
    time: Timestamp,
    format: &DateTimeFormat,
) -> Option<Command> {
    let sex = player.sex();
    let birth = player.birth();
    let inventory = player.inventory();
    let mut command = None;

    egui::Panel::left("character_panel")
        .resizable(false)
        .exact_size(PANEL_WIDTH)
        .show(ui, |ui| {
            ui.add_space(PANEL_PADDING);
            ui.add(
                egui::Label::new(RichText::new(player.name().to_string()).heading().strong())
                    .wrap(),
            );
            ui.separator();

            egui::Grid::new("character_facts")
                .num_columns(2)
                .spacing(FACT_SPACING)
                .show(ui, |ui| {
                    fact(ui, "Sex", format!("{} {sex}", icons::sex(sex)));
                    fact(ui, "Age", format_age(time, birth));
                    fact(ui, "Born", format.date(birth));
                    ui.label(RichText::new("Carrying").weak());
                    if inventory.is_empty() {
                        ui.label("Nothing");
                    } else {
                        meter::show(ui, fullness(player));
                    }
                    ui.end_row();
                });

            ui.add_space(ITEM_SPACING);
            for (item, count) in inventory.items() {
                if let Some(dropped) = carried(ui, item, count) {
                    command = Some(Command::Drop {
                        item,
                        count: dropped,
                    });
                }
            }
        });

    command
}

fn fact(ui: &mut egui::Ui, key: &str, value: String) {
    ui.label(RichText::new(key).weak());
    ui.label(value);
    ui.end_row();
}

fn carried(ui: &mut egui::Ui, item: Item, count: u32) -> Option<u32> {
    let id = ui.make_persistent_id(("amount to drop", item));
    let most = at_most(count).max(1);
    let mut amount = ui
        .data(|data| data.get_temp(id))
        .unwrap_or(1)
        .clamp(1, most);
    let mut dropped = None;

    ui.horizontal(|ui| {
        let controls = AMOUNT_WIDTH + DROP_WIDTH + 2.0 * ui.spacing().item_spacing.x;
        let text_width = ui.available_width() - controls;
        ui.allocate_ui(egui::vec2(text_width, 0.0), |ui| {
            ui.set_min_width(text_width);
            let carried = format!("{} {}", about(count), item_name(item, count));
            ui.add(egui::Label::new(carried).wrap());
        });

        let height = ui.spacing().interact_size.y;
        let speed = (f64::from(count) / AMOUNT_DRAG).max(0.05);
        let picker = egui::DragValue::new(&mut amount)
            .range(1..=most)
            .speed(speed);
        ui.add_sized([AMOUNT_WIDTH, height], picker)
            .on_hover_text("How many to drop");
        let drop = ui
            .add_sized([DROP_WIDTH, height], egui::Button::new("Drop"))
            .on_hover_text(format!(
                "Drop {amount} {}\nShift-click to drop all",
                item_name(item, amount)
            ));
        if drop.clicked() {
            let all = ui.input(|input| input.modifiers.shift);
            dropped = Some(if all { count } else { amount });
        }
    });
    ui.data_mut(|data| data.insert_temp(id, amount));
    ui.add_space(ITEM_SPACING);

    dropped
}

fn fullness(player: &Human) -> u8 {
    let room = player.carrying_room().max(1);
    let taken = player.inventory().bulk().min(room);

    ((taken * 100).div_ceil(room)) as u8
}

fn format_age(now: Timestamp, birth: Timestamp) -> String {
    let today = Date::from(now);
    let birthday = Date::from(birth);

    let years = today.years_since(birthday);
    let months = today.months_since(birthday);
    if years > 0 {
        count(years, "year")
    } else if months > 0 {
        count(months, "month")
    } else {
        count(now.seconds_since(birth) / SECONDS_PER_DAY, "day")
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::world::{GroundType, Patch, Terrain};
    use lords_sim::{Location, Name, Sex, Simulation, State, World};

    use super::*;
    use crate::scenes::harness::Harness;

    const fn days(days: u64) -> Timestamp {
        Timestamp::from_seconds(days * SECONDS_PER_DAY)
    }

    #[test]
    fn age_is_given_in_the_largest_completed_unit() {
        let birth = Timestamp::EPOCH;

        assert_eq!(format_age(birth, birth), "0 days");
        assert_eq!(format_age(Timestamp::from_seconds(86_399), birth), "0 days");
        assert_eq!(format_age(days(1), birth), "1 day");
        assert_eq!(format_age(days(30), birth), "30 days");
        // 1 February of the first year.
        assert_eq!(format_age(days(31), birth), "1 month");
        assert_eq!(format_age(days(364), birth), "11 months");
        assert_eq!(format_age(days(365), birth), "1 year");
        assert_eq!(format_age(days(2 * 365), birth), "2 years");
    }

    #[test]
    fn age_never_counts_backwards() {
        assert_eq!(format_age(Timestamp::EPOCH, days(400)), "0 days");
    }

    /// Returns a player who has gathered pebbles on a beach for `seconds`.
    fn gatherer(seconds: u32) -> Human {
        let gravel = Patch::new(GroundType::Gravel, None, 4_000);
        let mut world = World::default();
        world.set_terrain(Terrain::new(Location::new(-2_000, -2_000), 1, [gravel]));
        let mut sim = Simulation::from_state(State {
            world,
            ..State::new(Name::default(), Sex::Female)
        });
        sim.apply(Command::Gather(Item::Pebble));
        for _ in 0..seconds {
            sim.tick();
        }

        sim.view().player
    }

    #[test]
    fn fullness_is_rounded_up_to_a_whole_percent() {
        let pebbles = |player: &Human| player.inventory().count(Item::Pebble);
        let some = gatherer(10);
        let full = gatherer(1_000);
        let expected = (pebbles(&some) * 20_000 * 100).div_ceil(3_000_000) as u8;

        assert_eq!(fullness(&gatherer(0)), 0);
        assert!(pebbles(&some) > 0);
        assert_eq!(fullness(&some), expected);
        assert_eq!(fullness(&full), 100);
    }

    #[test]
    fn panel_draws_with_nothing_and_something_carried() {
        let harness = Harness::new();
        let format = DateTimeFormat::default();
        let empty = Simulation::new(Name::default(), Sex::Male).view().player;
        let carrying = gatherer(30);

        harness.show(|ui| assert_eq!(show(ui, &empty, Timestamp::EPOCH, &format), None));
        harness.show(|ui| assert_eq!(show(ui, &carrying, Timestamp::EPOCH, &format), None));
    }
}
