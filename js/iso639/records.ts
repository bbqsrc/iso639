// The records the lookups return, as plain objects rows.ts makes of the
// module's rows, so there is nothing to free.

/** A row of the autonym table, from `bcp47-autonyms.tsv`. */
export interface AutonymRecord {
  /** The ISO 639-3 or 639-5 code. */
  tag3: string;
  /** The ISO 639-1 code, if the language has one. */
  tag1: string | null;
  /** The ISO 15924 script, if the row is not in the language's default script. */
  script: string | null;
  /** The ISO 3166-1 or UN M.49 region, if the row only applies to that region. */
  region: string | null;
  /** What the speakers call their language in English. */
  commonName: string;
  /** What the speakers call their language in that language, if it differs from `commonName`. */
  autonym: string | null;
  /** `autonym`, or `commonName` if there is no separate autonym. */
  displayName: string;
  /** Where the row comes from, such as `cldr` or `ethnologue`. */
  source: string;
}

/** A row of the default script table, from `iso639-default-script.tsv`. */
export interface ScriptRecord {
  /** The ISO 639-3 or 639-5 code. */
  tag3: string;
  /** The ISO 639-1 code, if the language has one. */
  tag1: string | null;
  /** The ISO 15924 script the language is written in when a tag names none. */
  script: string;
  /** Where the row comes from. */
  source: string;
}

/** A row of the Windows LCID table, from `iso639-lcids.tsv`. */
export interface LcidRecord {
  /** The ISO 639-3 or 639-5 code. */
  tag3: string;
  /** The ISO 639-1 code, if the language has one. */
  tag1: string | null;
  /** The ISO 15924 script of the locale, if it names one. */
  script: string | null;
  /** The region of the locale, if it names one. */
  region: string | null;
  /** The LCID. */
  lcid: number;
}

/** What a pseudo-LCID encodes. */
export interface PseudoLcid {
  /** The ISO 639-3 code. */
  tag: string;
  /** The ISO 3166-1 or UN M.49 region, if it encodes one. */
  region: string | null;
}

/** The script and region `getLcid` looks up, in place of the ones in its tag. */
export interface LcidOptions {
  /** An ISO 15924 script, such as `Latn`. */
  script?: string;
  /** An ISO 3166-1 or UN M.49 region, such as `RS` or `419`. */
  region?: string;
}
