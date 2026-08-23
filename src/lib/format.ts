/**
 * Shared formatting helpers and the v1 verdict scale.
 * The scoring engine itself lives in Rust (src-tauri/src/scoring.rs, Phase 1);
 * this TS side mirrors the display thresholds so UI and core agree.
 */

export const VERDICT_THRESHOLDS = {
  negotiate: 5, // below → walk-away
  goodBuy: 7, // at/above → good-buy
} as const;

export type Verdict = "walk-away" | "negotiate" | "good-buy";

export function verdictForScore(score: number): Verdict {
  if (score >= VERDICT_THRESHOLDS.goodBuy) return "good-buy";
  if (score >= VERDICT_THRESHOLDS.negotiate) return "negotiate";
  return "walk-away";
}

export function formatMemory(mb: number, language: string): string {
  return new Intl.NumberFormat(language === "bn" ? "bn-BD" : "en-US", {
    maximumFractionDigits: 0,
  }).format(mb);
}
