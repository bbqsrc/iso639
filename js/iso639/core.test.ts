// Core on a fake glue: the start, the not-initialised check and poisoning,
// none of which need a build.

import { assert, assertEqual, rejectedCode, runTests, test, thrownCode } from "../test/harness.ts";
import { Core } from "./core.ts";
import { Iso639Error } from "./errors.ts";
import type { Glue } from "./glue.ts";

function fakeGlue(overrides: Partial<Glue> = {}): Glue & { starts: number } {
  const glue = {
    starts: 0,
    default: async () => {
      glue.starts += 1;
    },
    version: () => "0.0.0",
    getAutonym: () => undefined,
    getScript: () => undefined,
    getLcid: () => undefined,
    getByLcid: () => undefined,
    makePseudoLcid: () => 0,
    parsePseudoLcid: () => undefined,
    ...overrides,
  };
  return glue;
}

const BYTES = new Uint8Array([0, 97, 115, 109]);
const unused = new URL("https://example.invalid/iso639.wasm");

test("a call before init() throws not-initialised without entering the module", () => {
  let entered = false;
  const core = new Core(fakeGlue(), unused);
  const code = thrownCode(Iso639Error, () =>
    core.call(() => {
      entered = true;
    }),
  );
  assertEqual(code, "not-initialised", "code");
  assert(!entered, "the call did not run");
});

test("init() starts once, and every later call returns the first promise", async () => {
  const glue = fakeGlue();
  const core = new Core(glue, unused);
  const first = core.init(BYTES);
  assert(core.init() === first, "the second call returns the first promise");
  await first;
  assertEqual(glue.starts, 1, "starts");
  assertEqual(core.call((g) => g.version()), "0.0.0", "a call after the start");
});

test("a failed start rejects with init-failed, keeps its cause, and the next init() tries again", async () => {
  let fail = true;
  const glue = fakeGlue({
    default: async () => {
      if (fail) throw new Error("bad module");
    },
  });
  const core = new Core(glue, unused);
  const failed = core.init(BYTES);
  assertEqual(await rejectedCode(Iso639Error, failed), "init-failed", "code");
  const error = await failed.catch((e: unknown) => e);
  assert(error instanceof Iso639Error && error.cause instanceof Error, "the cause is kept");
  assert(error.message.includes("bad module"), `the message says why: ${error.message}`);
  fail = false;
  await core.init(BYTES);
  assertEqual(core.call((g) => g.version()), "0.0.0", "started on the second try");
});

test("a Response that is not ok fails the start", async () => {
  const core = new Core(fakeGlue(), unused);
  const code = await rejectedCode(Iso639Error, core.init(new Response("gone", { status: 410 })));
  assertEqual(code, "init-failed", "code");
});

test("a trap poisons the instance, and later calls throw poisoned without entering it", async () => {
  let entered = 0;
  const glue = fakeGlue({
    getAutonym: () => {
      entered += 1;
      throw new WebAssembly.RuntimeError("unreachable");
    },
  });
  const core = new Core(glue, unused);
  await core.init(BYTES);
  assertEqual(thrownCode(Iso639Error, () => core.call((g) => g.getAutonym("se"))), "poisoned", "the trapping call");
  assertEqual(thrownCode(Iso639Error, () => core.call((g) => g.version())), "poisoned", "a later call");
  assertEqual(entered, 1, "calls into the module");
});

test("an error that is not a trap passes through unchanged and does not poison", async () => {
  const core = new Core(fakeGlue(), unused);
  await core.init(BYTES);
  const thrown = new TypeError("not a trap");
  let caught: unknown;
  try {
    core.call(() => {
      throw thrown;
    });
  } catch (error) {
    caught = error;
  }
  assert(caught === thrown, "the same error");
  assertEqual(core.call((g) => g.version()), "0.0.0", "still usable");
});

await runTests("iso639/core");
