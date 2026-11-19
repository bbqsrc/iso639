/**
 * Every code an {@link Iso639Error} can carry. Code may test `error.code`
 * against these; they do not change between releases.
 *
 * - `not-initialised`: a call came before `init()` had resolved.
 * - `init-failed`: `init()` could not load, compile or start the module.
 * - `poisoned`: the module trapped. Its memory cannot be trusted after that,
 *   so every later call throws this without entering it.
 * - `invalid-argument`: an argument the package cannot use, such as a tag
 *   that is not a string, an LCID that is not a 32-bit unsigned integer, or
 *   an option it does not know.
 * - `invalid-tag`: `makePseudoLcid` was given a tag the databases have no
 *   language for.
 * - `invalid-region`: `makePseudoLcid` was given a region that is neither
 *   two letters nor three digits.
 */
export const ISO639_ERROR_CODES = [
  "not-initialised",
  "init-failed",
  "poisoned",
  "invalid-argument",
  "invalid-tag",
  "invalid-region",
] as const;

export type Iso639ErrorCode = (typeof ISO639_ERROR_CODES)[number];

/** An error from `@divvun/iso639`. `code` says what went wrong, in a form code can test. */
export class Iso639Error extends Error {
  readonly code: Iso639ErrorCode;

  constructor(code: Iso639ErrorCode, message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "Iso639Error";
    this.code = code;
  }
}

/** The message of whatever was thrown. */
export function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
