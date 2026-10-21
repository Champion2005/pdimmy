//! Hybrid Seamless Controller for pdimmy
//!
//! Orchestrates DDC/CI hardware backlight, WMI ACPI controls, and Win32 layered overlays.
//! Includes anti-blackout hard caps, non-blocking debounced hardware channels,
//! and emergency panic failsafe execution.

use super::hardware_ddc::DdcController;
use super::hardware_wmi::WmiController;
use super::win32_overlay::{MonitorRect, TintPreset, Win32OverlayEngine, MAX_OVERLAY_OPACITY_ALPHA};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimmingMode {
    HardwareDDC,
    HardwareWMI,
    SoftwareOverlay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorState {
    pub id: String,
    pub name: String,
    pub is_primary: bool,
    pub rect: MonitorRect,
    #[serde(rename = "supportedMode")]
    pub supported_mode: DimmingMode,
    pub brightness: u32, // 0 to 100
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "masterEnabled")]
    pub master_enabled: bool,
    pub tint: TintPreset,
    #[serde(rename = "launchOnStartup")]
    pub launch_on_startup: bool,
    pub monitors: HashMap<String, MonitorConfigEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfigEntry {
    pub brightness: u32,
    pub enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            master_enabled: true,
            tint: TintPreset::Neutral,
            launch_on_startup: false,
            monitors: HashMap::new(),
        }
    }
}

enum HardwareTask {
    SetBrightnessDdc { hmonitor_ptr: usize, target: u32 },
    SetBrightnessWmi { target: u32 },
    PanicResetAll,
}

pub struct HybridDimmerController {
    overlay_engine: Win32OverlayEngine,
    monitors: Arc<RwLock<Vec<MonitorState>>>,
    config: Arc<RwLock<AppConfig>>,
    tx_hardware: mpsc::Sender<HardwareTask>,
}

