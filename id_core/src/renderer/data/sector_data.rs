use std::collections::HashMap;

use id_map_format::{lump_from_namespace, Linedef, LumpNamespace, Map, Wad};

use anyhow::Result;
use encase::ShaderType;
use multimap::MultiMap;

use ultraviolet::Vec2;
use wgpu::BufferUsages;

use crate::{
    helpers::geom::{Graph2d, PolygonShape2d},
    renderer::helpers::gpu::{
        GpuIndexBuffer, GpuStorageBuffer, GpuU8StorageBuffer, GpuVertexBuffer,
    },
};

#[repr(C)]
#[derive(ShaderType)]
pub struct SectorVertexData {
    pub position: Vec2,
    /// This is the index in the sector storage buffer.
    pub storage_index: u32,
}

#[derive(ShaderType)]
pub struct SectorStorageData {
    pub floor_height: f32,
    pub ceiling_height: f32,

    // Indexes in the flat array.
    pub floor_flat_index: u32,
    pub ceiling_flat_index: u32,

    // Light level.
    pub light_level: u32,
}

pub struct SectorData {
    /// Can be used to draw either floors or ceilings.
    pub vertex_buf: GpuVertexBuffer<SectorVertexData>,
    /// Assumed to be wound with +y = CW.
    pub index_buf: GpuIndexBuffer,
    /// Stores auxiliary information about each sector.
    pub storage_buf: GpuStorageBuffer<SectorStorageData>,

    /// Stores "flat images"
    pub flat_storage_buf: GpuU8StorageBuffer,
}

