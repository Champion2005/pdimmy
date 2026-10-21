//! Global Hotkey Daemon for pdimmy
//!
//! Uses a dedicated Win32 thread running a message pump with RegisterHotKey
//! to ensure instant, reliable response regardless of UI thread state.

use super::hybrid_controller::HybridDimmerController;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::Emitter;
use tracing::{error, info};

pub const HOTKEY_ID_PANIC: i32 = 1001;
pub const HOTKEY_ID_TOGGLE: i32 = 1002;
pub const HOTKEY_ID_UP: i32 = 1003;
pub const HOTKEY_ID_DOWN: i32 = 1004;

pub struct HotkeyDaemon;

impl HotkeyDaemon {
    pub fn start(controller: Arc<HybridDimmerController>, app_handle: AppHandle) {
        std::thread::Builder::new()
            .name("pdimmy-hotkey-daemon".to_string())
            .spawn(move || {
                #[cfg(windows)]
                {
                    use windows::Win32::Foundation::HWND;
                    use windows::Win32::UI::Input::KeyboardAndMouse::{
                        RegisterHotKey, UnregisterHotKey,
                        HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
                        VK_BACK, VK_DOWN, VK_UP,
                    };
                    use windows::Win32::UI::WindowsAndMessaging::{
                        DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
                    };

                    let vk_d = 0x44u32; // 'D' key virtual key code

                    unsafe {
                        // 1. Emergency Panic Hotkey: Ctrl + Shift + Alt + Backspace
                        let panic_mod = MOD_CONTROL | MOD_SHIFT | MOD_ALT | MOD_NOREPEAT;
                        let reg_panic = RegisterHotKey(
                            HWND(0),
                            HOTKEY_ID_PANIC,
                            HOT_KEY_MODIFIERS(panic_mod.0),
                            VK_BACK.0 as u32,
                        );
                        if reg_panic.is_err() {
                            error!("Failed to register Emergency Panic Hotkey (Ctrl+Shift+Alt+Backspace)");
                        } else {
                            info!("Registered Emergency Panic Hotkey: Ctrl + Shift + Alt + Backspace");
                        }

                        // 2. Toggle Dimmer: Ctrl + Alt + D
                        let _ = RegisterHotKey(
                            HWND(0),
                            HOTKEY_ID_TOGGLE,
                            HOT_KEY_MODIFIERS((MOD_CONTROL | MOD_ALT | MOD_NOREPEAT).0),
                            vk_d,
                        );

                        // 3. Step Up: Ctrl + Alt + Up
                        let _ = RegisterHotKey(
                            HWND(0),
                            HOTKEY_ID_UP,
                            HOT_KEY_MODIFIERS((MOD_CONTROL | MOD_ALT | MOD_NOREPEAT).0),
                            VK_UP.0 as u32,
                        );

                        // 4. Step Down: Ctrl + Alt + Down
                        let _ = RegisterHotKey(
                            HWND(0),
                            HOTKEY_ID_DOWN,
                            HOT_KEY_MODIFIERS((MOD_CONTROL | MOD_ALT | MOD_NOREPEAT).0),
                            VK_DOWN.0 as u32,
                        );

                        let mut msg = MSG::default();
                        while GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                            if msg.message == WM_HOTKEY {
                                let hotkey_id = msg.wParam.0 as i32;
                                match hotkey_id {
                                    HOTKEY_ID_PANIC => {
                                        controller.emergency_panic_reset();
                                        let _ = app_handle.emit("emergency_panic_triggered", ());
                                    }
                                    HOTKEY_ID_TOGGLE => {
                                        controller.toggle_master();
                                        let _ = app_handle.emit("monitors_changed", controller.get_monitors());
                                    }
                                    HOTKEY_ID_UP => {
                                        controller.step_all_monitors(5);
                                        let _ = app_handle.emit("monitors_changed", controller.get_monitors());
                                    }
                                    HOTKEY_ID_DOWN => {
                                        controller.step_all_monitors(-5);
                                        let _ = app_handle.emit("monitors_changed", controller.get_monitors());
                                    }
                                    _ => {}
                                }
                            }
                            TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }

                        // Cleanup hotkeys on exit
                        let _ = UnregisterHotKey(HWND(0), HOTKEY_ID_PANIC);
                        let _ = UnregisterHotKey(HWND(0), HOTKEY_ID_TOGGLE);
                        let _ = UnregisterHotKey(HWND(0), HOTKEY_ID_UP);
                        let _ = UnregisterHotKey(HWND(0), HOTKEY_ID_DOWN);
                    }
                }
            })
            .expect("Failed to spawn hotkey daemon thread");
    }
}
