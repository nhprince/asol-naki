import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { LanguageToggle } from "./components/LanguageToggle";

interface BasicHardwareInfo {
  cpu_name: string;
  cpu_threads: number;
  total_memory_mb: number;
  os_name: string;
  os_version: string;
  kernel_version: string;
  hostname: string;
}

function formatMemory(mb: number, locale: string): string {
  return new Intl.NumberFormat(locale === "bn" ? "bn-BD" : "en-US", {
    maximumFractionDigits: 0,
  }).format(mb);
}

export function App() {
  const { t, i18n } = useTranslation();
  const [info, setInfo] = useState<BasicHardwareInfo | null>(null);
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runScan = useCallback(async () => {
    setScanning(true);
    setError(null);
    try {
      const result = await invoke<BasicHardwareInfo>("scan_hardware_basic");
      setInfo(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setScanning(false);
    }
  }, []);

  // Auto-scan on launch so the window never looks empty.
  useEffect(() => {
    void runScan();
  }, [runScan]);

  const bnDigits = i18n.language === "bn";

  return (
    <div className="min-h-full bg-gradient-to-b from-brand-700 to-brand-900 text-white">
      <header className="flex items-center justify-between px-6 py-4">
        <div>
          <h1
            className="text-xl font-semibold tracking-tight"
            data-testid="app-title"
          >
            {t("app.title")}
          </h1>
          <p className="text-xs text-white/60">{t("app.subtitle")}</p>
        </div>
        <LanguageToggle />
      </header>

      <main className="mx-auto flex max-w-xl flex-col gap-6 px-6 pb-10">
        <section className="rounded-2xl bg-white/5 p-6 ring-1 ring-white/10">
          <h2 className="text-lg font-medium">{t("home.heading")}</h2>
          <p className="mt-2 text-sm leading-relaxed text-white/70">
            {t("home.description")}
          </p>
          <button
            type="button"
            data-testid="scan-button"
            onClick={() => void runScan()}
            disabled={scanning}
            className="mt-5 w-full rounded-xl bg-emerald-500 px-4 py-3 font-semibold text-emerald-950 transition-colors hover:bg-emerald-400 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {scanning ? t("scan.running") : t("home.scanButton")}
          </button>
        </section>

        {error && (
          <section
            role="alert"
            className="rounded-2xl bg-red-500/15 p-4 text-sm text-red-200 ring-1 ring-red-400/30"
          >
            <p>{t("error.scanFailed", { message: error })}</p>
            <button
              type="button"
              onClick={() => void runScan()}
              className="mt-2 rounded-lg bg-red-400/20 px-3 py-1 font-medium hover:bg-red-400/30"
            >
              {t("error.retry")}
            </button>
          </section>
        )}

        {info && (
          <section className="overflow-hidden rounded-2xl ring-1 ring-white/10">
            <dl className="divide-y divide-white/5 bg-white/5 text-sm">
              <Row
                label={t("scan.cpu")}
                value={info.cpu_name}
                testId="cpu-name"
              />
              <Row
                label={t("scan.threads")}
                value={String(info.cpu_threads)}
                bn={bnDigits}
              />
              <Row
                label={t("scan.memory")}
                value={`${formatMemory(info.total_memory_mb, i18n.language)} MB`}
                bn={bnDigits}
              />
              <Row
                label={t("scan.os")}
                value={`${info.os_name} ${info.os_version}`}
              />
              <Row label={t("scan.kernel")} value={info.kernel_version} />
              <Row label={t("scan.hostname")} value={info.hostname} />
            </dl>
          </section>
        )}

        <p className="text-center text-xs text-white/40">
          {t("footer.phaseNote")}
        </p>
      </main>
    </div>
  );
}

function Row({
  label,
  value,
  bn = false,
  testId,
}: {
  label: string;
  value: string;
  bn?: boolean;
  testId?: string;
}) {
  return (
    <div className="flex items-baseline justify-between gap-4 px-5 py-3">
      <dt className="shrink-0 text-white/60">{label}</dt>
      <dd
        className={`text-right font-medium ${bn ? "bengali-digits" : ""}`}
        data-testid={testId}
      >
        {value}
      </dd>
    </div>
  );
}

export default App;