impl SectorData {
    pub fn new(device: &wgpu::Device, wad: &Wad, map: &Map) -> Result<Self> {
        // Create the vertex and index buffers for section data.

        let mut sidedefs_by_sector: MultiMap<usize, usize> = MultiMap::new();
        for (i, sidedef) in map.sidedefs.iter().enumerate() {
            sidedefs_by_sector.insert(sidedef.sector_idx as usize, i);
        }

        let mut linedefs_by_sidedef: MultiMap<usize, &Linedef> = MultiMap::new();
        for linedef in map.linedefs.iter() {
            if let Some(sidedef_idx) = linedef.left_sidedef_idx {
                linedefs_by_sidedef.insert(sidedef_idx as usize, linedef);
            }
            if let Some(sidedef_idx) = linedef.right_sidedef_idx {
                linedefs_by_sidedef.insert(sidedef_idx as usize, linedef);
            }
        }

        let mut sector_vertexes: Vec<SectorVertexData> = Vec::new();
        let mut sector_indices: Vec<u32> = Vec::new();
        let mut sector_storage: Vec<SectorStorageData> = Vec::new();

        let mut flat_storage: Vec<u8> = Vec::new();
        let mut flat_storage_by_index: HashMap<String, usize> = HashMap::new();

        // Flat storage i=0 is reserved for empty.
        flat_storage.append(&mut vec![0u8; 4096]);

        // Flat storage i=1 is reserved for sky texture.
        if let Ok(lump) = lump_from_namespace(&LumpNamespace::Flat, "F_SKY1", wad) {
            flat_storage_by_index.insert("F_SKY1".to_string(), 4096);
            flat_storage.append(&mut lump.bytes().to_vec());

            // Fill the rest of the buffer with zeros, if it's not a multiple of 4096.
            let remaining = 4096 - lump.bytes().len();
            flat_storage.append(&mut vec![0u8; remaining]);
        }

        // Populate sectors.
        for (i, sector) in map.sectors.iter().enumerate() {
            let ceiling_height = sector.ceiling_height as f32;
            let floor_height = sector.floor_height as f32;

            let ceiling_flat_index = match flat_storage_by_index.get(&sector.ceiling_flat) {
                Some(idx) => *idx / 4096,
                None => {
                    let idx = flat_storage.len();
                    if let Ok(lump) =
                        lump_from_namespace(&LumpNamespace::Flat, &sector.ceiling_flat, wad)
                    {
                        flat_storage_by_index.insert(sector.ceiling_flat.clone(), idx);
                        flat_storage.append(&mut lump.bytes().to_vec());

                        // Fill the rest of the buffer with zeros, if it's not a multiple of 4096.
                        let remaining = 4096 - lump.bytes().len();
                        flat_storage.append(&mut vec![0u8; remaining]);

                        idx / 4096
                    } else {
                        0
                    }
                }
            };

            let floor_flat_index = match flat_storage_by_index.get(&sector.floor_flat) {
                Some(idx) => *idx / 4096,
                None => {
                    let idx = flat_storage.len();

                    match lump_from_namespace(&LumpNamespace::Flat, &sector.floor_flat, wad) {
                        Ok(lump) => {
                            flat_storage_by_index.insert(sector.floor_flat.clone(), idx);
                            flat_storage.append(&mut lump.bytes().to_vec());

                            // Fill the rest of the buffer with zeros, if it's not a multiple of 4096.
                            let remaining = 4096 - lump.bytes().len();
                            flat_storage.append(&mut vec![0u8; remaining]);

                            idx / 4096
                        }
                        Err(e) => {
                            println!("Error loading flat {}: {}", &sector.floor_flat, e);
                            0
                        }
                    }
                }
            };

            let storage_index = sector_storage.len();
            sector_storage.push(SectorStorageData {
                floor_height,
                ceiling_height,
                floor_flat_index: floor_flat_index as u32,
                ceiling_flat_index: ceiling_flat_index as u32,
                light_level: sector.light_level as u32,
            });

            let mut vertices: Vec<Vec2> = Vec::new();
            let mut edges: Vec<(usize, usize)> = Vec::new();

            let mut vert_wad_mapping: HashMap<id_map_format::Vertex, usize> = HashMap::new();

            let sidedefs = match sidedefs_by_sector.get_vec(&i) {
                Some(sidedefs) => sidedefs,
                None => continue,
            };

            for sidedef_idx in sidedefs {
                let linedefs = match linedefs_by_sidedef.get_vec(sidedef_idx) {
                    Some(linedefs) => linedefs,
                    None => continue,
                };

                for linedef in linedefs {
                    let start_vertex = map.vertices[linedef.start_vertex_idx as usize];
                    let start_vertex_idx = match vert_wad_mapping.get(&start_vertex) {
                        Some(idx) => *idx,
                        None => {
                            let idx = vertices.len();
                            vertices.push(Vec2::new(start_vertex.x as f32, start_vertex.y as f32));
                            vert_wad_mapping.insert(start_vertex, idx);
                            idx
                        }
                    };

                    let end_vertex = map.vertices[linedef.end_vertex_idx as usize];
                    let end_vertex_idx = match vert_wad_mapping.get(&end_vertex) {
                        Some(idx) => *idx,
                        None => {
                            let idx = vertices.len();
                            vertices.push(Vec2::new(end_vertex.x as f32, end_vertex.y as f32));
                            vert_wad_mapping.insert(end_vertex, idx);
                            idx
                        }
                    };

                    edges.push((start_vertex_idx, end_vertex_idx));
                }
            }

            let polygons = Graph2d::new(vertices, edges).detect_polygons();
            let polygon_shapes = PolygonShape2d::from_polygons(&polygons);

            for shape in polygon_shapes {
                let (vertices, indices) = match shape.tessellate() {
                    Ok((vertices, indices)) => (vertices, indices),
                    Err(e) => panic!("{}", e.to_string()),
                };

                let mut vec_data = vertices
                    .iter()
                    .map(|v| SectorVertexData {
                        position: *v,
                        storage_index: storage_index as u32,
                    })
                    .collect::<Vec<SectorVertexData>>();

                let offset = sector_vertexes.len() as u32;
                sector_vertexes.append(&mut vec_data);

                sector_indices.append(&mut indices.iter().map(|x| *x + offset).collect());
            }
        }

        Ok(Self {
            vertex_buf: GpuVertexBuffer::new_vec(
                BufferUsages::VERTEX,
                device,
                sector_vertexes,
                Some("SectorData::vertex_buf"),
            )?,
            index_buf: GpuIndexBuffer::new_vec(
                BufferUsages::INDEX,
                device,
                sector_indices,
                Some("SectorData::index_buf"),
            )?,
            storage_buf: GpuStorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                sector_storage,
                Some("SectorData::storage_buf"),
            )?,
            flat_storage_buf: GpuU8StorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                flat_storage,
                Some("SectorData::flat_storage_buf"),
            )?,
        })
    }
}
