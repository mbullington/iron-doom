use bvh_arena::{volumes::Aabb, Bvh};
use ultraviolet::Vec2;

use crate::{components::CSector, helpers::geom::Bounds2d};

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
