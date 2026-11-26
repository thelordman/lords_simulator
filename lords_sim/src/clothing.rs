use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Garment {
    Shirt,
    Trousers,
    Jacket,
    Boots,
}

impl Garment {
    /// Returns the room in its pockets, in cubic millimeters.
    pub fn pocket_room(self) -> u64 {
        match self {
            Self::Shirt | Self::Boots => 0,
            Self::Trousers => 1_000_000,
            Self::Jacket => 1_500_000,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Clothes {
    worn: Vec<Garment>,
}

impl Clothes {
    pub fn everyday() -> Self {
        Self {
            worn: vec![
                Garment::Shirt,
                Garment::Trousers,
                Garment::Jacket,
                Garment::Boots,
            ],
        }
    }

    pub fn worn(&self) -> &[Garment] {
        &self.worn
    }

    /// Returns the room in all their pockets, in cubic millimeters.
    pub fn pocket_room(&self) -> u64 {
        self.worn.iter().map(|garment| garment.pocket_room()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pockets_hold_what_the_clothes_worn_have_room_for() {
        assert_eq!(Clothes::default().pocket_room(), 0);
        assert_eq!(Clothes::everyday().pocket_room(), 2_500_000);
    }

    #[test]
    fn clothes_read_back_as_they_were_written() {
        let clothes = Clothes::everyday();

        let written = ron::to_string(&clothes).expect("the clothes encode");
        let read: Clothes = ron::from_str(&written).expect("the clothes decode");

        assert_eq!(read, clothes);
    }
}
