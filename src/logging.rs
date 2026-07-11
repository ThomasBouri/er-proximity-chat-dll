use windows_sys::Win32::System::Console::AllocConsole;
use simplelog::{CombinedLogger, TermLogger, WriteLogger, Config, LevelFilter, TerminalMode, ColorChoice};
use std::fs::File;
use log::{info, error};

pub fn init() {
    // 1) create console window
    unsafe { let _ = AllocConsole(); }

    // 2) wire log output to console + file
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("mod_debug.log").unwrap()),
    ]).unwrap();

    // 3) arm the panic hook
    std::panic::set_hook(Box::new(|info| { error!("panic: {}", info); }));

    // 4) write a confirmation line
    info!("Logging Initialized");
}