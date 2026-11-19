// The records as the module hands them over: one string each, the fields in
// the order iso639-wasm/src/lib.rs gives, joined by tabs, with an empty field
// for none. Building the object in the module would cross the boundary once
// a field, and a string crosses once.

import type { AutonymRecord, LcidRecord, PseudoLcid, ScriptRecord } from "./records.ts";

function nullable(field: string): string | null {
  return field === "" ? null : field;
}

export function autonymRecord(row: string): AutonymRecord {
  const [tag3 = "", tag1 = "", script = "", region = "", commonName = "", autonym = "", source = ""] = row.split("\t");
  return {
    tag3,
    tag1: nullable(tag1),
    script: nullable(script),
    region: nullable(region),
    commonName,
    autonym: nullable(autonym),
    displayName: autonym === "" ? commonName : autonym,
    source,
  };
}

export function scriptRecord(row: string): ScriptRecord {
  const [tag3 = "", tag1 = "", script = "", source = ""] = row.split("\t");
  return { tag3, tag1: nullable(tag1), script, source };
}

export function lcidRecord(row: string): LcidRecord {
  const [tag3 = "", tag1 = "", script = "", region = "", lcid = ""] = row.split("\t");
  return { tag3, tag1: nullable(tag1), script: nullable(script), region: nullable(region), lcid: Number(lcid) };
}

export function pseudoLcid(row: string): PseudoLcid {
  const [tag = "", region = ""] = row.split("\t");
  return { tag, region: nullable(region) };
}
