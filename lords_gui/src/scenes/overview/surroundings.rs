mod ground;

use egui::RichText;
use lords_sim::object::{Activity, Living};
use lords_sim::perception::Sighting;
use lords_sim::world::Kind;
use lords_sim::{Command, Subject, View};

use super::Pointed;
use super::words::{compass_point, distance, gathering, kind_name, roughly};

const ROW_SPACING: egui::Vec2 = egui::vec2(16.0, 6.0);
const SECTION_SPACING: f32 = 12.0;

const COLUMN_WIDTH: f32 = 90.0;

/// How many rows of a section show at most without scrolling.
const ROWS_SHOWN: f32 = 5.0;

pub fn show(ui: &mut egui::Ui, view: &View, highlighted: Option<Subject>) -> Pointed {
    let mut pointed = Pointed::default();
    let highlighted = highlighted.and_then(|subject| row_of(view, subject));

    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if activity(ui, view) {
                    pointed.command = Some(Command::Stop);
                }
                if let Some(command) = ground::show(ui, view.ground.as_ref(), &view.player) {
                    pointed.command = Some(command);
                }

                let surroundings = &view.surroundings;
                for category in Category::ALL {
                    let reachable = surroundings.within_reach.iter();
                    let farther = surroundings.nearby.iter().chain(&surroundings.in_sight);
                    let rows: Vec<Row<'_>> = reachable
                        .map(|sighting| (sighting, true))
                        .chain(farther.map(|sighting| (sighting, false)))
                        .filter(|(sighting, _)| Category::of(sighting.kind) == category)
                        .map(|(sighting, within_reach)| Row {
                            sighting,
                            within_reach,
                            highlighted: highlighted == Some(sighting.subject),
                        })
                        .collect();

                    let section = section(ui, category, &rows);
                    if let Some(subject) = section.chosen {
                        pointed.command = Some(Command::WalkTo(subject));
                    }
                    pointed.hovered = pointed.hovered.or(section.hovered);
                }
            });
    });

    pointed
}

fn row_of(view: &View, subject: Subject) -> Option<Subject> {
    let surroundings = &view.surroundings;
    if surroundings.sighting(subject).is_some() {
        return Some(subject);
    }

    let mark = view
        .map
        .things
        .iter()
        .find(|mark| mark.subject == subject)?;
    surroundings
        .in_sight
        .iter()
        .find(|sighting| mark.in_sight() && sighting.kind == mark.kind)
        .map(|sighting| sighting.subject)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Category {
    Trees,
    Rocks,
    Water,
}

impl Category {
    const ALL: [Self; 3] = [Self::Trees, Self::Rocks, Self::Water];

    fn of(kind: Kind) -> Self {
        match kind {
            Kind::Plant(_) => Self::Trees,
            Kind::Rock => Self::Rocks,
            Kind::Lake => Self::Water,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Trees => "Trees",
            Self::Rocks => "Rocks",
            Self::Water => "Water",
        }
    }
}

struct Row<'a> {
    sighting: &'a Sighting,
    within_reach: bool,
    highlighted: bool,
}

#[derive(Default)]
struct Section {
    chosen: Option<Subject>,
    hovered: Option<Subject>,
}

fn activity(ui: &mut egui::Ui, view: &View) -> bool {
    let mut stop = false;

    ui.horizontal(|ui| {
        ui.set_min_height(ui.spacing().interact_size.y);
        match doing(view) {
            Some(doing) => {
                ui.label(doing);
                stop = ui.button("Stop").clicked();
            }
            None => {
                ui.label(RichText::new("Standing still").weak());
            }
        }
    });
    ui.add_space(SECTION_SPACING);

    stop
}

fn doing(view: &View) -> Option<String> {
    match view.player.activity()? {
        Activity::Walking { .. } => {
            let walk = view.walk.as_ref()?;
            Some(format!(
                "Walking to the {}, {} to go",
                kind_name(walk.toward).to_lowercase(),
                distance(walk.remaining)
            ))
        }
        Activity::Gathering { item, gathered, .. } => Some(gathering(item, gathered)),
    }
}

fn section(ui: &mut egui::Ui, category: Category, rows: &[Row<'_>]) -> Section {
    let mut section = Section::default();
    let seen: u32 = rows.iter().map(|row| row.sighting.count).sum();
    let row_height = ui.spacing().interact_size.y + ROW_SPACING.y;

    ui.horizontal(|ui| {
        ui.label(RichText::new(category.title()).strong());
        ui.label(RichText::new(roughly(seen)).weak());
    });
    egui::ScrollArea::vertical()
        .id_salt(category.title())
        .max_height(ROWS_SHOWN * row_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            if rows.is_empty() {
                ui.label(RichText::new("None in sight").weak());
                return;
            }

            egui::Grid::new(category.title())
                .num_columns(4)
                .min_col_width(COLUMN_WIDTH)
                .min_row_height(ui.spacing().interact_size.y)
                .spacing(ROW_SPACING)
                .show(ui, |ui| {
                    for row in rows {
                        let sighting = row.sighting;
                        let label = RichText::new(label(sighting));
                        let mut shown = ui.label(if row.highlighted {
                            label.strong()
                        } else {
                            label
                        });
                        shown |= ui.label(RichText::new(compass_point(sighting.bearing)).weak());
                        if row.within_reach {
                            shown |= ui.label(RichText::new("within reach").weak());
                        } else {
                            shown |= ui.label(RichText::new(distance_to(sighting)).weak());
                            let walk = ui.button("Walk to");
                            if walk.clicked() {
                                section.chosen = Some(sighting.subject);
                            }
                            shown |= walk;
                        }
                        if ui.rect_contains_pointer(shown.rect) {
                            section.hovered = Some(sighting.subject);
                        }
                        ui.end_row();
                    }
                });
        });
    ui.add_space(SECTION_SPACING);

    section
}

