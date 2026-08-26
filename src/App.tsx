import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { LanguageToggle } from "./components/LanguageToggle";
import { PrintButton } from "./components/PrintButton";
import { ReportCard } from "./components/ReportCard";
import { useScan } from "./lib/useScan";
import type {
  DisplayInfo,
  StorageInfo,
} from "./lib/types";

function formatMemory(mb: number, locale: string): string {
  return new Intl.NumberFormat(locale === "bn" ? "bn-BD" : "en-US", {
    maximumFractionDigits: 0,
  }).format(mb);
}

function formatCapacity(bytes: number | undefined): string {
  if (!bytes) return "—";
  const gb = bytes / 1024 ** 3;
  return `${gb >= 100 ? Math.round(gb) : gb.toFixed(1)} GB`;
}

const VERDICT_KEY: Record<string, string> = {
  "good-buy": "score.verdictGoodBuy",
  negotiate: "score.verdictNegotiate",
  "walk-away": "score.verdictWalkAway",
};

export function App() {
  const { t, i18n } = useTranslation();
  const scan = useScan();
  const runScan = scan.run;

  // Auto-scan on launch so the window never looks empty.
  useEffect(() => {
    void runScan();
  }, [runScan]);

  const rescan = useCallback(() => void runScan(), [runScan]);
  const bn = i18n.language === "bn";

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

      <main className="mx-auto flex max-w-xl flex-col gap-5 px-6 pb-10">
        {/* Score card */}
        {scan.score != null && scan.verdict && (
          <section
            data-testid="score-card"
            className="rounded-2xl bg-white/10 p-6 text-center ring-1 ring-white/20"
          >
            <p className="text-xs uppercase tracking-widest text-white/60">
              {t("score.title")}
            </p>
            <p
              className={`mt-1 text-5xl font-bold ${
                scan.cappedByCritical || scan.verdict === "walk-away"
                  ? "text-red-400"
                  : scan.verdict === "negotiate"
                    ? "text-amber-300"
                    : "text-emerald-400"
              }`}
            >
              {scan.score.toFixed(1)}
              <span className="text-xl text-white/50">/10</span>
            </p>
            <p className="mt-1 font-medium">{t(VERDICT_KEY[scan.verdict])}</p>
            {scan.cappedByCritical && (
              <p className="mt-2 rounded-lg bg-red-500/20 px-3 py-1 text-xs text-red-200">
                {t("score.cappedNote")}
              </p>
            )}
            <div className="mt-4 flex justify-center gap-2">
              <ReportCard
                hardware={scan.hardware!}
                battery={scan.battery}
                storage={scan.storage}
                score={scan.score}
                verdict={scan.verdict}
                flags={scan.flags}
              />
              <PrintButton />
            </div>
          </section>
        )}

        {/* Fraud flags */}
        {scan.flags.length > 0 && (
          <section
            data-testid="fraud-flags"
            className="overflow-hidden rounded-2xl ring-1 ring-white/10"
          >
            <p className="bg-white/10 px-5 py-2 text-xs uppercase tracking-widest text-white/70">
              {t("fraud.title")}
            </p>
            <ul className="divide-y divide-white/5 bg-white/5 text-sm">
              {scan.flags.map((f) => {
                const msgKey = f.message_key;
                const sevLabel = t(
                  f.severity === "critical"
                    ? "fraud.severityCritical"
                    : f.severity === "warning"
                      ? "fraud.severityWarning"
                      : "fraud.severityInfo",
                );
                return (
                  <li
                    key={f.check_id}
                    className={`flex items-start gap-3 px-5 py-3 ${
                      f.severity === "critical" ? "bg-red-500/10" : ""
                    }`}
                  >
                    <span
                      className={`mt-0.5 shrink-0 rounded px-2 py-0.5 text-[10px] font-bold ${
                        f.severity === "critical"
                          ? "bg-red-500/30 text-red-200"
                          : f.severity === "warning"
                            ? "bg-amber-500/30 text-amber-200"
                            : "bg-sky-500/30 text-sky-200"
                      }`}
                    >
                      {sevLabel}
                    </span>
                    <span>{t(msgKey) === msgKey ? msgKey.split(".").pop() : t(msgKey)}</span>
                  </li>
                );
              })}
            </ul>
          </section>
        )}

        <button
          type="button"
          data-testid="scan-button"
          onClick={rescan}
          disabled={scan.running}
          className="rounded-xl bg-emerald-500 px-4 py-3 font-semibold text-emerald-950 transition-colors hover:bg-emerald-400 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {scan.running ? t("scan.running") : t("home.scanButton")}
        </button>

        {/* Per-section errors — one failed module never hides the rest */}
        {scan.errors.map((e) => (
          <p
            key={e.section}
            role="alert"
            className="rounded-lg bg-amber-500/15 px-3 py-2 text-xs text-amber-200 ring-1 ring-amber-400/30"
          >
            {t("error.sectionFailed", {
              section: e.section,
              message: e.message,
            })}
          </p>
        ))}

        {scan.hardware && (
          <Section title={t("hostname") + ": " + scan.hardware.hostname}>
            <Row label={t("scan.cpu")} value={scan.hardware.cpu_name} testId="cpu-name" />
            {scan.hardware.cpu_cores_physical != null && (
              <Row
                label={t("scan.cpuCores")}
                value={fmtNum(scan.hardware.cpu_cores_physical, bn)}
              />
            )}
            <Row label={t("scan.threads")} value={fmtNum(scan.hardware.cpu_threads, bn)} />
            <Row
              label={t("scan.memory")}
              value={`${formatMemory(scan.hardware.total_memory_mb, i18n.language)} MB`}
              bn={bn}
            />
            {scan.hardware.motherboard && (
              <Row label={t("scan.motherboard")} value={scan.hardware.motherboard} />
            )}
            {scan.hardware.bios_vendor && (
              <Row
                label={t("scan.bios")}
                value={
                  scan.hardware.bios_version
                    ? `${scan.hardware.bios_vendor} ${scan.hardware.bios_version}`
                    : scan.hardware.bios_vendor
                }
              />
            )}
            {scan.hardware.gpus?.map((g) => (
              <Row key={g.name} label={t("scan.gpu")} value={g.name} />
            ))}
            <Row
              label={t("scan.os")}
              value={`${scan.hardware.os_name} ${scan.hardware.os_version}`}
            />
          </Section>
        )}

        {scan.battery && (
          <Section title={t("scan.batteryTitle")}>
            {scan.battery.health_percent != null && (
              <Row
                label={t("scan.batteryHealth")}
                value={`${fmtNum(Math.round(scan.battery.health_percent), bn)}%`}
                bn={bn}
              />
            )}
            {scan.battery.design_capacity_mwh != null && (
              <Row
                label={t("scan.batteryDesign")}
                value={`${fmtNum(Math.round(scan.battery.design_capacity_mwh / 1000), bn)} Wh`}
                bn={bn}
              />
            )}
            {scan.battery.cycle_count != null && (
              <Row
                label={t("scan.batteryCycles")}
                value={fmtNum(scan.battery.cycle_count, bn)}
                bn={bn}
              />
            )}
          </Section>
        )}

        {scan.storage?.map((d, i) => (
          <StorageCard key={`${d.model_name ?? "disk"}-${i}`} d={d} />
        ))}

        {scan.display
          ?.filter((d) => d.edid_valid)
          .map((d, i) => (
            <DisplayCard key={`display-${i}`} d={d} />
          ))}

        <p className="text-center text-xs text-white/40">
          {t("footer.phaseNote")}
        </p>
      </main>
    </div>
  );
}

