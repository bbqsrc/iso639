//! Every row of every table, found through the public API with the values the
//! databases give it.

use iso639::{autonym, lcid, script};

/// The rows of a table in iso639-databases, as maps from header to field.
fn rows(name: &str) -> Vec<std::collections::HashMap<String, String>> {
    let path = format!("{}/iso639-databases/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    let headers: Vec<_> = lines.next().unwrap().split('\t').collect();
    lines
        .map(|line| headers.iter().map(|h| h.to_string()).zip(line.split('\t').map(String::from)).collect())
        .collect()
}

fn opt(value: &str) -> Option<&str> {
    Some(value).filter(|v| !v.is_empty())
}

fn tag(row: &std::collections::HashMap<String, String>) -> String {
    ["tag3", "script", "region"]
        .iter()
        .filter_map(|c| opt(&row[*c]))
        .collect::<Vec<_>>()
        .join("-")
}

#[test]
fn every_autonym_row() {
    let rows = rows("bcp47-autonyms.tsv");
    assert!(rows.len() > 8000);
    for row in &rows {
        let r = autonym::get(&tag(row)).unwrap_or_else(|| panic!("{} is not found", tag(row)));
        assert_eq!(
            (r.tag3, r.tag1, r.script, r.region),
            (&*row["tag3"], opt(&row["tag1"]), opt(&row["script"]), opt(&row["region"])),
            "{}",
            tag(row)
        );
        assert_eq!(
            (r.common_name, r.autonym, r.source),
            (&*row["common_name"], opt(&row["autonym"]), &*row["source"]),
            "{}",
            tag(row)
        );
        if let Some(tag1) = r.tag1 {
            let by_tag1 = tag(row).replacen(r.tag3, tag1, 1);
            assert_eq!(autonym::get(&by_tag1), Some(r), "{by_tag1}");
        }
    }
}

#[test]
fn every_default_script_row() {
    for row in &rows("iso639-default-script.tsv") {
        let r = script::get(&row["tag3"]).unwrap_or_else(|| panic!("{} is not found", row["tag3"]));
        assert_eq!(
            (r.tag3, r.tag1, r.script, r.source),
            (&*row["tag3"], opt(&row["tag1"]), &*row["script"], &*row["source"])
        );
    }
}

#[test]
fn every_lcid_row() {
    for row in &rows("iso639-lcids.tsv") {
        let lcid: u32 = row["lcid"].parse().unwrap();
        let expected = (&*row["tag3"], opt(&row["tag1"]), opt(&row["script"]), opt(&row["region"]), lcid);
        let by_lcid = lcid::get_by_lcid(lcid).unwrap();
        assert_eq!(by_lcid.lcid, lcid);
        // lcid::get takes a region as BCP 47 writes one, which ES-valencia is not.
        if row["region"] != row["region"].to_ascii_uppercase() {
            continue;
        }
        let r = lcid::get(&row["tag3"], opt(&row["script"]), opt(&row["region"]))
            .unwrap_or_else(|| panic!("{expected:?} is not found"));
        assert_eq!((r.tag3, r.tag1, r.script, r.region, r.lcid), expected);
    }
}
