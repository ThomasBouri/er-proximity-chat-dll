//! Return the local player's live state from the game every frame.

use eldenring::cs::{GameDataMan, WorldChrMan, BlockId};
use fromsoftware_shared::FromStatic;

use serde::Serialize;

use log::info;

// The Data Struct
// For now, position + map_id --> Eventually, death_count + igt as well
#[derive(Clone, Serialize)]
pub struct PlayerState{
    pub position: [f32; 3],
    pub block_id: i32, // i32 as otherwise we couldn't serialize it

}
// The reading
// A function to exist within the PostPhysics task.
// 1. get WorldChrMan; bail (return None) if not ready
// 2. get main_player from it; bail if not present
// 3. read physics.position from the player
// 4. read the map_id from the player
// 5. pack both into the struct and return it > Will return Option<PlayerState>

pub unsafe fn get_player_state() -> Option<PlayerState> {
    let Ok(wcm) = (unsafe {WorldChrMan::instance() }) else {return None};
    let Some(player) = wcm.main_player.as_ref() else { return None};
    let module_container = &*player.chr_ins.modules;
    let physics = &*module_container.physics;
    let position = [physics.position.0, physics.position.1, physics.position.2];

    // by doing the following, we get rid of the convenient BlockId methods. but in coords.rs we can
    // use the fact that the crate has: From<i32> for BlockId --> BlockId::from(state.block_origin)
    let block_id = player.chr_ins.block_id;

    info!(
        "position {:?} | block_id: {} | block_id_override: {} | block_origin_override: {} | block_origin: {}",
        position,
        BlockId::from(player.chr_ins.block_id),
        BlockId::from(player.chr_ins.block_id_override),
        BlockId::from(player.chr_ins.block_origin_override),
        BlockId::from(player.chr_ins.block_origin),
    );


    Some(PlayerState{position, block_id:block_id.into()})
}
