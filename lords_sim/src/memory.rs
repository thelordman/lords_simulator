use serde::{Deserialize, Serialize};

use crate::time::Timestamp;
use crate::world::{AreaId, Store, Subject, ThingId};

const BITS: u32 = u64::BITS;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    /// When each thing was last seen, in the order of their ids.
    things: Store<Option<Timestamp>>,
    /// For each area, the edges of its outline that have been seen.
    shores: Vec<Marks>,
    /// The patches of the terrain that have been seen, by their index.
    #[serde(default)]
    ground: Marks,
}

impl Memory {
    pub fn knows(&self, subject: Subject) -> bool {
        match subject {
            Subject::Thing(id) => self.knows_thing(id),
            Subject::Area(id) => self.shore(id).is_some_and(|shore| !shore.is_empty()),
        }
    }

    pub fn knows_thing(&self, id: ThingId) -> bool {
        self.last_seen(id).is_some()
    }

    pub fn last_seen(&self, id: ThingId) -> Option<Timestamp> {
        self.things.get(id.index()).copied().flatten()
    }

    /// Returns whether the edge of the outline of `area` that starts from its point
    /// `edge` has been seen.
    pub fn knows_edge(&self, area: AreaId, edge: u32) -> bool {
        self.shore(area).is_some_and(|shore| shore.has(edge))
    }

    /// Returns whether the patch of the terrain at `index` has been seen.
    pub fn knows_patch(&self, index: u32) -> bool {
        self.ground.has(index)
    }

    /// Remembers the thing as seen at `now`.
    pub(crate) fn note_thing(&mut self, id: ThingId, now: Timestamp) {
        if self.last_seen(id) == Some(now) {
            return;
        }

        while self.things.len() <= id.index() as usize {
            self.things.push(None);
        }
        if let Some(last_seen) = self.things.get_mut(id.index()) {
            *last_seen = Some(now);
        }
    }

    pub(crate) fn note_edge(&mut self, area: AreaId, edge: u32) {
        let area = area.index() as usize;
        if self.shores.len() <= area {
            self.shores.resize_with(area + 1, Marks::default);
        }
        self.shores[area].mark(edge);
    }

    pub(crate) fn note_patch(&mut self, index: u32) {
        self.ground.mark(index);
    }

    fn shore(&self, area: AreaId) -> Option<&Marks> {
        self.shores.get(area.index() as usize)
    }
}

/// A set of numbers, kept as a bit for each.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct Marks {
    words: Store<u64>,
}

impl Marks {
    fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    fn has(&self, number: u32) -> bool {
        self.words
            .get(number / BITS)
            .is_some_and(|word| word & (1 << (number % BITS)) != 0)
    }

    fn mark(&mut self, number: u32) {
        if self.has(number) {
            return;
        }

        while self.words.len() <= (number / BITS) as usize {
            self.words.push(0);
        }
        if let Some(word) = self.words.get_mut(number / BITS) {
            *word |= 1 << (number % BITS);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Object;
    use crate::space::Length;
    use crate::world::{Area, AreaKind, Rock, Thing, World};

    const NOW: Timestamp = Timestamp::from_seconds(600);

    fn ids() -> (Vec<ThingId>, Vec<AreaId>) {
        let mut world = World::default();
        let lake = || Area::new(AreaKind::Lake, Vec::new());
        let pebble = || Thing::Rock(Rock::new(Object::default(), Length::ZERO, Length::ZERO));
        let areas = (0..3).map(|_| world.add_area(lake())).collect();
        let things = (0..20_000).map(|_| world.add_thing(pebble())).collect();

        (things, areas)
    }

    #[test]
    fn only_what_was_noted_is_known() {
        let (things, areas) = ids();
        let mut memory = Memory::default();

        memory.note_thing(things[0], NOW);
        memory.note_thing(things[255], NOW);
        memory.note_thing(things[256], NOW);
        memory.note_thing(things[19_999], NOW);
        memory.note_edge(areas[1], 63);
        memory.note_edge(areas[1], 70);
        memory.note_patch(9_000);

        for (index, thing) in things.iter().enumerate() {
            let noted = [0, 255, 256, 19_999].contains(&index);
            assert_eq!(memory.knows_thing(*thing), noted, "thing {index}");
            assert_eq!(memory.knows(Subject::Thing(*thing)), noted);
        }
        assert!(memory.knows_edge(areas[1], 63));
        assert!(memory.knows_edge(areas[1], 70));
        assert!(!memory.knows_edge(areas[1], 64));
        assert!(!memory.knows_edge(areas[1], 69));
        assert!(!memory.knows_edge(areas[0], 70));
        assert!(!memory.knows_edge(areas[2], 70));
        assert!(memory.knows_patch(9_000));
        assert!(!memory.knows_patch(8_999));
        assert!(!memory.knows_patch(0));
        assert!(memory.knows(Subject::Area(areas[1])));
        assert!(!memory.knows(Subject::Area(areas[0])));
        assert!(!memory.knows(Subject::Area(areas[2])));
    }

    #[test]
    fn thing_is_remembered_from_when_it_was_last_seen() {
        let (things, _) = ids();
        let later = Timestamp::from_seconds(900);
        let mut memory = Memory::default();
        assert_eq!(memory.last_seen(things[5]), None);

        memory.note_thing(things[5], NOW);
        memory.note_thing(things[6], NOW);
        memory.note_thing(things[5], later);

        assert_eq!(memory.last_seen(things[5]), Some(later));
        assert_eq!(memory.last_seen(things[6]), Some(NOW));
        assert_eq!(memory.last_seen(things[7]), None);
    }

    #[test]
    fn memory_reads_back_as_it_was_written() {
        let (things, areas) = ids();
        let mut memory = Memory::default();
        memory.note_thing(things[5], NOW);
        memory.note_thing(things[19_999], Timestamp::EPOCH);
        memory.note_edge(areas[2], 3);
        memory.note_patch(42);

        let written = ron::to_string(&memory).expect("the memory encodes");
        let read: Memory = ron::from_str(&written).expect("the memory decodes");

        assert_eq!(read, memory);
    }

    #[test]
    fn noting_what_is_known_already_changes_nothing() {
        let (things, areas) = ids();
        let mut memory = Memory::default();
        memory.note_thing(things[5], NOW);
        memory.note_edge(areas[0], 3);
        memory.note_patch(7);
        let before = memory.clone();

        memory.note_thing(things[5], NOW);
        memory.note_edge(areas[0], 3);
        memory.note_patch(7);

        assert_eq!(memory, before);
    }
}
