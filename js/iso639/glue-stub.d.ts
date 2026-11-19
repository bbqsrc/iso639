// What deno check sees in place of pkg/iso639_wasm.js, which only exists after
// a build. The root deno.json maps that path here; a member's own import map
// would not do, because deno resolves a member's keys against the workspace
// root. tsc compiles against the real one.
import type { Glue } from "./glue.ts";

declare const init: Glue["default"];
export default init;
export declare const version: Glue["version"];
export declare const getAutonym: Glue["getAutonym"];
export declare const getScript: Glue["getScript"];
export declare const getLcid: Glue["getLcid"];
export declare const getByLcid: Glue["getByLcid"];
export declare const makePseudoLcid: Glue["makePseudoLcid"];
export declare const parsePseudoLcid: Glue["parsePseudoLcid"];
