    //! Return the local player's live state from the game every frame.

    use eldenring::cs::{GameDataMan, WorldChrMan};
    use fromsoftware_shared::FromStatic;
    use eldenring;

    use serde::Serialize;

    use log::info;
    use std::sync::atomic::{AtomicU32, Ordering};
    static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);



    // The Data Struct
    #[derive(Clone, Serialize)]
    pub struct PlayerPosition{
        pub block_id: i32, // i32 as otherwise we couldn't serialize it
        pub bp_x: f32,
        pub bp_y: f32,
        pub bp_z: f32,
    }
    // The reading
    // A function to exist within the PostPhysics task.
    // 1. get WorldChrMan; bail (return None) if not ready
    // 2. get main_player from it; bail if not present
    // 3. get values, all from  

    pub unsafe fn get_player_position() -> Option<PlayerPosition> {
        let Ok(wcm) = (unsafe {WorldChrMan::instance() }) else {return None};
        let Some(player) = wcm.main_player.as_ref() else { return None};

        // by doing the following, we get rid of the convenient BlockId methods. but in coords.rs we can
        // use the fact that the crate has: From<i32> for BlockId --> BlockId::from(state.block_origin)
        let block_id = player.current_block_id;
        let block_position = player.block_position;
        
        let bp_x = block_position.x;
        let bp_y = block_position.y;
        let bp_z = block_position.z;
        
        Some(PlayerPosition{block_id:block_id.into(), bp_x, bp_y, bp_z})
    }


    #[derive(Clone, Serialize)]
    pub struct PlayerStats{
        pub death_count: u32,
        pub igt: u32,
    }

    pub unsafe fn get_player_stats() -> Option<PlayerStats> {
        let Ok(gdm) = (unsafe {GameDataMan::instance() }) else {return None};
        let death_count = gdm.death_count;
        let igt = gdm.play_time;

        let n: u32 = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
        if n % 60 == 0 { info!("Death Count: {}| IGT: {}", death_count, igt); }
        Some(PlayerStats{death_count, igt})
    }