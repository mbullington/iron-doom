use bitflags::bitflags;
use ultraviolet::Vec2;

bitflags! {
    #[derive(Debug)]
    pub struct CWallType: u32 {
        const Upper = 0b00000001;
        const Middle = 0b00000010;
        const Lower = 0b00000100;
    }
}

#[derive(Debug)]
pub struct CWall {
    pub wall_type: CWallType,

    pub start_vert: Vec2,
    pub end_vert: Vec2,

    pub sector_index: usize,

    pub x_offset: i16,
    pub y_offset: i16,
}

#[derive(Debug)]
pub struct CWallTwoSided {
    pub back_sector_index: usize,
}
