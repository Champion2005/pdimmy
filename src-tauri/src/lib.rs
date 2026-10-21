//! Core Application Library for pdimmy

pub mod commands;
pub mod engine;

use commands::*;
use engine::hotkey_daemon::HotkeyDaemon;
use engine::hybrid_controller::HybridDimmerController;
use engine::topology_listener::TopologyListener;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tracing::info;

pub fn run() {
    tracing_subscriber::fmt::init();
    info!("Starting pdimmy Windows screen dimmer...");

    let controller = HybridDimmerController::new();

    tauri::Builder::default()
        .manage(controller.clone())
        .setup({
            let controller = controller.clone();
            move |app| {
                let app_handle = app.handle().clone();

                // Start Win32 background hotkey daemon
                HotkeyDaemon::start(controller.clone(), app_handle.clone());

                // Start Win32 display change and power listener
                TopologyListener::start(controller.clone(), app_handle.clone());

                // Setup Tray Menu
                let quit_i = MenuItem::with_id(app, "quit", "Quit pdimmy", true, None::<&str>)?;
                let reset_i = MenuItem::with_id(
                    app,
                    "reset",
                    "Emergency Reset (100%)",
                    true,
                    None::<&str>,
                )?;
                let menu = Menu::with_items(app, &[&reset_i, &quit_i])?;

                // Build System Tray Icon
                let _tray = TrayIconBuilder::with_id("pdimmy-tray")
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event({
                        let controller = controller.clone();
                        move |app, event| match event.id.as_ref() {
                            "quit" => {
                                app.exit(0);
                            }
                            "reset" => {
                                controller.emergency_panic_reset();
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Button {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                if let Ok(is_visible) = window.is_visible() {
                                    if is_visible {
                                        let _ = window.hide();
                                    } else {
                                        // Position window above taskbar on primary monitor before showing
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        }
                    })
                    .build(app)?;

                Ok(())
            }
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(false) = event {
                // Auto-hide flyout window when it loses focus
                let _ = window.hide();

                // Aggressively trim working set memory when dormant to ensure <25MB footprint
                #[cfg(windows)]
                {
                    use windows::Win32::System::ProcessStatus::SetProcessWorkingSetSize;
                    use windows::Win32::System::Threading::GetCurrentProcess;
                    unsafe {
                        let _ = SetProcessWorkingSetSize(
                            GetCurrentProcess(),
                            usize::MAX,
                            usize::MAX,
                        );
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_monitors,
            get_config,
            set_monitor_brightness,
            toggle_monitor,
            set_master_enabled,
            set_tint,
            set_launch_on_startup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running pdimmy application");
}
