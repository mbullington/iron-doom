use anyhow::Result;
use id_map_format::Wad;
use ultraviolet::{Vec2, Vec3};

use crate::{
    cvars::{CVarsMap, DEFAULT_CVARS},
    entities::{init_sector_entities, init_wall_entities},
    helpers::Camera,
    Stopwatch,
};

pub struct World {
    // Actual game state is maintained in the ECS "world".
    pub world: hecs::World,

    pub wad: Wad,
    pub map: id_map_format::Map,

    pub player_start: Vec2,
    pub camera: Camera,

    pub cvars: CVarsMap,
}

impl World {
    pub fn new(wad: Wad, map_name: &str) -> Result<Self> {
        let map = wad.parse_map(map_name)?;
        let player_start = match map.things.iter().find(|thing| thing.thing_type == 1) {
            Some(thing) => Vec2::new(thing.x as f32, thing.y as f32),
            None => panic!("No player start found!"),
        };

        let camera = Camera {
            pos: Vec3 {
                x: player_start.x,
                y: 16.,
                z: player_start.y,
            },
            ..Default::default()
        };

        let mut world = hecs::World::new();

        // Time how long it takes to spawn the entities.
        let mut stopwatch = Stopwatch::new();

        // Add walls to the world.
        init_wall_entities(&mut world, &map);
        // Add sectors to the world.
        init_sector_entities(&mut world, &map);

        let setup_time = stopwatch.lap();

        println!("Added {} entities to the world.", world.len());
        println!("Setup time: {:?}", setup_time);

        Ok(Self {
            world,

            wad,
            map,

            player_start,
            camera,

            cvars: DEFAULT_CVARS.iter().copied().collect::<CVarsMap>(),
        })
    }
}
