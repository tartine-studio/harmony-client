import { en } from "./en";
import { fr, type Messages } from "./fr";
import { interpolate, lookup, pickLocale, type Key, type Params } from "./translate";

export type { Key } from "./translate";

const catalogs = { fr, en } satisfies Record<string, Messages>;
export type Locale = keyof typeof catalogs;
export const locales = Object.keys(catalogs) as Locale[];

const detected =
  typeof navigator === "undefined" ? "fr" : pickLocale(navigator.languages ?? [navigator.language], locales, "fr");

const state = $state<{ locale: Locale }>({ locale: detected });

export function getLocale(): Locale {
  return state.locale;
}

export function setLocale(locale: Locale) {
  state.locale = locale;
  if (typeof document !== "undefined") document.documentElement.lang = locale;
}

/** Translates a key in the current locale. Reactive in templates. */
export function t(key: Key, params?: Params): string {
  return interpolate(lookup(catalogs[state.locale], key) ?? key, params);
}

/** Translates an error code, falling back to a generic message. */
export function errorText(code: string | undefined): string {
  if (code?.startsWith("HTTP_")) return t("errors.INVALID_RESPONSE");
  if (code === "INVALID_TOKEN" || code === "UNAUTHORIZED") return t("errors.NOT_AUTHENTICATED");
  const text = code ? lookup(catalogs[state.locale], `errors.${code}`) : undefined;
  return text ?? t("errors.unknown");
}

const timeFormats = new Map<string, Intl.DateTimeFormat>();

function format(options: Intl.DateTimeFormatOptions): Intl.DateTimeFormat {
  const id = state.locale + JSON.stringify(options);
  let f = timeFormats.get(id);
  if (!f) {
    f = new Intl.DateTimeFormat(state.locale, options);
    timeFormats.set(id, f);
  }
  return f;
}

export function formatTime(date: Date): string {
  return format({ hour: "2-digit", minute: "2-digit" }).format(date);
}

export function formatDay(date: Date): string {
  return format({ weekday: "long", day: "numeric", month: "long", year: "numeric" }).format(date);
}

export function formatFull(date: Date): string {
  return format({ dateStyle: "full", timeStyle: "short" }).format(date);
}
