pub mod autonym {
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
}

pub mod script {
    #[derive(Debug, Clone, Copy)]
    pub struct Record {
        pub tag3: &'static str,
        pub tag1: Option<&'static str>,
        pub script: &'static str,
        pub source: &'static str,
    }
    
    include!(concat!(env!("OUT_DIR"), "/script_db.rs"));
    
    pub fn get(tag: &str) -> Option<&'static Record> {
        SCRIPTS.get(tag).map(|x| *x)
    }
}

pub mod lcid {
    #[derive(Debug, Clone, Copy)]
    pub struct Record {
        pub tag3: &'static str,
        pub tag1: Option<&'static str>,
        pub script: Option<&'static str>,
        pub region: Option<&'static str>,
        pub lcid: u32,
    }
    
    include!(concat!(env!("OUT_DIR"), "/lcid_db.rs"));
    
    pub fn get(tag: &str, script: Option<&str>, region: Option<&str>) -> Option<&'static Record> {
        let tag = match super::autonym::get(tag) {
            Some(v) => v.tag3,
            None => return None
        };

        LCIDS.binary_search_by_key(&(tag, script, region), |record| {
            (record.tag3, record.script, record.region)
        }).map(|i| LCIDS[i]).ok()
    }
}