function StorageCard({ d }: { d: StorageInfo }) {
  const { t } = useTranslation();
  return (
    <Section title={t("scan.storageTitle")}>
      {d.model_name && <Row label={t("scan.storageModel")} value={d.model_name} />}
      <Row
        label={t("scan.storageCapacity")}
        value={formatCapacity(d.total_capacity_bytes)}
      />
      {d.smart_status && (
        <Row label={t("scan.storageStatus")} value={d.smart_status} />
      )}
      {d.nvme_percentage_used != null && (
        <Row
          label={t("scan.storageUsedPct")}
          value={`${Math.round(d.nvme_percentage_used)}%`}
        />
      )}
      {d.power_on_hours != null && (
        <Row label={t("scan.storageHours")} value={String(d.power_on_hours)} />
      )}
    </Section>
  );
}

function DisplayCard({ d }: { d: DisplayInfo }) {
  const { t } = useTranslation();
  const resolution =
    d.horizontal_px && d.vertical_px
      ? `${d.horizontal_px} × ${d.vertical_px}`
      : undefined;
  return (
    <Section title={t("scan.displayTitle")}>
      {d.manufacturer && <Row label={t("scan.displayVendor")} value={d.manufacturer} />}
      {resolution && (
        <Row label={t("scan.displayResolution")} value={resolution} />
      )}
      {d.preferred_refresh_hz != null && (
        <Row
          label={t("scan.displayRefresh")}
          value={`${Math.round(d.preferred_refresh_hz)} Hz`}
        />
      )}
      {d.diagonal_cm != null && (
        <Row
          label={t("scan.displaySize")}
          value={`≈ ${Math.round(d.diagonal_cm / 2.54)}″`}
        />
      )}
      {d.manufacture_year != null && (
        <Row
          label={t("scan.displayMade")}
          value={
            d.manufacture_week && d.manufacture_week > 0
              ? `W${d.manufacture_week} ${d.manufacture_year}`
              : String(d.manufacture_year)
          }
        />
      )}
    </Section>
  );
}

function Section({
  title,
  children,
}: {
  title?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="overflow-hidden rounded-2xl ring-1 ring-white/10">
      {title && (
        <p className="bg-white/10 px-5 py-2 text-xs uppercase tracking-widest text-white/70">
          {title}
        </p>
      )}
      <dl className="divide-y divide-white/5 bg-white/5 text-sm">{children}</dl>
    </section>
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
        className="text-right font-medium"
        data-testid={testId}
      >
        {bn ? toBengaliDigits(value) : value}
      </dd>
    </div>
  );
}

function fmtNum(n: number, _bn: boolean): string {
  // Digits are localized by toBengaliDigits at render time.
  return new Intl.NumberFormat("en-US").format(n);
}

function toBengaliDigits(s: string): string {
  const map = "০১২৩৪৫৬৭৮৯";
  if (!/[0-9]/.test(s)) return s;
  return s.replace(/[0-9]/g, (d) => map[Number(d)]);
}

export default App;
