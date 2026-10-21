//! Tauri IPC Commands for pdimmy

use crate::engine::hybrid_controller::{AppConfig, HybridDimmerController, MonitorState};
use crate::engine::startup::StartupManager;
use crate::engine::win32_overlay::TintPreset;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn get_monitors(controller: State<'_, Arc<HybridDimmerController>>) -> Vec<MonitorState> {
    controller.get_monitors()
}

#[tauri::command]
pub fn get_config(controller: State<'_, Arc<HybridDimmerController>>) -> AppConfig {
    let mut cfg = controller.get_config();
    cfg.launch_on_startup = StartupManager::is_enabled();
    cfg
}

#[tauri::command]
pub fn set_monitor_brightness(
    controller: State<'_, Arc<HybridDimmerController>>,
    id: String,
    brightness: u32,
) {
    controller.set_monitor_brightness(&id, brightness);
}

#[tauri::command]
pub fn toggle_monitor(
    controller: State<'_, Arc<HybridDimmerController>>,
    id: String,
    enabled: bool,
) {
    controller.toggle_monitor(&id, enabled);
}

#[tauri::command]
pub fn set_master_enabled(
    controller: State<'_, Arc<HybridDimmerController>>,
    enabled: bool,
) {
    controller.set_master_enabled(enabled);
}

#[tauri::command]
pub fn set_tint(
    controller: State<'_, Arc<HybridDimmerController>>,
    tint: TintPreset,
) {
    controller.set_tint(tint);
}

#[tauri::command]
pub fn set_launch_on_startup(enabled: bool) -> Result<(), String> {
    StartupManager::set_enabled(enabled)
}
