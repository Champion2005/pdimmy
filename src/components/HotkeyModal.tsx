import React from "react";
import { X, ShieldAlert, Command, ArrowUp, ArrowDown, Power } from "lucide-react";

interface HotkeyModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const HotkeyModal: React.FC<HotkeyModalProps> = ({ isOpen, onClose }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-xs flex items-center justify-center p-4">
      <div className="bg-[#111111] border border-[#262626] rounded-xl w-full max-w-sm overflow-hidden shadow-2xl">
        <div className="flex items-center justify-between px-4 py-3 border-b border-[#222222] bg-[#161616]">
          <div className="flex items-center space-x-2">
            <Command size={15} className="text-white" />
            <h2 className="text-xs font-semibold text-white tracking-wide uppercase">
              Global Hotkeys & Guardrails
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded text-[#777777] hover:text-white hover:bg-[#252525] transition-colors"
          >
            <X size={15} />
          </button>
        </div>

        <div className="p-4 space-y-4 text-xs">
          {/* Emergency Panic Box */}
          <div className="p-3 rounded-lg border border-red-950/60 bg-red-950/20 space-y-1.5">
            <div className="flex items-center space-x-2 text-red-400 font-semibold text-xs">
              <ShieldAlert size={14} />
              <span>Emergency Rescue Hotkey</span>
            </div>
            <div className="flex items-center justify-between pt-1">
              <span className="text-[#AAAAAA] text-[11px]">Instant Failsafe Reset</span>
              <kbd className="px-2 py-0.5 rounded bg-[#200A0A] border border-red-900/40 text-red-200 font-mono text-[11px] font-bold">
                Ctrl + Shift + Alt + Backspace
              </kbd>
            </div>
            <p className="text-[10px] text-[#888888] leading-relaxed pt-0.5">
              Instantly destroys all software overlays, unregisters screen locks, and resets all hardware monitors to 100% brightness.
            </p>
          </div>

          {/* Standard Hotkeys */}
          <div className="space-y-2">
            <div className="text-[11px] font-medium text-[#777777] uppercase tracking-wider">
              Navigation & Adjustment
            </div>

            <div className="space-y-1.5">
              <div className="flex items-center justify-between p-2 rounded bg-[#181818] border border-[#222222]">
                <div className="flex items-center space-x-2 text-[#CCCCCC]">
                  <Power size={13} className="text-[#888888]" />
                  <span>Toggle Dimmer On / Off</span>
                </div>
                <kbd className="px-1.5 py-0.5 rounded bg-[#262626] text-white font-mono text-[10px]">
                  Ctrl + Alt + D
                </kbd>
              </div>

              <div className="flex items-center justify-between p-2 rounded bg-[#181818] border border-[#222222]">
                <div className="flex items-center space-x-2 text-[#CCCCCC]">
                  <ArrowUp size={13} className="text-[#888888]" />
                  <span>Step Brightness Up (+5%)</span>
                </div>
                <kbd className="px-1.5 py-0.5 rounded bg-[#262626] text-white font-mono text-[10px]">
                  Ctrl + Alt + Up
                </kbd>
              </div>

              <div className="flex items-center justify-between p-2 rounded bg-[#181818] border border-[#222222]">
                <div className="flex items-center space-x-2 text-[#CCCCCC]">
                  <ArrowDown size={13} className="text-[#888888]" />
                  <span>Step Brightness Down (-5%)</span>
                </div>
                <kbd className="px-1.5 py-0.5 rounded bg-[#262626] text-white font-mono text-[10px]">
                  Ctrl + Alt + Down
                </kbd>
              </div>
            </div>
          </div>

          <div className="pt-1">
            <button
              onClick={onClose}
              className="w-full py-1.5 rounded-lg bg-white text-black font-semibold text-xs hover:bg-[#E5E5E5] transition-colors"
            >
              Done
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
