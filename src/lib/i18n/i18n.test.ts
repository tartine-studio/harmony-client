import { describe, expect, it } from "vitest";
import { en } from "./en";
import { fr } from "./fr";
import { interpolate, lookup, pickLocale } from "./translate";

function keys(obj: object, prefix = ""): string[] {
  return Object.entries(obj).flatMap(([k, v]) =>
    typeof v === "string" ? [`${prefix}${k}`] : keys(v as object, `${prefix}${k}.`),
  );
}

describe("i18n", () => {
  it("has the same keys in every locale", () => {
    expect(keys(en).sort()).toEqual(keys(fr).sort());
  });

  it("looks up dotted keys", () => {
    expect(lookup(fr, "auth.email")).toBe("E-mail");
    expect(lookup(fr, "auth.nope")).toBeUndefined();
    expect(lookup(fr, "auth")).toBeUndefined();
  });

  it("interpolates params and leaves unknown ones", () => {
    expect(interpolate("Message #{name}", { name: "général" })).toBe("Message #général");
    expect(interpolate("{count} / {max}", { count: 3 })).toBe("3 / {max}");
  });

  it("picks the best available locale", () => {
    expect(pickLocale(["en-US", "fr"], ["fr", "en"], "fr")).toBe("en");
    expect(pickLocale(["de-DE"], ["fr", "en"], "fr")).toBe("fr");
    expect(pickLocale(["FR-ca"], ["fr", "en"], "en")).toBe("fr");
  });
});