fn label(sighting: &Sighting) -> String {
    let name = kind_name(sighting.kind);
    if sighting.count > 1 {
        format!("{name} ×{}", roughly(sighting.count))
    } else {
        name
    }
}

fn distance_to(sighting: &Sighting) -> String {
    let distance = distance(sighting.distance);
    if sighting.count > 1 {
        format!("from {distance}")
    } else {
        distance
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::world::{
        Area, AreaKind, GroundType, Patch, Species, Terrain, Vegetation, VegetationKind,
    };
    use lords_sim::{Bearing, Item, Length, Location, Name, Sex, Simulation, State, World};

    use super::*;
    use crate::scenes::harness::Harness;

    fn sighting(kind: Kind, count: u32, millimeters: i64) -> Sighting {
        let anything = World::default().add_area(Area::new(AreaKind::Lake, Vec::new()));

        Sighting {
            subject: Subject::Area(anything),
            kind,
            count,
            distance: Length::from_millimeters(millimeters),
            bearing: Bearing::North,
        }
    }

    #[test]
    fn sighting_of_several_says_how_many_and_how_near() {
        let one = sighting(Kind::Rock, 1, 4_000);
        let few = sighting(Kind::Plant(Species::Pine), 12, 15_000);
        let many = sighting(Kind::Plant(Species::Spruce), 41, 23_000);

        assert_eq!(label(&one), "Boulder");
        assert_eq!(distance_to(&one), "4 m");
        assert_eq!(label(&few), "Pine ×12");
        assert_eq!(label(&many), "Spruce ×~40");
        assert_eq!(distance_to(&many), "from 25 m");
    }

    #[test]
    fn list_draws_for_an_empty_world_a_forest_and_a_walk() {
        let harness = Harness::new();
        let empty = Simulation::new(Name::default(), Sex::Female).view();
        let mut forest = Simulation::generate(Name::default(), Sex::Female, 1);
        let target = forest
            .view()
            .surroundings
            .in_sight
            .first()
            .expect("there is something in sight")
            .subject;

        harness.show(|ui| assert_eq!(show(ui, &empty, None), Pointed::default()));
        harness.show(|ui| {
            assert_eq!(show(ui, &forest.view(), Some(target)), Pointed::default());
        });

        forest.apply(Command::WalkTo(target));
        forest.tick();
        let walking = forest.view();
        assert!(walking.walk.is_some());
        harness.show(|ui| assert_eq!(show(ui, &walking, None), Pointed::default()));
    }

    #[test]
    fn doing_says_where_the_walk_goes_and_what_gathering_has_turned_up() {
        let meadow = Vegetation::typical(VegetationKind::ShoreMeadow);
        let mud = Patch::new(GroundType::Mud, Some(meadow), 4_000);
        let mut world = World::default();
        world.set_terrain(Terrain::new(Location::new(-2_000, -2_000), 1, [mud]));
        let lake = world.add_area(Area::new(
            AreaKind::Lake,
            [(-5_000, 20_000), (5_000, 20_000), (0, 30_000)]
                .into_iter()
                .map(|(x, y)| Location::new(x, y))
                .collect(),
        ));
        let mut sim = Simulation::from_state(State {
            world,
            ..State::new(Name::default(), Sex::Female)
        });
        assert_eq!(doing(&sim.view()), None);

        sim.apply(Command::Gather(Item::Pebble));
        assert_eq!(doing(&sim.view()).as_deref(), Some("Searching for pebbles"));
        while sim.state().player.inventory().is_empty() {
            sim.tick();
        }
        assert_eq!(
            doing(&sim.view()).as_deref(),
            Some("Searching for pebbles, 1 found")
        );

        sim.apply(Command::Gather(Item::Picked(Species::BottleSedge)));
        while sim
            .state()
            .player
            .inventory()
            .count(Item::Picked(Species::BottleSedge))
            < 2
        {
            sim.tick();
        }
        assert_eq!(
            doing(&sim.view()).as_deref(),
            Some("Picking tufts of bottle sedge, 2 picked")
        );

        sim.apply(Command::WalkTo(Subject::Area(lake)));
        assert_eq!(
            doing(&sim.view()).as_deref(),
            Some("Walking to the lake, 20 m to go")
        );
    }

    #[test]
    fn row_of_something_far_away_is_that_of_the_nearest_of_its_kind() {
        let view = Simulation::generate(Name::default(), Sex::Female, 1).view();
        let group = view
            .surroundings
            .in_sight
            .iter()
            .find(|sighting| sighting.count > 1)
            .expect("there are several of some kind in sight");
        let another = view
            .map
            .things
            .iter()
            .filter(|mark| mark.kind == group.kind)
            .find(|mark| view.surroundings.sighting(mark.subject).is_none())
            .expect("the group has another in it");
        let nearby = view.surroundings.sightings().next().expect("the first");

        assert_eq!(row_of(&view, group.subject), Some(group.subject));
        assert_eq!(row_of(&view, another.subject), Some(group.subject));
        assert_eq!(row_of(&view, nearby.subject), Some(nearby.subject));
    }
}
