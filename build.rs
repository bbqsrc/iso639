//! Compiles the iso639-databases tables into `tables.rs`, as sorted columns of
//! small integers that index a handful of shared strings, so the tables cost
//! a few bytes a row rather than a pointer and a length a field. `src/lib.rs`
//! reads them; the layout is described where each one is written.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DATABASES: &str = "iso639-databases";

/// A TSV table whose fields are looked up by their header name.
struct Table {
    name: &'static str,
    headers: csv::StringRecord,
    records: Vec<csv::StringRecord>,
}

impl Table {
    fn open(name: &'static str) -> Self {
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
        Table {
            name,
            headers,
            records,
        }
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
            .unwrap_or_else(|| panic!("{} has no column {column}", self.table.name));
        &self.record[index]
    }

    fn opt(&self, column: &str) -> Option<&'a str> {
        Some(self.get(column)).filter(|v| !v.is_empty())
    }
}

/// The languages a tag can name: every ISO 639-3 code with a row of its own in
/// the autonym table, with no script or region. A language is its index in
/// the order of the codes.
struct Languages {
    tag1: BTreeMap<&'static str, Option<&'static str>>,
    index: HashMap<&'static str, u16>,
}

impl Languages {
    fn new(autonyms: &'static Table) -> Self {
        let mut tag1 = BTreeMap::new();
        for row in autonyms.rows() {
            if row.opt("script").is_none() && row.opt("region").is_none() {
                let (tag3, code) = (row.get("tag3"), row.opt("tag1"));
                assert!(is_code(tag3, 3), "{tag3} is not an ISO 639-3 code");
                assert!(code.is_none_or(|c| is_code(c, 2)), "{code:?} is not an ISO 639-1 code");
                tag1.insert(tag3, code);
            }
        }
        assert!(tag1.len() <= usize::from(u16::MAX), "too many languages for a u16");
        let index = tag1.keys().enumerate().map(|(i, t)| (*t, i as u16)).collect();
        Languages { tag1, index }
    }

    /// The language of a row, which must give the language's ISO 639-1 code.
    fn of(&self, row: &Row<'_>) -> Option<u16> {
        let tag3 = row.get("tag3");
        let index = *self.index.get(tag3)?;
        assert_eq!(
            row.opt("tag1"),
            self.tag1[tag3],
            "{} gives {tag3} another ISO 639-1 code than bcp47-autonyms.tsv",
            row.table.name
        );
        Some(index)
    }
}

fn is_code(code: &str, len: usize) -> bool {
    code.len() == len && code.bytes().all(|b| b.is_ascii_lowercase())
}

