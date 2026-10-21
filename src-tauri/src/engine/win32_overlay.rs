//! Win32 Layered Window Overlay Engine for pdimmy
//!
//! Provides ultra-low-overhead, hardware-accelerated, transparent, click-through overlays
//! using native Win32 layered windows (WS_EX_LAYERED | WS_EX_TRANSPARENT).
//! This completely avoids running WebView2 processes for screen darkening, ensuring 0% idle CPU
//! and virtually 0 MB memory overhead per display.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TintPreset {
    Neutral, // Pure black overlay
    Amber,   // Warm blue-light reducing tint
    Red,     // Deep night vision red tint
}

impl Default for TintPreset {
    fn default() -> Self {
        Self::Neutral
    }
}

impl TintPreset {
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            TintPreset::Neutral => (0, 0, 0),
            TintPreset::Amber => (217, 119, 6),
            TintPreset::Red => (220, 38, 38),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorRect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

// Anti-blackout hard cap: 85% maximum opacity (alpha <= 217 out of 255)
// Ensures minimum 15% display luminance is ALWAYS visible so users can never be trapped in a black screen.
pub const MAX_OVERLAY_OPACITY_ALPHA: u8 = 217;

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::mem::size_of;
    use windows::core::{s, w, PCWSTR};
    use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        CreateSolidBrush, DeleteObject, FillRect, BeginPaint, EndPaint,
        HBRUSH, HDC, PAINTSTRUCT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
        PostMessageW, RegisterClassExW, SetLayeredWindowAttributes, SetWindowPos,
        ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, HMENU,
        HWND_TOPMOST, LWA_ALPHA, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNA, WM_DESTROY, WM_ERASEBKGND,
        WM_PAINT, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    };

    static WINDOW_CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);
    const OVERLAY_CLASS_NAME: PCWSTR = w!("PDIMMY_OVERLAY_WINDOW_CLASS");

    unsafe extern "system" fn overlay_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_ERASEBKGND => LRESULT(1), // Suppress flicker
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                let mut rect = RECT::default();
                windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rect);

                // Use the brush associated with the current tint or solid black
                let brush = CreateSolidBrush(COLORREF(0x00000000));
                FillRect(hdc, &rect, brush);
                DeleteObject(brush);

                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_DESTROY => LRESULT(0),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    pub struct NativeOverlayHandle {
        pub hwnd: HWND,
        pub current_alpha: u8,
        pub rect: MonitorRect,
    }

    unsafe impl Send for NativeOverlayHandle {}
    unsafe impl Sync for NativeOverlayHandle {}

    pub struct OverlayManagerInner {
        handles: HashMap<String, NativeOverlayHandle>,
        current_tint: TintPreset,
    }

    impl OverlayManagerInner {
        pub fn new() -> Self {
            Self::ensure_class_registered();
            Self {
                handles: HashMap::new(),
                current_tint: TintPreset::Neutral,
            }
        }

        fn ensure_class_registered() {
            if WINDOW_CLASS_REGISTERED.load(Ordering::SeqCst) {
                return;
            }

            unsafe {
                let wc = WNDCLASSEXW {
                    cbSize: size_of::<WNDCLASSEXW>() as u32,
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(overlay_wndproc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                        .unwrap_or_default()
                        .into(),
                    hIcon: Default::default(),
                    hCursor: Default::default(),
                    hbrBackground: HBRUSH(0),
                    lpszMenuName: PCWSTR::null(),
                    lpszClassName: OVERLAY_CLASS_NAME,
                    hIconSm: Default::default(),
                };

                let _ = RegisterClassExW(&wc);
                WINDOW_CLASS_REGISTERED.store(true, Ordering::SeqCst);
            }
        }

        pub fn get_or_create(&mut self, monitor_id: &str, rect: &MonitorRect) -> &mut NativeOverlayHandle {
            if !self.handles.contains_key(monitor_id) {
                unsafe {
                    let hwnd = CreateWindowExW(
                        WS_EX_LAYERED
                            | WS_EX_TRANSPARENT
                            | WS_EX_TOOLWINDOW
                            | WS_EX_NOACTIVATE
                            | WS_EX_TOPMOST,
                        OVERLAY_CLASS_NAME,
                        w!("pdimmy_dimmer_overlay"),
                        WS_POPUP,
                        rect.left,
                        rect.top,
                        rect.width,
                        rect.height,
                        HWND(0),
                        HMENU(0),
                        windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                            .unwrap_or_default(),
                        None,
                    );

                    // Set initial transparency to completely transparent (alpha = 0)
                    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA);
                    let _ = ShowWindow(hwnd, SW_HIDE);

                    self.handles.insert(
                        monitor_id.to_string(),
                        NativeOverlayHandle {
                            hwnd,
                            current_alpha: 0,
                            rect: rect.clone(),
                        },
                    );
                }
            }

            self.handles.get_mut(monitor_id).unwrap()
        }

        pub fn set_opacity(&mut self, monitor_id: &str, rect: &MonitorRect, target_alpha: u8) {
            // Apply anti-blackout hard cap
            let safe_alpha = target_alpha.min(MAX_OVERLAY_OPACITY_ALPHA);
            let handle = self.get_or_create(monitor_id, rect);

            unsafe {
                if safe_alpha == 0 {
                    let _ = ShowWindow(handle.hwnd, SW_HIDE);
                } else {
                    // Update bounding box if monitor changed geometry
                    if handle.rect.left != rect.left
                        || handle.rect.top != rect.top
                        || handle.rect.width != rect.width
                        || handle.rect.height != rect.height
                    {
                        let _ = SetWindowPos(
                            handle.hwnd,
                            HWND_TOPMOST,
                            rect.left,
                            rect.top,
                            rect.width,
                            rect.height,
                            SWP_NOACTIVATE | SWP_SHOWWINDOW,
                        );
                        handle.rect = rect.clone();
                    }

                    let (r, g, b) = self.current_tint.rgb();
                    let color = COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16));

                    let _ = SetLayeredWindowAttributes(handle.hwnd, color, safe_alpha, LWA_ALPHA);
                    let _ = ShowWindow(handle.hwnd, SW_SHOWNA);
                    let _ = SetWindowPos(
                        handle.hwnd,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    );
                }
                handle.current_alpha = safe_alpha;
            }
        }

        pub fn set_tint(&mut self, tint: TintPreset) {
            self.current_tint = tint;
            let handles_snapshot: Vec<(String, MonitorRect, u8)> = self
                .handles
                .iter()
                .map(|(k, v)| (k.clone(), v.rect.clone(), v.current_alpha))
                .collect();

            for (id, rect, alpha) in handles_snapshot {
                if alpha > 0 {
                    self.set_opacity(&id, &rect, alpha);
                }
            }
        }

        pub fn hide_all(&mut self) {
            for (_, handle) in self.handles.iter_mut() {
                unsafe {
                    let _ = ShowWindow(handle.hwnd, SW_HIDE);
                    let _ = SetLayeredWindowAttributes(handle.hwnd, COLORREF(0), 0, LWA_ALPHA);
                }
                handle.current_alpha = 0;
            }
        }

        pub fn destroy_all(&mut self) {
            for (_, handle) in self.handles.drain() {
                unsafe {
                    let _ = DestroyWindow(handle.hwnd);
                }
            }
        }
    }

    impl Drop for OverlayManagerInner {
        fn drop(&mut self) {
            self.destroy_all();
        }
    }
}

