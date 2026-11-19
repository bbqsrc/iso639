// A small test harness for Node, Bun, Deno and browsers. Nothing here has
// node: types, so deno check can check the suite without @types/node. Each
// scenario runs in a process of its own, because a module instance, once
// started, stays started for the life of the process.

import type * as Entry from "../iso639/index.ts";

/** What the package exports. */
export type Package = typeof Entry;

/** What `run.mjs` hands each scenario process, as JSON in its first argument. */
export interface ScenarioArgs {
  /** URL of the package's entry module. */
  module: string;
  /** `file:` URL of the package's `.wasm` file. */
  wasm: string;
  /** `http:` URL the runner serves the same `.wasm` file at. */
  http: string;
  /** The version the package declares. */
  version: string;
  /** Which variant of the scenario to run. */
  variant: string;
  /** Names the scenario and variant in the test output. */
  label: string;
}

const deno: unknown = Reflect.get(globalThis, "Deno");

/** The runtime this is, by the global each one alone defines. */
export const runtime =
  deno !== undefined
    ? "deno"
    : Reflect.has(globalThis, "Bun")
      ? "bun"
      : Reflect.has(globalThis, "process")
        ? "node"
        : "browser";

/** Deno hands a script its own arguments; Node and Bun put the runtime and script paths first. */
export function scenarioArgs(): ScenarioArgs {
  const argv: unknown =
    typeof deno === "object" && deno !== null
      ? Reflect.get(deno, "args")
      : Reflect.get(Reflect.get(globalThis, "process"), "argv");
  const own = runtime === "deno" ? argv : Array.isArray(argv) ? argv.slice(2) : undefined;
  if (!Array.isArray(own) || typeof own[0] !== "string") {
    throw new Error("a scenario takes its arguments as JSON in its first argument");
  }
  return JSON.parse(own[0]);
}

export async function loadPackage(module: string): Promise<Package> {
  return await import(module);
}

interface NodeFs {
  readFile(path: URL): Promise<Uint8Array<ArrayBuffer>>;
}

// A variable specifier keeps deno check from asking for node:fs types.
export async function readBytes(url: string | URL): Promise<Uint8Array<ArrayBuffer>> {
  const specifier = "node:fs/promises";
  const fs: NodeFs = await import(specifier);
  return await fs.readFile(new URL(url));
}

type Body = () => void | Promise<void>;

const tests: { name: string; body: Body }[] = [];

export function test(name: string, body: Body): void {
  tests.push({ name, body });
}

/** Runs the registered tests in order and returns a line for each, and whether all passed. */
export async function collect(label: string): Promise<{ lines: string[]; passed: boolean }> {
  const lines: string[] = [];
  let failed = 0;
  for (const { name, body } of tests) {
    try {
      await body();
      lines.push(`ok - ${label}: ${name}`);
    } catch (error) {
      failed += 1;
      lines.push(`not ok - ${label}: ${name}`);
      lines.push(error instanceof Error ? (error.stack ?? error.message) : String(error));
    }
  }
  if (failed > 0) lines.push(`${label}: ${failed} of ${tests.length} tests failed`);
  return { lines, passed: failed === 0 };
}

/** Runs the registered tests, prints what each did, and throws if any failed. */
export async function runTests(name: string): Promise<void> {
  const { lines, passed } = await collect(`${runtime} ${name}`);
  for (const line of lines) console.log(line);
  if (!passed) throw new Error(`${runtime} ${name} failed`);
}

export function assert(condition: boolean, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

export function assertEqual<T>(actual: T, expected: T, what: string): void {
  if (!Object.is(actual, expected)) {
    throw new Error(`${what}: expected ${String(expected)}, got ${String(actual)}`);
  }
}

/** Compares two plain values by their JSON. */
export function assertJson(actual: unknown, expected: unknown, what: string): void {
  const [a, e] = [JSON.stringify(actual), JSON.stringify(expected)];
  if (a !== e) throw new Error(`${what}: expected ${e}, got ${a}`);
}

/** The `Iso639Error` class under test: the package's export, or the module's own. */
export type ErrorClass = abstract new (...args: never[]) => Error & { code: string };

/** The `code` of the `Iso639Error` that `f` throws. Fails if it returns or throws anything else. */
export function thrownCode(errors: ErrorClass, f: () => unknown): string {
  try {
    f();
  } catch (error) {
    assert(error instanceof errors, `expected an Iso639Error, got ${String(error)}`);
    return error.code;
  }
  throw new Error("expected an Iso639Error, but nothing was thrown");
}

export async function rejectedCode(errors: ErrorClass, promise: Promise<unknown>): Promise<string> {
  try {
    await promise;
  } catch (error) {
    assert(error instanceof errors, `expected an Iso639Error, got ${String(error)}`);
    return error.code;
  }
  throw new Error("expected an Iso639Error rejection, but the promise resolved");
}
