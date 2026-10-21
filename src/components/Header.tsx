import React from "react";
import { Moon, Sun, Keyboard } from "lucide-react";

interface HeaderProps {
  masterEnabled: boolean;
  onToggleMaster: () => void;
  onOpenHotkeys: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  masterEnabled,
  onToggleMaster,
  onOpenHotkeys,
}) => {
  return (
    <div className="flex items-center justify-between px-4 py-3 border-b border-[#1F1F1F] bg-[#0E0E0E]">
      <div className="flex items-center space-x-2.5">
        <div className="w-6 h-6 rounded bg-white flex items-center justify-center text-black font-black text-xs tracking-tighter">
          pD
        </div>
        <div>
          <div className="flex items-center space-x-1.5">
            <h1 className="text-sm font-semibold tracking-wide text-white">pdimmy</h1>
            <span className="text-[10px] px-1.5 py-0.2 bg-[#222222] text-[#888888] rounded font-mono">
              v1.0
            </span>
          </div>
          <p className="text-[11px] text-[#666666]">Windows Display Dimmer</p>
        </div>
      </div>

      <div className="flex items-center space-x-2">
        <button
          onClick={onOpenHotkeys}
          title="Hotkey Shortcuts & Failsafe"
          className="p-1.5 rounded bg-[#181818] hover:bg-[#252525] text-[#AAAAAA] hover:text-white transition-colors"
        >
          <Keyboard size={15} />
        </button>

        {/* Master Toggle */}
        <button
          onClick={onToggleMaster}
          className={`flex items-center space-x-1.5 px-2.5 py-1 rounded-full text-xs font-medium transition-all ${
            masterEnabled
              ? "bg-white text-black shadow-[0_0_12px_rgba(255,255,255,0.25)]"
              : "bg-[#1C1C1C] text-[#666666] hover:bg-[#262626]"
          }`}
        >
          {masterEnabled ? <Moon size={13} /> : <Sun size={13} />}
          <span>{masterEnabled ? "Active" : "Bypassed"}</span>
        </button>
      </div>
    </div>
  );
};
