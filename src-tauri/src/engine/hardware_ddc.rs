//! DDC/CI Hardware Backlight Driver for External Displays
//!
//! Uses dxva2.dll to query and set hardware backlight brightness on external monitors
//! connected via DisplayPort, HDMI, or USB-C Alt Mode.

use super::win32_overlay::MonitorRect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdcMonitorDescriptor {
    pub id: String,
    pub name: String,
    pub is_primary: boolean_t,
    pub rect: MonitorRect,
    pub min_brightness: u32,
    pub current_brightness: u32,
    pub max_brightness: u32,
}

type boolean_t = bool;

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::mem::size_of;
    use windows::Win32::Foundation::{BOOL, HANDLE, HWND, LPARAM, POINT, RECT};
    use windows::Win32::Graphics::Dxva::{
        DestroyPhysicalMonitors, GetMonitorBrightness, GetNumberOfPhysicalMonitorsFromHMONITOR,
        GetPhysicalMonitorsFromHMONITOR, SetMonitorBrightness, PHYSICAL_MONITOR,
    };
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
        MONITOR_DEFAULTTONEAREST,
    };

    pub struct PhysicalMonitorGuard {
        pub h_physical: HANDLE,
        pub description: String,
    }

    pub struct DdcController {
        // Cached mapping from monitor device ID to physical monitor handles
    }

    impl DdcController {
        pub fn new() -> Self {
            Self {}
        }

        pub fn test_and_get_brightness(hmonitor: HMONITOR) -> Option<(u32, u32, u32)> {
            unsafe {
                let mut num_physical: u32 = 0;
                if GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut num_physical).is_err() || num_physical == 0 {
                    return None;
                }

                let mut physical_monitors = vec![PHYSICAL_MONITOR::default(); num_physical as usize];
                if GetPhysicalMonitorsFromHMONITOR(hmonitor, &mut physical_monitors).is_err() {
                    return None;
                }

                let mut min = 0u32;
                let mut cur = 0u32;
                let mut max = 100u32;

                let success = if let Some(first) = physical_monitors.first() {
                    GetMonitorBrightness(first.hPhysicalMonitor, &mut min, &mut cur, &mut max).is_ok()
                } else {
                    false
                };

                let _ = DestroyPhysicalMonitors(&physical_monitors);

                if success {
                    Some((min, cur, max))
                } else {
                    None
                }
            }
        }

        pub fn set_hardware_brightness(hmonitor: HMONITOR, target_brightness: u32) -> Result<(), String> {
            unsafe {
                let mut num_physical: u32 = 0;
                if GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut num_physical).is_err() || num_physical == 0 {
                    return Err("Failed to query physical monitor count".to_string());
                }

                let mut physical_monitors = vec![PHYSICAL_MONITOR::default(); num_physical as usize];
                if GetPhysicalMonitorsFromHMONITOR(hmonitor, &mut physical_monitors).is_err() {
                    return Err("Failed to acquire physical monitor handles".to_string());
                }

                let mut last_err = None;
                for pm in &physical_monitors {
                    let mut min = 0u32;
                    let mut cur = 0u32;
                    let mut max = 100u32;
                    if GetMonitorBrightness(pm.hPhysicalMonitor, &mut min, &mut cur, &mut max).is_ok() {
                        let clamped = target_brightness.clamp(min, max);
                        if let Err(e) = SetMonitorBrightness(pm.hPhysicalMonitor, clamped) {
                            last_err = Some(format!("SetMonitorBrightness error: {:?}", e));
                        }
                    }
                }

                let _ = DestroyPhysicalMonitors(&physical_monitors);

                match last_err {
                    Some(err) => Err(err),
                    None => Ok(()),
                }
            }
        }
    }
}

#[cfg(not(windows))]
mod non_windows_stub {
    use super::*;

    pub struct DdcController;

    impl DdcController {
        pub fn new() -> Self {
            Self
        }
        pub fn set_hardware_brightness(_id: &str, _brightness: u32) -> Result<(), String> {
            Ok(())
        }
    }
}

#[cfg(windows)]
pub use windows_impl::DdcController;
#[cfg(not(windows))]
pub use non_windows_stub::DdcController;
