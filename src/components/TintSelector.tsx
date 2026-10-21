import React from "react";
import { TintPreset } from "../types.ts";

interface TintSelectorProps {
  currentTint: TintPreset;
  onSelectTint: (tint: TintPreset) => void;
  disabled: boolean;
}

const TINTS: { id: TintPreset; label: string; desc: string; dotColor: string }[] = [
  {
    id: "neutral",
    label: "Neutral",
    desc: "Pure Black",
    dotColor: "bg-[#444444]",
  },
  {
    id: "amber",
    label: "Warm Amber",
    desc: "Blue-blocker",
    dotColor: "bg-[#D97706]",
  },
  {
    id: "red",
    label: "Deep Red",
    desc: "Night Vision",
    dotColor: "bg-[#DC2626]",
  },
];

export const TintSelector: React.FC<TintSelectorProps> = ({
  currentTint,
  onSelectTint,
  disabled,
}) => {
  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between text-[11px] text-[#777777] font-medium px-0.5">
        <span>Software Overlay Tint</span>
        <span className="font-mono text-[10px] text-[#555555]">Active under 20%</span>
      </div>

      <div className="grid grid-cols-3 gap-1.5">
        {TINTS.map((t) => {
          const isSelected = currentTint === t.id;
          return (
            <button
              key={t.id}
              disabled={disabled}
              onClick={() => onSelectTint(t.id)}
              className={`flex items-center space-x-2 px-2.5 py-1.5 rounded-md border text-left transition-all ${
                isSelected
                  ? "bg-[#1C1C1C] border-white text-white shadow-sm"
                  : "bg-[#111111] border-[#222222] text-[#888888] hover:border-[#333333] hover:text-[#CCCCCC]"
              } ${disabled ? "opacity-40 cursor-not-allowed" : ""}`}
            >
              <div className={`w-2.5 h-2.5 rounded-full ${t.dotColor} shrink-0`} />
              <div className="truncate">
                <div className="text-xs font-medium leading-none">{t.label}</div>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
};
