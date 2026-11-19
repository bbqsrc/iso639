// Checks the npm package must pass, run against the built package directory.

import { isPublicDeclaration, usesAny } from "../test/declarations.ts";
import { assert, assertEqual, readBytes, runTests, test } from "../test/harness.ts";

/** The fields of package.json these checks read. */
interface Manifest {
  name: string;
  version: string;
  license: string;
  type: string;
  files: string[];
  exports: Record<string, { types: string; default: string }>;
}

interface NodeFs {
  readdir(path: URL, options: { recursive: true }): Promise<string[]>;
}

const dir = new URL("./", import.meta.url);
const dist = new URL("dist/", dir);

async function readText(url: URL): Promise<string> {
  return new TextDecoder().decode(await readBytes(url));
}

async function readManifest(): Promise<Manifest> {
  return JSON.parse(await readText(new URL("package.json", dir)));
}

async function listFiles(dir: URL): Promise<string[]> {
  const specifier = "node:fs/promises";
  const fs: NodeFs = await import(specifier);
  return await fs.readdir(dir, { recursive: true });
}

test("package.json names the package, its licence and its one entry point", async () => {
  const manifest = await readManifest();
  assertEqual(manifest.name, "@divvun/iso639", "name");
  assert(/^\d+\.\d+\.\d+$/.test(manifest.version), `a release version: ${manifest.version}`);
  assertEqual(manifest.license, "MIT OR Apache-2.0", "license");
  assertEqual(manifest.type, "module", "type");
  assertEqual(manifest.files.join(" "), "dist", "files");
  assertEqual(Object.keys(manifest.exports).join(" "), ".", "entry points");
});

test("the entry module and its declarations are where the exports map says", async () => {
  const files = await listFiles(dir);
  for (const path of Object.values((await readManifest()).exports).flatMap((e) => [e.default, e.types])) {
    assert(files.includes(path.replace(/^\.\//, "")), `${path} exists`);
  }
});

test("the package has one .wasm file, which the binding names beside it", async () => {
  const wasm = (await listFiles(dist)).filter((file) => file.endsWith(".wasm"));
  assertEqual(wasm.join(" "), "pkg/iso639_wasm_bg.wasm", ".wasm files");
  const binding = await readText(new URL("binding.js", dist));
  assert(binding.includes('new URL("./pkg/iso639_wasm_bg.wasm", import.meta.url)'), "binding.js names it");
});

test("the package carries the code's licences and the data's", async () => {
  assert((await readText(new URL("LICENSE-MIT", dist))).includes("Permission is hereby granted"), "MIT");
  assert((await readText(new URL("LICENSE-APACHE", dist))).includes("Version 2.0, January 2004"), "Apache-2.0");
  assert((await readText(new URL("LICENSE-DATA", dist))).includes("CC0 1.0 Universal"), "the data's CC0");
});

test("the package ships a README naming it", async () => {
  assert((await readText(new URL("README.md", dir))).startsWith("# @divvun/iso639\n"), "README.md is the package's");
});

test("no declaration of the public API uses any", async () => {
  const declarations = (await listFiles(dist)).filter(isPublicDeclaration);
  assert(declarations.length > 0, "the package has declarations");
  for (const path of declarations) {
    assert(!usesAny(await readText(new URL(path, dist))), `${path} uses any`);
  }
});

test("the entry module exports the API and nothing else", async () => {
  const entry: Record<string, unknown> = await import(new URL("index.js", dist).href);
  const names = Object.keys(entry).sort().join(" ");
  assertEqual(
    names,
    "Iso639Error getAutonym getByLcid getLcid getScript init makePseudoLcid parsePseudoLcid version",
    "exports",
  );
});

await runTests("iso639/package");