/// The distinct values of a column, in order. A row gives the index of its
/// value plus one, or 0 for none.
struct Values(Vec<&'static str>);

impl Values {
    fn new(values: impl IntoIterator<Item = &'static str>) -> Self {
        let values: BTreeSet<_> = values.into_iter().filter(|v| !v.is_empty()).collect();
        assert!(values.len() < usize::from(u8::MAX), "too many values for a u8");
        Values(values.into_iter().collect())
    }

    fn index(&self, value: Option<&str>) -> u8 {
        value.map_or(0, |v| self.0.iter().position(|x| *x == v).unwrap() as u8 + 1)
    }

    /// The index of a row's source, which every row must give.
    fn source(&self, row: &Row<'_>) -> u8 {
        let source = row.opt("source");
        assert!(source.is_some(), "{} has a row with no source for {}", row.table.name, row.get("tag3"));
        self.index(source) - 1
    }
}

/// A row of the autonym table, as `tables.rs` keeps it.
struct Autonym {
    common_name: &'static str,
    autonym: Option<&'static str>,
    source: u8,
}

fn key(language: u16, script: u8, region: u8) -> u32 {
    u32::from(language) << 16 | u32::from(script) << 8 | u32::from(region)
}

struct Out(BufWriter<File>);

impl Out {
    fn doc(&mut self, doc: &str) {
        for line in doc.lines() {
            writeln!(self.0, "/// {line}").unwrap();
        }
    }

    fn str(&mut self, name: &str, value: &str, doc: &str) {
        self.doc(doc);
        writeln!(self.0, "pub(crate) static {name}: &str = {value:?};\n").unwrap();
    }

    fn array<T: Display>(&mut self, name: &str, ty: &str, items: &[T], doc: &str) {
        self.doc(doc);
        writeln!(self.0, "pub(crate) static {name}: [{ty}; {}] = [", items.len()).unwrap();
        for chunk in items.chunks(16) {
            let line: Vec<_> = chunk.iter().map(ToString::to_string).collect();
            writeln!(self.0, "    {},", line.join(", ")).unwrap();
        }
        writeln!(self.0, "];\n").unwrap();
    }

    fn strs(&mut self, name: &str, values: &[&str], doc: &str) {
        let quoted: Vec<_> = values.iter().map(|v| format!("{v:?}")).collect();
        self.array(name, "&str", &quoted, doc);
    }
}

fn main() {
    // The tables live as long as the build script; leaking them lets every
    // column borrow its strings for 'static.
    let autonyms: &'static Table = Box::leak(Box::new(Table::open("bcp47-autonyms.tsv")));
    let scripts: &'static Table = Box::leak(Box::new(Table::open("iso639-default-script.tsv")));
    let lcids: &'static Table = Box::leak(Box::new(Table::open("iso639-lcids.tsv")));

    let languages = Languages::new(autonyms);
    let script_values = Values::new(
        [autonyms, scripts, lcids]
            .into_iter()
            .flat_map(|t| t.rows().map(|r| r.get("script"))),
    );
    let region_values = Values::new(
        [autonyms, lcids]
            .into_iter()
            .flat_map(|t| t.rows().map(|r| r.get("region"))),
    );
    let sources = Values::new(
        [autonyms, scripts]
            .into_iter()
            .flat_map(|t| t.rows().map(|r| r.get("source"))),
    );

    // A row for a language no tag can name is never found, so it is left out.
    // The LCID table is the exception: get_by_lcid finds every row.
    let mut own_rows: Vec<Option<Autonym>> = languages.tag1.keys().map(|_| None).collect();
    let mut variant_rows = BTreeMap::new();
    for row in autonyms.rows() {
        let Some(language) = languages.of(&row) else {
            continue;
        };
        let common_name = row.get("common_name");
        assert!(
            !common_name.is_empty() && common_name.len() <= 255,
            "{common_name:?} is empty or longer than 255 bytes"
        );
        let autonym = Autonym {
            common_name,
            autonym: row.opt("autonym"),
            source: sources.source(&row),
        };
        match (script_values.index(row.opt("script")), region_values.index(row.opt("region"))) {
            (0, 0) => own_rows[usize::from(language)] = Some(autonym),
            (script, region) => {
                variant_rows.insert(key(language, script, region), autonym);
            }
        }
    }
    // Every language has a row of its own, since that is what makes it one.
    let autonym_rows: Vec<&Autonym> = own_rows
        .iter()
        .map(|row| row.as_ref().unwrap())
        .chain(variant_rows.values())
        .collect();

    let mut default_scripts = BTreeMap::new();
    for row in scripts.rows() {
        let Some(language) = languages.of(&row) else {
            continue;
        };
        let script = script_values.index(Some(row.get("script")));
        assert_ne!(script, 0, "{} has no script for {}", scripts.name, row.get("tag3"));
        let source = sources.source(&row);
        default_scripts.insert(language, [script, source]);
    }

    let mut lcid_rows = BTreeMap::new();
    for row in lcids.rows() {
        let tag3 = row.get("tag3");
        let language = languages
            .of(&row)
            .unwrap_or_else(|| panic!("{} has {tag3}, which bcp47-autonyms.tsv has no row for", lcids.name));
        let script = script_values.index(row.opt("script"));
        let region = region_values.index(row.opt("region"));
        let lcid: u32 = row.get("lcid").parse().unwrap();
        let previous = lcid_rows.insert(key(language, script, region), lcid);
        assert!(previous.is_none(), "{} has two rows for {tag3} {:?} {:?}", lcids.name, row.opt("script"), row.opt("region"));
    }

    let path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("tables.rs");
    let mut out = Out(BufWriter::new(File::create(path).unwrap()));

    let tag3: String = languages.tag1.keys().copied().collect();
    out.str("TAG3", &tag3, "Every language's ISO 639-3 code, three bytes each. A language is its index here.");
    let tag1: Vec<(&str, u16)> = languages
        .tag1
        .iter()
        .enumerate()
        .filter_map(|(language, (_, code))| code.map(|c| (c, language as u16)))
        .collect();
    let codes: String = tag1.iter().map(|(c, _)| *c).collect();
    out.str("TAG1", &codes, "The ISO 639-1 codes, two bytes each, in the order of their languages.");
    let tag1_languages: Vec<u16> = tag1.iter().map(|(_, l)| *l).collect();
    out.array("TAG1_LANGUAGES", "u16", &tag1_languages, "The language of each code in [`TAG1`], in order.");
    let mut by_code: Vec<u8> = (0..u8::try_from(tag1.len()).unwrap()).collect();
    by_code.sort_by_key(|&i| tag1[usize::from(i)].0);
    out.array("TAG1_BY_CODE", "u8", &by_code, "The indices of the codes in [`TAG1`], in the order of the codes.");

    out.strs("SCRIPTS", &script_values.0, "The ISO 15924 scripts, in order. A row's script is its index here plus one, or 0 for none.");
    out.strs("REGIONS", &region_values.0, "The regions, in order. A row's region is its index here plus one, or 0 for none.");
    out.strs("SOURCES", &sources.0, "Where rows come from, in order. A row's source is its index here.");
    let keys = "Keys are a row's language shifted up 16 bits, its script shifted up 8 and its\nregion, in order.";
    let variant_keys: Vec<u32> = variant_rows.keys().copied().collect();
    out.array(
        "AUTONYM_VARIANT_KEYS",
        "u32",
        &variant_keys,
        &format!(
            "The autonym table's rows are each language's own, in the order of the\n\
             languages, then those with a script or region, which are found by these\n\
             keys. {keys}"
        ),
    );
    let mut names = String::new();
    let mut offsets = vec![];
    for row in &autonym_rows {
        offsets.push(u32::try_from(names.len()).unwrap());
        names.push_str(row.common_name);
        names.push_str(row.autonym.unwrap_or_default());
    }
    offsets.push(u32::try_from(names.len()).unwrap());
    out.str("NAMES", &names, "Each row's common name, then its autonym, in the order of the rows.");
    out.array(
        "NAME_OFFSETS",
        "u32",
        &offsets,
        "Where each row's names start in [`NAMES`]. They end where the next row's start.",
    );
    let lengths: Vec<usize> = autonym_rows.iter().map(|row| row.common_name.len()).collect();
    out.array(
        "COMMON_NAME_LENGTHS",
        "u8",
        &lengths,
        "The length of each row's common name in bytes. The rest of its names is its\nautonym, which is none if it is empty.",
    );
    let autonym_sources: Vec<u8> = autonym_rows.iter().map(|row| row.source).collect();
    out.array("AUTONYM_SOURCES", "u8", &autonym_sources, "Each row's source.");

    let script_languages: Vec<u16> = default_scripts.keys().copied().collect();
    out.array("DEFAULT_SCRIPT_LANGUAGES", "u16", &script_languages, "The languages the default script table has, in order.");
    let script_rows: Vec<String> = default_scripts.values().map(|v| format!("{v:?}")).collect();
    out.array("DEFAULT_SCRIPTS", "[u8; 2]", &script_rows, "Each language's default script and the row's source.");

    let lcid_keys: Vec<u32> = lcid_rows.keys().copied().collect();
    out.array("LCID_KEYS", "u32", &lcid_keys, &format!("The keys of the LCID table's rows. {keys}"));
    let lcid_values: Vec<u32> = lcid_rows.values().copied().collect();
    out.array("LCIDS", "u32", &lcid_values, "Each row's LCID.");
}
