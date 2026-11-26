use std::sync::Arc;

use crate::item::Item;
use crate::map::{self, Map, Surface};
use crate::object::{Activity, Human, Living, Name, Sex};
use crate::perception::{self, Surroundings};
use crate::space::{Length, Location};
use crate::state::State;
use crate::time::{TICKS_PER_SECOND, Timestamp};
use crate::view::{Ground, Growth, View, Walk};
use crate::world::index::Grid;
use crate::world::{Area, AreaKind, Subject, ThingId, Vegetation, World};
use crate::{gather, route};

pub struct Simulation {
    state: State,
    grid: Grid,
    /// The triangles covering each area of the world, in the order of the areas.
    surfaces: Vec<Surface>,
    surroundings: Arc<Surroundings>,
    map: Arc<Map>,
    in_sight: Vec<ThingId>,
    route: Vec<Location>,
}

impl Simulation {
    /// Returns a simulation of a player in an empty world.
    pub fn new(name: Name, sex: Sex) -> Self {
        Self::from_state(State::new(name, sex))
    }

    /// Returns a simulation of a player in the world that [`World::generate`] makes of `seed`.
    pub fn generate(name: Name, sex: Sex, seed: u64) -> Self {
        Self::from_state(State {
            world: World::generate(seed),
            ..State::new(name, sex)
        })
    }

    pub fn from_state(state: State) -> Self {
        let grid = Grid::new(&state.world);
        let surfaces = state.world.areas().map(|_| Surface::default()).collect();

        let mut simulation = Self {
            state,
            grid,
            surfaces,
            surroundings: Arc::default(),
            map: Arc::default(),
            in_sight: Vec::new(),
            route: Vec::new(),
        };
        simulation.look();
        simulation
    }

    /// Returns the state of the simulation.
    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn view(&self) -> View {
        View {
            time: self.state.time,
            player: self.state.player.clone(),
            surroundings: Arc::clone(&self.surroundings),
            map: Arc::clone(&self.map),
            walk: self.walking(),
            ground: self.ground(),
        }
    }

    /// Advances the simulation by one tick.
    pub fn tick(&mut self) {
        self.state.time.advance();
        self.walk();

        let State {
            player,
            world,
            time,
            ..
        } = &mut self.state;
        let season = world.season(*time);
        gather::tick(player, world.terrain_mut(), *time, season);
    }

