//! The wasm-bindgen glue behind `@divvun/iso639`. `js/iso639` wraps it and
//! `js/build.mjs` builds it.
//!
//! A record crosses as one string: its fields in a fixed order, joined by
//! tabs, with an empty field for none, which `js/iso639/rows.ts` splits back
//! into an object. Building the object from in here would cross the boundary
//! once a field, and a string crosses once. No field can hold a tab, since
//! every one comes from a TSV table, and no field that is there is empty. A
//! missing record is `undefined`.

use wasm_bindgen::prelude::*;

fn row<const N: usize>(fields: [Option<&str>; N]) -> String {
    let mut row = String::with_capacity(fields.iter().map(|f| f.map_or(0, str::len) + 1).sum());
    for (i, field) in fields.into_iter().enumerate() {
        if i > 0 {
            row.push('\t');
        }
        row.push_str(field.unwrap_or_default());
    }
    row
}

/// The version of `@divvun/iso639`.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// tag3, tag1, script, region, common name, autonym, source.
#[wasm_bindgen(js_name = getAutonym)]
pub fn get_autonym(tag: &str) -> Option<String> {
    iso639::autonym::get(tag).map(|r| {
        row([
            Some(r.tag3),
            r.tag1,
            r.script,
            r.region,
            Some(r.common_name),
            r.autonym,
            Some(r.source),
        ])
    })
}

/// tag3, tag1, script, source.
#[wasm_bindgen(js_name = getScript)]
pub fn get_script(tag: &str) -> Option<String> {
    iso639::script::get(tag).map(|r| row([Some(r.tag3), r.tag1, Some(r.script), Some(r.source)]))
}

/// tag3, tag1, script, region, LCID.
fn lcid_row(r: iso639::lcid::Record) -> String {
    row([Some(r.tag3), r.tag1, r.script, r.region, Some(&r.lcid.to_string())])
}

#[wasm_bindgen(js_name = getLcid)]
pub fn get_lcid(tag: &str, script: Option<String>, region: Option<String>) -> Option<String> {
    iso639::lcid::get(tag, script.as_deref(), region.as_deref()).map(lcid_row)
}

#[wasm_bindgen(js_name = getByLcid)]
pub fn get_by_lcid(lcid: u32) -> Option<String> {
    iso639::lcid::get_by_lcid(lcid).map(lcid_row)
}

/// Throws the code of the `Iso639Error` the wrapper turns it into.
#[wasm_bindgen(js_name = makePseudoLcid)]
pub fn make_pseudo_lcid(tag: &str, region: Option<String>) -> Result<u32, JsValue> {
    iso639::make_pseudo_lcid(tag, region.as_deref()).map_err(|error| {
        JsValue::from_str(match error {
            iso639::Error::InvalidTag => "invalid-tag",
            iso639::Error::InvalidRegion => "invalid-region",
        })
    })
}

/// tag, region.
#[wasm_bindgen(js_name = parsePseudoLcid)]
pub fn parse_pseudo_lcid(lcid: u32) -> Option<String> {
    let (tag, region) = iso639::parse_pseudo_lcid(lcid).ok()?;
    Some(row([Some(&tag), region.as_deref()]))
}
