import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./en.json";
import bn from "./bn.json";

export const supportedLanguages = ["en", "bn"] as const;
export type Language = (typeof supportedLanguages)[number];

void i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    bn: { translation: bn },
  },
  lng: "en",
  fallbackLng: "en",
  interpolation: {
    escapeValue: false, // React already escapes
  },
});

export default i18n;
