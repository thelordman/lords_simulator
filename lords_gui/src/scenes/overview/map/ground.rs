use egui::{Color32, Pos2, Rect, Shape};
use lords_sim::Location;
use lords_sim::map::Tile;
use lords_sim::world::{GroundType, Terrain, VegetationKind};

use crate::scenes::overview::words::{ground_type_name, vegetation_name};

use super::Projection;

const VEGETATION_TINT: f32 = 0.5;

pub(super) fn draw(tiles: &[Tile], projection: Projection, bounds: Rect, dark_mode: bool) -> Shape {
    let mut mesh = egui::Mesh::default();
    for tile in tiles {
        let square = square(tile, projection);
        if square.intersects(bounds) {
            mesh.add_colored_rect(square, color(tile.ground_type, tile.vegetation, dark_mode));
        }
    }

    Shape::mesh(mesh)
}

pub(super) fn under(pointer: Pos2, tiles: &[Tile], projection: Projection) -> Option<&Tile> {
    tiles
        .iter()
        .find(|tile| square(tile, projection).contains(pointer))
}

pub(super) fn describe(tile: &Tile) -> String {
    let ground = ground_type_name(tile.ground_type);
    match tile.vegetation {
        Some(kind) => format!("{} on {}", vegetation_name(kind), ground.to_lowercase()),
        None => ground.to_owned(),
    }
}

fn square(tile: &Tile, projection: Projection) -> Rect {
    let side = Terrain::PATCH_SIDE.as_millimeters();
    let corner = tile.corner;
    let opposite = Location::new(corner.x() + side, corner.y() + side);

    Rect::from_two_pos(projection.position(corner), projection.position(opposite))
}

fn color(ground_type: GroundType, vegetation: Option<VegetationKind>, dark_mode: bool) -> Color32 {
    let pick = |(dark, light): ([u8; 3], [u8; 3])| {
        let [red, green, blue] = if dark_mode { dark } else { light };
        Color32::from_rgb(red, green, blue)
    };
    let bare = pick(match ground_type {
        GroundType::Till => ([60, 58, 50], [196, 190, 170]),
        GroundType::Sand => ([110, 100, 72], [238, 222, 170]),
        GroundType::Gravel => ([86, 86, 84], [204, 202, 198]),
        GroundType::Mud => ([66, 58, 44], [190, 176, 150]),
        GroundType::Peat => ([74, 52, 40], [196, 160, 136]),
    });
    let Some(kind) = vegetation else {
        return bare;
    };
    let overgrowth = pick(match kind {
        VegetationKind::BilberryHeath => ([24, 70, 40], [150, 200, 150]),
        VegetationKind::LingonberryHeath => ([52, 74, 30], [180, 200, 130]),
        VegetationKind::LichenHeath => ([96, 104, 88], [225, 228, 205]),
        VegetationKind::HerbRichHeath => ([20, 84, 50], [140, 210, 160]),
        VegetationKind::PineBog => ([90, 70, 50], [210, 170, 150]),
        VegetationKind::ShoreMeadow => ([60, 80, 40], [180, 205, 140]),
    });

    bare.lerp_to_gamma(overgrowth, VEGETATION_TINT)
}

#[cfg(test)]
mod tests {
    use super::super::Palette;
    use super::*;

    const PROJECTION: Projection = Projection {
        center: Pos2::new(200.0, 150.0),
        origin: Location::new(5_000, -3_000),
        points_per_meter: 2.0,
    };

    fn tile(x: i64, y: i64, ground_type: GroundType, vegetation: Option<VegetationKind>) -> Tile {
        Tile {
            corner: Location::new(x, y),
            ground_type,
            vegetation,
        }
    }

    #[test]
    fn ground_under_the_pointer_is_named_by_what_it_is_and_what_grows_on_it() {
        let heath = tile(
            5_000,
            -3_000,
            GroundType::Till,
            Some(VegetationKind::BilberryHeath),
        );
        let beach = tile(1_000, -3_000, GroundType::Gravel, None);
        let tiles = [heath, beach];
        let pointed = |x, y| under(Pos2::new(x, y), &tiles, PROJECTION).copied();

        assert_eq!(pointed(204.0, 146.0), Some(heath));
        assert_eq!(pointed(196.0, 146.0), Some(beach));
        assert_eq!(pointed(204.0, 152.0), None);
        assert_eq!(describe(&heath), "Bilberry heath on till");
        assert_eq!(describe(&beach), "Gravel");
    }

    #[test]
    fn every_kind_of_ground_the_forest_has_is_its_own_color() {
        let sites = [
            (GroundType::Till, Some(VegetationKind::BilberryHeath)),
            (GroundType::Till, Some(VegetationKind::LingonberryHeath)),
            (GroundType::Till, Some(VegetationKind::HerbRichHeath)),
            (GroundType::Sand, Some(VegetationKind::LichenHeath)),
            (GroundType::Peat, Some(VegetationKind::PineBog)),
            (GroundType::Mud, Some(VegetationKind::ShoreMeadow)),
            (GroundType::Gravel, None),
            (GroundType::Sand, None),
        ];

        for visuals in [egui::Visuals::dark(), egui::Visuals::light()] {
            let unseen = Palette::of(&visuals).ground;
            let mut colors: Vec<Color32> = sites
                .iter()
                .map(|(ground, vegetation)| color(*ground, *vegetation, visuals.dark_mode))
                .chain([unseen])
                .collect();
            colors.sort_unstable_by_key(Color32::to_array);
            colors.dedup();

            assert_eq!(colors.len(), sites.len() + 1);
        }
    }
}
