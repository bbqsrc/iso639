# @divvun/iso639

Autonyms, default scripts and Windows LCIDs for BCP 47 language tags, from
[iso639-databases](https://github.com/divvun/iso639-databases). It is the
[iso639](https://github.com/bbqsrc/iso639) Rust crate, built as WebAssembly,
and answers every lookup as the crate does. It runs in browsers, Web Workers,
Deno, Node 20 and later, and Bun. It is one ES module and one `.wasm` file
holding the tables, and loads nothing else.

```sh
npm install @divvun/iso639
```

## Looking up a tag

```js
import { getAutonym, getLcid, getScript, init } from "@divvun/iso639";

await init();

getAutonym("se")?.displayName; // "Davvisámegiella"
getAutonym("sr-Latn")?.displayName; // "Srpski"
getScript("sr")?.script; // "Cyrl"
getLcid("se-NO")?.lcid; // 1083
```

Call `init()` once and wait for it before anything else. A call that comes
first throws `Iso639Error` with code `not-initialised`.

Lookups take tags such as `se`, `sme`, `sr-Latn` or `rmn-SE`, in any case,
with `-` or `_` between subtags. An ISO 639-1 and an ISO 639-3 code find the
same language. An extended language subtag stands for the language, so
`zh-yue` is `yue`. Variants, extensions and private use subtags are ignored.
A tag the databases have nothing for gives `null`, which the `?.` above
allows for.

### Autonyms

`getAutonym(tag)` returns the autonym table's row for `tag`:

```js
getAutonym("sr-Latn");
// {
//   tag3: "srp",
//   tag1: "sr",
//   script: "Latn",
//   region: null,
//   commonName: "Serbian",
//   autonym: "Srpski",
//   displayName: "Srpski",
//   source: "cldr",
// }
```

`commonName` is what the speakers call their language in English, and
`autonym` what they call it in that language, or `null` when it is the same.
`displayName` is the autonym, or the common name when there is no separate
autonym, which is usually the field to show.

Without a row for the exact tag, the lookup falls back by dropping the region,
then the script. The language's default script counts as no script, so
`sr-Cyrl` finds the `sr` row and `se-FI` the `se` row.

### Default scripts

`getScript(tag)` returns the ISO 15924 script the language of `tag` is
written in when a tag names none, whatever script `tag` names:
`{ tag3, tag1, script, source }`.

### LCIDs

`getLcid(tag, options?)` returns the Windows LCID table's row for the locale:
`{ tag3, tag1, script, region, lcid }`. The table holds locales, so the lookup
is exact, with no fallback. A `script` or `region` in `options` replaces the
one in `tag`:

```js
getLcid("sr", { script: "Latn", region: "RS" })?.lcid; // 9242, as getLcid("sr-Latn-RS")
```

`getByLcid(lcid)` returns the row for an LCID.

### Pseudo-LCIDs

Some software wants an LCID for a locale Windows has none for.
`makePseudoLcid(tag, region?)` gives a constant one, with its top bit set, so
it is not an LCID as [MS-LCID] defines them. `region` is an ISO 3166-1 or UN
M.49 region. `parsePseudoLcid(lcid)` reads one back, or gives `null` for a
number that is not one:

```js
makePseudoLcid("fkv", "NO"); // 2517634637
parsePseudoLcid(2517634637); // { tag: "fkv", region: "NO" }
```

`makePseudoLcid` throws `invalid-tag` for a tag the databases have no
language for, and `invalid-region` for a region that is neither two letters
nor three digits.

## Starting the module

With no argument, `init()` loads the `.wasm` file that ships beside the
package, by a URL relative to the package's own module. That works unchanged
in Deno, Node and Bun, and in bundlers that follow
`new URL("…", import.meta.url)`, such as Vite and webpack 5. Wherever the
file ends up elsewhere, pass its location or contents:

```js
await init("/assets/iso639_wasm_bg.wasm"); // a URL, absolute or relative to the page
await init(new URL("./iso639.wasm", import.meta.url));
await init(await fetch("/assets/iso639_wasm_bg.wasm")); // a Response
await init(bytes); // an ArrayBuffer or typed array
await init(module); // a WebAssembly.Module
```

A `file:` URL is read from disk, an `http:` or `https:` one fetched. Calling
`init()` again returns the first call's promise, whatever its argument. A
start that fails rejects with `init-failed`, and the next `init()` tries
again.

### Deno

```js
import { getAutonym, init } from "npm:@divvun/iso639";

await init();
console.log(getAutonym("smn")?.displayName); // "Anarâškielâ"
```

```sh
deno run --allow-read main.ts
```

The default `init()` reads the `.wasm` file from disk, in Deno's npm cache or
in `node_modules`, so it needs read access to it: `--allow-read` gives it, or
name the package's directory to grant that alone. Given the module's bytes,
`init(bytes)` reads nothing.

## Errors

Every error the package throws is an `Iso639Error`, whose `code` is one of:

- `not-initialised`: a call came before `init()` had resolved.
- `init-failed`: `init()` could not load, compile or start the module. The
  error's `cause` is what went wrong.
- `invalid-argument`: an argument the package cannot use, such as a tag that
  is not a string, an LCID that is not an integer from 0 to 2³² − 1, or an
  option `getLcid` does not know.
- `invalid-tag` and `invalid-region`: from `makePseudoLcid`, as above.
- `poisoned`: the module trapped, and its memory cannot be trusted. Every
  later call throws this. Load the page or process again to recover.

## Versions

The package is versioned on its own, apart from the iso639 crate, and
`version()` gives its version. The tables are the iso639-databases the
release was built from.

## Licence

The code is MIT or Apache-2.0, at your option; `dist/LICENSE-MIT` and
`dist/LICENSE-APACHE` have the texts. The tables compiled into the module are
iso639-databases', which dedicates them to the public domain under CC0 1.0
(`dist/LICENSE-DATA`).

## Developing

The package is built from `iso639-wasm` in the
[iso639](https://github.com/bbqsrc/iso639) repository:

```sh
git submodule update --init
node js/build.mjs --test   # writes js/iso639/dist, and the browser check under target/js-test
node js/test/run.mjs       # Node, Bun, Deno and Chromium, each skipped if missing
deno check                 # from the repository root; needs no build
```

The build needs the `wasm32-unknown-unknown` Rust target, the `wasm-bindgen`
CLI at the version `iso639-wasm` pins, `wasm-opt` from binaryen, and Node 22.18
or later. `--require-all` makes a missing runtime fail the run instead.
