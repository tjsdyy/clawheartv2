import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zh from "@/locales/zh.json";
import en from "@/locales/en.json";
import ja from "@/locales/ja.json";
import ko from "@/locales/ko.json";
import de from "@/locales/de.json";
import fr from "@/locales/fr.json";
import es from "@/locales/es.json";
import pt from "@/locales/pt.json";
import ru from "@/locales/ru.json";
import it from "@/locales/it.json";

export const SUPPORTED_LANGUAGES = [
  { code: "zh", label: "中文" },
  { code: "en", label: "English" },
  { code: "ja", label: "日本語" },
  { code: "ko", label: "한국어" },
  { code: "de", label: "Deutsch" },
  { code: "fr", label: "Français" },
  { code: "es", label: "Español" },
  { code: "pt", label: "Português" },
  { code: "ru", label: "Русский" },
  { code: "it", label: "Italiano" },
];

const stored = localStorage.getItem("clawheart-lang");
const browserLang = navigator.language?.toLowerCase() ?? "";
// 浏览器语言匹配（按代码前缀）
const browserMatch = SUPPORTED_LANGUAGES.find((l) =>
  browserLang.startsWith(l.code),
)?.code;
const defaultLang = stored || browserMatch || "en";

i18n.use(initReactI18next).init({
  resources: {
    zh: { translation: zh },
    en: { translation: en },
    ja: { translation: ja },
    ko: { translation: ko },
    de: { translation: de },
    fr: { translation: fr },
    es: { translation: es },
    pt: { translation: pt },
    ru: { translation: ru },
    it: { translation: it },
  },
  lng: defaultLang,
  fallbackLng: "en",
  interpolation: { escapeValue: false },
});

export function setLanguage(lang: string) {
  i18n.changeLanguage(lang);
  localStorage.setItem("clawheart-lang", lang);
  if (typeof document !== "undefined") {
    document.documentElement.lang = lang;
  }
}

// 启动时初始化 HTML lang
if (typeof document !== "undefined") {
  document.documentElement.lang = defaultLang;
}

export default i18n;
