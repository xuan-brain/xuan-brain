// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let run_result = xuan_brain_lib::run();
    run_result.expect("Failed to run application");
}
