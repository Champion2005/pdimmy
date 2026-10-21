//! Windows Startup Registry Manager for pdimmy
//!
//! Controls HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run so the utility
//! starts automatically on boot without requiring administrative privileges.

pub struct StartupManager;

impl StartupManager {
    pub fn is_enabled() -> bool {
        #[cfg(windows)]
        {
            use windows::core::w;
            use windows::Win32::System::Registry::{
                RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
                KEY_READ, REG_SZ,
            };

            unsafe {
                let mut hkey = HKEY::default();
                let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
                if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut hkey).is_err() {
                    return false;
                }

                let value_name = w!("pdimmy");
                let mut value_type = REG_SZ;
                let mut data_len = 0u32;
                let exists = RegQueryValueExW(
                    hkey,
                    value_name,
                    None,
                    Some(&mut value_type),
                    None,
                    Some(&mut data_len),
                )
                .is_ok();

                let _ = RegCloseKey(hkey);
                exists
            }
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        #[cfg(windows)]
        {
            use std::env;
            use windows::core::w;
            use windows::Win32::System::Registry::{
                RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY,
                HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
            };

            unsafe {
                let mut hkey = HKEY::default();
                let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
                if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_SET_VALUE, &mut hkey).is_err() {
                    return Err("Failed to open HKCU Run registry key".to_string());
                }

                let value_name = w!("pdimmy");

                let res = if enabled {
                    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
                    let cmd_str = format!("\"{}\" --minimized\0", exe_path.to_string_lossy());
                    let utf16: Vec<u16> = cmd_str.encode_utf16().collect();
                    let byte_slice = std::slice::from_raw_parts(
                        utf16.as_ptr() as *const u8,
                        utf16.len() * 2,
                    );

                    RegSetValueExW(
                        hkey,
                        value_name,
                        0,
                        REG_SZ,
                        Some(byte_slice),
                    )
                } else {
                    RegDeleteValueW(hkey, value_name)
                };

                let _ = RegCloseKey(hkey);
                res.map_err(|e| format!("Registry write error: {:?}", e))?;
                Ok(())
            }
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }
}
