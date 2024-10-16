use crate::helpers::geom::Triangles2d;

#[derive(Debug)]
pub struct CSector {
    pub triangles: Vec<Triangles2d>,

    pub floor_height: i16,
    pub ceiling_height: i16,

    pub light_level: i16,

    pub special_type: u16,
    pub sector_tag: u16,
}
