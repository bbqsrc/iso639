// The check that no declaration of the public API uses any. js/build.mjs runs
// it on every build, and the package tests run it again on what was built. It
// takes the declarations' text, so it needs no node: types and deno check can
// check it.

/** Whether the TypeScript declarations in `text` use `any` outside comments. */
export function usesAny(text: string): boolean {
  const code = text.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, "");
  return /\bany\b/.test(code);
}

/**
 * Whether the file at `path`, relative to the package's `dist`, is one of the
 * wrapper's declarations. The generated glue under `pkg` is internal and is
 * not checked.
 */
export function isPublicDeclaration(path: string): boolean {
  return path.endsWith(".d.ts") && !path.split(/[\\/]/).includes("pkg");
}
