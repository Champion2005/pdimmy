# pdimmy

<div align="center">

**The ultra-lightweight, zero-lag screen dimmer for Windows.**

Dim your displays below their physical hardware limits — without lag, mouse hitches, or the risk of getting trapped in a black screen.

[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-black.svg)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Footprint](https://img.shields.io/badge/Idle%20Memory-%3C25MB-emerald.svg)]()

</div>

---

## Why pdimmy?

Most screen dimmers either just slap a heavy transparent window over your screen (which slows down your PC and can trap your mouse) or only let you adjust hardware backlight down to around 30% brightness.

**pdimmy combines the best of both:**
- **True Hardware Control First:** Lowers your monitor's actual physical backlight (via DDC/CI or laptop WMI) from 100% down to 20% to save power and preserve contrast.
- **Ultra-Dim Night Mode Below 20%:** Automatically blends in a transparent GPU-accelerated overlay when you need to go darker than your monitor allows.
- **100% Click-Through:** Mouse clicks, games, gestures, and hotkeys pass straight through without delay or focus drops.
- **Never Get Trapped:** An anti-blackout floor ensures your screen is always readable, plus an instant panic rescue hotkey if you ever need to reset everything.
- **Dormant & Fast:** Sits quietly in your system tray, using 0% CPU and less than 25 MB of RAM when idle.

---

## Keyboard Shortcuts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Alt</kbd> + <kbd>Backspace</kbd> | **Panic Rescue Reset** | Instantly removes all dimming and resets all screens to 100% brightness. |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>D</kbd> | **Toggle Dimmer** | Turn dimming on or off across all screens. |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>Up</kbd> | **Brighten Screen** | Increase brightness by 5%. |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>Down</kbd> | **Dim Screen** | Decrease brightness by 5%. |

---

## Features

- **Multi-Monitor Support:** Automatically detects each connected monitor (external displays, USB-C portable screens, and built-in laptop panels) with independent brightness sliders.
- **Night Tints:** Switch between **Neutral** (pure black), **Warm Amber** (blue-light reduction), and **Deep Red** (night vision protection).
- **Dock & Sleep Resilient:** Automatically adapts when you unplug, re-dock, or wake your PC from sleep.
- **Start with Windows:** Easily toggle automatic launch on boot from the tray menu.
- **No Admin Rights Required:** Runs safely as a standard Windows user without UAC prompts.

---

## Frequently Asked Questions

<details>
<summary><b>Why won't the slider let my screen go 100% pitch black?</b></summary>
This is an intentional safety feature. pdimmy caps software darkening at 85% opacity so that screen text and cursor controls always remain visible, preventing accidental lockouts.
</details>

<details>
<summary><b>What if my screen becomes unreadable?</b></summary>
Press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Alt</kbd> + <kbd>Backspace</kbd> at any time. This emergency rescue shortcut instantly destroys all dark overlays and sets all monitor backlights back to 100%.
</details>

<details>
<summary><b>Does it work with laptop screens?</b></summary>
Yes. Laptop internal panels are controlled directly through Windows ACPI/WMI brightness APIs, while external monitors use DDC/CI.
</details>

---

## Developer / Building from Source

If you want to build or run pdimmy from source on Windows:

### Prerequisites
1. **Node.js** (v20 or newer)
2. **Rust & C++ Build Tools:**
   - Install Rust via [rustup.rs](https://rustup.rs) (`x86_64-pc-windows-msvc`).
   - Install "Desktop development with C++" via the Visual Studio Installer.

### Running & Building
```powershell
# Install dependencies
npm install

# Run in development mode
npx tauri dev

# Build standalone executable & setup installer
npx tauri build
```

Compiled binaries will be created in `src-tauri/target/release/`.

---

## License
MIT License. Free for personal and commercial use.
