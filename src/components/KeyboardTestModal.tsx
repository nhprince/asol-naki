import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

interface KeyboardTestModalProps {
  isOpen: boolean;
  onClose: (result?: { pass: boolean; note?: string }) => void;
}

const KEY_ROWS = [
  ["Esc", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "Del"],
  ["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", "Backspace"],
  ["Tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "[", "]", "\\"],
  ["Caps", "A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "'", "Enter"],
  ["Shift", "Z", "X", "C", "V", "B", "N", "M", ",", ".", "/", "Shift"],
  ["Ctrl", "Win", "Alt", "Space", "Alt", "Fn", "Ctrl", "←", "↑", "↓", "→"],
];

function normalizeKey(code: string, key: string): string {
  if (code.startsWith("Key")) return code.replace("Key", "");
  if (code.startsWith("Digit")) return code.replace("Digit", "");
  if (code.startsWith("F") && code.length <= 3) return code;
  if (code === "Space") return "Space";
  if (code === "Backspace") return "Backspace";
  if (code === "Tab") return "Tab";
  if (code === "CapsLock") return "Caps";
  if (code === "Enter") return "Enter";
  if (code.startsWith("Shift")) return "Shift";
  if (code.startsWith("Control")) return "Ctrl";
  if (code.startsWith("Alt")) return "Alt";
  if (code.startsWith("Meta") || code.startsWith("OS")) return "Win";
  if (code === "Escape") return "Esc";
  if (code === "Delete") return "Del";
  if (code === "ArrowLeft") return "←";
  if (code === "ArrowUp") return "↑";
  if (code === "ArrowDown") return "↓";
  if (code === "ArrowRight") return "→";
  return key.toUpperCase();
}

export function KeyboardTestModal({ isOpen, onClose }: KeyboardTestModalProps) {
  const { t } = useTranslation();
  const [pressedKeys, setPressedKeys] = useState<Set<string>>(new Set());
  const [activeKey, setActiveKey] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      const k = normalizeKey(e.code, e.key);
      setActiveKey(k);
      setPressedKeys((prev) => new Set(prev).add(k));
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      e.preventDefault();
      setActiveKey(null);
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
    };
  }, [isOpen]);

  if (!isOpen) return null;

  const totalTested = pressedKeys.size;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 p-4 backdrop-blur-md">
      <div className="flex w-full max-w-4xl flex-col gap-5 rounded-2xl bg-slate-900 p-6 text-white shadow-2xl ring-1 ring-white/10">
        <div className="flex items-center justify-between border-b border-white/10 pb-4">
          <div>
            <h2 className="text-lg font-bold text-emerald-400">
              {t("guidedKeyboard.title", "Interactive Keyboard Tester")}
            </h2>
            <p className="text-xs text-white/60">
              {t("guidedKeyboard.subtitle", "Press every key on the keyboard to verify responsiveness.")}
            </p>
          </div>
          <div className="rounded-xl bg-white/10 px-3 py-1.5 text-xs font-semibold">
            {t("guidedKeyboard.keysTested", "Tested")}:{" "}
            <span className="text-emerald-400">{totalTested}</span>
          </div>
        </div>

        <div className="flex flex-col gap-2 overflow-x-auto py-2">
          {KEY_ROWS.map((row, rIdx) => (
            <div key={`row-${rIdx}`} className="flex justify-center gap-1.5">
              {row.map((label, cIdx) => {
                const isPressed = pressedKeys.has(label);
                const isActive = activeKey === label;
                let widthClass = "w-10";
                if (label === "Space") widthClass = "w-52";
                else if (label === "Backspace" || label === "Enter") widthClass = "w-20";
                else if (label === "Shift" || label === "Caps" || label === "Tab") widthClass = "w-16";

                return (
                  <div
                    key={`key-${rIdx}-${cIdx}`}
                    className={`flex h-10 ${widthClass} items-center justify-center rounded-lg border text-xs font-medium transition-all duration-100 ${
                      isActive
                        ? "scale-105 border-emerald-400 bg-emerald-400 text-slate-950 shadow-lg shadow-emerald-500/50"
                        : isPressed
                          ? "border-emerald-500/50 bg-emerald-500/20 text-emerald-300"
                          : "border-white/10 bg-white/5 text-white/70"
                    }`}
                  >
                    {label}
                  </div>
                );
              })}
            </div>
          ))}
        </div>

        <div className="flex items-center justify-between border-t border-white/10 pt-4">
          <button
            type="button"
            onClick={() => {
              setPressedKeys(new Set());
              setActiveKey(null);
            }}
            className="rounded-lg bg-white/10 px-3 py-2 text-xs font-medium text-white/70 hover:bg-white/20"
          >
            {t("guidedKeyboard.reset", "Reset Layout")}
          </button>
          <div className="flex gap-3">
            <button
              type="button"
              onClick={() => onClose({ pass: false, note: `${totalTested} keys tested` })}
              className="rounded-xl bg-red-500/80 px-4 py-2 text-sm font-semibold text-white transition hover:bg-red-500"
            >
              {t("guidedKeyboard.fail", "Keys Unresponsive")}
            </button>
            <button
              type="button"
              onClick={() => onClose({ pass: true, note: `${totalTested} keys verified` })}
              className="rounded-xl bg-emerald-500 px-4 py-2 text-sm font-semibold text-emerald-950 transition hover:bg-emerald-400"
            >
              {t("guidedKeyboard.pass", "All Keys Working")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
