use lords_sim::world::{Abundance, GroundType, Kind, Part, Species, VegetationKind};
use lords_sim::{Bearing, Item, Length};

const COUNTED: u32 = 20;

pub(super) fn kind_name(kind: Kind) -> String {
    match kind {
        Kind::Plant(species) => capitalized(species_name(species)),
        Kind::Rock => "Boulder".to_owned(),
        Kind::Lake => "Lake".to_owned(),
    }
}

pub(super) fn distance(length: Length) -> String {
    let millimeters = length.as_millimeters();
    if millimeters < 1_000 {
        return "under 1 m".to_owned();
    }

    let precision = if millimeters < 10_000 {
        1_000
    } else if millimeters < 100_000 {
        5_000
    } else {
        10_000
    };
    let rounded = (millimeters + precision / 2) / precision * precision;
    format!("{} m", rounded / 1_000)
}

pub(super) fn compass_point(bearing: Bearing) -> &'static str {
    match bearing {
        Bearing::North => "N",
        Bearing::Northeast => "NE",
        Bearing::East => "E",
        Bearing::Southeast => "SE",
        Bearing::South => "S",
        Bearing::Southwest => "SW",
        Bearing::West => "W",
        Bearing::Northwest => "NW",
    }
}

pub(super) fn ground_type_name(ground_type: GroundType) -> &'static str {
    match ground_type {
        GroundType::Till => "Till",
        GroundType::Sand => "Sand",
        GroundType::Gravel => "Gravel",
        GroundType::Mud => "Mud",
        GroundType::Peat => "Peat",
    }
}

pub(super) fn ground_type_description(ground_type: GroundType) -> &'static str {
    match ground_type {
        GroundType::Till => "Sand, gravel, and stones mixed together",
        GroundType::Sand => "Loose, fine grains",
        GroundType::Gravel => "Small stones washed smooth by the water",
        GroundType::Mud => "Soft, wet silt",
        GroundType::Peat => "Wet, spongy remains of moss",
    }
}

pub(super) fn vegetation_name(kind: VegetationKind) -> &'static str {
    match kind {
        VegetationKind::BilberryHeath => "Bilberry heath",
        VegetationKind::LingonberryHeath => "Lingonberry heath",
        VegetationKind::LichenHeath => "Lichen heath",
        VegetationKind::HerbRichHeath => "Herb-rich heath",
        VegetationKind::PineBog => "Pine bog",
        VegetationKind::ShoreMeadow => "Shore meadow",
    }
}

pub(super) fn abundance(abundance: Abundance) -> &'static str {
    match abundance {
        Abundance::Plenty => "plenty",
        Abundance::Moderate => "some",
        Abundance::Scarce => "a few",
        Abundance::Gone => "none",
    }
}

pub(super) fn species_name(species: Species) -> &'static str {
    match species {
        Species::Spruce => "spruce",
        Species::Pine => "pine",
        Species::Birch => "birch",
        Species::Aspen => "aspen",
        Species::GreyAlder => "grey alder",
        Species::Bilberry => "bilberry",
        Species::Lingonberry => "lingonberry",
        Species::Crowberry => "crowberry",
        Species::Heather => "heather",
        Species::BogBilberry => "bog bilberry",
        Species::LabradorTea => "Labrador tea",
        Species::Cranberry => "cranberry",
        Species::BogRosemary => "bog rosemary",
        Species::Cloudberry => "cloudberry",
        Species::MayLily => "May lily",
        Species::ArcticStarflower => "arctic starflower",
        Species::WoodSorrel => "wood sorrel",
        Species::WoodCranesbill => "wood cranesbill",
        Species::Meadowsweet => "meadowsweet",
        Species::YellowLoosestrife => "yellow loosestrife",
        Species::WavyHairGrass => "wavy hair-grass",
        Species::TuftedHairGrass => "tufted hair-grass",
        Species::Cottongrass => "cottongrass",
        Species::BottleSedge => "bottle sedge",
        Species::CommonReed => "common reed",
        Species::OakFern => "oak fern",
        Species::FeatherMoss => "red-stemmed feather-moss",
        Species::StairStepMoss => "stair-step moss",
        Species::PeatMoss => "peat moss",
        Species::ReindeerLichen => "reindeer lichen",
    }
}

