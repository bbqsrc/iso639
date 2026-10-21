use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DATABASES: &str = "iso639-databases";

/// A TSV table whose fields are looked up by their header name.
struct Table {
    headers: csv::StringRecord,
    records: Vec<csv::StringRecord>,
}

impl Table {
    fn open(name: &str) -> Self {
        let path = Path::new(DATABASES).join(name);
        println!("cargo::rerun-if-changed={}", path.display());
        let file = File::open(&path).unwrap_or_else(|e| {
            panic!(
                "could not open {}: {e}. Run `git submodule update --init` to fetch the databases.",
                path.display()
            )
        });
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .from_reader(file);
        let headers = reader.headers().unwrap().clone();
        let records = reader.records().map(Result::unwrap).collect();
        Table { headers, records }
    }

    fn rows(&self) -> impl Iterator<Item = Row<'_>> {
        self.records.iter().map(|record| Row {
            table: self,
            record,
        })
    }
}

struct Row<'a> {
    table: &'a Table,
    record: &'a csv::StringRecord,
}

impl<'a> Row<'a> {
    fn get(&self, column: &str) -> &'a str {
        let index = self
            .table
            .headers
            .iter()
            .position(|h| h == column)
            .unwrap_or_else(|| panic!("missing column {column}"));
        &self.record[index]
    }

    fn opt(&self, column: &str) -> Option<&'a str> {
        Some(self.get(column)).filter(|v| !v.is_empty())
    }
}

/// The lookup key for a language, script and region, e.g. `srp-Latn` or `rmn-SE`.
fn key(tag3: &str, script: Option<&str>, region: Option<&str>) -> String {
    [Some(tag3), script, region]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("-")
}

fn out_file(name: &str) -> BufWriter<File> {
    let path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(name);
    BufWriter::new(File::create(path).unwrap())
}

fn build_autonym_db() {
    let table = Table::open("bcp47-autonyms.tsv");
    let mut autonyms = BTreeMap::new();
    let mut languages = BTreeMap::new();

    for row in table.rows() {
        let tag3 = row.get("tag3");
        let tag1 = row.opt("tag1");
        let script = row.opt("script");
        let region = row.opt("region");

        if script.is_none() && region.is_none() {
            languages.insert(tag3.to_string(), format!("{tag3:?}"));
            if let Some(tag1) = tag1 {
                languages.insert(tag1.to_string(), format!("{tag3:?}"));
            }
        }

        let record = format!(
            "&Record {{ tag3: {tag3:?}, tag1: {tag1:?}, script: {script:?}, region: {region:?}, \
             common_name: {:?}, autonym: {:?}, source: {:?} }}",
            row.get("common_name"),
            row.opt("autonym"),
            row.get("source"),
        );
        autonyms.insert(key(tag3, script, region), record);
    }

    let mut db = out_file("autonym_db.rs");
    let mut map = phf_codegen::Map::new();
    for (k, v) in &autonyms {
        map.entry(k.as_str(), v.as_str());
    }
    writeln!(
        db,
        "static AUTONYMS: phf::Map<&str, &Record> = {};",
        map.build()
    )
    .unwrap();

    let mut map = phf_codegen::Map::new();
    for (k, v) in &languages {
        map.entry(k.as_str(), v.as_str());
    }
    writeln!(
        db,
        "pub(crate) static LANGUAGES: phf::Map<&str, &str> = {};",
        map.build()
    )
    .unwrap();
}

fn build_script_db() {
    let table = Table::open("iso639-default-script.tsv");
    let mut scripts = BTreeMap::new();

    for row in table.rows() {
        let tag3 = row.get("tag3");
        let record = format!(
            "&Record {{ tag3: {tag3:?}, tag1: {:?}, script: {:?}, source: {:?} }}",
            row.opt("tag1"),
            row.get("script"),
            row.get("source"),
        );
        scripts.insert(tag3.to_string(), record);
    }

    let mut db = out_file("script_db.rs");
    let mut map = phf_codegen::Map::new();
    for (k, v) in &scripts {
        map.entry(k.as_str(), v.as_str());
    }
    writeln!(
        db,
        "static SCRIPTS: phf::Map<&str, &Record> = {};",
        map.build()
    )
    .unwrap();
}

fn build_lcid_db() {
    let table = Table::open("iso639-lcids.tsv");
    let mut records: Vec<_> = table
        .rows()
        .map(|row| {
            let lcid: u32 = row.get("lcid").parse().unwrap();
            (
                row.get("tag3"),
                row.opt("script"),
                row.opt("region"),
                row.opt("tag1"),
                lcid,
            )
        })
        .collect();
    records.sort();

    let mut db = out_file("lcid_db.rs");
    writeln!(db, "static LCIDS: &[Record] = &[").unwrap();
    for (tag3, script, region, tag1, lcid) in records {
        writeln!(
            db,
            "    Record {{ tag3: {tag3:?}, tag1: {tag1:?}, script: {script:?}, region: {region:?}, lcid: {lcid} }},"
        )
        .unwrap();
    }
    writeln!(db, "];").unwrap();
}

fn main() {
    build_autonym_db();
    build_script_db();
    build_lcid_db();
}
