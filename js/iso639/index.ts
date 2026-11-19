/**
 * `@divvun/iso639`: autonyms, default scripts and Windows LCIDs for BCP 47
 * language tags, from
 * [iso639-databases](https://github.com/divvun/iso639-databases). It is the
 * Rust iso639 crate built as WebAssembly, and runs in browsers, Web Workers,
 * Deno, Node and Bun.
 *
 * Lookups take tags such as `se`, `sme`, `sr-Latn` or `rmn-SE`, in any case,
 * with `-` or `_` between subtags. A tag the databases have nothing for gives
 * `null`.
 *
 * @module
 */
import { core } from "./binding.ts";
import type { InitSource } from "./core.ts";
import { Iso639Error } from "./errors.ts";
import type { AutonymRecord, LcidOptions, LcidRecord, PseudoLcid, ScriptRecord } from "./records.ts";
import { autonymRecord, lcidRecord, pseudoLcid, scriptRecord } from "./rows.ts";

// This module declares the whole public API, and js/build.mjs fails a build
// whose declarations use any.

/**
 * Loads and starts the package's WebAssembly module. Call it once and wait
 * for it before anything else. With no argument it loads the `.wasm` file
 * that ships beside the package. Calling it again returns the same promise.
 */
export function init(source?: InitSource): Promise<void> {
  return core.init(source);
}

/** The package's version. The package is versioned on its own, apart from the iso639 crate it carries. */
export function version(): string {
  return core.call((glue) => glue.version());
}

/**
 * The autonym table's row for `tag`. Without a row for the exact tag, this
 * falls back by dropping the region, then the script. The language's default
 * script counts as no script, so `sr-Cyrl` finds the `sr` row.
 */
export function getAutonym(tag: string): AutonymRecord | null {
  const row = core.call((glue) => glue.getAutonym(checkTag(tag)));
  return row === undefined ? null : autonymRecord(row);
}

/** The default script of the language of `tag`, whatever script `tag` names. */
export function getScript(tag: string): ScriptRecord | null {
  const row = core.call((glue) => glue.getScript(checkTag(tag)));
  return row === undefined ? null : scriptRecord(row);
}

/**
 * The Windows LCID of `tag`. The table holds locales, so this looks up the
 * exact language, script and region, with no fallback. A `script` or `region`
 * in `options` replaces the one in `tag`: `getLcid("sr", { script: "Latn",
 * region: "RS" })` is `getLcid("sr-Latn-RS")`.
 */
export function getLcid(tag: string, options: LcidOptions = {}): LcidRecord | null {
  const row = core.call((glue) => {
    const { script, region } = checkLcidOptions(options);
    return glue.getLcid(checkTag(tag), script ?? null, region ?? null);
  });
  return row === undefined ? null : lcidRecord(row);
}

/** The LCID table's row for `lcid`. */
export function getByLcid(lcid: number): LcidRecord | null {
  const row = core.call((glue) => glue.getByLcid(checkLcid(lcid)));
  return row === undefined ? null : lcidRecord(row);
}

/**
 * A constant pseudo-LCID for the language of `tag` and `region`, for software
 * that wants an LCID for a locale Windows has none for. It is not an LCID as
 * [MS-LCID] defines them: its top bit is set. `region` is an ISO 3166-1 or UN
 * M.49 region, such as `NO` or `150`.
 *
 * Throws `invalid-tag` when the databases have no language for `tag`, and
 * `invalid-region` when `region` is neither two letters nor three digits.
 */
export function makePseudoLcid(tag: string, region?: string): number {
  return core.call((glue) => {
    checkTag(tag);
    if (region !== undefined) checkString(region, "a region");
    try {
      return glue.makePseudoLcid(tag, region ?? null);
    } catch (error) {
      if (error === "invalid-tag") {
        throw new Iso639Error("invalid-tag", `the databases have no language for ${JSON.stringify(tag)}`);
      }
      if (error === "invalid-region") {
        throw new Iso639Error(
          "invalid-region",
          `${JSON.stringify(region)} is not a region: give two letters or three digits`,
        );
      }
      throw error;
    }
  });
}

/** What the pseudo-LCID `lcid` encodes, or `null` if `lcid` is not one. */
export function parsePseudoLcid(lcid: number): PseudoLcid | null {
  const row = core.call((glue) => glue.parsePseudoLcid(checkLcid(lcid)));
  return row === undefined ? null : pseudoLcid(row);
}

function checkString(value: unknown, what: string): string {
  if (typeof value !== "string") {
    throw new Iso639Error("invalid-argument", `${what} is a string, not ${describe(value)}`);
  }
  return value;
}

function checkTag(tag: unknown): string {
  return checkString(tag, "a language tag");
}

function checkLcid(lcid: unknown): number {
  if (typeof lcid !== "number" || !Number.isInteger(lcid) || lcid < 0 || lcid > 0xffff_ffff) {
    throw new Iso639Error("invalid-argument", `an LCID is an integer from 0 to 2³² − 1, not ${describe(lcid)}`);
  }
  return lcid;
}

const LCID_OPTIONS = ["script", "region"];

function checkLcidOptions(options: unknown): LcidOptions {
  if (typeof options !== "object" || options === null) {
    throw new Iso639Error("invalid-argument", `getLcid's options are an object, not ${describe(options)}`);
  }
  for (const key of Object.keys(options)) {
    if (!LCID_OPTIONS.includes(key)) {
      throw new Iso639Error(
        "invalid-argument",
        `getLcid has no option ${JSON.stringify(key)}; it has ${LCID_OPTIONS.join(" and ")}`,
      );
    }
  }
  const { script, region } = options as Record<string, unknown>;
  if (script !== undefined) checkString(script, "a script");
  if (region !== undefined) checkString(region, "a region");
  return options;
}

function describe(value: unknown): string {
  if (value === null) return "null";
  if (typeof value === "string") return JSON.stringify(value);
  if (typeof value === "number") return String(value);
  return `a value of type ${typeof value}`;
}

export { Iso639Error } from "./errors.ts";
export type { Iso639ErrorCode } from "./errors.ts";
export type { InitSource } from "./core.ts";
export type { AutonymRecord, LcidOptions, LcidRecord, PseudoLcid, ScriptRecord } from "./records.ts";
