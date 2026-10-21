use iso639::{autonym, lcid, script};

fn name(tag: &str) -> Option<&'static str> {
    autonym::get(tag).map(|r| r.display_name())
}

#[test]
fn autonym_by_language() {
    assert_eq!(name("se"), Some("Davvisámegiella"));
    assert_eq!(name("sme"), Some("Davvisámegiella"));
    assert_eq!(name("SE"), Some("Davvisámegiella"));
    assert_eq!(name("zz"), None);
    assert_eq!(name(""), None);
}

#[test]
fn autonym_by_script() {
    assert_eq!(name("sr"), Some("Српски"));
    assert_eq!(name("sr-Latn"), Some("Srpski"));
    assert_eq!(name("sr_latn"), Some("Srpski"));
    assert_eq!(name("ikt"), Some("Inuinnaqtun"));
    assert_eq!(name("ikt-Cans"), Some("ᐃᓄᐃᓐᓇᖅᑐᓐ"));
}

#[test]
fn autonym_default_script_is_no_script() {
    assert_eq!(name("sr-Cyrl"), Some("Српски"));
    assert_eq!(name("ikt-Latn"), Some("Inuinnaqtun"));
}

#[test]
fn autonym_falls_back() {
    assert_eq!(name("sr-Latn-RS"), Some("Srpski"));
    assert_eq!(name("se-FI"), Some("Davvisámegiella"));
    assert_eq!(name("se-Cyrl"), Some("Davvisámegiella"));
    assert_eq!(name("se-FI-x-private"), Some("Davvisámegiella"));
}

#[test]
fn autonym_extlang() {
    assert_eq!(autonym::get("zh-yue").map(|r| r.tag3), Some("yue"));
}

#[test]
fn autonym_record() {
    let r = autonym::get("sr-Latn").unwrap();
    assert_eq!(
        (r.tag3, r.tag1, r.script, r.region),
        ("srp", Some("sr"), Some("Latn"), None)
    );
    assert_eq!(r.common_name, "Serbian");
}

#[test]
fn default_script() {
    assert_eq!(script::get("sr").map(|r| r.script), Some("Cyrl"));
    assert_eq!(script::get("crk").map(|r| r.script), Some("Latn"));
    assert_eq!(script::get("sr-Latn-RS").map(|r| r.script), Some("Cyrl"));
}

#[test]
fn lcids() {
    let get = |tag, script, region| lcid::get(tag, script, region).map(|r| r.lcid);
    assert_eq!(get("se", None, Some("NO")), Some(1083));
    assert_eq!(get("sme", None, Some("no")), Some(1083));
    assert_eq!(get("se-NO", None, None), Some(1083));
    assert_eq!(get("smn", None, Some("FI")), Some(9275));
    assert_eq!(get("sr", Some("Latn"), Some("RS")), Some(9242));
    assert_eq!(get("sr-Latn-RS", None, None), Some(9242));
    assert_eq!(get("sje", None, None), None);
    assert_eq!(
        lcid::get_by_lcid(1083).map(|r| (r.tag3, r.region)),
        Some(("sme", None))
    );
}
