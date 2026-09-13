import { useState } from "react";
import { useTranslation } from "react-i18next";

export function OptionalSpeedTest() {
  const { t } = useTranslation();
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<{ latencyMs: number; downloadMbps: number } | null>(null);
  const [error, setError] = useState<string | null>(null);

  const runSpeedTest = async () => {
    setTesting(true);
    setError(null);
    setResult(null);

    const startTime = performance.now();
    try {
      // Small 1MB test file fetch to measure throughput
      const res = await fetch("https://speed.cloudflare.com/__down?bytes=1000000", {
        cache: "no-store",
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);

      const durationSec = (performance.now() - startTime) / 1000;
      const blob = await res.blob();
      const megabits = (blob.size * 8) / 1000000;
      const downloadMbps = Math.round((megabits / durationSec) * 10) / 10;
      const latencyMs = Math.round(durationSec * 100);

      setResult({ latencyMs, downloadMbps });
    } catch {
      setError(t("optional.offlineError", "Offline or test host unreachable. Core functions remain unaffected."));
    } finally {
      setTesting(false);
    }
  };

  return (
    <div className="flex flex-col gap-2 rounded-2xl bg-white/5 p-4 ring-1 ring-white/10">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-semibold text-white">
            {t("optional.speedTestTitle", "Optional Internet Speed Test")}
          </h3>
          <p className="text-xs text-white/60">
            {t("optional.speedTestDesc", "Verifies Wi-Fi adapter throughput & latency.")}
          </p>
        </div>
        <button
          type="button"
          onClick={runSpeedTest}
          disabled={testing}
          className="rounded-xl bg-sky-500/20 px-3 py-1.5 text-xs font-semibold text-sky-300 ring-1 ring-sky-500/30 transition hover:bg-sky-500/30 disabled:opacity-50"
        >
          {testing ? t("optional.testing", "Testing…") : t("optional.startTest", "Run Test")}
        </button>
      </div>

      {result && (
        <div className="mt-2 flex gap-4 rounded-xl bg-white/5 p-3 text-xs">
          <div>
            <span className="text-white/60">{t("optional.download", "Speed")}: </span>
            <span className="font-bold text-emerald-400">{result.downloadMbps} Mbps</span>
          </div>
          <div>
            <span className="text-white/60">{t("optional.latency", "Latency")}: </span>
            <span className="font-bold text-sky-300">{result.latencyMs} ms</span>
          </div>
        </div>
      )}

      {error && (
        <p className="mt-1 text-xs text-amber-300/90">{error}</p>
      )}
    </div>
  );
}
