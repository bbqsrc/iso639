import { Core } from "./core.ts";
import * as glue from "./pkg/iso639_wasm.js";

// The .wasm file ships in pkg/, beside the compiled entry module, so a URL
// relative to this module finds it in every runtime, and bundlers that know
// the new URL(..., import.meta.url) pattern carry it along.
/** The package's module instance. Making it loads nothing. */
export const core: Core = new Core(glue, new URL("./pkg/iso639_wasm_bg.wasm", import.meta.url));