    /// Applies a change requested from outside the simulation.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Reset => self.state.time = Timestamp::EPOCH,
            Command::WalkTo(subject) => self.walk_to(subject),
            Command::Gather(item) => self.gather(item),
            Command::Drop { item, count } => self.drop_items(item, count),
            Command::Stop => self.set_activity(None),
        }
    }

    fn walking(&self) -> Option<Walk> {
        let player = &self.state.player;
        let Some(Activity::Walking {
            toward,
            destination,
        }) = player.activity()
        else {
            return None;
        };

        let mut remaining = Length::ZERO;
        let mut at = player.location();
        for stop in self.route.iter().copied().chain([destination]) {
            remaining = remaining + at.distance_to(stop);
            at = stop;
        }

        Some(Walk {
            toward: self.state.world.kind(toward)?,
            destination,
            via: self.route.clone(),
            remaining,
        })
    }

    fn ground(&self) -> Option<Ground> {
        let season = self.state.world.season(self.state.time);
        let terrain = self.state.world.terrain();
        let patch = terrain.patch_at(self.state.player.location())?;
        let vegetation = patch.vegetation();
        let shares = vegetation.map_or(&[][..], Vegetation::shares);

        Some(Ground {
            ground_type: patch.ground_type(),
            vegetation: vegetation.map(Vegetation::kind),
            plants: shares
                .iter()
                .map(|(species, _)| Growth {
                    species: *species,
                    share: patch.share(*species),
                    crop: gather::abundance(patch, *species, season),
                })
                .collect(),
        })
    }

    fn set_activity(&mut self, activity: Option<Activity>) {
        self.state.player.animal_mut().set_activity(activity);
        self.plan_route();
    }

    fn walk_to(&mut self, subject: Subject) {
        if !self.state.memory.knows(subject) {
            return;
        }

        let from = self.state.player.location();
        let destination = self.state.world.approach(subject, from, Human::REACH);
        let activity = destination
            .filter(|destination| *destination != from)
            .map(|destination| Activity::Walking {
                toward: subject,
                destination,
            });

        self.set_activity(activity);
    }

    fn walk(&mut self) {
        let Some(Activity::Walking { destination, .. }) = self.state.player.activity() else {
            return;
        };
        let start = self.state.player.location();
        let mut at = start;
        let mut left = Human::WALKING_SPEED / TICKS_PER_SECOND as i64;
        let mut learned = false;

        while left > Length::ZERO {
            let target = self.route.first().copied().unwrap_or(destination);
            let next = at.toward(target, left);

            if let Some((id, lake)) = self.state.world.lake_at(next) {
                let memory = &mut self.state.memory;
                let found = lake
                    .nearest_on_shore(next)
                    .map(|(edge, _)| edge)
                    .filter(|edge| !memory.knows_edge(id, *edge));
                match found {
                    Some(edge) => {
                        memory.note_edge(id, edge);
                        learned = true;
                    }
                    None => self.state.player.animal_mut().set_activity(None),
                }
                break;
            }

            left = left - at.distance_to(next);
            at = next;
            if next != target {
                break;
            }
            if self.route.is_empty() {
                self.state.player.animal_mut().set_activity(None);
                break;
            }
            self.route.remove(0);
        }

        if at != start || learned {
            self.state.player.animal_mut().move_to(at);
            self.look();
        }
    }

    fn gather(&mut self, item: Item) {
        let State {
            player,
            world,
            time,
            ..
        } = &self.state;

        let season = world.season(*time);
        if let Some(gathering) = gather::begin(player, world.terrain(), item, *time, season) {
            self.set_activity(Some(gathering));
        }
    }

    fn drop_items(&mut self, item: Item, count: u32) {
        let State { player, world, .. } = &mut self.state;
        let dropped = player.inventory_mut().remove(item, count);
        if dropped == 0 {
            return;
        }

        if let Some(patch) = world.terrain_mut().patch_at_mut(player.location()) {
            patch.leave(item, dropped);
        }
    }

    fn plan_route(&mut self) {
        let Some(Activity::Walking { destination, .. }) = self.state.player.activity() else {
            self.route.clear();
            return;
        };

        self.route = route::plan(self.state.player.location(), destination, &self.lakes());
    }

    fn lakes(&self) -> Vec<Area> {
        self.state
            .world
            .areas()
            .zip(&self.surfaces)
            .filter(|((_, area), _)| area.kind() == AreaKind::Lake)
            .map(|((_, area), surface)| surface.shape(area))
            .filter(|shape| shape.outline().len() >= 2)
            .collect()
    }

    /// Takes in what the player sees from where they are, and commits it to memory.
    fn look(&mut self) {
        let State {
            player,
            world,
            memory,
            time,
        } = &mut self.state;
        let eye = player.location();
        let seen = perception::look(world, &self.grid, eye);

        for thing in self.in_sight.iter().chain(&seen.things) {
            memory.note_thing(*thing, *time);
        }
        for (area, edge) in &seen.shores {
            memory.note_edge(*area, *edge);
        }
        for patch in &seen.ground {
            memory.note_patch(*patch);
        }
        let mut shore_learned = false;
        for ((id, area), surface) in world.areas().zip(&mut self.surfaces) {
            shore_learned |= surface.update(id, area, memory);
        }

        self.map = Arc::new(map::chart(
            world,
            &self.grid,
            &self.surfaces,
            memory,
            &seen,
            eye,
        ));
        self.surroundings = Arc::new(seen.surroundings);
        self.in_sight = seen.things;
        if shore_learned {
            self.plan_route();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Reset,
    WalkTo(Subject),
    Gather(Item),
    Drop { item: Item, count: u32 },
    Stop,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{Mark, Tile};
    use crate::object::Object;
    use crate::world::{
        Abundance, GroundType, Kind, Patch, Plant, Rock, Season, Species, Terrain, Thing,
        VegetationKind,
    };

    fn birch(x: i64, y: i64) -> Thing {
        Thing::Plant(Plant::new(
            Object::new(Location::new(x, y), Default::default()),
            Species::Birch,
            Length::from_meters(15),
            Length::from_millimeters(300),
        ))
    }

    fn lake(south: i64, north: i64) -> Area {
        let corners = [
            (-20_000, south),
            (20_000, south),
            (20_000, north),
            (-20_000, north),
        ];
        Area::new(
            AreaKind::Lake,
            corners
                .into_iter()
                .map(|(x, y)| Location::new(x, y))
                .collect(),
        )
    }

    fn tall_boulder(x: i64, y: i64) -> Thing {
        Thing::Rock(Rock::new(
            Object::new(Location::new(x, y), Default::default()),
            Length::from_millimeters(2_500),
            Length::from_meters(2),
        ))
    }

    /// Returns a world whose only ground is `patch`, around the center.
    fn ground(patch: Patch) -> World {
        let mut world = World::default();
        let corner = Location::new(-2_000, -2_000);
        world.set_terrain(Terrain::new(corner, 1, [patch]));
        world
    }

    fn gravel() -> Patch {
        Patch::new(GroundType::Gravel, None, 4_000)
    }

    fn pebbles(sim: &Simulation) -> u32 {
        sim.state().player.inventory().count(Item::Pebble)
    }

    fn mark(sim: &Simulation, subject: Subject) -> Option<Mark> {
        let map = sim.view().map;
        map.things
            .iter()
            .copied()
            .find(|mark| mark.subject == subject)
    }

    /// A simulation of a player who has walked south out of sight of a tree to the north,
    /// along with that tree, one that was behind it all the way, and the one walked to.
    fn walk_away() -> (Simulation, [Subject; 3]) {
        let mut world = World::default();
        let left_behind = Subject::Thing(world.add_thing(birch(0, 20_000)));
        let hidden = Subject::Thing(world.add_thing(birch(0, 40_000)));
        let walked_to = Subject::Thing(world.add_thing(birch(0, -85_000)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(walked_to));
        walk_until_done(&mut sim);

        (sim, [left_behind, hidden, walked_to])
    }

    fn simulation(world: World) -> Simulation {
        Simulation::from_state(State {
            world,
            ..State::new(Name::default(), Sex::Female)
        })
    }

    /// Ticks until the player stops walking, and returns where they were after each tick.
    fn walk_until_done(sim: &mut Simulation) -> Vec<Location> {
        let mut path = Vec::new();
        while sim.state().player.activity().is_some() {
            assert!(path.len() < 1_000, "the walk never ended");
            sim.tick();

            let at = sim.state().player.location();
            assert!(
                !sim.state().world.is_water(at),
                "walked into water at {at:?}"
            );
            path.push(at);
        }
        path
    }

    fn within_reach(sim: &Simulation, subject: Subject) -> bool {
        let view = sim.view();
        view.surroundings
            .within_reach
            .iter()
            .any(|sighting| sighting.subject == subject)
    }

    #[test]
    fn simulation_carries_on_from_a_state() {
        let mut original = Simulation::new(Name::default(), Sex::Male);
        original.tick();
        original.tick();

        let mut resumed = Simulation::from_state(original.state().clone());
        assert_eq!(resumed.state(), original.state());

        resumed.tick();
        assert_eq!(resumed.state().time, Timestamp::from_seconds(3));
    }

    #[test]
    fn view_shows_the_player_at_the_current_time() {
        let name = Name::new("Ada".to_owned(), Vec::new(), "Lord".to_owned());
        let mut sim = Simulation::new(name, Sex::Female);
        sim.tick();

        let view = sim.view();

        assert_eq!(view.time, Timestamp::from_seconds(1));
        assert_eq!(view.player, sim.state().player);
        assert!(view.surroundings.is_empty());
        assert_eq!(*view.map, Map::default());
        assert_eq!(view.walk, None);
        assert_eq!(view.ground, None);
    }

    #[test]
    fn generated_simulation_starts_with_things_in_sight() {
        let sim = Simulation::generate(Name::default(), Sex::Male, 5);

        assert!(!sim.view().surroundings.is_empty());
        assert_eq!(sim.state().player.location(), Location::default());
    }

    #[test]
    fn generated_simulation_starts_on_overgrown_ground() {
        let sim = Simulation::generate(Name::default(), Sex::Male, 5);

        let ground = sim.view().ground.expect("the start has ground");
        assert!(ground.vegetation.is_some());
    }

    #[test]
    fn view_shows_the_ground_underfoot() {
        let meadow = Vegetation::new(
            VegetationKind::ShoreMeadow,
            vec![(Species::BottleSedge, 60), (Species::CommonReed, 40)],
        );
        let mut sim = simulation(ground(Patch::new(GroundType::Mud, Some(meadow), 50)));
        let expected = |share, crop| Ground {
            ground_type: GroundType::Mud,
            vegetation: Some(VegetationKind::ShoreMeadow),
            plants: vec![
                Growth {
                    species: Species::BottleSedge,
                    share: 60,
                    crop: Abundance::Plenty,
                },
                Growth {
                    species: Species::CommonReed,
                    share,
                    crop,
                },
            ],
        };
        assert_eq!(sim.view().ground, Some(expected(40, Abundance::Plenty)));

        sim.apply(Command::Gather(Item::Picked(Species::CommonReed)));
        while sim.state().player.activity().is_some() {
            sim.tick();
        }

        assert_eq!(sim.view().ground, Some(expected(38, Abundance::Plenty)));
    }

    #[test]
    fn searching_turns_up_pebbles_that_the_player_takes_off_the_ground() {
        let mut sim = simulation(ground(gravel()));

        sim.apply(Command::Gather(Item::Pebble));
        for _ in 0..60 {
            sim.tick();
        }

        let found = pebbles(&sim);
        assert!((15..=45).contains(&found), "{found} found");
        assert!(matches!(
            sim.state().player.activity(),
            Some(Activity::Gathering { item: Item::Pebble, gathered, .. }) if gathered == found
        ));
        let left = sim.state().world.terrain().patch_at(Location::default());
        assert_eq!(
            left.map(|patch| patch.left(Item::Pebble, Season::usual(0))),
            Some(4_000 - found)
        );
    }

    #[test]
    fn search_that_turns_up_nothing_is_given_up() {
        let vegetation = Vegetation::typical(VegetationKind::PineBog);
        let mut sim = simulation(ground(Patch::new(GroundType::Peat, Some(vegetation), 0)));

        sim.apply(Command::Gather(Item::Pebble));
        for _ in 1..gather::PATIENCE {
            sim.tick();
        }
        assert!(matches!(
            sim.state().player.activity(),
            Some(Activity::Gathering { gathered: 0, .. })
        ));

        sim.tick();
        assert_eq!(sim.state().player.activity(), None);
        assert_eq!(pebbles(&sim), 0);
    }

    #[test]
    fn nobody_gathers_where_there_is_no_ground() {
        let mut sim = simulation(World::default());

        sim.apply(Command::Gather(Item::Pebble));

        assert_eq!(sim.state().player.activity(), None);
    }

    #[test]
    fn search_carries_on_from_a_state() {
        let mut sim = simulation(ground(Patch::new(GroundType::Sand, None, 300)));
        sim.apply(Command::Gather(Item::Pebble));
        for _ in 0..20 {
            sim.tick();
        }

        let mut resumed = Simulation::from_state(sim.state().clone());
        for _ in 0..100 {
            sim.tick();
            resumed.tick();
        }

        assert_eq!(resumed.state(), sim.state());
        assert!(pebbles(&sim) > 0);
    }

    #[test]
    fn picking_takes_what_grows_until_none_is_seen_to_be_left() {
        let heath = Vegetation::typical(VegetationKind::BilberryHeath);
        let mut sim = simulation(ground(Patch::new(GroundType::Till, Some(heath), 0)));
        let moss = Item::Picked(Species::FeatherMoss);

        sim.apply(Command::Gather(moss));
        let mut ticks = 0;
        while sim.state().player.activity().is_some() {
            assert!(ticks < 2_000, "the picking never ended");
            sim.tick();
            ticks += 1;
        }

        let inventory = sim.state().player.inventory();
        assert_eq!(inventory.count(moss), 30, "the hands are full first");
        let patch = sim.state().world.terrain().patch_at(Location::default());
        assert_eq!(
            patch.map(|patch| patch.left(moss, Season::usual(0))),
            Some(50)
        );
    }

    #[test]
    fn nothing_is_picked_where_it_does_not_grow() {
        let mut sim = simulation(ground(gravel()));

        sim.apply(Command::Gather(Item::Picked(Species::Bilberry)));

        assert_eq!(sim.state().player.activity(), None);
    }

    #[test]
    fn gathering_stops_once_the_hands_are_full_and_starts_again_once_emptied() {
        let mut sim = simulation(ground(gravel()));
        sim.apply(Command::Gather(Item::Pebble));
        for _ in 0..2_000 {
            sim.tick();
        }

        assert_eq!(sim.state().player.activity(), None);
        assert_eq!(pebbles(&sim), 150);
        assert!(!sim.state().player.has_room_for(Item::Pebble));
        sim.apply(Command::Gather(Item::Pebble));
        assert_eq!(sim.state().player.activity(), None);

        sim.apply(Command::Drop {
            item: Item::Pebble,
            count: 150,
        });
        sim.apply(Command::Gather(Item::Pebble));

        assert_eq!(pebbles(&sim), 0);
        assert!(sim.state().player.activity().is_some());
        let patch = sim.state().world.terrain().patch_at(Location::default());
        assert_eq!(
            patch.map(|patch| patch.left(Item::Pebble, Season::usual(0))),
            Some(4_000)
        );
    }

    #[test]
    fn some_of_what_is_carried_can_be_dropped_and_no_more_than_is_carried() {
        let mut sim = simulation(ground(gravel()));
        sim.apply(Command::Gather(Item::Pebble));
        while pebbles(&sim) < 5 {
            sim.tick();
        }
        sim.apply(Command::Stop);
        let drop = |count| Command::Drop {
            item: Item::Pebble,
            count,
        };
        let left = |sim: &Simulation| {
            let patch = sim.state().world.terrain().patch_at(Location::default());
            patch.map(|patch| patch.left(Item::Pebble, Season::usual(0)))
        };

        sim.apply(drop(2));
        assert_eq!(pebbles(&sim), 3);
        assert_eq!(left(&sim), Some(3_997));

        sim.apply(drop(0));
        assert_eq!(pebbles(&sim), 3);

        sim.apply(drop(10));
        assert_eq!(pebbles(&sim), 0);
        assert_eq!(left(&sim), Some(4_000));
    }

    #[test]
    fn dropped_plants_are_gone() {
        let meadow = Vegetation::typical(VegetationKind::ShoreMeadow);
        let mut sim = simulation(ground(Patch::new(GroundType::Mud, Some(meadow), 0)));
        let reed = Item::Picked(Species::CommonReed);
        sim.apply(Command::Gather(reed));
        while sim.state().player.inventory().is_empty() {
            sim.tick();
        }

        sim.apply(Command::Drop {
            item: reed,
            count: 1,
        });

        assert!(sim.state().player.inventory().is_empty());
        let patch = sim.state().world.terrain().patch_at(Location::default());
        assert_eq!(
            patch.map(|patch| patch.left(reed, Season::usual(0))),
            Some(127)
        );
    }

    #[test]
    fn walking_off_ends_a_search() {
        let mut world = ground(gravel());
        let tree = Subject::Thing(world.add_thing(birch(0, 20_000)));
        let mut sim = simulation(world);
        sim.apply(Command::Gather(Item::Pebble));
        sim.tick();

        sim.apply(Command::WalkTo(tree));
        let carried = pebbles(&sim);
        walk_until_done(&mut sim);

        assert_eq!(pebbles(&sim), carried);
        assert!(within_reach(&sim, tree));
    }

    #[test]
    fn walk_covers_walking_speed_each_tick_and_ends_within_reach() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(0, 20_000)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(tree));
        assert_eq!(
            sim.view().walk,
            Some(Walk {
                toward: Kind::Plant(Species::Birch),
                destination: Location::new(0, 19_450),
                via: Vec::new(),
                remaining: Length::from_millimeters(19_450),
            })
        );

        sim.tick();
        assert_eq!(sim.state().player.location(), Location::new(0, 1_400));

        assert_eq!(walk_until_done(&mut sim).len(), 13);
        assert_eq!(sim.state().player.location(), Location::new(0, 19_450));
        assert_eq!(sim.view().walk, None);
        assert_eq!(sim.view().surroundings.within_reach.len(), 1);
    }

    /// A simulation of a player who has set off for a tree behind a lake, a little east of
    /// straight across it, along with that tree.
    fn walk_across_water() -> (Simulation, Subject) {
        let mut world = World::default();
        world.add_area(lake(30_000, 50_000));
        let tree = Subject::Thing(world.add_thing(birch(5_000, 60_000)));
        let mut sim = simulation(world);
        sim.apply(Command::WalkTo(tree));

        (sim, tree)
    }

    #[test]
    fn walk_goes_around_water_by_the_shorter_side() {
        let (mut sim, tree) = walk_across_water();
        assert_eq!(sim.view().walk.map(|walk| walk.via.len()), Some(2));

        let path = walk_until_done(&mut sim);

        assert!(path.iter().any(|at| at.x() > 20_000));
        assert!(path.iter().all(|at| at.x() > -20_000));
        assert!(within_reach(&sim, tree));
    }

    #[test]
    fn walk_turns_aside_once_the_water_in_the_way_is_seen() {
        let mut world = World::default();
        world.add_area(lake(310_000, 330_000));
        let tree = world.add_thing(birch(5_000, 340_000));
        let mut state = State {
            world,
            ..State::new(Name::default(), Sex::Female)
        };
        state.memory.note_thing(tree, Timestamp::EPOCH);
        let mut sim = Simulation::from_state(state);

        sim.apply(Command::WalkTo(Subject::Thing(tree)));
        assert_eq!(sim.view().walk.map(|walk| walk.via), Some(Vec::new()));

        let path = walk_until_done(&mut sim);

        assert!(path.iter().any(|at| at.x() > 20_000));
        assert!(within_reach(&sim, Subject::Thing(tree)));
    }

    #[test]
    fn walk_around_water_carries_on_from_a_state() {
        let (mut sim, _) = walk_across_water();
        for _ in 0..30 {
            sim.tick();
        }

        let mut resumed = Simulation::from_state(sim.state().clone());

        assert_eq!(resumed.view().walk, sim.view().walk);
        assert_eq!(walk_until_done(&mut resumed), walk_until_done(&mut sim));
    }

    #[test]
    fn walk_to_a_lake_ends_with_the_water_within_reach() {
        let mut world = World::default();
        let water = Subject::Area(world.add_area(lake(30_000, 50_000)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(water));
        walk_until_done(&mut sim);

        let view = sim.view();
        assert_eq!(view.surroundings.within_reach.len(), 1);
        assert_eq!(view.surroundings.within_reach[0].subject, water);
    }

    #[test]
    fn stopping_ends_a_walk_where_it_has_got_to() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(0, 20_000)));
        let mut sim = simulation(world);
        sim.apply(Command::WalkTo(tree));
        sim.tick();

        sim.apply(Command::Stop);
        sim.tick();

        assert_eq!(sim.state().player.activity(), None);
        assert_eq!(sim.state().player.location(), Location::new(0, 1_400));
    }

    #[test]
    fn nobody_walks_to_what_they_do_not_perceive() {
        let mut world = World::default();
        world.add_thing(birch(0, 5_000));
        let hidden = Subject::Thing(world.add_thing(birch(0, 20_000)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(hidden));

        assert_eq!(sim.state().player.activity(), None);
    }

    #[test]
    fn map_marks_what_is_in_sight_where_it_stands() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(3_000, 20_000)));
        let sim = simulation(world);

        assert_eq!(
            mark(&sim, tree),
            Some(Mark {
                subject: tree,
                kind: Kind::Plant(Species::Birch),
                location: Location::new(3_000, 20_000),
                footprint: Length::from_millimeters(150),
                last_seen: None,
            })
        );
    }

    #[test]
    fn map_keeps_what_was_seen_and_never_has_what_was_not() {
        let (sim, [left_behind, hidden, walked_to]) = walk_away();

        assert!(sim.view().surroundings.sighting(left_behind).is_none());
        assert_eq!(
            mark(&sim, left_behind).map(|mark| mark.in_sight()),
            Some(false)
        );
        assert_eq!(
            mark(&sim, walked_to).map(|mark| mark.in_sight()),
            Some(true)
        );
        assert_eq!(mark(&sim, hidden), None);
    }

    #[test]
    fn thing_left_behind_is_remembered_from_when_it_went_out_of_sight() {
        let (mut sim, [left_behind, ..]) = walk_away();
        let lost = Some(Timestamp::from_seconds(51));
        assert_eq!(
            mark(&sim, left_behind).map(|mark| mark.last_seen),
            Some(lost)
        );

        for _ in 0..100 {
            sim.tick();
        }

        assert_eq!(
            mark(&sim, left_behind).map(|mark| mark.last_seen),
            Some(lost)
        );
    }

    #[test]
    fn thing_watched_while_standing_still_is_remembered_from_when_the_player_left() {
        let mut world = World::default();
        let barely_seen = Subject::Thing(world.add_thing(birch(0, 90_000)));
        let walked_to = Subject::Thing(world.add_thing(birch(0, -50_000)));
        let mut sim = simulation(world);
        for _ in 0..100 {
            sim.tick();
        }
        assert_eq!(
            mark(&sim, barely_seen).map(|mark| mark.last_seen),
            Some(None)
        );

        sim.apply(Command::WalkTo(walked_to));
        sim.tick();

        assert_eq!(
            mark(&sim, barely_seen).map(|mark| mark.last_seen),
            Some(Some(Timestamp::from_seconds(101)))
        );
    }

    #[test]
    fn map_has_the_ground_seen_on_the_way_and_none_that_was_not() {
        let mut world = World::default();
        let heath = Vegetation::typical(VegetationKind::LichenHeath);
        let patches = (0..10 * 50).map(|_| Patch::new(GroundType::Sand, Some(heath.clone()), 0));
        world.set_terrain(Terrain::new(Location::new(-20_000, -100_000), 10, patches));
        let tree = Subject::Thing(world.add_thing(birch(0, 80_000)));
        let mut sim = simulation(world);
        let known = |sim: &Simulation, x, y| {
            let map = sim.view().map;
            map.ground
                .iter()
                .any(|tile| tile.corner == Location::new(x, y))
        };
        assert!(known(&sim, 0, 0));
        assert!(known(&sim, 0, 24_000));
        assert!(!known(&sim, 0, 60_000));

        sim.apply(Command::WalkTo(tree));
        walk_until_done(&mut sim);

        assert!(known(&sim, 0, 0));
        assert!(known(&sim, 0, 60_000));
        assert!(!known(&sim, 0, -40_000));
        assert_eq!(
            sim.view().map.ground.first(),
            Some(&Tile {
                corner: Location::new(-16_000, -28_000),
                ground_type: GroundType::Sand,
                vegetation: Some(VegetationKind::LichenHeath),
            })
        );
    }

    #[test]
    fn map_reaches_as_far_diagonally_as_it_does_straight() {
        let mut world = World::default();
        let walked_to = Subject::Thing(world.add_thing(tall_boulder(250_000, 0)));
        let off_to_the_corner = Subject::Thing(world.add_thing(tall_boulder(0, 250_000)));
        let left_behind = Subject::Thing(world.add_thing(tall_boulder(-60_000, 0)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(walked_to));
        walk_until_done(&mut sim);

        assert!(mark(&sim, off_to_the_corner).is_some());
        assert_eq!(mark(&sim, left_behind), None);
    }

    #[test]
    fn memory_is_part_of_the_state_a_simulation_carries_on_from() {
        let (sim, [left_behind, ..]) = walk_away();

        let resumed = Simulation::from_state(sim.state().clone());

        assert_eq!(resumed.state(), sim.state());
        assert_eq!(resumed.view().map, sim.view().map);
        assert!(mark(&resumed, left_behind).is_some());
    }

    #[test]
    fn walk_can_head_for_what_is_only_remembered() {
        let (mut sim, [left_behind, hidden, _]) = walk_away();

        sim.apply(Command::WalkTo(hidden));
        assert_eq!(sim.state().player.activity(), None);

        sim.apply(Command::WalkTo(left_behind));
        walk_until_done(&mut sim);
        assert_eq!(sim.state().player.location(), Location::new(0, 19_450));
    }

    #[test]
    fn shore_is_mapped_where_it_has_been_seen() {
        let mut world = World::default();
        let water = Subject::Area(world.add_area(lake(30_000, 50_000)));
        world.add_thing(tall_boulder(4_000, 10_000));
        world.add_thing(tall_boulder(-4_000, 10_000));
        let mut sim = simulation(world);
        let corner = |x, y| Location::new(x, y);

        let map = sim.view().map;
        assert_eq!(map.waters.len(), 1);
        assert_eq!(map.waters[0].subject, water);
        assert_eq!(map.waters[0].kind, Kind::Lake);
        assert_eq!(
            map.waters[0].shore,
            [
                [corner(-20_000, 30_000), corner(20_000, 30_000)],
                [corner(20_000, 30_000), corner(20_000, 50_000)],
                [corner(-20_000, 50_000), corner(-20_000, 30_000)],
            ]
        );
        assert_eq!(map.waters[0].surface.len(), 2);

        sim.apply(Command::WalkTo(water));
        walk_until_done(&mut sim);

        assert_eq!(sim.view().map.waters[0].shore.len(), 4);
    }

    #[test]
    fn water_is_cut_straight_across_where_its_shore_has_not_been_seen() {
        let mut world = World::default();
        let water = Subject::Area(world.add_area(lake(30_000, 50_000)));
        world.add_thing(tall_boulder(-4_000, 6_000));
        world.add_thing(tall_boulder(-4_000, 10_000));
        world.add_thing(tall_boulder(4_000, 10_000));
        let mut sim = simulation(world);
        let southwest = Location::new(-20_000, 30_000);
        let southeast = Location::new(20_000, 30_000);
        let northeast = Location::new(20_000, 50_000);

        let map = sim.view().map;
        assert_eq!(
            map.waters[0].shore,
            [[southwest, southeast], [southeast, northeast]]
        );
        assert_eq!(map.waters[0].surface.len(), 1);
        for corner in [southwest, southeast, northeast] {
            assert!(map.waters[0].surface[0].contains(&corner));
        }

        sim.apply(Command::WalkTo(water));
        walk_until_done(&mut sim);

        assert_eq!(sim.view().map.waters[0].surface.len(), 2);
    }

    #[test]
    fn single_stretch_of_shore_has_no_water_drawn_inside_it() {
        let mut world = World::default();
        world.add_area(lake(30_000, 50_000));
        for x in [-4_000, 4_000] {
            world.add_thing(tall_boulder(x, 6_000));
            world.add_thing(tall_boulder(x, 10_000));
        }

        let map = simulation(world).view().map;

        assert_eq!(map.waters[0].shore.len(), 1);
        assert!(map.waters[0].surface.is_empty());
    }

    #[test]
    fn water_never_seen_is_not_on_the_map() {
        let mut world = World::default();
        world.add_area(lake(400_000, 420_000));

        assert!(simulation(world).view().map.waters.is_empty());
    }

    #[test]
    fn walking_to_what_is_within_reach_goes_nowhere() {
        let mut world = World::default();
        let tree = Subject::Thing(world.add_thing(birch(0, 500)));
        let mut sim = simulation(world);

        sim.apply(Command::WalkTo(tree));

        assert_eq!(sim.state().player.activity(), None);
    }
}
