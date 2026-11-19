// The lookups the package must answer as the iso639 crate does (tests/*.rs),
// against a started package. lookups.ts runs them in Node, Bun and Deno, and
// browser.ts on a page and in a Web Worker.

import { assert, assertEqual, assertJson, type Package, test, thrownCode } from "./harness.ts";

// Pseudo-LCIDs are meant to be constant, so these values must never change.
const KNOWN_PSEUDO_LCIDS: [string, string | undefined, number][] = [
  ["sje", undefined, 2147496054],
  ["fkv", "NO", 2517634637],
  ["rmn", "SE", 2643471913],
  ["sjd", "RU", 2632986741],
  ["sje", "150", 2304815222],
];

/** Registers the lookup tests against `pkg`, which `version` must be the version of. */
export function lookupTests(pkg: Package, version: string): void {
  const name = (tag: string) => pkg.getAutonym(tag)?.displayName ?? null;

  test("version() is the package's", () => {
    assertEqual(pkg.version(), version, "version");
  });

  test("autonyms by language, in either code and any case", () => {
    assertEqual(name("se"), "Davvisámegiella", "se");
    assertEqual(name("sme"), "Davvisámegiella", "sme");
    assertEqual(name("SE"), "Davvisámegiella", "SE");
    assertEqual(name("zz"), null, "zz");
    assertEqual(name(""), null, "the empty tag");
  });

  test("autonyms by script, with the default script as no script", () => {
    assertEqual(name("sr"), "Српски", "sr");
    assertEqual(name("sr-Latn"), "Srpski", "sr-Latn");
    assertEqual(name("sr_latn"), "Srpski", "sr_latn");
    assertEqual(name("sr-Cyrl"), "Српски", "sr-Cyrl");
    assertEqual(name("ikt-Cans"), "ᐃᓄᐃᓐᓇᖅᑐᓐ", "ikt-Cans");
    assertEqual(name("ikt-Latn"), "Inuinnaqtun", "ikt-Latn");
  });

  test("autonyms fall back by region, then script", () => {
    assertEqual(name("sr-Latn-RS"), "Srpski", "sr-Latn-RS");
    assertEqual(name("se-FI"), "Davvisámegiella", "se-FI");
    assertEqual(name("se-Cyrl"), "Davvisámegiella", "se-Cyrl");
    assertEqual(name("se-FI-x-private"), "Davvisámegiella", "se-FI-x-private");
    assertEqual(pkg.getAutonym("zh-yue")?.tag3, "yue", "an extended language subtag");
  });

  test("an autonym record is a plain object with every field", () => {
    const record = pkg.getAutonym("sr-Latn");
    assert(record !== null, "sr-Latn has a row");
    assertEqual(Object.getPrototypeOf(record), Object.prototype, "prototype");
    assertJson(
      record,
      {
        tag3: "srp",
        tag1: "sr",
        script: "Latn",
        region: null,
        commonName: "Serbian",
        autonym: "Srpski",
        displayName: "Srpski",
        source: "cldr",
      },
      "sr-Latn",
    );
  });

  test("every kind of record has each field, with null for a field the row leaves empty", () => {
    assertJson(
      pkg.getAutonym("af"),
      {
        tag3: "afr",
        tag1: "af",
        script: null,
        region: null,
        commonName: "Afrikaans",
        autonym: null,
        displayName: "Afrikaans",
        source: "cldr",
      },
      "af, which has no separate autonym",
    );
    assertJson(pkg.getScript("crk"), { tag3: "crk", tag1: null, script: "Latn", source: "giellalt" }, "crk");
    assertJson(
      pkg.getLcid("sr-Latn-RS"),
      { tag3: "srp", tag1: "sr", script: "Latn", region: "RS", lcid: 9242 },
      "sr-Latn-RS",
    );
    assertJson(pkg.parsePseudoLcid(2147496054), { tag: "sje", region: null }, "a pseudo-LCID with no region");
  });

  test("default scripts", () => {
    assertEqual(pkg.getScript("sr")?.script, "Cyrl", "sr");
    assertEqual(pkg.getScript("crk")?.script, "Latn", "crk");
    assertEqual(pkg.getScript("sr-Latn-RS")?.script, "Cyrl", "sr-Latn-RS");
    assertEqual(pkg.getScript("zz"), null, "zz");
  });

  test("LCIDs, from the tag or from options", () => {
    const lcid = (tag: string, options?: { script?: string; region?: string }) =>
      pkg.getLcid(tag, options)?.lcid ?? null;
    assertEqual(lcid("se", { region: "NO" }), 1083, "se, NO");
    assertEqual(lcid("sme", { region: "no" }), 1083, "sme, no");
    assertEqual(lcid("se-NO"), 1083, "se-NO");
    assertEqual(lcid("smn", { region: "FI" }), 9275, "smn, FI");
    assertEqual(lcid("sr", { script: "Latn", region: "RS" }), 9242, "sr, Latn, RS");
    assertEqual(lcid("sr", { script: "latn", region: "rs" }), 9242, "sr, latn, rs");
    assertEqual(lcid("sr-Latn-RS"), 9242, "sr-Latn-RS");
    assertEqual(lcid("sje"), null, "sje");
    assertEqual(lcid("sr", { script: "" }), null, "an empty script");
    assertEqual(lcid("sr", { script: "é", region: "RS" }), null, "a script that is not ASCII");
    assertJson(pkg.getByLcid(1083), { tag3: "sme", tag1: "se", script: null, region: null, lcid: 1083 }, "1083");
    assertEqual(pkg.getByLcid(0), null, "LCID 0");
  });

  test("pseudo-LCIDs are the crate's, and parse back", () => {
    for (const [tag, region, lcid] of KNOWN_PSEUDO_LCIDS) {
      assertEqual(pkg.makePseudoLcid(tag, region), lcid, `${tag} ${String(region)}`);
      assertJson(pkg.parsePseudoLcid(lcid), { tag, region: region ?? null }, `parse ${lcid}`);
    }
    assertEqual(pkg.parsePseudoLcid(1083), null, "a real LCID");
  });

  test("a pseudo-LCID refuses a tag or region it cannot encode", () => {
    for (const region of ["²1", "1²", "", "N", "NOR", "1234", "١٢"]) {
      assertEqual(thrownCode(pkg.Iso639Error, () => pkg.makePseudoLcid("sme", region)), "invalid-region", region);
    }
    assertEqual(thrownCode(pkg.Iso639Error, () => pkg.makePseudoLcid("zz")), "invalid-tag", "zz");
  });

  test("arguments of the wrong type are refused before they reach the module", () => {
    const loose = pkg as unknown as Record<string, (...args: unknown[]) => unknown>;
    const refused = (f: string, ...args: unknown[]) =>
      thrownCode(pkg.Iso639Error, () => loose[f]?.(...args));
    for (const f of ["getAutonym", "getScript", "getLcid", "makePseudoLcid"]) {
      assertEqual(refused(f, 7), "invalid-argument", `${f}(7)`);
      assertEqual(refused(f, undefined), "invalid-argument", `${f}(undefined)`);
    }
    for (const lcid of [-1, 1.5, 2 ** 32, Number.NaN, "1083"]) {
      assertEqual(refused("getByLcid", lcid), "invalid-argument", `getByLcid(${String(lcid)})`);
      assertEqual(refused("parsePseudoLcid", lcid), "invalid-argument", `parsePseudoLcid(${String(lcid)})`);
    }
    assertEqual(refused("getLcid", "sr", { Region: "RS" }), "invalid-argument", "an unknown option");
    assertEqual(refused("getLcid", "sr", null), "invalid-argument", "null options");
    assertEqual(refused("getLcid", "sr", { script: 1 }), "invalid-argument", "a script that is not a string");
    assertEqual(refused("makePseudoLcid", "sme", 578), "invalid-argument", "a region that is not a string");
    assertEqual(pkg.getAutonym("se")?.tag3, "sme", "the module still answers");
  });
}
