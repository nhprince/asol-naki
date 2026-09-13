import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

interface DisplayTestModalProps {
  isOpen: boolean;
  onClose: (result?: { pass: boolean; note?: string }) => void;
}

const COLORS = [
  { name: "Red", bg: "bg-red-600" },
  { name: "Green", bg: "bg-green-600" },
  { name: "Blue", bg: "bg-blue-600" },
  { name: "White", bg: "bg-white text-black" },
  { name: "Black", bg: "bg-black text-white" },
];

export function DisplayTestModal({ isOpen, onClose }: DisplayTestModalProps) {
  const { t } = useTranslation();
  const [colorIndex, setColorIndex] = useState(0);
  const [showControls, setShowControls] = useState(true);

  const nextColor = useCallback(() => {
    setColorIndex((prev) => (prev + 1) % COLORS.length);
  }, []);

  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setShowControls((prev) => !prev);
      } else if (e.key === " " || e.key === "ArrowRight") {
        nextColor();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, nextColor]);

  if (!isOpen) return null;

  const currentColor = COLORS[colorIndex];

  return (
    <div
      className={`fixed inset-0 z-50 flex flex-col justify-between p-6 transition-colors duration-200 ${currentColor.bg}`}
      onClick={nextColor}
    >
      {/* Top Banner overlay */}
      <div
        className="mx-auto flex max-w-xl items-center justify-between rounded-xl bg-slate-900/80 px-4 py-2 text-xs text-white backdrop-blur-md shadow-lg"
        onClick={(e) => e.stopPropagation()}
      >
        <div>
          <span className="font-semibold text-emerald-400">
            {t("guidedDisplay.title", "Display Pixel & Bleed Check")}
          </span>
          <span className="ml-2 text-white/70">
            ({colorIndex + 1}/{COLORS.length}: {currentColor.name})
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setShowControls((prev) => !prev)}
            className="rounded bg-white/10 px-2 py-0.5 text-[11px] hover:bg-white/20"
          >
            {showControls ? "Hide Card" : "Show Card"}
          </button>
          <span className="text-[11px] text-white/60">
            {t("guidedDisplay.instruction", "Click or Space to cycle color, Esc to toggle card")}
          </span>
        </div>
      </div>

      {/* Bottom Verdict Controls overlay */}
      {showControls && (
        <div
          className="mx-auto flex max-w-md flex-col items-center gap-3 rounded-2xl bg-slate-900/90 p-5 text-center text-white backdrop-blur-lg shadow-2xl ring-1 ring-white/20"
          onClick={(e) => e.stopPropagation()}
        >
          <p className="text-sm font-medium">
            {t("guidedDisplay.question", "Did you notice any dead pixels, stuck pixels, or backlight bleed?")}
          </p>
          <div className="flex w-full gap-3">
            <button
              type="button"
              onClick={() => onClose({ pass: true })}
              className="flex-1 rounded-xl bg-emerald-500 py-2 text-sm font-semibold text-emerald-950 transition hover:bg-emerald-400"
            >
              {t("guidedDisplay.pass", "Screen Clean (Pass)")}
            </button>
            <button
              type="button"
              onClick={() => onClose({ pass: false, note: "Defects found" })}
              className="flex-1 rounded-xl bg-red-500/80 py-2 text-sm font-semibold text-white transition hover:bg-red-500"
            >
              {t("guidedDisplay.fail", "Defects Found")}
            </button>
          </div>
          <button
            type="button"
            onClick={() => onClose()}
            className="text-xs text-white/50 hover:text-white"
          >
            {t("guidedDisplay.cancel", "Cancel Test")}
          </button>
        </div>
      )}
    </div>
  );
}
