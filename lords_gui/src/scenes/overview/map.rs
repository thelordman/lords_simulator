mod ground;

use std::iter;
use std::ops::RangeInclusive;

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke};
use lords_sim::map::{Map, Mark, RANGE, Water};
use lords_sim::object::Living;
use lords_sim::time::gregorian::{SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use lords_sim::world::{Kind, Species};
use lords_sim::{Command, Length, Location, Subject, Timestamp, View};

use crate::scenes::count;

use super::Pointed;
use super::words::{distance, kind_name};

/// How many meters there are from the player to the edges of the map at first.
pub const DEFAULT_REACH: f32 = 50.0;
const REACH_LIMITS: RangeInclusive<f32> = 10.0..=RANGE.as_millimeters() as f32 / 1_000.0;
const ZOOM_PER_SCROLLED_POINT: f32 = 0.004;

const PANEL_WIDTH: f32 = 320.0;
const PANEL_MIN_WIDTH: f32 = 160.0;

/// The width left to the rest of the overview, without which the map is not shown.
const ROOM_FOR_THE_LIST: f32 = 380.0;

/// How many times wider than its trunk a tree is drawn.
const CANOPY_SPREAD: f32 = 8.0;
const SMALLEST_DOT: f32 = 2.0;

const FRESHEST: f32 = 0.6;
const FAINTEST: f32 = 0.15;

const FADING_HALF_LIFE: f32 = SECONDS_PER_HOUR as f32;

/// How far outside a dot or a shore the pointer still points at it.
const POINTING_SLACK: f32 = 4.0;

const PLAYER_RADIUS: f32 = 4.0;
const HIGHLIGHT_GAP: f32 = 3.0;
const CORNER_RADIUS: f32 = 4.0;
const MARGIN: f32 = 10.0;
const LABEL_SIZE: f32 = 12.0;

/// The lengths a scale bar may have, in meters.
const SCALE_BARS: [f32; 6] = [2.0, 5.0, 10.0, 20.0, 50.0, 100.0];

pub fn show(
    ui: &mut egui::Ui,
    view: &View,
    reach: &mut f32,
    highlighted: Option<Subject>,
) -> Pointed {
    let room = ui.available_width() - ROOM_FOR_THE_LIST;
    if room < PANEL_MIN_WIDTH {
        return Pointed::default();
    }

    egui::Panel::right("map_panel")
        .default_size(PANEL_WIDTH.min(room))
        .size_range(PANEL_MIN_WIDTH..=room)
        .show(ui, |ui| chart(ui, view, reach, highlighted))
        .inner
}

fn chart(ui: &mut egui::Ui, view: &View, reach: &mut f32, highlighted: Option<Subject>) -> Pointed {
    let side = ui.available_width().min(ui.available_height());
    ui.take_available_width();
    let (response, painter) = ui.allocate_painter(egui::Vec2::splat(side), Sense::click());
    let rect = response.rect;
    if response.hovered() {
        zoom(ui, reach);
    }

    let palette = Palette::of(ui.visuals());
    let map = &*view.map;
    let projection = Projection {
        center: rect.center(),
        origin: view.player.location(),
        points_per_meter: side / 2.0 / *reach,
    };
    let dots: Vec<Dot<'_>> = map
        .things
        .iter()
        .map(|mark| Dot::of(mark, projection))
        .filter(|dot| rect.expand(dot.radius).contains(dot.center))
        .collect();

    let pointed_at = response
        .hover_pos()
        .and_then(|pointer| pointed_at(pointer, &dots, &map.waters, projection));
    let lit = |subject| pointed_at == Some(subject) || highlighted == Some(subject);

    painter.rect_filled(rect, CORNER_RADIUS, palette.ground);
    painter.add(ground::draw(
        &map.ground,
        projection,
        rect,
        ui.visuals().dark_mode,
    ));
    for water in &map.waters {
        let width = if lit(water.subject) { 3.0 } else { 1.5 };
        painter.add(surface(water, projection, palette.water));
        for ends in &water.shore {
            let ends = ends.map(|end| projection.position(end));
            painter.line_segment(ends, Stroke::new(width, palette.shore));
        }
    }
    for dot in &dots {
        let color = palette.of_kind(dot.mark.kind);
        let (fill, edge) = match dot.mark.last_seen {
            None => (color, palette.ground),
            Some(last_seen) => {
                let age = view.time.seconds_since(last_seen);
                (color.gamma_multiply(opacity(age)), Color32::TRANSPARENT)
            }
        };

        painter.circle_filled(dot.center, dot.radius, fill);
        if dot.mark.kind == Kind::Rock {
            painter.circle_stroke(dot.center, dot.radius, Stroke::new(1.0, edge));
        }
        if lit(dot.mark.subject) {
            let ring = Stroke::new(1.5, palette.strong);
            painter.circle_stroke(dot.center, dot.radius + HIGHLIGHT_GAP, ring);
        }
    }
    if let Some(walk) = &view.walk {
        let turns = walk.via.iter().chain([&walk.destination]);
        let route = iter::once(rect.center())
            .chain(turns.map(|turn| projection.position(*turn)))
            .collect();
        painter.add(Shape::line(route, Stroke::new(1.0, palette.strong)));
    }
    painter.circle_filled(rect.center(), PLAYER_RADIUS, palette.strong);
    painter.circle_stroke(
        rect.center(),
        PLAYER_RADIUS,
        Stroke::new(1.5, palette.ground),
    );
    legend(&painter, rect, projection, &palette);

    let clicked = response.clicked();
    let description = match pointed_at {
        Some(subject) => {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            describe(subject, map, projection, view.time)
        }
        None => response
            .hover_pos()
            .and_then(|pointer| ground::under(pointer, &map.ground, projection))
            .map(ground::describe),
    };
    if let Some(description) = description {
        response.on_hover_text_at_pointer(description);
    }

    Pointed {
        command: pointed_at.filter(|_| clicked).map(Command::WalkTo),
        hovered: pointed_at,
    }
}

fn zoom(ui: &egui::Ui, reach: &mut f32) {
    let (scrolled, pinched) = ui.input(|input| (input.smooth_scroll_delta.y, input.zoom_delta()));
    let closer = pinched * (scrolled * ZOOM_PER_SCROLLED_POINT).exp();

    *reach = (*reach / closer).clamp(*REACH_LIMITS.start(), *REACH_LIMITS.end());
}

/// Draws which way north is and how long a round distance is on the map.
fn legend(painter: &egui::Painter, rect: Rect, projection: Projection, palette: &Palette) {
    let font = FontId::proportional(LABEL_SIZE);
    let north = rect.center_top() + egui::vec2(0.0, MARGIN / 2.0);
    painter.text(north, Align2::CENTER_TOP, "N", font.clone(), palette.weak);

    let longest = rect.width() / 3.0 / projection.points_per_meter;
    let meters = SCALE_BARS
        .into_iter()
        .rev()
        .find(|meters| *meters <= longest)
        .unwrap_or(SCALE_BARS[0]);
    let start = rect.left_bottom() + egui::vec2(MARGIN, -MARGIN);
    let end = start + egui::vec2(meters * projection.points_per_meter, 0.0);
    painter.line_segment([start, end], Stroke::new(1.5, palette.weak));
    painter.text(
        start + egui::vec2(0.0, -MARGIN / 2.0),
        Align2::LEFT_BOTTOM,
        format!("{meters} m"),
        font,
        palette.weak,
    );
}

fn surface(water: &Water, projection: Projection, color: Color32) -> Shape {
    let mut mesh = egui::Mesh::default();
    for triangle in &water.surface {
        let first = mesh.vertices.len() as u32;
        for corner in triangle {
            mesh.colored_vertex(projection.position(*corner), color);
        }
        mesh.add_triangle(first, first + 1, first + 2);
    }

    Shape::mesh(mesh)
}

/// Returns what the map shows nearest to under `pointer`, things before water.
fn pointed_at(
    pointer: Pos2,
    dots: &[Dot<'_>],
    waters: &[Water],
    projection: Projection,
) -> Option<Subject> {
    let dot = dots
        .iter()
        .filter(|dot| dot.center.distance(pointer) <= dot.radius + POINTING_SLACK)
        .min_by(|a, b| {
            let (a, b) = (a.center.distance(pointer), b.center.distance(pointer));
            a.total_cmp(&b)
        });
    let water = || {
        waters.iter().find(|water| {
            let on_the_surface = water
                .surface
                .iter()
                .any(|triangle| is_inside(pointer, triangle.map(|at| projection.position(at))));
            on_the_surface || distance_to_shore(pointer, water, projection) <= POINTING_SLACK
        })
    };

    dot.map(|dot| dot.mark.subject)
        .or_else(|| water().map(|water| water.subject))
}

fn opacity(age: u64) -> f32 {
    let left_to_fade = 0.5_f32.powf(age as f32 / FADING_HALF_LIFE);

    FAINTEST + (FRESHEST - FAINTEST) * left_to_fade
}

/// Returns the words for how long ago `age` in-game seconds were.
fn ago(age: u64) -> String {
    let minutes = age / SECONDS_PER_MINUTE;
    let hours = age / SECONDS_PER_HOUR;
    let days = age / SECONDS_PER_DAY;

    if minutes == 0 {
        "moments ago".to_owned()
    } else if hours == 0 {
        format!("{} ago", count(minutes, "minute"))
    } else if days == 0 {
        format!("{} ago", count(hours, "hour"))
    } else {
        format!("{} ago", count(days, "day"))
    }
}

/// Returns the words for what `subject` is, how far from the player, and when a thing no
/// longer in sight was last seen.
fn describe(subject: Subject, map: &Map, projection: Projection, now: Timestamp) -> Option<String> {
    let mark = map.things.iter().find(|mark| mark.subject == subject);
    let water = map.waters.iter().find(|water| water.subject == subject);

    if let Some(mark) = mark {
        let away = projection.origin.distance_to(mark.location) - mark.footprint;
        let seen = mark
            .last_seen
            .map(|last_seen| format!(", seen {}", ago(now.seconds_since(last_seen))))
            .unwrap_or_default();

        Some(format!(
            "{}, {}{seen}",
            kind_name(mark.kind),
            distance(away.max(Length::ZERO))
        ))
    } else {
        let water = water?;
        let points = distance_to_shore(projection.center, water, projection);
        let away =
            Length::from_millimeters((points / projection.points_per_meter * 1_000.0) as i64);

        Some(format!("{}, {}", kind_name(water.kind), distance(away)))
    }
}

/// Returns how many points there are on the map from `point` to the known shore of `water`.
fn distance_to_shore(point: Pos2, water: &Water, projection: Projection) -> f32 {
    water
        .shore
        .iter()
        .map(|ends| distance_to_segment(point, ends.map(|end| projection.position(end))))
        .fold(f32::INFINITY, f32::min)
}

fn distance_to_segment(point: Pos2, [start, end]: [Pos2; 2]) -> f32 {
    let along = end - start;
    let length_squared = along.length_sq();
    if length_squared == 0.0 {
        return start.distance(point);
    }

    let share = ((point - start).dot(along) / length_squared).clamp(0.0, 1.0);
    (start + along * share).distance(point)
}

fn is_inside(point: Pos2, [a, b, c]: [Pos2; 3]) -> bool {
    let turn = |from: Pos2, to: Pos2| {
        (to.x - from.x) * (point.y - from.y) - (to.y - from.y) * (point.x - from.x)
    };
    let turns = [turn(a, b), turn(b, c), turn(c, a)];

    turns.iter().all(|turn| *turn >= 0.0) || turns.iter().all(|turn| *turn <= 0.0)
}

#[derive(Copy, Clone, Debug)]
struct Projection {
    center: Pos2,
    origin: Location,
    points_per_meter: f32,
}

impl Projection {
    fn position(self, location: Location) -> Pos2 {
        let east = (location.x() - self.origin.x()) as f32 / 1_000.0;
        let north = (location.y() - self.origin.y()) as f32 / 1_000.0;

        self.center + egui::vec2(east, -north) * self.points_per_meter
    }

    fn length(self, length: Length) -> f32 {
        length.as_millimeters() as f32 / 1_000.0 * self.points_per_meter
    }
}

#[derive(Copy, Clone, Debug)]
struct Dot<'a> {
    mark: &'a Mark,
    center: Pos2,
    radius: f32,
}

impl<'a> Dot<'a> {
    fn of(mark: &'a Mark, projection: Projection) -> Self {
        let spread = match mark.kind {
            Kind::Plant(_) => CANOPY_SPREAD,
            Kind::Rock | Kind::Lake => 1.0,
        };

        Self {
            mark,
            center: projection.position(mark.location),
            radius: (projection.length(mark.footprint) * spread).max(SMALLEST_DOT),
        }
    }
}

struct Palette {
    ground: Color32,
    water: Color32,
    shore: Color32,
    spruce: Color32,
    pine: Color32,
    birch: Color32,
    /// For any other plant.
    plant: Color32,
    rock: Color32,
    strong: Color32,
    weak: Color32,
}

impl Palette {
    fn of(visuals: &egui::Visuals) -> Self {
        let strong = visuals.strong_text_color();
        let weak = visuals.weak_text_color();

        if visuals.dark_mode {
            Self {
                ground: Color32::from_rgb(33, 40, 33),
                water: Color32::from_rgb(40, 84, 128),
                shore: Color32::from_rgb(112, 164, 212),
                spruce: Color32::from_rgb(44, 128, 82),
                pine: Color32::from_rgb(122, 158, 62),
                birch: Color32::from_rgb(214, 204, 110),
                plant: Color32::from_rgb(96, 150, 96),
                rock: Color32::from_rgb(146, 146, 154),
                strong,
                weak,
            }
        } else {
            Self {
                ground: Color32::from_rgb(228, 232, 216),
                water: Color32::from_rgb(150, 194, 232),
                shore: Color32::from_rgb(54, 106, 158),
                spruce: Color32::from_rgb(24, 98, 58),
                pine: Color32::from_rgb(104, 146, 44),
                birch: Color32::from_rgb(178, 160, 30),
                plant: Color32::from_rgb(70, 124, 70),
                rock: Color32::from_rgb(118, 118, 126),
                strong,
                weak,
            }
        }
    }

    fn of_kind(&self, kind: Kind) -> Color32 {
        match kind {
            Kind::Plant(Species::Spruce) => self.spruce,
            Kind::Plant(Species::Pine) => self.pine,
            Kind::Plant(Species::Birch) => self.birch,
            Kind::Plant(_) => self.plant,
            Kind::Rock => self.rock,
            Kind::Lake => self.water,
        }
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::object::Object;
    use lords_sim::world::{Rock, Thing};
    use lords_sim::{Name, Sex, Simulation, World};

    use super::*;
    use crate::scenes::harness::Harness;

    const PROJECTION: Projection = Projection {
        center: Pos2::new(200.0, 150.0),
        origin: Location::new(5_000, -3_000),
        points_per_meter: 2.0,
    };

    fn subjects() -> [Subject; 2] {
        let mut world = World::default();
        let pebble = || Thing::Rock(Rock::new(Object::default(), Length::ZERO, Length::ZERO));

        [(); 2].map(|()| Subject::Thing(world.add_thing(pebble())))
    }

    fn mark(subject: Subject, kind: Kind, x: i64, y: i64, footprint: i64) -> Mark {
        Mark {
            subject,
            kind,
            location: Location::new(x, y),
            footprint: Length::from_millimeters(footprint),
            last_seen: None,
        }
    }

    #[test]
    fn player_is_in_the_middle_with_north_up_and_east_right() {
        let at = |x: i64, y: i64| PROJECTION.position(Location::new(5_000 + x, -3_000 + y));

        assert_eq!(at(0, 0), Pos2::new(200.0, 150.0));
        assert_eq!(at(0, 10_000), Pos2::new(200.0, 130.0));
        assert_eq!(at(10_000, 0), Pos2::new(220.0, 150.0));
        assert_eq!(at(-2_500, -5_000), Pos2::new(195.0, 160.0));
        assert_eq!(PROJECTION.length(Length::from_millimeters(1_500)), 3.0);
    }

    #[test]
    fn trees_are_drawn_wider_than_their_trunks_and_nothing_too_small_to_see() {
        let [subject, _] = subjects();
        let radius =
            |kind, footprint| Dot::of(&mark(subject, kind, 0, 0, footprint), PROJECTION).radius;

        assert_eq!(radius(Kind::Plant(Species::Pine), 200), 3.2);
        assert_eq!(radius(Kind::Rock, 1_250), 2.5);
        assert_eq!(radius(Kind::Rock, 150), SMALLEST_DOT);
        assert_eq!(radius(Kind::Plant(Species::Pine), 40), SMALLEST_DOT);
    }

    #[test]
    fn pointer_picks_the_nearest_thing_within_its_slack() {
        let [near, far] = subjects();
        let marks = [
            mark(far, Kind::Rock, 8_000, 0, 150),
            mark(near, Kind::Rock, 5_000, 2_000, 150),
        ];
        let dots = marks.each_ref().map(|mark| Dot::of(mark, PROJECTION));
        let picked = |x, y| pointed_at(Pos2::new(x, y), &dots, &[], PROJECTION);

        assert_eq!(picked(200.0, 141.0), Some(near));
        assert_eq!(picked(202.0, 142.0), Some(near));
        assert_eq!(picked(205.0, 143.0), Some(far));
        assert_eq!(picked(200.0, 160.0), None);
    }

    #[test]
    fn pointer_is_on_water_inside_a_triangle_or_by_a_shore() {
        let triangle = [
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(0.0, 10.0),
        ];
        let reversed = [triangle[2], triangle[1], triangle[0]];

        assert!(is_inside(Pos2::new(2.0, 2.0), triangle));
        assert!(is_inside(Pos2::new(2.0, 2.0), reversed));
        assert!(!is_inside(Pos2::new(8.0, 8.0), triangle));
        assert!(!is_inside(Pos2::new(-1.0, 2.0), reversed));

        let shore = [Pos2::new(0.0, 0.0), Pos2::new(10.0, 0.0)];
        assert_eq!(distance_to_segment(Pos2::new(4.0, 3.0), shore), 3.0);
        assert_eq!(distance_to_segment(Pos2::new(14.0, 3.0), shore), 5.0);
        assert_eq!(distance_to_segment(Pos2::new(-3.0, -4.0), shore), 5.0);
    }

    #[test]
    fn map_draws_for_an_empty_world_a_forest_and_a_walk() {
        let harness = Harness::new();
        let mut reach = DEFAULT_REACH;
        let empty = Simulation::new(Name::default(), Sex::Female).view();
        let mut forest = Simulation::generate(Name::default(), Sex::Female, 1);
        let target = forest.view().map.things[0].subject;

        harness.show(|ui| assert_eq!(show(ui, &empty, &mut reach, None), Pointed::default()));
        for mut reach in [*REACH_LIMITS.start(), DEFAULT_REACH, *REACH_LIMITS.end()] {
            let view = forest.view();
            harness.show(|ui| {
                assert_eq!(
                    show(ui, &view, &mut reach, Some(target)),
                    Pointed::default()
                );
            });
        }

        forest.apply(Command::WalkTo(target));
        forest.tick();
        let walking = forest.view();
        assert!(walking.walk.is_some());
        harness.show(|ui| assert_eq!(show(ui, &walking, &mut reach, None), Pointed::default()));
        assert_eq!(reach, DEFAULT_REACH);
    }

    #[test]
    fn thing_is_described_by_what_it_is_and_how_far() {
        let view = Simulation::generate(Name::default(), Sex::Female, 1).view();
        let nearest = view
            .surroundings
            .sightings()
            .min_by_key(|sighting| sighting.distance)
            .copied()
            .expect("there is something in sight");
        let projection = Projection {
            origin: view.player.location(),
            ..PROJECTION
        };

        let described = format!(
            "{}, {}",
            kind_name(nearest.kind),
            distance(nearest.distance)
        );
        let later = Timestamp::from_seconds(view.time.as_seconds() + 2 * 3_600);
        let mut remembered = (*view.map).clone();
        for mark in &mut remembered.things {
            mark.last_seen = Some(view.time);
        }

        assert_eq!(
            describe(nearest.subject, &view.map, projection, later),
            Some(described.clone())
        );
        assert_eq!(
            describe(nearest.subject, &remembered, projection, later),
            Some(format!("{described}, seen 2 hours ago"))
        );
        assert_eq!(
            describe(nearest.subject, &Map::default(), projection, later),
            None
        );
    }

    #[test]
    fn remembered_thing_fades_fast_at_first_and_never_all_the_way() {
        assert_eq!(opacity(0), FRESHEST);
        assert_eq!(opacity(3_600), (FRESHEST + FAINTEST) / 2.0);
        assert!(opacity(60) > opacity(600));
        assert!(opacity(3_600) - opacity(7_200) < opacity(0) - opacity(3_600));
        assert!(opacity(86_400 * 365) >= FAINTEST);
        assert!(opacity(86_400 * 365) < FAINTEST + 0.001);
    }

    #[test]
    fn time_since_a_thing_was_seen_is_told_in_its_largest_whole_unit() {
        assert_eq!(ago(0), "moments ago");
        assert_eq!(ago(59), "moments ago");
        assert_eq!(ago(60), "1 minute ago");
        assert_eq!(ago(3_599), "59 minutes ago");
        assert_eq!(ago(3_600), "1 hour ago");
        assert_eq!(ago(86_399), "23 hours ago");
        assert_eq!(ago(86_400), "1 day ago");
        assert_eq!(ago(86_400 * 40), "40 days ago");
    }
}
