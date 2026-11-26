use egui::RichText;
use lords_sim::object::{Activity, Living};
use lords_sim::view::Ground;
use lords_sim::world::Abundance;
use lords_sim::{Command, Human, Item};

use crate::scenes::overview::meter;
use crate::scenes::overview::words::{
    abundance, capitalized, gather_order, ground_type_description, ground_type_name, species_name,
    vegetation_name,
};

use super::{ROW_SPACING, SECTION_SPACING};

const NO_ROOM: &str = "No room to carry more";
const NONE: &str = "None to pick here";

pub fn show(ui: &mut egui::Ui, ground: Option<&Ground>, player: &Human) -> Option<Command> {
    let mut command = None;
    let mut gather = |ui: &mut egui::Ui, label: String, item: Item, gone: bool| {
        if gather_button(ui, label, item, player, gone).clicked() {
            command = Some(Command::Gather(item));
        }
    };
    ui.label(RichText::new("Ground").strong());

    let Some(ground) = ground else {
        ui.label(RichText::new("No ground").weak());
        ui.add_space(SECTION_SPACING);
        return None;
    };

    egui::Grid::new("ground")
        .num_columns(5)
        .min_row_height(ui.spacing().interact_size.y)
        .spacing(ROW_SPACING)
        .show(ui, |ui| {
            ui.label(RichText::new("Type").weak());
            ui.label(ground_type_name(ground.ground_type))
                .on_hover_text(ground_type_description(ground.ground_type));
            ui.end_row();

            ui.label(RichText::new("Vegetation").weak());
            let Some(vegetation) = ground.vegetation else {
                ui.label(RichText::new("None").weak());
                ui.end_row();
                return;
            };
            ui.label(vegetation_name(vegetation));
            ui.end_row();
            for plant in &ground.plants {
                let gone = plant.crop == Abundance::Gone;
                ui.label("");
                ui.label(capitalized(species_name(plant.species)));
                meter::show(ui, plant.share);
                ui.label(RichText::new(abundance(plant.crop)).weak());
                gather(ui, "Pick".to_owned(), Item::Picked(plant.species), gone);
                ui.end_row();
            }
        });

    gather(ui, gather_order(Item::Pebble), Item::Pebble, false);
    ui.add_space(SECTION_SPACING);

    command
}

fn gather_button(
    ui: &mut egui::Ui,
    label: String,
    item: Item,
    player: &Human,
    gone: bool,
) -> egui::Response {
    let gathering = matches!(
        player.activity(),
        Some(Activity::Gathering { item: gathered, .. }) if gathered == item
    );
    let room = player.has_room_for(item);
    let hint = if gone {
        NONE.to_owned()
    } else if room {
        gather_order(item)
    } else {
        NO_ROOM.to_owned()
    };

    ui.add_enabled(room && !gone && !gathering, egui::Button::new(label))
        .on_hover_text(&hint)
        .on_disabled_hover_text(&hint)
}

#[cfg(test)]
mod tests {
    use lords_sim::view::Growth;
    use lords_sim::world::{GroundType, Patch, Species, Terrain, VegetationKind, World};
    use lords_sim::{Location, Name, Sex, Simulation, State};

    use super::*;
    use crate::scenes::harness::Harness;

    #[test]
    fn ground_draws_with_and_without_a_patch_or_vegetation() {
        let harness = Harness::new();
        let growth = |species, crop| Growth {
            species,
            share: 50,
            crop,
        };
        let heath = Ground {
            ground_type: GroundType::Till,
            vegetation: Some(VegetationKind::LingonberryHeath),
            plants: vec![
                growth(Species::Lingonberry, Abundance::Plenty),
                growth(Species::FeatherMoss, Abundance::Gone),
            ],
        };
        let beach = Ground {
            ground_type: GroundType::Gravel,
            vegetation: None,
            plants: Vec::new(),
        };
        let mut world = World::default();
        let gravel = Patch::new(GroundType::Gravel, None, 4_000);
        world.set_terrain(Terrain::new(Location::new(-2_000, -2_000), 1, [gravel]));
        let mut sim = Simulation::from_state(State {
            world,
            ..State::new(Name::default(), Sex::Female)
        });
        let idle = sim.view().player;
        sim.apply(Command::Gather(Item::Pebble));
        let searching = sim.view().player;

        harness.show(|ui| assert_eq!(show(ui, None, &idle), None));
        harness.show(|ui| assert_eq!(show(ui, Some(&heath), &idle), None));
        harness.show(|ui| assert_eq!(show(ui, Some(&beach), &searching), None));
    }
}
