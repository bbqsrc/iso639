// Builds the npm package @divvun/iso639 from iso639-wasm.
//
//   node js/build.mjs          the package
//   node js/build.mjs --test   the package, then the browser check under target/js-test
//
// This writes js/iso639/pkg (the wasm-bindgen output) and js/iso639/dist (the
// compiled wrapper, with a copy of pkg where the compiled binding imports it
// from, and the licences). It needs cargo with the wasm32-unknown-unknown
// target, the wasm-bindgen CLI at the version iso639-wasm pins, wasm-opt
// (binaryen) and npx.

import { execFileSync } from "node:child_process";
import { cpSync, readdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isPublicDeclaration, usesAny } from "./test/declarations.ts";

const js = dirname(fileURLToPath(import.meta.url));
const root = dirname(js);
const packageDir = join(js, "iso639");
const testBuild = join(root, "target", "js-test");
const CRATE = "iso639-wasm";

function run(command, args) {
  console.log(`$ ${command} ${args.join(" ")}`);
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function manifest() {
  return readFileSync(join(root, CRATE, "Cargo.toml"), "utf8");
}

// The CLI writes glue for its own version of the wasm-bindgen ABI, so it must
// be the version the crate links.
function assertBindgenVersion() {
  const installed = execFileSync("wasm-bindgen", ["--version"], { encoding: "utf8" }).trim().split(" ")[1];
  const pinned = /^wasm-bindgen = "=([^"]+)"$/m.exec(manifest())?.[1];
  if (pinned === undefined) throw new Error(`${CRATE}/Cargo.toml does not pin wasm-bindgen`);
  if (installed !== pinned) throw new Error(`the wasm-bindgen CLI is ${installed}; ${CRATE} pins ${pinned}`);
}

// version() reports the crate's version, so it must be the one npm publishes.
function assertPackageVersion() {
  const crate = /^version = "([^"]+)"$/m.exec(manifest())?.[1];
  const npm = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8")).version;
  if (crate !== npm) throw new Error(`js/iso639/package.json is ${npm}; ${CRATE} is ${crate}`);
}

function cargoBuild() {
  run("cargo", ["build", "--target", "wasm32-unknown-unknown", "--profile", "wasm-release", "-p", CRATE]);
  return join(root, "target", "wasm32-unknown-unknown", "wasm-release", `${CRATE.replaceAll("-", "_")}.wasm`);
}

function bindgen(wasm, outDir) {
  rmSync(outDir, { recursive: true, force: true });
  run("wasm-bindgen", ["--target", "web", "--out-dir", outDir, wasm]);
  return join(outDir, `${CRATE.replaceAll("-", "_")}_bg.wasm`);
}

// The features rustc enables by default for wasm32-unknown-unknown; wasm-opt
// refuses a module that uses one it was not told about.
const WASM_FEATURES = [
  "--enable-bulk-memory",
  "--enable-nontrapping-float-to-int",
  "--enable-sign-ext",
  "--enable-reference-types",
  "--enable-multivalue",
  "--enable-mutable-globals",
];

function tsc(project, outDir) {
  rmSync(outDir, { recursive: true, force: true });
  run("npx", ["-y", "-p", "typescript@5", "tsc", "-p", project, "--outDir", outDir]);
}

function* files(dir) {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) yield* files(path);
    else yield path;
  }
}

// The wrapper's declarations are the public API, so none of them may use any.
function assertNoAny(dist) {
  const offenders = [...files(dist)].filter(
    (file) => isPublicDeclaration(relative(dist, file)) && usesAny(readFileSync(file, "utf8")),
  );
  if (offenders.length > 0) throw new Error(`declarations use any: ${offenders.join(", ")}`);
}

/** Builds the package and returns the size of its .wasm file in bytes. */
function buildPackage() {
  assertBindgenVersion();
  assertPackageVersion();
  const pkgDir = join(packageDir, "pkg");
  const module = bindgen(cargoBuild(), pkgDir);
  run("wasm-opt", ["-Oz", ...WASM_FEATURES, module, "-o", module]);
  const dist = join(packageDir, "dist");
  tsc(join(packageDir, "tsconfig.json"), dist);
  cpSync(pkgDir, join(dist, "pkg"), { recursive: true });
  // The code is MIT or Apache-2.0; the tables compiled into the module are
  // iso639-databases', under CC0.
  cpSync(join(root, "LICENSE-MIT"), join(dist, "LICENSE-MIT"));
  cpSync(join(root, "LICENSE-APACHE"), join(dist, "LICENSE-APACHE"));
  cpSync(join(root, "iso639-databases", "LICENSE"), join(dist, "LICENSE-DATA"));
  assertNoAny(dist);
  return statSync(module).size;
}

// A browser runs no TypeScript, so the check it runs is compiled.
function buildTestPackage() {
  tsc(join(js, "test", "tsconfig.browser.json"), join(testBuild, "browser"));
}

if (process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const bytes = buildPackage();
  if (process.argv.includes("--test")) buildTestPackage();
  console.log(`@divvun/iso639: ${bytes} bytes of WebAssembly (${(bytes / 1024).toFixed(1)} KiB)`);
}
