//! Autonyms, default scripts and Windows LCIDs for languages, from
//! [iso639-databases](https://github.com/divvun/iso639-databases).
//!
//! Lookups take BCP 47 language tags such as `se`, `sme`, `sr-Latn` or `rmn-SE`.

/// The language, script and region of a BCP 47 language tag, in the case the databases use.
struct Subtags {
    tag3: &'static str,
    script: Option<String>,
    region: Option<String>,
}

fn is_alpha(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_alphabetic())
}

fn normalise_script(script: &str) -> String {
    let mut script = script.to_ascii_lowercase();
    script[..1].make_ascii_uppercase();
    script
}

/// Parses the language, script and region of a BCP 47 language tag. An extended
/// language subtag replaces the language, so `zh-yue` is `yue`. Variants,
/// extensions and private use subtags are ignored.
fn parse(tag: &str) -> Option<Subtags> {
    let mut parts = tag.split(['-', '_']).peekable();
    let mut language = parts.next()?;
    if !is_alpha(language, 2) && !is_alpha(language, 3) {
        return None;
    }
    if let Some(extlang) = parts.next_if(|p| is_alpha(p, 3)) {
        language = extlang;
    }
    let script = parts.next_if(|p| is_alpha(p, 4)).map(normalise_script);
    let region = parts
        .next_if(|p| is_alpha(p, 2) || (p.len() == 3 && p.bytes().all(|b| b.is_ascii_digit())))
        .map(str::to_ascii_uppercase);
    let tag3 = autonym::LANGUAGES
        .get(&*language.to_ascii_lowercase())
        .copied()?;
    Some(Subtags {
        tag3,
        script,
        region,
    })
}

/// The database key for a language, script and region, e.g. `srp-Latn` or `rmn-SE`.
fn key(tag3: &str, script: Option<&str>, region: Option<&str>) -> String {
    [Some(tag3), script, region]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("-")
}

pub mod autonym {
    /// A row of the autonym table.
    #[derive(Debug, Clone, Copy)]
    pub struct Record {
        pub tag3: &'static str,
        pub tag1: Option<&'static str>,
        /// The ISO 15924 script, if the row is not in the language's default script.
        pub script: Option<&'static str>,
        /// The ISO 3166-1 or UN M.49 region, if the row only applies to that region.
        pub region: Option<&'static str>,
        /// What the speakers call their language in English.
        pub common_name: &'static str,
        /// What the speakers call their language in that language, if it differs
        /// from the common name.
        pub autonym: Option<&'static str>,
        pub source: &'static str,
    }

    impl Record {
        /// The autonym, or the common name if there is no separate autonym.
        pub fn display_name(&self) -> &'static str {
            self.autonym.unwrap_or(self.common_name)
        }
    }

    include!(concat!(env!("OUT_DIR"), "/autonym_db.rs"));

    /// Looks up a BCP 47 language tag. Without a row for the exact tag, this
    /// falls back by dropping the region, then the script. The language's default
    /// script counts as no script, so `sr-Cyrl` finds the `sr` row.
    pub fn get(tag: &str) -> Option<&'static Record> {
        let crate::Subtags {
            tag3,
            script,
            region,
        } = crate::parse(tag)?;
        let default_script = crate::script::by_tag3(tag3).map(|r| r.script);
        let script = script.as_deref().filter(|s| Some(*s) != default_script);
        let region = region.as_deref();

        [(script, region), (script, None), (None, None)]
            .into_iter()
            .find_map(|(s, r)| AUTONYMS.get(&*crate::key(tag3, s, r)).copied())
    }
}

pub mod script {
    /// A row of the default script table.
    #[derive(Debug, Clone, Copy)]
    pub struct Record {
        pub tag3: &'static str,
        pub tag1: Option<&'static str>,
        pub script: &'static str,
        pub source: &'static str,
    }

    include!(concat!(env!("OUT_DIR"), "/script_db.rs"));

