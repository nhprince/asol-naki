import { useState } from "react";
import { useTranslation } from "react-i18next";

interface PortTestModalProps {
  isOpen: boolean;
  onClose: (result?: { pass: boolean; note?: string }) => void;
}

interface PortCheckItem {
  id: string;
  labelKey: string;
  defaultLabel: string;
  tested: boolean;
  passed: boolean;
}

export function PortTestModal({ isOpen, onClose }: PortTestModalProps) {
  const { t } = useTranslation();
  const [items, setItems] = useState<PortCheckItem[]>([
    { id: "usb", labelKey: "guidedPorts.usb", defaultLabel: "USB Ports (A / C)", tested: false, passed: false },
    { id: "hdmi", labelKey: "guidedPorts.hdmi", defaultLabel: "HDMI / Display Output", tested: false, passed: false },
    { id: "audio", labelKey: "guidedPorts.audio", defaultLabel: "3.5mm Headphone Jack / Audio Output", tested: false, passed: false },
    { id: "wifi", labelKey: "guidedPorts.wifi", defaultLabel: "Wi-Fi Connectivity", tested: false, passed: false },
    { id: "bluetooth", labelKey: "guidedPorts.bluetooth", defaultLabel: "Bluetooth Adapter", tested: false, passed: false },
    { id: "charger", labelKey: "guidedPorts.charger", defaultLabel: "AC Adapter / Charging Port", tested: false, passed: false },
  ]);

  if (!isOpen) return null;

  const toggleItem = (id: string, pass: boolean) => {
    setItems((prev) =>
      prev.map((item) =>
        item.id === id ? { ...item, tested: true, passed: pass } : item,
      ),
    );
  };

  const allPassed = items.every((i) => i.tested && i.passed);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 p-4 backdrop-blur-md">
      <div className="flex w-full max-w-lg flex-col gap-5 rounded-2xl bg-slate-900 p-6 text-white shadow-2xl ring-1 ring-white/10">
        <div className="border-b border-white/10 pb-4">
          <h2 className="text-lg font-bold text-emerald-400">
            {t("guidedPorts.title", "Ports & Connectivity Check")}
          </h2>
          <p className="text-xs text-white/60">
            {t("guidedPorts.subtitle", "Plug in a drive, headphones, or display cable to verify physical port function.")}
          </p>
        </div>

        <div className="flex flex-col gap-3">
          {items.map((item) => (
            <div
              key={item.id}
              className="flex items-center justify-between rounded-xl bg-white/5 px-4 py-3 ring-1 ring-white/5"
            >
              <span className="text-sm font-medium">
                {t(item.labelKey, item.defaultLabel)}
              </span>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={() => toggleItem(item.id, true)}
                  className={`rounded-lg px-3 py-1 text-xs font-semibold transition ${
                    item.tested && item.passed
                      ? "bg-emerald-500 text-emerald-950"
                      : "bg-white/10 text-white/60 hover:bg-white/20"
                  }`}
                >
                  {t("guidedPorts.working", "Working ✓")}
                </button>
                <button
                  type="button"
                  onClick={() => toggleItem(item.id, false)}
                  className={`rounded-lg px-3 py-1 text-xs font-semibold transition ${
                    item.tested && !item.passed
                      ? "bg-red-500 text-white"
                      : "bg-white/10 text-white/60 hover:bg-white/20"
                  }`}
                >
                  {t("guidedPorts.failed", "Failed ✗")}
                </button>
              </div>
            </div>
          ))}
        </div>

        <div className="flex items-center justify-between border-t border-white/10 pt-4">
          <button
            type="button"
            onClick={() => onClose()}
            className="text-xs text-white/50 hover:text-white"
          >
            {t("guidedPorts.cancel", "Cancel")}
          </button>
          <button
            type="button"
            onClick={() =>
              onClose({
                pass: allPassed,
                note: `${items.filter((i) => i.passed).length}/${items.length} ports passed`,
              })
            }
            className="rounded-xl bg-emerald-500 px-5 py-2 text-sm font-semibold text-emerald-950 hover:bg-emerald-400"
          >
            {t("guidedPorts.done", "Save Checklist Results")}
          </button>
        </div>
      </div>
    </div>
  );
}
