use eldenring::cs::CSEventFlagMan;
use fromsoftware_shared::FromStatic;

pub unsafe fn get_flag(flag_id: u32) -> Option<bool> {
    let Ok(efm) = (unsafe {CSEventFlagMan::instance()}) else { return None };
    Some(efm.virtual_memory_flag.get_flag(flag_id))
}

