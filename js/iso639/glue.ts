// The generated wasm-bindgen glue, as the wrapper uses it. tsc checks at
// build time that the generated module satisfies these types when binding.ts
// hands it to Core. Declaring them here also lets the unit tests run the
// wrapper on a fake glue, and lets deno check the wrapper on a checkout that
// has no build output.

/** What the generated init function is given. The wrapper resolves URLs itself. */
export type GlueInitInput = BufferSource | Response | WebAssembly.Module;

/** The exports of `iso639-wasm`, the module behind `@divvun/iso639`. */
export interface Glue {
  /** Instantiates the module. The generated glue keeps the instance it makes. */
  default(options: { module_or_path: GlueInitInput }): Promise<unknown>;
  version(): string;
  // Each record is a row, as rows.ts reads it, or undefined for none.
  getAutonym(tag: string): string | undefined;
  getScript(tag: string): string | undefined;
  getLcid(tag: string, script: string | null, region: string | null): string | undefined;
  getByLcid(lcid: number): string | undefined;
  /** Throws the string `invalid-tag` or `invalid-region` for a tag or region it cannot encode. */
  makePseudoLcid(tag: string, region: string | null): number;
  parsePseudoLcid(lcid: number): string | undefined;
}
