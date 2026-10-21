export type DimmingMode = "HardwareDDC" | "HardwareWMI" | "SoftwareOverlay";

export type TintPreset = "neutral" | "amber" | "red";

export interface MonitorInfo {
  id: string;
  name: string;
  isPrimary: boolean;
  rect: {
    left: number;
    top: number;
    width: number;
    height: number;
  };
  supportedMode: DimmingMode;
  brightness: number; // 0 to 100
  enabled: boolean;
}

export interface AppConfig {
  masterEnabled: boolean;
  tint: TintPreset;
  launchOnStartup: boolean;
  monitors: Record<string, { brightness: number; enabled: boolean }>;
}

export interface HotkeyInfo {
  name: string;
  keys: string;
  description: string;
}
