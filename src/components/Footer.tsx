import React from "react";

interface FooterProps {
  launchOnStartup: boolean;
  onToggleStartup: () => void;
}

export const Footer: React.FC<FooterProps> = ({
  launchOnStartup,
  onToggleStartup,
}) => {
  return (
    <div className="px-4 py-3 border-t border-[#1C1C1C] bg-[#0E0E0E] flex items-center justify-between">
      <label className="flex items-center space-x-2 cursor-pointer select-none">
        <input
          type="checkbox"
          checked={launchOnStartup}
          onChange={onToggleStartup}
          className="sr-only peer"
        />
        <div className="w-7 h-4 bg-[#262626] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3 after:w-3 after:transition-all peer-checked:bg-white peer-checked:after:bg-black peer-checked:after:border-black relative"></div>
        <span className="text-[11px] text-[#888888] hover:text-[#CCCCCC] transition-colors">
          Start with Windows
        </span>
      </label>

      <span className="text-[10px] text-[#555555] font-mono">
        Rescue: <span className="text-[#777777]">Ctrl+Shift+Alt+Bksp</span>
      </span>
    </div>
  );
};
