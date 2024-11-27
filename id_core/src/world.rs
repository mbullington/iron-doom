use anyhow::Result;
use bvh_arena::{volumes::Aabb, Bvh};
use id_map_format::Wad;
use ultraviolet::Vec2;

use crate::{
    components::{CSector, CWorldPos},
    cvars::{CVarsMap, DEFAULT_CVARS},
    entities::{init_player_entities, init_sector_entities, init_wall_entities},
    helpers::geom::Bounds2d,
    Stopwatch,
};

pub struct World {
    /// Actual game state is maintained in the ECS "world".
    pub world: hecs::World,
    /// Acceleration structure for looking up sectors by their bounding boxes.
    pub sector_accel: SectorAccel,

    pub player: hecs::Entity,

    pub wad: Wad,
    pub map: id_map_format::Map,

    pub cvars: CVarsMap,
}

impl World {
    pub fn new(wad: Wad, map_name: &str) -> Result<Self> {
        let map = wad.parse_map(map_name)?;

        let mut world = hecs::World::new();

        // Time how long it takes to spawn the entities.
        let mut stopwatch = Stopwatch::new();

        // Add walls to the world.
        init_wall_entities(&mut world, &map);
        // Add sectors to the world.
        init_sector_entities(&mut world, &map);

        // Build acceleration structure for sectors.
        let sector_accel = SectorAccel::new(&world);

        // Add entities to the world.
        // Requires we've already initialized sector accel.
        let player = init_player_entities(&mut world, &sector_accel, &map)?;

        let setup_time = stopwatch.lap();

        println!("Added {} entities to the world.", world.len());
        println!("Setup time: {:?}", setup_time);

        Ok(Self {
            world,
            sector_accel,
            player,

            wad,
            map,

            cvars: DEFAULT_CVARS.iter().copied().collect::<CVarsMap>(),
        })
    }

    pub fn with_player_pos<RT, F: FnOnce(&mut CWorldPos) -> RT>(
        &mut self,
        callback: F,
    ) -> Result<RT> {
        let player_pos = self.world.query_one_mut::<&mut CWorldPos>(self.player)?;
        Ok(callback(player_pos))
    }
}

pub struct SectorAccel {
    bvh: Bvh<hecs::Entity, Aabb<2>>,
}

impl SectorAccel {
    pub fn new(world: &hecs::World) -> Self {
        let mut bvh = Bvh::default();
        for (id, sector) in &mut world.query::<&CSector>() {
            let bbox = Bounds2d::from_iter(sector.triangles.iter().map(|tri| tri.bbox()));
            if bbox.min.x > bbox.max.x || bbox.min.y > bbox.max.y {
                eprintln!("Sector {} has invalid bounding box", id.id());
            } else {
                let aabb = Aabb::from_min_max(bbox.min, bbox.max);
                bvh.insert(id, aabb);
            }
        }
        Self { bvh }
    }

    pub fn query(&self, world: &hecs::World, point_xz: Vec2) -> Option<hecs::Entity> {
        let mut found_sector: Option<hecs::Entity> = None;
        self.bvh
            .for_each_overlaps(&Aabb::from_min_max(point_xz, point_xz), |sector| {
                if found_sector.is_some() {
                    return;
                }

                let candidate_sector = world.get::<&CSector>(*sector).unwrap();
                for triangle in &candidate_sector.triangles {
                    if triangle.has_point(point_xz) {
                        found_sector = Some(*sector);
                        break;
                    }
                }
            });

        found_sector
    }
}