pub(super) fn item_name(item: Item, count: u32) -> String {
    let species = match item {
        Item::Pebble => return plural(count, "pebble", "pebbles").to_owned(),
        Item::Picked(species) => species,
    };
    let species_name = species_name(species);

    match species.crop().map(|crop| crop.part) {
        None => species_name.to_owned(),
        Some(Part::Berry) if count == 1 => species_name.to_owned(),
        Some(Part::Berry) => match species_name.strip_suffix('y') {
            Some(stem) => format!("{stem}ies"),
            None => format!("{species_name}s"),
        },
        Some(part) => format!("{} of {species_name}", part_name(part, count)),
    }
}

fn part_name(part: Part, count: u32) -> &'static str {
    match part {
        Part::Berry => plural(count, "berry", "berries"),
        Part::Sprig => plural(count, "sprig", "sprigs"),
        Part::Leaf => plural(count, "leaf", "leaves"),
        Part::Stem => plural(count, "stem", "stems"),
        Part::Tuft => plural(count, "tuft", "tufts"),
        Part::Stalk => plural(count, "stalk", "stalks"),
        Part::Frond => plural(count, "frond", "fronds"),
        Part::Clump => plural(count, "clump", "clumps"),
    }
}

fn plural(count: u32, one: &'static str, many: &'static str) -> &'static str {
    if count == 1 { one } else { many }
}

pub(super) fn gather_order(item: Item) -> String {
    let [order, ..] = verbs(item);
    format!("{order} {}", item_name(item, 2))
}

pub(super) fn gathering(item: Item, gathered: u32) -> String {
    let [_, doing, done] = verbs(item);
    let doing = format!("{doing} {}", item_name(item, 2));

    match gathered {
        0 => doing,
        gathered => format!("{doing}, {} {done}", about(gathered)),
    }
}

pub(super) fn about(count: u32) -> String {
    match estimate(count) {
        Some(rough) => format!("about {}", grouped(rough)),
        None => count.to_string(),
    }
}

pub(super) fn roughly(count: u32) -> String {
    match estimate(count) {
        Some(rough) => format!("~{}", grouped(rough)),
        None => count.to_string(),
    }
}

pub(super) fn at_most(count: u32) -> u32 {
    if count <= COUNTED {
        return count;
    }
    let step = rounding(count);
    count.div_ceil(step).saturating_mul(step)
}

fn estimate(count: u32) -> Option<u32> {
    if count <= COUNTED {
        return None;
    }
    let step = u64::from(rounding(count));
    let rounded = (u64::from(count) + step / 2) / step * step;

    Some(u32::try_from(rounded).unwrap_or(u32::MAX))
}

fn rounding(count: u32) -> u32 {
    match count.checked_ilog10() {
        None | Some(0 | 1) => 10,
        Some(digits) => 5 * 10_u32.pow(digits - 1),
    }
}

