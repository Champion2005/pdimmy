//! Topology and Power Broadcast Listener for pdimmy
//!
//! Listens for WM_DISPLAYCHANGE and WM_POWERBROADCAST to dynamically adapt to
//! docking stations, monitor disconnects/reconnects, and system resume from sleep.

use super::hybrid_controller::HybridDimmerController;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

pub struct TopologyListener;

impl TopologyListener {
    pub fn start(controller: Arc<HybridDimmerController>, app_handle: AppHandle) {
        std::thread::Builder::new()
            .name("pdimmy-topology-listener".to_string())
            .spawn(move || {
                #[cfg(windows)]
                {
                    use std::mem::size_of;
                    use windows::core::{w, PCWSTR};
                    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
                    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
                    use windows::Win32::UI::WindowsAndMessaging::{
                        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
                        RegisterClassExW, TranslateMessage, HMENU, MSG, WNDCLASSEXW,
                        WM_DISPLAYCHANGE, WM_POWERBROADCAST,
                    };

                    const LISTENER_CLASS_NAME: PCWSTR = w!("PDIMMY_TOPOLOGY_LISTENER_CLASS");

                    unsafe extern "system" fn listener_wndproc(
                        hwnd: HWND,
                        msg: u32,
                        wparam: WPARAM,
                        lparam: LPARAM,
                    ) -> LRESULT {
                        match msg {
                            WM_DISPLAYCHANGE => {
                                info!("Display topology change detected (WM_DISPLAYCHANGE).");
                                LRESULT(0)
                            }
                            WM_POWERBROADCAST => {
                                info!("Power state change detected (WM_POWERBROADCAST).");
                                LRESULT(1)
                            }
                            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
                        }
                    }

                    unsafe {
                        let hinstance = GetModuleHandleW(None).unwrap_or_default();
                        let wc = WNDCLASSEXW {
                            cbSize: size_of::<WNDCLASSEXW>() as u32,
                            style: Default::default(),
                            lpfnWndProc: Some(listener_wndproc),
                            cbClsExtra: 0,
                            cbWndExtra: 0,
                            hInstance: hinstance.into(),
                            hIcon: Default::default(),
                            hCursor: Default::default(),
                            hbrBackground: Default::default(),
                            lpszMenuName: PCWSTR::null(),
                            lpszClassName: LISTENER_CLASS_NAME,
                            hIconSm: Default::default(),
                        };

                        let _ = RegisterClassExW(&wc);

                        // Create message-only window
                        let hwnd = CreateWindowExW(
                            Default::default(),
                            LISTENER_CLASS_NAME,
                            w!("pdimmy_topology_msg_window"),
                            Default::default(),
                            0,
                            0,
                            0,
                            0,
                            HWND(-3 as _), // HWND_MESSAGE
                            HMENU(0),
                            hinstance,
                            None,
                        );

                        let mut msg = MSG::default();
                        while GetMessageW(&mut msg, hwnd, 0, 0).as_bool() {
                            if msg.message == WM_DISPLAYCHANGE || msg.message == WM_POWERBROADCAST {
                                controller.refresh_monitors();
                                let _ = app_handle.emit("monitors_changed", controller.get_monitors());
                            }
                            TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    }
                }
            })
            .expect("Failed to spawn topology listener thread");
    }
}
