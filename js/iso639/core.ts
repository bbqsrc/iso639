import { Iso639Error, messageOf } from "./errors.ts";
import type { Glue, GlueInitInput } from "./glue.ts";

/**
 * Where `init()` loads the module from: a URL (a string is read as one), a
 * `Response`, the module's bytes, or a compiled `WebAssembly.Module`.
 */
export type InitSource = string | URL | Response | BufferSource | WebAssembly.Module;

/**
 * The WebAssembly module instance and the state the wrapper keeps about it.
 * The package makes exactly one, in binding.ts, and every call into the
 * module goes through it.
 */
export class Core {
  readonly #glue: Glue;
  readonly #defaultSource: URL;
  #starting: Promise<void> | null = null;
  #ready = false;
  /** Set by the first trap, and never cleared. */
  #poison: WebAssembly.RuntimeError | null = null;

  /** `defaultSource` is the package's own `.wasm` file. Nothing is loaded yet. */
  constructor(glue: Glue, defaultSource: URL) {
    this.#glue = glue;
    this.#defaultSource = defaultSource;
  }

  /**
   * Loads and starts the module. Every call after the first returns the first
   * call's promise, whatever its argument, so there is only ever one instance.
   * A start that fails leaves no instance, and the next call tries again.
   */
  init(source?: InitSource): Promise<void> {
    if (this.#starting === null) {
      const starting = this.#start(source ?? this.#defaultSource);
      this.#starting = starting;
      starting.catch(() => {
        if (this.#starting === starting) this.#starting = null;
      });
    }
    return this.#starting;
  }

  async #start(source: InitSource): Promise<void> {
    try {
      await this.#glue.default({ module_or_path: await load(source) });
    } catch (error) {
      throw new Iso639Error("init-failed", `cannot start @divvun/iso639: ${messageOf(error)}`, {
        cause: error,
      });
    }
    this.#ready = true;
  }

  /**
   * Runs `f` against the glue once `init()` has resolved. A trap poisons the
   * instance: that call and every later one throw `poisoned`, and no later
   * one runs `f`.
   */
  call<T>(f: (glue: Glue) => T): T {
    if (this.#poison !== null) throw poisoned(this.#poison);
    if (!this.#ready) {
      throw new Iso639Error(
        "not-initialised",
        "@divvun/iso639 is not initialised: call its init() and wait for it to resolve first",
      );
    }
    try {
      return f(this.#glue);
    } catch (error) {
      if (!(error instanceof WebAssembly.RuntimeError)) throw error;
      this.#poison = error;
      throw poisoned(error);
    }
  }
}

function poisoned(trap: WebAssembly.RuntimeError): Iso639Error {
  return new Iso639Error(
    "poisoned",
    `the module trapped: ${trap.message}. Load the page or process again to recover.`,
    { cause: trap },
  );
}

async function load(source: InitSource): Promise<GlueInitInput> {
  if (typeof source === "string" || source instanceof URL) {
    const url = typeof source === "string" && URL.canParse(source) ? new URL(source) : source;
    if (url instanceof URL && url.protocol === "file:") return await readFile(url);
    return checked(await fetch(url));
  }
  if (source instanceof Response) return checked(source);
  return source;
}

function checked(response: Response): Response {
  if (!response.ok) {
    throw new Error(`${response.url || "the response"} answered HTTP ${response.status}`);
  }
  return response;
}

/** The part of `node:fs/promises` that {@link readFile} uses. */
interface NodeFs {
  readFile(path: URL): Promise<Uint8Array<ArrayBuffer>>;
}

// Node's fetch cannot read file: URLs, so they are read with node:fs. The
// specifier is a variable so that bundlers and type checkers do not follow it:
// code that never passes a file URL never asks for node:fs.
async function readFile(url: URL): Promise<Uint8Array<ArrayBuffer>> {
  const specifier = "node:fs/promises";
  const fs: NodeFs = await import(/* @vite-ignore */ /* webpackIgnore: true */ specifier);
  return await fs.readFile(url);
}
