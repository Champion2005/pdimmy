import React from "react";
import { Monitor, Cpu, Layers, Sparkles, Minus, Plus } from "lucide-react";
import { MonitorInfo, DimmingMode } from "../types.ts";

interface MonitorCardProps {
  monitor: MonitorInfo;
  masterEnabled: boolean;
  onChangeBrightness: (id: string, brightness: number) => void;
  onToggleMonitor: (id: string) => void;
}

const getModeBadge = (mode: DimmingMode) => {
  switch (mode) {
    case "HardwareDDC":
      return {
        label: "DDC/CI Hardware",
        icon: <Cpu size={11} className="mr-1 text-white" />,
        bg: "bg-[#181818] border-[#2A2A2A] text-white",
      };
    case "HardwareWMI":
      return {
        label: "WMI ACPI",
        icon: <Sparkles size={11} className="mr-1 text-white" />,
        bg: "bg-[#181818] border-[#2A2A2A] text-white",
      };
    case "SoftwareOverlay":
    default:
      return {
        label: "Layered Overlay",
        icon: <Layers size={11} className="mr-1 text-[#888888]" />,
        bg: "bg-[#141414] border-[#202020] text-[#999999]",
      };
  }
};

export const MonitorCard: React.FC<MonitorCardProps> = ({
  monitor,
  masterEnabled,
  onChangeBrightness,
  onToggleMonitor,
}) => {
  const badge = getModeBadge(monitor.supportedMode);
  const isEffectivelyActive = masterEnabled && monitor.enabled;

  const handleStep = (delta: number) => {
    if (!isEffectivelyActive) return;
    const next = Math.max(0, Math.min(100, monitor.brightness + delta));
    onChangeBrightness(monitor.id, next);
  };

  // Determine dimming stage description in hybrid mode
  const getStageDescription = () => {
    if (!isEffectivelyActive) return "Display unmanaged";
    if (monitor.supportedMode === "SoftwareOverlay") {
      return `${100 - monitor.brightness}% Software Dim`;
    }
    if (monitor.brightness >= 20) {
      return "Hardware Backlight Dimming";
    }
    return "Ultra-Dim (Hardware Min + Layered Overlay)";
  };

  return (
    <div
      className={`p-3.5 rounded-lg border transition-all ${
        isEffectivelyActive
          ? "bg-[#111111] border-[#222222]"
          : "bg-[#0D0D0D] border-[#181818] opacity-60"
      }`}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center space-x-2">
          <Monitor size={15} className={isEffectivelyActive ? "text-white" : "text-[#555555]"} />
          <span className="text-xs font-medium text-white truncate max-w-[150px]">
            {monitor.name}
          </span>
          {monitor.isPrimary && (
            <span className="text-[9px] px-1 py-0.2 bg-[#222222] text-[#888888] rounded">
              MAIN
            </span>
          )}
        </div>

        <div className="flex items-center space-x-1.5">
          <span
            className={`inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-mono border ${badge.bg}`}
          >
            {badge.icon}
            {badge.label}
          </span>
          <button
            onClick={() => onToggleMonitor(monitor.id)}
            title={monitor.enabled ? "Disable for this monitor" : "Enable for this monitor"}
            className={`w-3.5 h-3.5 rounded-full border transition-all ${
              monitor.enabled
                ? "bg-white border-white"
                : "bg-transparent border-[#444444]"
            }`}
          />
        </div>
      </div>

      {/* Slider Control */}
      <div className="space-y-1.5 mt-2">
        <div className="flex items-center justify-between text-xs">
          <span className="text-[11px] text-[#777777] font-mono">
            {getStageDescription()}
          </span>
          <span className="text-sm font-bold font-mono text-white">
            {monitor.brightness}%
          </span>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={() => handleStep(-5)}
            disabled={!isEffectivelyActive || monitor.brightness <= 0}
            className="p-1 rounded bg-[#181818] hover:bg-[#252525] disabled:opacity-30 text-[#888888] hover:text-white transition-colors"
            title="Step down (-5%)"
          >
            <Minus size={12} />
          </button>

          <div className="relative flex-1">
            <input
              type="range"
              min={0}
              max={100}
              value={monitor.brightness}
              disabled={!isEffectivelyActive}
              onChange={(e) =>
                onChangeBrightness(monitor.id, parseInt(e.target.value, 10))
              }
              className="w-full disabled:opacity-40"
            />
            {/* Visual tick mark at 20% hybrid boundary if hardware supported */}
            {monitor.supportedMode !== "SoftwareOverlay" && (
              <div
                className="absolute top-1/2 -translate-y-1/2 w-0.5 h-2 bg-[#555555] pointer-events-none rounded"
                style={{ left: "20%" }}
                title="Hardware minimum boundary (20%)"
              />
            )}
          </div>

          <button
            onClick={() => handleStep(5)}
            disabled={!isEffectivelyActive || monitor.brightness >= 100}
            className="p-1 rounded bg-[#181818] hover:bg-[#252525] disabled:opacity-30 text-[#888888] hover:text-white transition-colors"
            title="Step up (+5%)"
          >
            <Plus size={12} />
          </button>
        </div>
      </div>
    </div>
  );
};