    pub(crate) fn by_tag3(tag3: &str) -> Option<&'static Record> {
        SCRIPTS.get(tag3).copied()
    }

    /// Looks up the default script of the language of a BCP 47 language tag.
    pub fn get(tag: &str) -> Option<&'static Record> {
        by_tag3(crate::parse(tag)?.tag3)
    }
}

pub mod lcid {
    /// A row of the LCID table.
    #[derive(Debug, Clone, Copy)]
    pub struct Record {
        pub tag3: &'static str,
        pub tag1: Option<&'static str>,
        pub script: Option<&'static str>,
        pub region: Option<&'static str>,
        pub lcid: u32,
    }

    include!(concat!(env!("OUT_DIR"), "/lcid_db.rs"));

    /// Looks up the LCID of a BCP 47 language tag. A `script` or `region` given
    /// here replaces the one in the tag.
    pub fn get(tag: &str, script: Option<&str>, region: Option<&str>) -> Option<&'static Record> {
        let subtags = crate::parse(tag)?;
        let script = script.map(crate::normalise_script).or(subtags.script);
        let region = region.map(str::to_ascii_uppercase).or(subtags.region);

        LCIDS
            .binary_search_by_key(&(subtags.tag3, script.as_deref(), region.as_deref()), |r| {
                (r.tag3, r.script, r.region)
            })
            .ok()
            .map(|i| &LCIDS[i])
    }

    pub fn get_by_lcid(lcid: u32) -> Option<&'static Record> {
        LCIDS.iter().find(|r| r.lcid == lcid)
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

/// Decodes lowercase ASCII letters as a base 26 number, where `a` is 0.
fn base26_decode(s: &str) -> Option<u16> {
    s.bytes().try_fold(0u16, |n, b| {
        b.is_ascii_lowercase()
            .then(|| n.checked_mul(26)?.checked_add((b - b'a') as u16))
            .flatten()
    })
}

/// Encodes a number in base 26 with lowercase ASCII letters, where `a` is 0.
fn base26_encode(mut n: u16) -> String {
    let mut out = vec![];
    loop {
        out.push(b'a' + (n % 26) as u8);
        n /= 26;
        if n == 0 {
            break;
        }
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

pub fn parse_pseudo_lcid(lcid: u32) -> Result<(String, Option<String>), LcidError> {
    // Check if maybe pseudo
    if lcid & 0b10000000_00000000_00000000_00000000 == 0 {
        return Err(LcidError::NotPseudoLcid);
    }

    // Get language part
    let tag_nr = (lcid & 0b01111111_11111111) as u16;
    let is_m49 = (lcid & 0b10000000_00000000) != 0;

    // Get region
    let region_nr = ((lcid >> 20) & 0b00000011_11111111) as u16;

    let tag: String = format!("{:a>3}", base26_encode(tag_nr));

    let region: Option<String> = if region_nr == 0 {
        None
    } else if is_m49 {
        Some(format!("{:03}", region_nr))
    } else {
        Some(format!(
            "{:A>2}",
            base26_encode(region_nr - 1).to_uppercase()
        ))
    };

    Ok((tag, region))
}

/// Generate a constant pseudo-LCID (that is technically non-compliant with [MS-LCID])
/// from a language, region (optional), and/or script (optional) to provide
/// interoperability with some broken piece of software.
pub fn make_pseudo_lcid(tag: &str, region: Option<&str>) -> Result<u32, Error> {
    let tag = parse(tag).ok_or(Error::InvalidTag)?.tag3;
    let region = region.map(|x| x.to_lowercase());

    let tag_nr: u16 = base26_decode(tag).ok_or(Error::InvalidTag)?;
    let mut is_m49 = false;
    let region_nr: u16 = match region {
        Some(x) => {
            if x.len() == 3 && x.chars().all(|c| c.is_numeric()) {
                is_m49 = true;
                x.parse::<u16>().unwrap()
            } else if x.len() == 2 && x.chars().all(|c| c.is_ascii_alphabetic()) {
                let v: u16 = base26_decode(&x).ok_or(Error::InvalidRegion)?;
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
