import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { buildReport, summaryText } from "../lib/report";
import type {
  BatteryInfo,
  FullHardwareInfo,
  StorageInfo,
  Verdict,
} from "../lib/types";
import type { FraudFlag } from "../lib/useScan";

/**
 * Report card action: copies the ScanReport plain-text summary to the
 * clipboard for quick sharing (Messenger/WhatsApp). PDF/print comes next.
 */
export function ReportCard({
  hardware,
  battery,
  storage,
  score,
  verdict,
  flags,
}: {
  hardware: FullHardwareInfo;
  battery: BatteryInfo | null;
  storage: StorageInfo[] | null;
  score: number;
  verdict: Verdict;
  flags: FraudFlag[];
}) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  const onCopy = useCallback(async () => {
    const report = buildReport({
      hardware,
      battery,
      storage,
      score,
      verdict,
      flags,
    });
    const text = summaryText(report, (k) => t(k));
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard API can be denied in some webview contexts; fall back to
      // the legacy path so the button always does something useful.
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }
  }, [hardware, battery, storage, score, verdict, flags, t]);

  return (
    <button
      type="button"
      onClick={() => void onCopy()}
      className="rounded-xl bg-white/10 px-4 py-2 text-sm font-medium text-white ring-1 ring-white/20 transition-colors hover:bg-white/15"
      data-testid="copy-report"
    >
      {copied ? t("report.copied") : t("report.copy")}
    </button>
  );
}