#[cfg(not(windows))]
mod non_windows_stub {
    use super::*;

    pub struct OverlayManagerInner;

    impl OverlayManagerInner {
        pub fn new() -> Self {
            Self
        }
        pub fn set_opacity(&mut self, _monitor_id: &str, _rect: &MonitorRect, _target_alpha: u8) {}
        pub fn set_tint(&mut self, _tint: TintPreset) {}
        pub fn hide_all(&mut self) {}
        pub fn destroy_all(&mut self) {}
    }
}

#[cfg(windows)]
pub use windows_impl::OverlayManagerInner as PlatformOverlayManager;
#[cfg(not(windows))]
pub use non_windows_stub::OverlayManagerInner as PlatformOverlayManager;

#[derive(Clone)]
pub struct Win32OverlayEngine {
    inner: Arc<Mutex<PlatformOverlayManager>>,
}

impl Win32OverlayEngine {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PlatformOverlayManager::new())),
        }
    }

    pub fn set_opacity(&self, monitor_id: &str, rect: &MonitorRect, target_alpha: u8) {
        self.inner.lock().set_opacity(monitor_id, rect, target_alpha);
    }

    pub fn set_tint(&self, tint: TintPreset) {
        self.inner.lock().set_tint(tint);
    }

    pub fn hide_all(&self) {
        self.inner.lock().hide_all();
    }

    pub fn destroy_all(&self) {
        self.inner.lock().destroy_all();
    }
}
