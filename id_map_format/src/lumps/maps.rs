use crate::{helpers::parse_bytes_cstr, lump_from_namespace, LumpNamespace, Wad, WadError};

#[derive(Debug)]
pub struct Thing {
    pub x: i16,
    pub y: i16,
    /// In degrees. Counter-clockwise from east.
    pub angle: u16,

    pub thing_type: u16,
    pub spawn_flags: u16,
}

#[derive(Debug)]
pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,

    pub floor_flat: String,
    pub ceiling_flat: String,

    pub light_level: i16,

    pub special_type: u16,
    pub sector_tag: u16,
}

#[derive(Debug)]
pub struct Sidedef {
    pub x_offset: i16,
    pub y_offset: i16,

    pub upper_texture: String,
    pub lower_texture: String,
    pub middle_texture: String,

    pub sector_idx: u16,
}

#[derive(Debug)]
pub struct Linedef {
    pub start_vertex_idx: u16,
    pub end_vertex_idx: u16,

    /// Flags are game (and engine) dependent.
    pub flags: u16,

    pub line_type: u16,
    pub sector_tag: u16,

    pub right_sidedef_idx: Option<u16>,
    pub left_sidedef_idx: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug)]
pub struct Map {
    pub name: String,

    /// "Things" indicate monsters, items, etc.
    pub things: Vec<Thing>,

    /// Sectors are geometry-agnostic, and defined with floor/ceiling height.
    pub sectors: Vec<Sector>,
    /// Sidedefs are keyed on their sector, and provide a geometry-agnostic
    /// texture name for the floor/ceiling.
    pub sidedefs: Vec<Sidedef>,
    /// Linedefs are keyed on their sidedefs, and connect a front/back face
    /// sidedef to a start/end vertex.
    pub linedefs: Vec<Linedef>,
    /// Vertices are just points in X,Y space.
    pub vertices: Vec<Vertex>,
}