fn grouped(number: u32) -> String {
    let digits = number.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

fn verbs(item: Item) -> [&'static str; 3] {
    match item {
        Item::Pebble => ["Search for", "Searching for", "found"],
        Item::Picked(_) => ["Pick", "Picking", "picked"],
    }
}

pub(super) fn capitalized(text: &str) -> String {
    let mut letters = text.chars();
    letters
        .next()
        .map(|first| first.to_uppercase().chain(letters).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_is_rounded_more_roughly_the_farther_it_is() {
        let worded = |millimeters| distance(Length::from_millimeters(millimeters));

        assert_eq!(worded(0), "under 1 m");
        assert_eq!(worded(999), "under 1 m");
        assert_eq!(worded(1_000), "1 m");
        assert_eq!(worded(3_600), "4 m");
        assert_eq!(worded(9_400), "9 m");
        assert_eq!(worded(12_000), "10 m");
        assert_eq!(worded(13_000), "15 m");
        assert_eq!(worded(97_000), "95 m");
        assert_eq!(worded(104_000), "100 m");
        assert_eq!(worded(138_000), "140 m");
    }

    #[test]
    fn item_is_named_by_how_many_there_are() {
        let picked = |species, count| item_name(Item::Picked(species), count);

        assert_eq!(item_name(Item::Pebble, 1), "pebble");
        assert_eq!(item_name(Item::Pebble, 0), "pebbles");
        assert_eq!(item_name(Item::Pebble, 7), "pebbles");
        assert_eq!(picked(Species::Bilberry, 1), "bilberry");
        assert_eq!(picked(Species::BogBilberry, 2), "bog bilberries");
        assert_eq!(picked(Species::Heather, 1), "sprig of heather");
        assert_eq!(picked(Species::WoodSorrel, 3), "leaves of wood sorrel");
        assert_eq!(picked(Species::MayLily, 2), "stems of May lily");
        assert_eq!(
            picked(Species::FeatherMoss, 2),
            "clumps of red-stemmed feather-moss"
        );
    }

    #[test]
    fn every_species_is_named_and_berries_are_named_as_berries() {
        for species in Species::ALL {
            let Some(crop) = species.crop() else {
                continue;
            };
            let one = item_name(Item::Picked(species), 1);
            let many = item_name(Item::Picked(species), 2);

            assert_ne!(one, many, "{species:?}");
            if crop.part == Part::Berry {
                assert!(many.ends_with("berries"), "{many}");
            } else {
                assert!(many.contains(" of "), "{many}");
            }
        }
    }

    #[test]
    fn gathering_is_worded_by_how_the_item_is_come_by() {
        let bilberry = Item::Picked(Species::Bilberry);

        assert_eq!(gather_order(Item::Pebble), "Search for pebbles");
        assert_eq!(gather_order(bilberry), "Pick bilberries");
        assert_eq!(gathering(Item::Pebble, 0), "Searching for pebbles");
        assert_eq!(gathering(Item::Pebble, 1), "Searching for pebbles, 1 found");
        assert_eq!(gathering(bilberry, 3), "Picking bilberries, 3 picked");
    }

    #[test]
    fn crop_left_is_worded_roughly() {
        assert_eq!(abundance(Abundance::Plenty), "plenty");
        assert_eq!(abundance(Abundance::Scarce), "a few");
        assert_eq!(abundance(Abundance::Gone), "none");
    }

    #[test]
    fn counts_are_told_exactly_while_few_and_estimated_ever_more_roughly_after() {
        assert_eq!(about(0), "0");
        assert_eq!(about(20), "20");
        assert_eq!(about(21), "about 20");
        assert_eq!(about(46), "about 50");
        assert_eq!(about(96), "about 100");
        assert_eq!(about(150), "about 150");
        assert_eq!(about(342), "about 350");
        assert_eq!(about(4_285), "about 4,500");
        assert_eq!(about(1_234_567), "about 1,000,000");
        assert_eq!(about(u32::MAX), "about 4,294,967,295");
        assert_eq!(roughly(424), "~400");
        assert_eq!(roughly(12), "12");
    }

    #[test]
    fn largest_estimate_does_not_give_away_the_count() {
        assert_eq!(at_most(7), 7);
        assert_eq!(at_most(21), 30);
        assert_eq!(at_most(4_285), 4_500);
        assert_eq!(at_most(4_500), 4_500);
        assert!(at_most(u32::MAX) >= u32::MAX - 1);
    }

    #[test]
    fn first_letter_is_made_a_capital() {
        assert_eq!(capitalized("bilberry"), "Bilberry");
        assert_eq!(capitalized("Labrador tea"), "Labrador tea");
        assert_eq!(capitalized("äkämä"), "Äkämä");
        assert_eq!(capitalized(""), "");
    }
}