impl HybridDimmerController {
    pub fn new() -> Arc<Self> {
        let (tx, mut rx) = mpsc::channel::<HardwareTask>(64);
        let overlay_engine = Win32OverlayEngine::new();
        let monitors = Arc::new(RwLock::new(Self::enumerate_all_monitors()));
        let config = Arc::new(RwLock::new(AppConfig::default()));

        // Spawn dedicated non-blocking worker thread for slow DDC/CI / WMI I/O
        tokio::spawn(async move {
            let mut pending_ddc: HashMap<usize, u32> = HashMap::new();
            let mut pending_wmi: Option<u32> = None;

            loop {
                // Wait for task or timeout for debouncing (30ms)
                tokio::select! {
                    Some(task) = rx.recv() => {
                        match task {
                            HardwareTask::SetBrightnessDdc { hmonitor_ptr, target } => {
                                pending_ddc.insert(hmonitor_ptr, target);
                            }
                            HardwareTask::SetBrightnessWmi { target } => {
                                pending_wmi = Some(target);
                            }
                            HardwareTask::PanicResetAll => {
                                pending_ddc.clear();
                                pending_wmi = None;
                                #[cfg(windows)]
                                {
                                    // Reset WMI if active
                                    let _ = WmiController::set_brightness(100);
                                }
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(30)), if !pending_ddc.is_empty() || pending_wmi.is_some() => {
                        // Flush debounced hardware commands
                        for (hmon_ptr, brightness) in pending_ddc.drain() {
                            #[cfg(windows)]
                            {
                                use windows::Win32::Graphics::Gdi::HMONITOR;
                                let hmon = HMONITOR(hmon_ptr as _);
                                let _ = DdcController::set_hardware_brightness(hmon, brightness);
                            }
                        }

                        if let Some(target) = pending_wmi.take() {
                            #[cfg(windows)]
                            {
                                let _ = WmiController::set_brightness(target);
                            }
                        }
                    }
                }
            }
        });

        let controller = Arc::new(Self {
            overlay_engine,
            monitors,
            config,
            tx_hardware: tx,
        });

        // Initialize dimming states
        controller.apply_all_dimming();
        controller
    }

    pub fn enumerate_all_monitors() -> Vec<MonitorState> {
        #[cfg(windows)]
        {
            use std::mem::size_of;
            use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
            use windows::Win32::Graphics::Gdi::{
                EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
                MONITORINFOF_PRIMARY,
            };

            let mut list = Vec::new();

            unsafe extern "system" fn enum_proc(
                hmon: HMONITOR,
                _hdc: HDC,
                _rect: *mut RECT,
                lparam: LPARAM,
            ) -> BOOL {
                let list_ptr = lparam.0 as *mut Vec<MonitorState>;
                let mut mi = MONITORINFOEXW::default();
                mi.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;

                if GetMonitorInfoW(hmon, &mut mi as *mut _ as *mut _).as_bool() {
                    let rc = mi.monitorInfo.rcMonitor;
                    let width = rc.right - rc.left;
                    let height = rc.bottom - rc.top;
                    let is_primary = (mi.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;

                    let raw_device = String::from_utf16_lossy(&mi.szDevice);
                    let device_name = raw_device.trim_matches(char::from(0)).to_string();
                    let id = format!("hmon-{:p}-{}", hmon.0, device_name);

                    // Test DDC/CI capability
                    let ddc_result = DdcController::test_and_get_brightness(hmon);
                    let (mode, initial_bright) = if let Some((_, cur, _)) = ddc_result {
                        (DimmingMode::HardwareDDC, cur)
                    } else if WmiController::is_available() && is_primary {
                        (
                            DimmingMode::HardwareWMI,
                            WmiController::get_brightness().unwrap_or(100),
                        )
                    } else {
                        (DimmingMode::SoftwareOverlay, 100)
                    };

                    let display_name = if mode == DimmingMode::HardwareDDC {
                        format!("External Display ({})", device_name)
                    } else if mode == DimmingMode::HardwareWMI {
                        "Built-in Laptop Screen".to_string()
                    } else {
                        format!("Display ({})", device_name)
                    };

                    let state = MonitorState {
                        id,
                        name: display_name,
                        is_primary,
                        rect: MonitorRect {
                            left: rc.left,
                            top: rc.top,
                            width,
                            height,
                        },
                        supported_mode: mode,
                        brightness: initial_bright,
                        enabled: true,
                    };

                    (*list_ptr).push(state);
                }
                BOOL(1)
            }

            unsafe {
                let _ = EnumDisplayMonitors(
                    HDC(0),
                    None,
                    Some(enum_proc),
                    LPARAM(&mut list as *mut _ as isize),
                );
            }

            if list.is_empty() {
                list.push(MonitorState {
                    id: "default-display".to_string(),
                    name: "Primary Monitor".to_string(),
                    is_primary: true,
                    rect: MonitorRect {
                        left: 0,
                        top: 0,
                        width: 1920,
                        height: 1080,
                    },
                    supported_mode: DimmingMode::SoftwareOverlay,
                    brightness: 100,
                    enabled: true,
                });
            }

            list
        }

        #[cfg(not(windows))]
        {
            vec![
                MonitorState {
                    id: "mock-mon-1".to_string(),
                    name: "External Display (DDC/CI)".to_string(),
                    is_primary: true,
                    rect: MonitorRect {
                        left: 0,
                        top: 0,
                        width: 2560,
                        height: 1440,
                    },
                    supported_mode: DimmingMode::HardwareDDC,
                    brightness: 80,
                    enabled: true,
                },
                MonitorState {
                    id: "mock-mon-2".to_string(),
                    name: "Built-in Screen (WMI)".to_string(),
                    is_primary: false,
                    rect: MonitorRect {
                        left: 2560,
                        top: 0,
                        width: 1920,
                        height: 1080,
                    },
                    supported_mode: DimmingMode::HardwareWMI,
                    brightness: 70,
                    enabled: true,
                },
            ]
        }
    }

    pub fn get_monitors(&self) -> Vec<MonitorState> {
        self.monitors.read().clone()
    }

    pub fn get_config(&self) -> AppConfig {
        self.config.read().clone()
    }

    pub fn set_tint(&self, tint: TintPreset) {
        self.config.write().tint = tint;
        self.overlay_engine.set_tint(tint);
    }

    pub fn set_master_enabled(&self, enabled: bool) {
        self.config.write().master_enabled = enabled;
        if !enabled {
            self.overlay_engine.hide_all();
            // Restore hardware backlights to 100%
            let monitors = self.monitors.read().clone();
            for m in monitors {
                self.dispatch_hardware_brightness(&m, 100);
            }
        } else {
            self.apply_all_dimming();
        }
    }

    pub fn toggle_monitor(&self, monitor_id: &str, enabled: bool) {
        {
            let mut list = self.monitors.write();
            if let Some(target) = list.iter_mut().find(|m| m.id == monitor_id) {
                target.enabled = enabled;
            }
        }
        self.apply_all_dimming();
    }

    pub fn set_monitor_brightness(&self, monitor_id: &str, target_brightness: u32) {
        let clamped = target_brightness.clamp(0, 100);
        let monitor_snapshot = {
            let mut list = self.monitors.write();
            if let Some(target) = list.iter_mut().find(|m| m.id == monitor_id) {
                target.brightness = clamped;
                Some(target.clone())
            } else {
                None
            }
        };

        if let Some(mon) = monitor_snapshot {
            self.apply_monitor_dimming(&mon);
        }
    }

    pub fn step_all_monitors(&self, delta: i32) {
        let monitors_snapshot: Vec<(String, u32)> = {
            let mut list = self.monitors.write();
            list.iter_mut()
                .map(|m| {
                    let next = (m.brightness as i32 + delta).clamp(0, 100) as u32;
                    m.brightness = next;
                    (m.id.clone(), next)
                })
                .collect()
        };

        for (id, val) in monitors_snapshot {
            self.set_monitor_brightness(&id, val);
        }
    }

    pub fn toggle_master(&self) {
        let current = self.config.read().master_enabled;
        self.set_master_enabled(!current);
    }

    /// Emergency Panic Reset Hotkey Handler
    /// Immediately destroys all software overlays and resets all hardware monitors to 100%.
    pub fn emergency_panic_reset(&self) {
        info!("🚨 EMERGENCY RESCUE HOTKEY TRIGGERED! Killing overlays and restoring 100% backlight.");
        self.overlay_engine.hide_all();
        self.config.write().master_enabled = false;

        let monitors = self.monitors.read().clone();
        for m in monitors {
            self.dispatch_hardware_brightness(&m, 100);
        }

        let _ = self.tx_hardware.try_send(HardwareTask::PanicResetAll);
    }

    pub fn refresh_monitors(&self) {
        let new_list = Self::enumerate_all_monitors();
        *self.monitors.write() = new_list;
        self.apply_all_dimming();
    }

    fn apply_all_dimming(&self) {
        let master = self.config.read().master_enabled;
        if !master {
            self.overlay_engine.hide_all();
            return;
        }

        let monitors = self.monitors.read().clone();
        for m in monitors {
            self.apply_monitor_dimming(&m);
        }
    }

    /// Hybrid Seamless Dimming Core Engine
    /// Computes hardware backlight vs. software overlay alpha.
    fn apply_monitor_dimming(&self, monitor: &MonitorState) {
        let master = self.config.read().master_enabled;
        if !master || !monitor.enabled {
            self.overlay_engine
                .set_opacity(&monitor.id, &monitor.rect, 0);
            return;
        }

        match monitor.supported_mode {
            DimmingMode::HardwareDDC | DimmingMode::HardwareWMI => {
                // Hybrid Seamless Mode:
                // 100% down to 20%: Modulate physical backlight; overlay alpha is 0.
                // Below 20%: Hardware is at minimum (0%); overlay alpha ramps up from 0 to MAX_CAP.
                if monitor.brightness >= 20 {
                    // Backlight scales from min to 100%
                    self.dispatch_hardware_brightness(monitor, monitor.brightness);
                    // Overlay is transparent
                    self.overlay_engine
                        .set_opacity(&monitor.id, &monitor.rect, 0);
                } else {
                    // Hardware at lowest backlight
                    self.dispatch_hardware_brightness(monitor, 0);

                    // Ratio from 20% down to 0%
                    let fraction = (20.0 - monitor.brightness as f32) / 20.0;
                    let alpha = (fraction * MAX_OVERLAY_OPACITY_ALPHA as f32) as u8;
                    self.overlay_engine
                        .set_opacity(&monitor.id, &monitor.rect, alpha);
                }
            }
            DimmingMode::SoftwareOverlay => {
                // Software Overlay Fallback:
                // 100% -> alpha 0; 0% -> alpha 217 (85% opacity cap)
                let fraction = (100.0 - monitor.brightness as f32) / 100.0;
                let alpha = (fraction * MAX_OVERLAY_OPACITY_ALPHA as f32) as u8;
                self.overlay_engine
                    .set_opacity(&monitor.id, &monitor.rect, alpha);
            }
        }
    }

    fn dispatch_hardware_brightness(&self, monitor: &MonitorState, target: u32) {
        match monitor.supported_mode {
            DimmingMode::HardwareDDC => {
                #[cfg(windows)]
                {
                    // Parse HMONITOR pointer address from monitor.id
                    if let Some(hmon_ptr) = monitor
                        .id
                        .split('-')
                        .nth(1)
                        .and_then(|s| usize::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                    {
                        let _ = self.tx_hardware.try_send(HardwareTask::SetBrightnessDdc {
                            hmonitor_ptr: hmon_ptr,
                            target,
                        });
                    }
                }
            }
            DimmingMode::HardwareWMI => {
                let _ = self
                    .tx_hardware
                    .try_send(HardwareTask::SetBrightnessWmi { target });
            }
            DimmingMode::SoftwareOverlay => {}
        }
    }
}
