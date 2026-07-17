//! Converts a block-relative PlayerPosition to a comparable world coordinate

use eldenring::cs::BlockId;
use serde::{Deserialize, Serialize};
use crate::er::playerstate::PlayerPosition;

use log::{info, warn};

use std::sync::atomic::{AtomicU32, Ordering};
static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);

use std::sync::OnceLock;
use csv::Reader;

const TILE_SIZE: f32 = 256.0;
const MAIN_WORLD_ID: u8 = 60;
const DLC_WORLD_ID: u8 = 61;
const MAP_CONV_CSV: &str = include_str!("../../assets/WorldMapLegacyConvParamTrim.csv");
static MAP_CONV: OnceLock<Vec<MapConvEntry>> = OnceLock::new();


#[derive(Clone, Serialize, Deserialize)]
pub struct WorldPos {
    pub block_id: i32,
    pub bp_x: f32, pub bp_y: f32, pub bp_z: f32,
    pub global: Option<(f32, f32, f32)>,
}

pub fn globalize(pos: &PlayerPosition) -> Option<WorldPos> { 
    // section 1 - getting block_id, and checking if loading frame (m255_255_255_255)
    if pos.block_id == -1 { return None; }
    let block_id = BlockId::from(pos.block_id);
    let block_x = block_id.block();
    let block_z = block_id.region();


    //section 2 - overworld logic
    let global = if block_id.area() == MAIN_WORLD_ID || block_id.area() == DLC_WORLD_ID {
        Some((
            pos.bp_x + (block_x as f32)*TILE_SIZE,
            pos.bp_y,
            pos.bp_z + (block_z as f32)*TILE_SIZE,
        ))
    } else { 
        match find_map_conv_entry(block_id) {
        Some(e) => {
            let offset_x = pos.bp_x - e.src_pos_x;
            let offset_y = pos.bp_y - e.src_pos_y;
            let offset_z = pos.bp_z - e.src_pos_z;
            let target_x = e.dst_pos_x + offset_x;
            let target_y = e.dst_pos_y + offset_y;
            let target_z = e.dst_pos_z + offset_z;
        Some((
            target_x + e.dst_grid_x_no as f32 * TILE_SIZE,
            target_y,
            target_z + e.dst_grid_z_no as f32 * TILE_SIZE,
        ))
    }
    None => None,   // un-globalizable — but we still return a WorldPos below
        }
    };



    // section 4 - testing
    let n: u32 = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
    if n % 60 == 0 {
        match global {
            Some((x, y, z)) => info!("Global: {}, {}, {} | block: {}", x, y, z, block_id),
            None => warn!("NO GLOBAL for block {} | bp: {}, {}, {}", block_id, pos.bp_x, pos.bp_y, pos.bp_z),
    }
}
    Some(WorldPos { block_id: (pos.block_id), bp_x: (pos.bp_x), bp_y: (pos.bp_y), bp_z: (pos.bp_z), global })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MapConvEntry {
    src_area_no: u32, src_grid_x_no: u32, src_grid_z_no: u32, 
    src_pos_x: f32, src_pos_y: f32, src_pos_z: f32, 
    dst_area_no: u32, dst_grid_x_no: u32, dst_grid_z_no: u32, 
    dst_pos_x: f32, dst_pos_y: f32, dst_pos_z: f32, 
}


fn map_conv_table() -> &'static Vec<MapConvEntry> {
    MAP_CONV.get_or_init(|| {
        let mut rdr = Reader::from_reader(MAP_CONV_CSV.as_bytes());
        rdr.deserialize()
            .collect::<Result<Vec<MapConvEntry>, csv::Error>>()
            .expect("failed to parse WorldMapLegacyConvParam CSV")
    })
}

/// Finds the WorldMapLegacyConvParam row that converts an interior's local
/// coordinates into the overworld frame. Returns None if no direct conversion exists.
///
/// KNOWN GAP — this is single-hop by design. We only match rows whose destination
/// is the overworld (60) or DLC (61). Nine maps in the CSV chain through *other*
/// dungeons rather than reaching a world directly, e.g.
///     m12_03 (Deeproot Depths) -> m35_00 (Shunning-Grounds) -> m11_00 (Leyndell) -> overworld
/// so they return None here. Same for any map absent from the CSV entirely — it
/// only covers 98 of the game's maps.
///
/// This is deliberate, not a bug. Callers get `global: None` but still receive a
/// WorldPos carrying block_id + raw block_position, so the server falls back to
/// same-block local comparison. For sealed underground areas that IS the correct
/// behaviour — someone in Deeproot shouldn't hear someone in Limgrave. Resolving
/// the chain would mean recursively composing transforms, which buys nothing here.
///
/// Second quirk: ~10 maps (Farum Azula, Haligtree, m34_15) have multiple rows that
/// globalize to different points, disagreeing by thousands of units. `.find()` takes
/// the first match deterministically, so every client lands in the *same* frame and
/// relative distances within the map stay correct — only the absolute placement is
/// arbitrary, which is moot for self-contained late-game dungeons.

fn find_map_conv_entry(block_id: BlockId) -> Option<&'static MapConvEntry> {
    map_conv_table()
        .iter()
        .find(|e| e.src_area_no == block_id.area() as u32 
        && e.src_grid_x_no == block_id.block() as u32
        && e.src_grid_z_no == block_id.region() as u32 
        && (e.dst_area_no == MAIN_WORLD_ID as u32 || e.dst_area_no == DLC_WORLD_ID as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_loads() {
        assert_eq!(map_conv_table().len(), 196);
    }
    
    #[test]
    fn stormfoot_maps_to_correct_tile() {
        let e = find_map_conv_entry(BlockId::from_parts(30, 2, 0, 0))
            .expect("Stormfoot entry should exist");
        assert_eq!(e.dst_area_no, 60);
        assert_eq!(e.dst_grid_x_no, 41);
        assert_eq!(e.dst_grid_z_no, 37);
    }

    #[test]
    fn area_35_has_no_overworld_entry() {
        assert!(find_map_conv_entry(BlockId::from_parts(35, 0, 0, 0)).is_none());
    }

    #[test]
fn ungloblizable_map_still_returns_position() {
    let pos = PlayerPosition {
        block_id: i32::from(BlockId::from_parts(35, 0, 0, 0)),
        bp_x: 1.0, bp_y: 2.0, bp_z: 3.0,
        /* ..any other fields.. */
    };
    let w = globalize(&pos).expect("should still return a WorldPos");
    assert!(w.global.is_none());
    assert_eq!(w.bp_x, 1.0);   // local data survives
}
}