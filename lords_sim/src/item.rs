use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::world::Species;

/// Something small enough to pick up and carry.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Item {
    Pebble,
    /// What is picked of a plant of the species.
    Picked(Species),
}

impl Item {
    /// Returns the room one takes up when carried, in cubic millimeters.
    pub fn bulk(self) -> u32 {
        match self {
            Self::Pebble => 20_000,
            Self::Picked(species) => species.crop().map_or(0, |crop| crop.bulk),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Inventory {
    counts: BTreeMap<Item, u32>,
}

impl Inventory {
    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    pub fn count(&self, item: Item) -> u32 {
        self.counts.get(&item).copied().unwrap_or(0)
    }

    /// Returns each item carried with how many of it there are.
    pub fn items(&self) -> impl Iterator<Item = (Item, u32)> + '_ {
        self.counts.iter().map(|(item, count)| (*item, *count))
    }

    /// Returns the room all of it takes up, in cubic millimeters.
    pub fn bulk(&self) -> u64 {
        self.items()
            .map(|(item, count)| u64::from(item.bulk()) * u64::from(count))
            .sum()
    }

    pub(crate) fn add(&mut self, item: Item) {
        *self.counts.entry(item).or_default() += 1;
    }

    /// Takes out up to `count` of `item` and returns how many were taken.
    pub(crate) fn remove(&mut self, item: Item, count: u32) -> u32 {
        let Some(carried) = self.counts.get_mut(&item) else {
            return 0;
        };
        let taken = count.min(*carried);
        *carried -= taken;
        if *carried == 0 {
            self.counts.remove(&item);
        }

        taken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BILBERRY: Item = Item::Picked(Species::Bilberry);

    #[test]
    fn inventory_counts_what_is_added() {
        let mut inventory = Inventory::default();
        assert!(inventory.is_empty());
        assert_eq!(inventory.count(Item::Pebble), 0);

        inventory.add(Item::Pebble);
        inventory.add(BILBERRY);
        inventory.add(Item::Pebble);

        assert!(!inventory.is_empty());
        assert_eq!(inventory.count(Item::Pebble), 2);
        assert_eq!(
            inventory.items().collect::<Vec<_>>(),
            [(Item::Pebble, 2), (BILBERRY, 1)]
        );
    }

    #[test]
    fn inventory_takes_up_the_room_of_all_that_is_in_it() {
        let mut inventory = Inventory::default();
        assert_eq!(inventory.bulk(), 0);

        inventory.add(Item::Pebble);
        inventory.add(Item::Pebble);
        inventory.add(BILBERRY);

        assert_eq!(inventory.bulk(), 2 * 20_000 + 700);
    }

    #[test]
    fn removing_takes_out_no_more_than_there_is() {
        let mut inventory = Inventory::default();
        for _ in 0..3 {
            inventory.add(Item::Pebble);
        }
        inventory.add(BILBERRY);

        assert_eq!(inventory.remove(Item::Pebble, 1), 1);
        assert_eq!(inventory.count(Item::Pebble), 2);
        assert_eq!(inventory.remove(Item::Pebble, 0), 0);
        assert_eq!(inventory.remove(Item::Pebble, 5), 2);
        assert_eq!(inventory.remove(Item::Pebble, 1), 0);
        assert_eq!(inventory.items().collect::<Vec<_>>(), [(BILBERRY, 1)]);
    }

    #[test]
    fn inventory_reads_back_as_it_was_written() {
        let mut inventory = Inventory::default();
        inventory.add(Item::Pebble);
        inventory.add(BILBERRY);

        let written = ron::to_string(&inventory).expect("the inventory encodes");
        let read: Inventory = ron::from_str(&written).expect("the inventory decodes");

        assert_eq!(read, inventory);
    }
}
