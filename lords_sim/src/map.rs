use crate::memory::Memory;
use crate::perception::{SIGHT_LIMIT, Seen};
use crate::space::{Length, Location};
use crate::time::Timestamp;
use crate::world::index::Grid;
use crate::world::{Area, AreaId, GroundType, Kind, Subject, Vegetation, VegetationKind, World};

/// How far the map reaches to the north, east, south, and west of the character.
pub const RANGE: Length = SIGHT_LIMIT;

/// What a character sees or remembers in the square that reaches [`RANGE`] each way from
/// where it stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Map {
    pub things: Vec<Mark>,
    pub waters: Vec<Water>,
    pub ground: Vec<Tile>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    pub subject: Subject,
    pub kind: Kind,
    pub location: Location,
    /// The radius of the ground the thing stands on.
    pub footprint: Length,
    /// When the thing was last seen, or `None` if it is in sight.
    pub last_seen: Option<Timestamp>,
}

impl Mark {
    pub fn in_sight(&self) -> bool {
        self.last_seen.is_none()
    }
}

/// A patch of ground that has been seen.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tile {
    /// The south-west corner of the patch.
    pub corner: Location,
    pub ground_type: GroundType,
    pub vegetation: Option<VegetationKind>,
}

/// The known part of a body of water.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Water {
    pub subject: Subject,
    pub kind: Kind,
    pub shore: Vec<[Location; 2]>,
    pub surface: Vec<[Location; 3]>,
}

/// The triangles that cover an area between the stretches of its shore that are known, cut
/// straight across where the shore is not.
#[derive(Clone, Debug, Default)]
pub(crate) struct Surface {
    corners: Vec<u32>,
    triangles: Vec<[u32; 3]>,
}

impl Surface {
    /// Works the triangles out again if `memory` knows more of the shore than it did, and
    /// returns whether it did.
    pub(crate) fn update(&mut self, id: AreaId, area: &Area, memory: &Memory) -> bool {
        let corners: Vec<u32> = (0..area.outline().len() as u32)
            .filter(|corner| {
                memory.knows_edge(id, *corner) || memory.knows_edge(id, area.edge_before(*corner))
            })
            .collect();

        let changed = corners != self.corners;
        if changed {
            self.triangles = area.triangles(&corners);
            self.corners = corners;
        }
        changed
    }

    /// Returns `area` with only the corners that are known.
    pub(crate) fn shape(&self, area: &Area) -> Area {
        let outline = area.outline();
        let corners = self
            .corners
            .iter()
            .map(|corner| outline[*corner as usize])
            .collect();

        Area::new(area.kind(), corners)
    }
}

/// Returns the map of a character at `eye` who remembers `memory` and now sees `seen`.
///
/// `surfaces` are those of the areas of `world`, in the order of the areas.
pub(crate) fn chart(
    world: &World,
    grid: &Grid,
    surfaces: &[Surface],
    memory: &Memory,
    seen: &Seen,
    eye: Location,
) -> Map {
    let in_range = |point: &Location| {
        let (east, north) = eye.offset_to(*point);
        east.abs().max(north.abs()) <= i128::from(RANGE.as_millimeters())
    };

    let things = grid
        .around(eye, RANGE)
        .filter_map(|id| {
            let thing = world.thing(id)?;
            let last_seen = memory.last_seen(id)?;
            if !in_range(&thing.location()) {
                return None;
            }
            let in_sight = seen.things.binary_search(&id).is_ok();

            Some(Mark {
                subject: Subject::Thing(id),
                kind: thing.kind(),
                location: thing.location(),
                footprint: thing.footprint(),
                last_seen: (!in_sight).then_some(last_seen),
            })
        })
        .collect();

    let waters = world
        .areas()
        .zip(surfaces)
        .filter_map(|((id, area), surface)| {
            let subject = Subject::Area(id);
            let outline = area.outline();
            let point = |corner: u32| outline[corner as usize];

            let shore: Vec<[Location; 2]> = (0..outline.len() as u32)
                .filter(|edge| memory.knows_edge(id, *edge))
                .map(|edge| [point(edge), point((edge + 1) % outline.len() as u32)])
                .filter(|ends| ends.iter().any(in_range))
                .collect();
            let surface = surface
                .triangles
                .iter()
                .map(|corners| corners.map(point))
                .filter(|corners| corners.iter().any(in_range))
                .collect();

            (!shore.is_empty()).then_some(Water {
                subject,
                kind: world.kind(subject)?,
                shore,
                surface,
            })
        })
        .collect();

    Map {
        things,
        waters,
        ground: ground(world, memory, eye),
    }
}

/// Returns the patches of the terrain of `world` in range of `eye` that `memory` has seen.
fn ground(world: &World, memory: &Memory, eye: Location) -> Vec<Tile> {
    let terrain = world.terrain();

    terrain
        .around(eye, RANGE)
        .filter(|(index, _)| memory.knows_patch(*index))
        .map(|(index, patch)| Tile {
            corner: terrain.corner_of(index),
            ground_type: patch.ground_type(),
            vegetation: patch.vegetation().map(Vegetation::kind),
        })
        .collect()
}
