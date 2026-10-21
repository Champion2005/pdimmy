import React, { useState, useEffect, useCallback } from "react";
import { Header } from "./components/Header.tsx";
import { MonitorCard } from "./components/MonitorCard.tsx";
import { TintSelector } from "./components/TintSelector.tsx";
import { HotkeyModal } from "./components/HotkeyModal.tsx";
import { Footer } from "./components/Footer.tsx";
import { MonitorInfo, TintPreset, AppConfig } from "./types.ts";

// Tauri v2 invoke helper with browser mockup fallback
const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  }
  return Promise.reject("Running in web browser mode");
}

const MOCK_MONITORS: MonitorInfo[] = [
  {
    id: "mon-1",
    name: "DELL U2720Q (DisplayPort)",
    isPrimary: true,
    rect: { left: 0, top: 0, width: 3840, height: 2160 },
    supportedMode: "HardwareDDC",
    brightness: 80,
    enabled: true,
  },
  {
    id: "mon-2",
    name: "Internal Laptop Display (eDP)",
    isPrimary: false,
    rect: { left: 3840, top: 0, width: 1920, height: 1080 },
    supportedMode: "HardwareWMI",
    brightness: 65,
    enabled: true,
  },
  {
    id: "mon-3",
    name: "Side USB-C Portable",
    isPrimary: false,
    rect: { left: -1920, top: 0, width: 1920, height: 1080 },
    supportedMode: "SoftwareOverlay",
    brightness: 90,
    enabled: true,
  },
];

export const App: React.FC = () => {
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [masterEnabled, setMasterEnabled] = useState(true);
  const [tint, setTint] = useState<TintPreset>("neutral");
  const [launchOnStartup, setLaunchOnStartup] = useState(false);
  const [hotkeyModalOpen, setHotkeyModalOpen] = useState(false);
  const [loading, setLoading] = useState(true);

  // Initialize and load state
  useEffect(() => {
    async function init() {
      try {
        if (isTauri) {
          const fetchedMonitors = await tauriInvoke<MonitorInfo[]>("get_monitors");
          const config = await tauriInvoke<AppConfig>("get_config");
          setMonitors(fetchedMonitors);
          setMasterEnabled(config.masterEnabled);
          setTint(config.tint);
          setLaunchOnStartup(config.launchOnStartup);

          // Setup Tauri event listeners for background hotkey trigger / panic
          const { listen } = await import("@tauri-apps/api/event");
          const unlistenPanic = await listen("emergency_panic_triggered", () => {
            setMasterEnabled(false);
            setMonitors((prev) =>
              prev.map((m) => ({ ...m, brightness: 100 }))
            );
          });
          const unlistenRefresh = await listen<MonitorInfo[]>("monitors_changed", (event) => {
            setMonitors(event.payload);
          });

          return () => {
            unlistenPanic();
            unlistenRefresh();
          };
        } else {
          // Dev mockup mode
          setMonitors(MOCK_MONITORS);
          setLaunchOnStartup(true);
        }
      } catch (err) {
        console.warn("Failed to invoke Tauri backend:", err);
        setMonitors(MOCK_MONITORS);
      } finally {
        setLoading(false);
      }
    }
    init();
  }, []);

  // Update monitor brightness
  const handleBrightnessChange = useCallback(
    async (id: string, brightness: number) => {
      setMonitors((prev) =>
        prev.map((m) => (m.id === id ? { ...m, brightness } : m))
      );

      if (isTauri) {
        try {
          await tauriInvoke("set_monitor_brightness", { id, brightness });
        } catch (err) {
          console.error("Error setting monitor brightness:", err);
        }
      }
    },
    []
  );

  // Toggle monitor enabled
  const handleToggleMonitor = useCallback(
    async (id: string) => {
      setMonitors((prev) =>
        prev.map((m) => (m.id === id ? { ...m, enabled: !m.enabled } : m))
      );

      if (isTauri) {
        const target = monitors.find((m) => m.id === id);
        if (target) {
          try {
            await tauriInvoke("toggle_monitor", { id, enabled: !target.enabled });
          } catch (err) {
            console.error("Error toggling monitor:", err);
          }
        }
      }
    },
    [monitors]
  );

  // Toggle master enabled
  const handleToggleMaster = useCallback(async () => {
    const next = !masterEnabled;
    setMasterEnabled(next);

    if (isTauri) {
      try {
        await tauriInvoke("set_master_enabled", { enabled: next });
      } catch (err) {
        console.error("Error toggling master switch:", err);
      }
    }
  }, [masterEnabled]);

  // Change overlay tint
  const handleSelectTint = useCallback(async (newTint: TintPreset) => {
    setTint(newTint);

    if (isTauri) {
      try {
        await tauriInvoke("set_tint", { tint: newTint });
      } catch (err) {
        console.error("Error updating tint:", err);
      }
    }
  }, []);

  // Toggle startup
  const handleToggleStartup = useCallback(async () => {
    const next = !launchOnStartup;
    setLaunchOnStartup(next);

    if (isTauri) {
      try {
        await tauriInvoke("set_launch_on_startup", { enabled: next });
      } catch (err) {
        console.error("Error setting startup:", err);
      }
    }
  }, [launchOnStartup]);

  return (
    <div className="w-[360px] h-[540px] flex flex-col bg-[#0A0A0A] text-white border border-[#262626] rounded-xl overflow-hidden shadow-2xl selection:bg-neutral-800">
      <Header
        masterEnabled={masterEnabled}
        onToggleMaster={handleToggleMaster}
        onOpenHotkeys={() => setHotkeyModalOpen(true)}
      />

      {/* Main Content Area */}
      <div className="flex-1 overflow-y-auto px-4 py-3 space-y-4">
        {/* Monitors List */}
        <div className="space-y-2">
          <div className="flex items-center justify-between text-[11px] text-[#777777] font-medium px-0.5">
            <span>Displays ({monitors.length})</span>
            <span className="font-mono text-[10px] text-[#555555]">
              Hybrid Auto-Detection
            </span>
          </div>

          {loading ? (
            <div className="py-8 text-center text-xs text-[#555555]">
              Enumerating display interfaces...
            </div>
          ) : monitors.length === 0 ? (
            <div className="py-8 text-center text-xs text-[#555555]">
              No active displays found.
            </div>
          ) : (
            <div className="space-y-2">
              {monitors.map((m) => (
                <MonitorCard
                  key={m.id}
                  monitor={m}
                  masterEnabled={masterEnabled}
                  onChangeBrightness={handleBrightnessChange}
                  onToggleMonitor={handleToggleMonitor}
                />
              ))}
            </div>
          )}
        </div>

        {/* Tint Selector */}
        <TintSelector
          currentTint={tint}
          onSelectTint={handleSelectTint}
          disabled={!masterEnabled}
        />
      </div>

      <Footer
        launchOnStartup={launchOnStartup}
        onToggleStartup={handleToggleStartup}
      />

      <HotkeyModal
        isOpen={hotkeyModalOpen}
        onClose={() => setHotkeyModalOpen(false)}
      />
    </div>
  );
};
export default App;
