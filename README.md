# `pdimmy` — Production Windows Screen Dimmer

<div align="center">

```
             _ _                           
  _ __   __| (_)_ __ ___  _ __ ___  _   _ 
 | '_ \ / _` | | '_ ` _ \| '_ ` _ \| | | |
 | |_) | (_| | | | | | | | | | | | | |_| |
 | .__/ \__,_|_|_| |_| |_|_| |_| |_|\__, |
 |_|                                |___/ 
```

**Ultra-lightweight, production-ready system tray utility for Windows that dims connected displays via hardware backlights and GPU-accelerated layered overlays.**

Built with **Rust**, **Win32 API**, and **Tauri v2**.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-black.svg)]()
[![Architecture](https://img.shields.io/badge/Architecture-x86__64-black.svg)]()
[![Footprint](https://img.shields.io/badge/Idle%20RAM-%3C25MB-emerald.svg)]()

</div>

---

## 1. Architectural Highlights

### A. Decoupled Win32 Layered Window Engine
Naive webview dimmers create a transparent browser window per display. That approach consumes 50–120+ MB of RAM and introduces mouse latency.

`pdimmy` decouples the UI from the overlay engine:
- **Tray & Flyout UI:** Sleek, monochromatic Tauri v2 window that stays dormant when hidden.
- **Display Overlays:** Created directly via pure Win32 API (`CreateWindowExW` with `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST`).
- **Overhead:** 0% idle CPU and virtually 0 MB memory per display. Click-through and hotkeys pass straight to background apps without focus stealing.

### B. Hybrid Seamless Dimming
Monitors have hardware limits (often ~30–50 nits at 0% backlight). `pdimmy` smoothly blends hardware backlight control with software dimming:
- **100% → 20% Luminance:** Modulates physical hardware backlight (`dxva2.dll` DDC/CI on external monitors or WMI/ACPI on laptop panels). Software overlay is 100% transparent.
- **20% → 0% Luminance:** Keeps hardware backlight at minimum and ramps up the Win32 layered overlay opacity down to the safety floor.
- **Non-DDC/CI Displays:** Seamlessly falls back to full-range software overlay dimming.

### C. Guardrails & Safety Constraints
- **Anti-Blackout Hard Cap:** Overlay opacity has an immutable hard ceiling of **85%** ($\alpha \le 217$), guaranteeing at least **15% display luminance** remains visible so you can never be trapped in an unreadable screen.
- **Emergency Rescue Hotkey:** Immutable system-wide failsafe (`Ctrl + Shift + Alt + Backspace`) running on a dedicated OS message pump. It instantly kills all overlays, unregisters locks, and restores hardware backlights to 100%.

### D. Asynchronous, Debounced Hardware Driver
DDC/CI communication over I2C/DisplayPort AUX channels takes 20ms to 200ms per round-trip. `pdimmy` updates software overlays synchronously (60fps immediate response) while queueing hardware commands through an asynchronous worker with a 30ms trailing-edge coalesce to eliminate slider stutter.

### E. Memory Trimming (<25MB Idle Working Set)
When the flyout loses focus, Tauri hides the window and invokes `SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX)`, trimming dormant memory working sets down to <20–25 MB.

---

## 2. Global Hotkeys

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Alt</kbd> + <kbd>Backspace</kbd> | **Emergency Panic Reset** | Kills all overlays, sets backlights to 100%, and disables dimming. |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>D</kbd> | **Toggle Dimmer** | Bypasses or enables dimming across all monitors. |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>Up</kbd> | **Step Brightness Up** | Increases brightness by +5% on all managed monitors. |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>Down</kbd> | **Step Brightness Down** | Decreases brightness by -5% on all managed monitors. |

---

## 3. Project Structure

```
pdimmy/
├── package.json              # Frontend scripts & dependencies
├── vite.config.ts            # Vite 8 configuration
├── tsconfig.json             # TypeScript configuration
├── index.html                # App shell HTML
├── src/                      # Monochromatic Flyout UI (React + Tailwind)
│   ├── App.tsx               # Main state container & IPC bridge
│   ├── main.tsx              # React DOM entrypoint
│   ├── index.css             # Monochromatic styling & slider styles
│   ├── types.ts              # TypeScript interface definitions
│   └── components/
│       ├── Header.tsx        # Title, status indicator, master toggle
│       ├── MonitorCard.tsx   # Per-display slider & hardware tag
│       ├── TintSelector.tsx  # Neutral / Amber / Red selector
│       ├── HotkeyModal.tsx   # Hotkey drawer with panic instructions
│       └── Footer.tsx        # Startup toggle & emergency reminder
└── src-tauri/                # High-performance Rust backend
    ├── Cargo.toml            # Rust dependencies & Windows features
    ├── tauri.conf.json       # Tauri v2 configuration
    ├── build.rs              # Build script
    ├── pdimmy.exe.manifest   # asInvoker & PerMonitorV2 manifest
    ├── icons/                # Multi-size app & tray icons
    └── src/
        ├── main.rs           # Entry point & DPI v2 initialization
        ├── lib.rs            # Tray setup, focus blur & memory trimming
        ├── commands.rs       # Tauri IPC commands
        └── engine/
            ├── mod.rs
            ├── win32_overlay.rs     # Pure Win32 WS_EX_LAYERED overlay engine
            ├── hardware_ddc.rs      # dxva2.dll DDC/CI monitor driver
            ├── hardware_wmi.rs      # WMI / ACPI laptop backlight driver
            ├── hybrid_controller.rs # Seamless dimming & debounced queue
            ├── hotkey_daemon.rs     # Win32 RegisterHotKey background pump
            ├── topology_listener.rs # WM_DISPLAYCHANGE & WM_POWERBROADCAST
            └── startup.rs           # Registry HKCU Run key manager
```

---

## 4. Setup & Build Instructions

### Prerequisites (on Windows Host)
1. **Rust:** Install via [rustup.rs](https://rustup.rs) (`stable-x86_64-pc-windows-msvc`).
2. **Visual Studio C++ Build Tools:** Ensure **Desktop development with C++** and the **Windows 10/11 SDK** are installed.
3. **Node.js & pnpm:** Node.js v20+ and pnpm v9+.
4. **WebView2 Runtime:** Pre-installed on Windows 10/11.

### Development Mode
```powershell
# 1. Clone repository
git clone https://github.com/your-username/pdimmy.git
cd pdimmy

# 2. Install frontend dependencies
pnpm install

# 3. Launch dev environment (hot-reload UI + Rust backend)
pnpm tauri dev
```

### Production Build
```powershell
# Build optimized standalone binary and NSIS installer
pnpm tauri build
```
Output artifacts:
- **Standalone Portable `.exe`:** `src-tauri/target/release/pdimmy.exe`
- **NSIS Setup Installer:** `src-tauri/target/release/bundle/nsis/pdimmy_1.0.0_x64-setup.exe`

> [!NOTE]
> The application uses `<requestedExecutionLevel level="asInvoker" />` and registry entries in `HKEY_CURRENT_USER`. It runs entirely as a standard user and will never trigger a UAC elevation prompt.

---

## 5. Verification & Testing Checklist

### 1. Click-Through & Focus Pass-Through Verification
1. Dim all displays to 30% or lower so software overlays are visibly active.
2. Click any desktop icon or interact with background apps directly beneath the dark overlay.
3. **Pass Criteria:** Mouse events, drag-select boxes, right-click context menus, and scroll gestures must pass immediately to underlying windows without lag or focus stealing.

### 2. Emergency Rescue Hotkey Verification
1. Turn dimmer active and set opacity high.
2. Press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Alt</kbd> + <kbd>Backspace</kbd>.
3. **Pass Criteria:** All overlays must vanish instantly, hardware brightness must jump to 100%, and the master toggle must switch to Bypassed.

### 3. Anti-Blackout Hard Cap Verification
1. Drag the brightness slider for any monitor down to `0%`.
2. **Pass Criteria:** Screen text, icons, and windows must remain clearly discernible (minimum 15% luminance maintained via the 85% alpha cap).

### 4. Monitor Disconnect / Reconnect Scenario
1. Unplug an external monitor or undock a laptop.
2. Reconnect the display or dock.
3. **Pass Criteria:** The `WM_DISPLAYCHANGE` listener automatically catches topology updates, refreshes monitor bounding boxes, re-establishes DDC/CI handles, and re-applies current dimming without crashing or visual tearing.

### 5. Memory Working Set Footprint
1. Open Windows Task Manager (`Ctrl + Shift + Esc`).
2. Click the tray icon to open the flyout, then click away to dismiss it.
3. Check the `pdimmy.exe` process in Task Manager.
4. **Pass Criteria:** Working set memory drops to `<25 MB` within seconds as memory trimming executes.

---

## License
MIT License. Free for personal and commercial use.
