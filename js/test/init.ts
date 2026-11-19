// Starting the package from each kind of source init() takes. One variant per
// process, since a started instance stays started. run.mjs runs each under
// Node, Bun and Deno.

import {
  assert,
  assertEqual,
  loadPackage,
  readBytes,
  rejectedCode,
  runTests,
  scenarioArgs,
  test,
  thrownCode,
} from "./harness.ts";

const args = scenarioArgs();
const pkg = await loadPackage(args.module);

async function source(): Promise<Parameters<typeof pkg.init>[0]> {
  switch (args.variant) {
    case "default":
      return undefined;
    case "bytes":
      return await readBytes(args.wasm);
    case "file-url":
      return new URL(args.wasm);
    case "file-url-string":
      return args.wasm;
    case "http-url":
      return args.http;
    case "response":
      return new Response(await readBytes(args.wasm), { headers: { "content-type": "application/wasm" } });
    case "module":
      return await WebAssembly.compile(await readBytes(args.wasm));
    default:
      throw new Error(`no init variant ${args.variant}`);
  }
}

test("a call before init() throws not-initialised", () => {
  assertEqual(thrownCode(pkg.Iso639Error, () => pkg.getAutonym("se")), "not-initialised", "code");
});

if (args.variant === "retry") {
  test("a start that fails rejects with init-failed, and the next init() tries again", async () => {
    const missing = new URL("missing.wasm", args.wasm);
    assertEqual(await rejectedCode(pkg.Iso639Error, pkg.init(missing)), "init-failed", "missing file");
    const notFound = new Response("no", { status: 404 });
    assertEqual(await rejectedCode(pkg.Iso639Error, pkg.init(notFound)), "init-failed", "HTTP 404");
    const garbage = new Uint8Array([0, 1, 2, 3]);
    assertEqual(await rejectedCode(pkg.Iso639Error, pkg.init(garbage)), "init-failed", "not a module");
    assertEqual(thrownCode(pkg.Iso639Error, () => pkg.version()), "not-initialised", "still not started");
    await pkg.init();
    assertEqual(pkg.getAutonym("se")?.displayName, "Davvisámegiella", "se");
  });
} else {
  const what: Record<string, string> = {
    default: "its own .wasm file",
    bytes: "the module's bytes",
    "file-url": "a file: URL",
    "file-url-string": "a file: URL as a string",
    "http-url": "an http: URL as a string",
    response: "a Response",
    module: "a WebAssembly.Module",
  };
  test(`init() starts the package from ${what[args.variant] ?? args.variant}`, async () => {
    const first = pkg.init(await source());
    assert(pkg.init() === first, "a second init() returns the first one's promise");
    await first;
    assertEqual(pkg.version(), args.version, "version");
    assertEqual(pkg.getAutonym("se")?.displayName, "Davvisámegiella", "se");
  });
}

await runTests(args.label);
