#[derive(Debug, Clone, Copy)]
pub struct Record {
    pub tag3: &'static str,
    pub tag1: Option<&'static str>,
    pub name: &'static str,
    pub autonym: Option<&'static str>,
    pub source: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/autonym_db.rs"));

pub fn get(tag: &str) -> Option<&'static Record> {
    AUTONYMS.get(tag).map(|x| *x)
}
