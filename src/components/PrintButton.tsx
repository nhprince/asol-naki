import { useTranslation } from "react-i18next";
import { useCallback } from "react";

/**
 * "Save as PDF" via the WebView2 print pipeline — no extra dependency
 * (plan.md §9: try webview print-to-PDF first). Opens the native print
 * dialog where the user picks "Microsoft Print to PDF".
 *
 * Print styling: a dedicated stylesheet hides chrome and lays out the
 * report on white paper (see print.css imported in index.css).
 */
export function PrintButton() {
  const { t } = useTranslation();

  const onPrint = useCallback(() => {
    document.body.classList.add("printing");
    // Give the class a tick to apply before the dialog snapshots layout.
    setTimeout(() => {
      window.print();
      document.body.classList.remove("printing");
    }, 50);
  }, []);

  return (
    <button
      type="button"
      onClick={onPrint}
      className="rounded-xl bg-white/10 px-4 py-2 text-sm font-medium text-white ring-1 ring-white/20 transition-colors hover:bg-white/15"
      data-testid="print-report"
    >
      {t("report.print")}
    </button>
  );
}
