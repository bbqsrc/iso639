// The lookups against the package as it ships, started from its default
// source. run.mjs runs this under Node, Bun and Deno.

import { lookupTests } from "./cases.ts";
import { loadPackage, runTests, scenarioArgs } from "./harness.ts";

const args = scenarioArgs();
const pkg = await loadPackage(args.module);
await pkg.init();
lookupTests(pkg, args.version);
await runTests(args.label);
