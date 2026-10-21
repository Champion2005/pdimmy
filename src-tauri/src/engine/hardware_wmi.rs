//! Windows WMI / ACPI Laptop Backlight Driver
//!
//! Controls built-in laptop displays (eDP panels) via WmiMonitorBrightnessMethods
//! in the root\\wmi namespace.

use std::process::Command;
use tracing::{debug, error, info};

pub struct WmiController;

impl WmiController {
    /// Detects if an internal laptop display is active via WMI.
    pub fn is_available() -> bool {
        #[cfg(windows)]
        {
            let output = Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "Get-CimInstance -Namespace root/wmi -ClassName WmiMonitorBrightness -ErrorAction SilentlyContinue | Select-Object -ExpandProperty CurrentBrightness",
                ])
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        text.parse::<u32>().is_ok()
                    } else {
                        false
                    }
                }
                Err(_) => false,
            }
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    /// Queries the current brightness percentage of the internal laptop display.
    pub fn get_brightness() -> Option<u32> {
        #[cfg(windows)]
        {
            let output = Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "(Get-CimInstance -Namespace root/wmi -ClassName WmiMonitorBrightness).CurrentBrightness",
                ])
                .output()
                .ok()?;

            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                s.parse::<u32>().ok()
            } else {
                None
            }
        }
        #[cfg(not(windows))]
        {
            None
        }
    }

    /// Sets the brightness of the internal laptop screen asynchronously.
    /// Uses WmiSetBrightness with a 1-second timeout.
    pub fn set_brightness(target: u32) -> Result<(), String> {
        #[cfg(windows)]
        {
            let clamped = target.clamp(0, 100);
            let cmd_str = format!(
                "(Get-CimInstance -Namespace root/wmi -ClassName WmiMonitorBrightnessMethods).WmiSetBrightness(1, {})",
                clamped
            );

            // Execute non-blocking or quick PowerShell command
            let output = Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", &cmd_str])
                .output()
                .map_err(|e| format!("Failed to invoke WMI: {}", e))?;

            if output.status.success() {
                Ok(())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }
}
