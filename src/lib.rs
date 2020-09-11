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
            None => return None,
        };

        LCIDS
            .binary_search_by_key(&(tag, script, region), |record| {
                (record.tag3, record.script, record.region)
            })
            .map(|i| LCIDS[i])
            .ok()
    }

    pub fn get_by_lcid(lcid: u32) -> Option<&'static Record> {
        for record in LCIDS {
            if record.lcid == lcid {
                return Some(record);
            }
        }
        None
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidTag,
    InvalidRegion,
}

#[derive(Debug)]
pub enum LcidError {
    NotPseudoLcid,
}

pub fn parse_pseudo_lcid(lcid: u32) -> Result<(String, Option<String>), LcidError> {
    use based::{Base, NumeralSystem};
    let base26: Base = "abcdefghijklmnopqrstuvwxyz".parse().unwrap();

    // Check if maybe pseudo
    if lcid & 0b10000000_00000000_00000000_00000000 == 0 {
        return Err(LcidError::NotPseudoLcid);
    }

    // Get language part
    let tag_nr = (lcid & 0b01111111_11111111) as u16;
    let is_m49 = (lcid & 0b10000000_00000000) != 0;

    // Get region
    let region_nr = ((lcid >> 20) & 0b00000011_11111111) as u16;

    let tag: String = format!("{:a>3}", base26.encode(tag_nr).unwrap());

    let region: Option<String> = if region_nr == 0 {
        None
    } else if is_m49 {
        Some(format!("{:03}", region_nr))
    } else {
        base26
            .encode(region_nr - 1)
            .ok()
            .map(|x| format!("{:A>2}", x.to_uppercase()))
    };

    Ok((tag, region))
}

/// Generate a constant pseudo-LCID (that is technically non-compliant with [MS-LCID])
/// from a language, region (optional), and/or script (optional) to provide
/// interoperability with some broken piece of software.
pub fn make_pseudo_lcid(tag: &str, region: Option<&str>) -> Result<u32, Error> {
    use based::{Base, NumeralSystem};
    let base26: Base = "abcdefghijklmnopqrstuvwxyz".parse().unwrap();

    let tag = match autonym::get(tag) {
        Some(v) => v.tag3,
        None => return Err(Error::InvalidTag),
    };
    let region = region.map(|x| x.to_lowercase());

    let tag_nr: u16 = base26.decode(tag).unwrap();
    let mut is_m49 = false;
    let region_nr: u16 = match region {
        Some(x) => {
            if x.len() == 3 && x.chars().all(|c| c.is_numeric()) {
                is_m49 = true;
                x.parse::<u16>().unwrap()
            } else if x.len() == 2 && x.chars().all(|c| c.is_ascii_alphabetic()) {
                let v: u16 = base26.decode(&x.to_ascii_lowercase()).unwrap();
                // Increment by one, as an empty region is an empty region, but "AA" == 0.
                v + 1
            } else {
                return Err(Error::InvalidRegion);
            }
        }
        None => 0,
    };

    // 1XRRRRRR_RRRR0000_TLLLLLLL_LLLLLLLL
    let mut x = tag_nr as u16;
    if is_m49 {
        x |= 0b1000_0000_0000_0000
    }
    let mut y = region_nr << 4;
    y |= 0b1000_0000_0000_0000;

    let mut out = (y as u32) << 16;
    out |= x as u32;

    Ok(out as u32)
}
