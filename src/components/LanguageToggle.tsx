import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { supportedLanguages, type Language } from "../i18n";

const labels: Record<Language, string> = {
  en: "English",
  bn: "বাংলা",
};

export function LanguageToggle() {
  const { i18n } = useTranslation();

  const change = useCallback(
    (lng: Language) => {
      void i18n.changeLanguage(lng);
    },
    [i18n],
  );

  return (
    <div
      className="flex gap-1 rounded-lg bg-white/10 p-1"
      role="group"
      aria-label="Language"
    >
      {supportedLanguages.map((lng) => (
        <button
          key={lng}
          type="button"
          onClick={() => change(lng)}
          aria-pressed={i18n.language === lng}
          className={`rounded-md px-3 py-1 text-sm font-medium transition-colors ${
            i18n.language === lng
              ? "bg-white text-brand-900"
              : "text-white/80 hover:text-white"
          }`}
          data-lang={lng}
        >{labels[lng]}
        </button>
      ))}
    </div>
  );
}
