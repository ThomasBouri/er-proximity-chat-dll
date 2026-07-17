mod logging;
mod er;

use er::playerstate::{get_player_position, get_player_stats};
use er::coords::globalize;

use std::thread;
use std::time::Duration;

use eldenring::{
    cs::{CSTaskGroupIndex, CSTaskImp},
    fd4::FD4TaskData,
};

use fromsoftware_shared::SharedTaskImpExt;


#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn DllMain(_hmodule: u64, reason: u32) -> bool {
    const DLL_PROCESS_ATTACH: u32 = 1;

    if reason == DLL_PROCESS_ATTACH {
        let _ = thread::spawn(|| {main_thread(); });
    }
    true
}

fn main_thread() {
    logging::init();
    let cs_task = CSTaskImp::wait_for_instance(Duration::MAX).unwrap();
    cs_task.run_recurring(
        |_: &FD4TaskData | {
            let Some(pos) = (unsafe { get_player_position() }) else { return; };
            let Some(_global) = globalize(&pos) else {return;};
            let Some(_stats) = (unsafe {get_player_stats()}) else { return; };
        }, CSTaskGroupIndex::ChrIns_PostPhysics
    );
}