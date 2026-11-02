import type { Messages } from "./fr";

/** Dotted paths to every string, e.g. `"auth.email"`. */
export type Key<T = Messages, P extends string = ""> = {
  [K in keyof T & string]: T[K] extends string ? `${P}${K}` : Key<T[K], `${P}${K}.`>;
}[keyof T & string];

export type Params = Record<string, string | number>;

export function lookup(messages: Messages, key: string): string | undefined {
  let node: unknown = messages;
  for (const part of key.split(".")) {
    if (node === null || typeof node !== "object") return undefined;
    node = (node as Record<string, unknown>)[part];
  }
  return typeof node === "string" ? node : undefined;
}

export function interpolate(template: string, params?: Params): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

export function pickLocale<L extends string>(preferred: readonly string[], available: readonly L[], fallback: L): L {
  for (const tag of preferred) {
    const base = tag.toLowerCase().split("-")[0];
    const found = available.find((l) => l === base);
    if (found) return found;
  }
  return fallback;
}
