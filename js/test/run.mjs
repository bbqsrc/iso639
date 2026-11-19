// Runs the JavaScript test suite against the built package: the unit and
// package tests and the scenarios (starting the package from each kind of
// source, and the lookups) under Node, Bun and Deno, and the lookups in a
// headless Chromium on a page and in a Web Worker.
//
//   node js/build.mjs --test && node js/test/run.mjs [--runtime node|bun|deno|chromium]... [--require-all]
//
// A runtime that is not installed is skipped with a message. --require-all,
// or ISO639_JS_TEST_REQUIRE_ALL=1, fails the run instead, and fails it when
// --runtime leaves any of the four out. ISO639_JS_TEST_BROWSER names the
// browser to run.
//
// Each scenario variant runs in a process of its own, once per runtime,
// because a started module instance cannot be reset. Deno runs each with no
// config file, import map or lock file, read access to the package alone,
// and network access to the runner's own server, which serves the package's
// .wasm file so the suite can start the package from an http: URL.

import { spawn, spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { createServer } from "node:http";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { findBrowser, runInBrowser } from "./browser.mjs";

const js = dirname(dirname(fileURLToPath(import.meta.url)));
const packageDir = join(js, "iso639");
const dist = join(packageDir, "dist");
const wasm = join(dist, "pkg", "iso639_wasm_bg.wasm");
const compiledBrowserCheck = join(dirname(js), "target", "js-test", "browser", "test");

const RUNTIMES = ["node", "bun", "deno", "chromium"];

const scenarios = [
  {
    file: "init.ts",
    variants: ["default", "bytes", "file-url", "file-url-string", "http-url", "response", "module", "retry"],
    network: true,
  },
  { file: "lookups.ts", variants: ["main"] },
];

/** Unit and package tests: every *.test.ts outside build output. */
function unitTests(dir = packageDir) {
  const found = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (!["dist", "pkg", "node_modules"].includes(entry.name)) found.push(...unitTests(path));
    } else if (/\.test\.ts$/.test(entry.name)) {
      found.push(path);
    }
  }
  return found.sort();
}

function chosenRuntimes(argv) {
  const chosen = [];
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--runtime" && argv[i + 1] !== undefined) chosen.push(argv[(i += 1)]);
  }
  return chosen.length > 0 ? chosen : RUNTIMES;
}

function installed(runtime) {
  const probe = spawnSync(runtime, ["--version"], { stdio: "ignore" });
  return probe.error === undefined && probe.status === 0;
}

/** Serves the package's .wasm file at /iso639.wasm. */
function serve() {
  const server = createServer((request, response) => {
    if (request.url !== "/iso639.wasm") {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, { "content-type": "application/wasm" });
    response.end(readFileSync(wasm));
  });
  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () => resolve(server));
  });
}

function command(runtime, path, args, { readable, network }) {
  if (runtime !== "deno") return [runtime, [path, ...args]];
  if (readable.some((dir) => dir.includes(","))) {
    throw new Error(`deno takes --allow-read as a comma-separated list, so it cannot name ${readable.join(" or ")}`);
  }
  const flags = ["run", "--no-config", "--no-prompt"];
  if (readable.length > 0) flags.push(`--allow-read=${readable.join(",")}`);
  if (network) flags.push("--allow-net=127.0.0.1");
  return ["deno", [...flags, path, ...args]];
}

function runFile(runtime, path, args, access) {
  return new Promise((resolve) => {
    const [program, argv] = command(runtime, path, args, access);
    const child = spawn(program, argv, { stdio: ["ignore", "inherit", "inherit"] });
    child.on("error", (error) => {
      console.log(`not ok - ${runtime} ${path}: cannot start ${runtime}: ${error.message}`);
      resolve(false);
    });
    child.on("close", (code) => resolve(code === 0));
  });
}

const argv = process.argv.slice(2);
const required = argv.includes("--require-all") || process.env.ISO639_JS_TEST_REQUIRE_ALL === "1";
const runtimes = chosenRuntimes(argv);
if (!existsSync(wasm)) {
  console.log(`not ok - ${wasm} is missing: run node js/build.mjs --test first`);
  process.exit(1);
}
const { version } = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8"));
const server = await serve();
const http = `http://127.0.0.1:${server.address().port}/iso639.wasm`;
let failures = 0;
let runs = 0;

function unavailable(runtime, label, why) {
  if (required) {
    console.log(`not ok - ${runtime} ${label}: ${why}, and every runtime is required`);
    runs += 1;
    failures += 1;
  } else {
    console.log(`ok - ${runtime} ${label} # SKIP ${why} (--require-all makes this a failure)`);
  }
}

for (const runtime of RUNTIMES.filter((name) => !runtimes.includes(name))) {
  unavailable(runtime, "@divvun/iso639", "--runtime leaves it out");
}

for (const runtime of runtimes.filter((name) => name !== "chromium")) {
  if (!installed(runtime)) {
    unavailable(runtime, "@divvun/iso639", `${runtime} is not on PATH`);
    continue;
  }
  for (const path of unitTests()) {
    runs += 1;
    if (!(await runFile(runtime, path, [], { readable: [packageDir], network: false }))) failures += 1;
  }
  for (const { file, variants, network = false } of scenarios) {
    for (const variant of variants) {
      const args = {
        module: pathToFileURL(join(dist, "index.js")).href,
        wasm: pathToFileURL(wasm).href,
        http,
        version,
        variant,
        label: `@divvun/iso639 ${file} ${variant}`,
      };
      runs += 1;
      const passed = await runFile(runtime, join(js, "test", file), [JSON.stringify(args)], {
        readable: [dist],
        network,
      });
      if (!passed) failures += 1;
    }
  }
}

if (runtimes.includes("chromium")) {
  const variants = ["main-thread", "worker"];
  const label = (variant) => `chromium @divvun/iso639 browser.ts ${variant}`;
  const browser = findBrowser();
  if (browser === null) {
    for (const variant of variants) unavailable("chromium", label(variant), "no Chromium or Chrome is installed");
  } else if (!existsSync(join(compiledBrowserCheck, "browser.js"))) {
    for (const variant of variants) unavailable("chromium", label(variant), "the browser check is not built: run node js/build.mjs --test");
  } else {
    console.log(`# chromium: ${browser}`);
    const outcome = await runInBrowser(browser, {
      packageDir: dist,
      compiled: compiledBrowserCheck,
      version,
      label,
      variants,
    });
    runs += outcome.runs;
    failures += outcome.failures;
  }
}

server.close();
console.log(`${runs - failures} of ${runs} test files passed`);
if (failures > 0) process.exitCode = 1;