impl Wad {
    pub fn parse_map(&self, map_name: &str) -> Result<Map, WadError> {
        let namespace = LumpNamespace::Map(map_name.to_string().clone());
        if !self.lump_namespaces.contains_key(&namespace) {
            return Err(WadError::MapDoesNotExist(map_name.to_string()));
        }

        // Parse THINGS.
        let mut things: Vec<Thing> = Vec::new();
        {
            let lump = lump_from_namespace(&namespace, "THINGS", self)?;
            let lump_bytes = lump.bytes();

            if lump.size % 10 != 0 {
                return Err(WadError::CorruptedLump(lump.name.clone()));
            }

            let num_things = lump.size / 10;
            things.reserve_exact(num_things);

            for i in 0..num_things {
                let thing_offset = i * 10;
                let thing_bytes = &lump_bytes[thing_offset..thing_offset + 10];

                let x = i16_le!(&thing_bytes[0..2]);
                let y = i16_le!(&thing_bytes[2..4]);

                let angle = u16_le!(&thing_bytes[4..6]);

                let thing_type = u16_le!(&thing_bytes[6..8]);
                let spawn_flags = u16_le!(&thing_bytes[8..10]);

                things.push(Thing {
                    x,
                    y,
                    angle,
                    thing_type,
                    spawn_flags,
                });
            }
        }

        // Parse SECTORS.
        let mut sectors: Vec<Sector> = Vec::new();
        {
            let lump = lump_from_namespace(&namespace, "SECTORS", self)?;
            let lump_bytes = lump.bytes();

            if lump.size % 26 != 0 {
                return Err(WadError::CorruptedLump(lump.name.clone()));
            }

            let num_sectors = lump.size / 26;
            sectors.reserve_exact(num_sectors);

            for i in 0..num_sectors {
                let sector_offset = i * 26;
                let sector_bytes = &lump_bytes[sector_offset..sector_offset + 26];

                let floor_height = i16_le!(&sector_bytes[0..2]);
                let ceiling_height = i16_le!(&sector_bytes[2..4]);

                let floor_flat = match parse_bytes_cstr(&sector_bytes[4..12]) {
                    Ok(floor_flat) => match floor_flat.to_str() {
                        Ok(floor_flat) => floor_flat.to_string(),
                        Err(_) => return Err(WadError::CorruptedString),
                    },
                    Err(_) => return Err(WadError::CorruptedBytes),
                };

                let ceiling_flat = match parse_bytes_cstr(&sector_bytes[12..20]) {
                    Ok(ceiling_flat) => match ceiling_flat.to_str() {
                        Ok(ceiling_flat) => ceiling_flat.to_string(),
                        Err(_) => return Err(WadError::CorruptedString),
                    },
                    Err(_) => return Err(WadError::CorruptedBytes),
                };

                let light_level = i16_le!(&sector_bytes[20..22]);

                let special_type = u16_le!(&sector_bytes[22..24]);
                let sector_tag = u16_le!(&sector_bytes[24..26]);

                sectors.push(Sector {
                    floor_height,
                    ceiling_height,
                    floor_flat,
                    ceiling_flat,
                    light_level,
                    special_type,
                    sector_tag,
                });
            }
        }

        // Parse SIDEDEFS.
        let mut sidedefs: Vec<Sidedef> = Vec::new();
        {
            let lump = lump_from_namespace(&namespace, "SIDEDEFS", self)?;
            let lump_bytes = lump.bytes();

            if lump.size % 30 != 0 {
                return Err(WadError::CorruptedLump(lump.name.clone()));
            }

            let num_sidedefs = lump.size / 30;
            sidedefs.reserve_exact(num_sidedefs);

            for i in 0..num_sidedefs {
                let sidedef_offset = i * 30;
                let sidedef_bytes = &lump_bytes[sidedef_offset..sidedef_offset + 30];

                let x_offset = i16_le!(&sidedef_bytes[0..2]);
                let y_offset = i16_le!(&sidedef_bytes[2..4]);

                let upper_texture = match parse_bytes_cstr(&sidedef_bytes[4..12]) {
                    Ok(upper_texture) => match upper_texture.to_str() {
                        Ok(upper_texture) => upper_texture.to_string(),
                        Err(_) => return Err(WadError::CorruptedString),
                    },
                    Err(_) => return Err(WadError::CorruptedBytes),
                };

                let lower_texture = match parse_bytes_cstr(&sidedef_bytes[12..20]) {
                    Ok(lower_texture) => match lower_texture.to_str() {
                        Ok(lower_texture) => lower_texture.to_string(),
                        Err(_) => return Err(WadError::CorruptedString),
                    },
                    Err(_) => return Err(WadError::CorruptedBytes),
                };

                let middle_texture = match parse_bytes_cstr(&sidedef_bytes[20..28]) {
                    Ok(middle_texture) => match middle_texture.to_str() {
                        Ok(middle_texture) => middle_texture.to_string(),
                        Err(_) => return Err(WadError::CorruptedString),
                    },
                    Err(_) => return Err(WadError::CorruptedBytes),
                };

                let sector_idx = u16_le!(&sidedef_bytes[28..30]);

                sidedefs.push(Sidedef {
                    x_offset,
                    y_offset,
                    upper_texture,
                    lower_texture,
                    middle_texture,
                    sector_idx,
                });
            }
        }

        // Parse LINEDEFS.
        let mut linedefs: Vec<Linedef> = Vec::new();
        {
            let lump = lump_from_namespace(&namespace, "LINEDEFS", self)?;
            let lump_bytes = lump.bytes();

            if lump.size % 14 != 0 {
                return Err(WadError::CorruptedLump(lump.name.clone()));
            }

            let num_linedefs = lump.size / 14;
            linedefs.reserve_exact(num_linedefs);

            for i in 0..num_linedefs {
                let linedef_offset = i * 14;
                let linedef_bytes = &lump_bytes[linedef_offset..linedef_offset + 14];

                let start_vertex_idx = u16_le!(&linedef_bytes[0..2]);
                let end_vertex_idx = u16_le!(&linedef_bytes[2..4]);

                let flags = u16_le!(&linedef_bytes[4..6]);

                let line_type = u16_le!(&linedef_bytes[6..8]);
                let sector_tag = u16_le!(&linedef_bytes[8..10]);

                let right_sidedef_idx = match u16_le!(&linedef_bytes[10..12]) {
                    0xFFFF => None,
                    idx => Some(idx),
                };
                let left_sidedef_idx = match u16_le!(&linedef_bytes[12..14]) {
                    0xFFFF => None,
                    idx => Some(idx),
                };

                linedefs.push(Linedef {
                    start_vertex_idx,
                    end_vertex_idx,
                    flags,
                    line_type,
                    sector_tag,
                    right_sidedef_idx,
                    left_sidedef_idx,
                });
            }
        }

        // Parse VERTEXES.
        let mut vertices: Vec<Vertex> = Vec::new();
        {
            let lump = lump_from_namespace(&namespace, "VERTEXES", self)?;
            let lump_bytes = lump.bytes();

            if lump.size % 4 != 0 {
                return Err(WadError::CorruptedLump(lump.name.clone()));
            }

            let num_vertices = lump.size / 4;
            vertices.reserve_exact(num_vertices);

            for i in 0..num_vertices {
                let vertex_offset = i * 4;
                let vertex_bytes = &lump_bytes[vertex_offset..vertex_offset + 4];

                let x = i16_le!(&vertex_bytes[0..2]);
                let y = i16_le!(&vertex_bytes[2..4]);

                vertices.push(Vertex { x, y });
            }
        }

        Ok(Map {
            name: map_name.to_string().clone(),
            things,
            sectors,
            sidedefs,
            linedefs,
            vertices,
        })
    }
}
