use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let f = std::fs::File::open("./iso639-autonyms.tsv").unwrap();
    let path = Path::new(&std::env::var("OUT_DIR").unwrap()).join("autonym_db.rs");
    let mut db = BufWriter::new(File::create(&path).unwrap());

    let mut names = std::collections::BTreeMap::new();

    let mut tsv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(f);

    for result in tsv_reader.records() {
        let record = result.unwrap();

        let rec_name = format!("RECORD_{}", record[0].to_uppercase());
        let tag3 = &record[0];
        let tag1 = if record[1].trim() == "" {
            None
        } else {
            Some(&record[1])
        };
        let name = &record[2];
        let autonym = if record[3].trim() == "" {
            None
        } else {
            Some(&record[3])
        };
        let source = &record[4];

        writeln!(&mut db, "const {}: &'static Record = &Record {{", &rec_name).unwrap();
        names.insert(record[0].to_string(), rec_name.to_string());
        names.insert(record[1].to_string(), rec_name.to_string());
        writeln!(
            &mut db,
            "    tag3: {:?}, tag1: {:?}, name: {:?}, autonym: {:?}, source: {:?}",
            tag3, tag1, name, autonym, source
        )
        .unwrap();
        writeln!(&mut db, "}};").unwrap();
    }

    write!(
        &mut db,
        "static AUTONYMS: phf::Map<&'static str, &'static Record> = "
    )
    .unwrap();

    let mut map = phf_codegen::Map::new();

    for (key, value) in names.iter() {
        map.entry(&**key, value);
    }

    writeln!(&mut db, "{}", map.build()).unwrap();
    write!(&mut db, ";\n").unwrap();
}
