//! Autonyms, default scripts and Windows LCIDs for languages, from
//! [iso639-databases](https://github.com/divvun/iso639-databases).
//!
//! Lookups take BCP 47 language tags such as `se`, `sme`, `sr-Latn` or `rmn-SE`.

/// The tables, as build.rs writes them.
mod tables {
    include!(concat!(env!("OUT_DIR"), "/tables.rs"));
}

/// A language: its index in [`tables::TAG3`].
#[derive(Clone, Copy)]
struct Language(u16);

impl Language {
    const COUNT: usize = tables::TAG3.len() / 3;

    /// The language whose ISO 639-1 or 639-3 code is `code`, which must be
    /// ASCII letters, in any case.
    fn find(code: &str) -> Option<Self> {
        let number = code.bytes().fold(0, |n, b| n << 8 | u32::from(b.to_ascii_lowercase()));
        match code.len() {
            3 => search(Self::COUNT, number, |i| code_number(tables::TAG3, 3, i)).map(|i| Language(i as u16)),
            2 => {
                let by_code = |i: usize| usize::from(tables::TAG1_BY_CODE[i]);
                let i = search(tables::TAG1_BY_CODE.len(), number, |i| code_number(tables::TAG1, 2, by_code(i)))?;
                Some(Language(tables::TAG1_LANGUAGES[by_code(i)]))
            }
            _ => None,
        }
    }

    fn tag3(self) -> &'static str {
        code_at(tables::TAG3, 3, self.0.into())
    }

    fn tag1(self) -> Option<&'static str> {
        let i = tables::TAG1_LANGUAGES.binary_search(&self.0).ok()?;
        Some(code_at(tables::TAG1, 2, i))
    }

    /// The index of the language's default script, if the table has one.
    fn default_script(self) -> Option<u8> {
        let i = tables::DEFAULT_SCRIPT_LANGUAGES.binary_search(&self.0).ok()?;
        Some(tables::DEFAULT_SCRIPTS[i][0])
    }
}

/// The `i`th of the `width`-byte codes `codes` holds.
fn code_at(codes: &'static str, width: usize, i: usize) -> &'static str {
    &codes[i * width..(i + 1) * width]
}

/// The `i`th of the `width`-byte codes `codes` holds, as a number that sorts as
/// the codes do.
fn code_number(codes: &str, width: usize, i: usize) -> u32 {
    codes.as_bytes()[i * width..(i + 1) * width]
        .iter()
        .fold(0, |n, &b| n << 8 | u32::from(b))
}

/// The index in `0..len` where `key` is `target`, given that `key` rises with
/// the index. Each step picks a half without a branch to mispredict.
fn search(len: usize, target: u32, key: impl Fn(usize) -> u32) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let (mut base, mut size) = (0, len);
    while size > 1 {
        let half = size / 2;
        if key(base + half) <= target {
            base += half;
        }
        size -= half;
    }
    (key(base) == target).then_some(base)
}

/// The index a row gives the value of `values` that `value` is once each of
/// its bytes is put in the case the table uses, or `None` if no row has it.
fn index_of(values: &[&str], value: Option<&str>, case: impl Fn(usize, u8) -> u8) -> Option<u8> {
    let Some(value) = value else {
        return Some(0);
    };
    let value = value.bytes().enumerate().map(|(i, b)| case(i, b));
    let i = values.binary_search_by(|v| v.bytes().cmp(value.clone())).ok()?;
    Some(i as u8 + 1)
}

/// The index a row gives `script`, which ISO 15924 writes in title case.
fn script_index(script: Option<&str>) -> Option<u8> {
    index_of(&tables::SCRIPTS, script, |i, b| match i {
        0 => b.to_ascii_uppercase(),
        _ => b.to_ascii_lowercase(),
    })
}

/// The index a row gives `region`, which ISO 3166-1 writes in upper case.
fn region_index(region: Option<&str>) -> Option<u8> {
    index_of(&tables::REGIONS, region, |_, b| b.to_ascii_uppercase())
}

/// The value of `values` that a row gives as `index`.
fn value_at(values: &[&'static str], index: u8) -> Option<&'static str> {
    index.checked_sub(1).map(|i| values[usize::from(i)])
}

fn key(language: Language, script: u8, region: u8) -> u32 {
    u32::from(language.0) << 16 | u32::from(script) << 8 | u32::from(region)
}

/// The language, script and region of a key.
fn unkey(key: u32) -> (Language, Option<&'static str>, Option<&'static str>) {
    (
        Language((key >> 16) as u16),
        value_at(&tables::SCRIPTS, (key >> 8) as u8),
        value_at(&tables::REGIONS, key as u8),
    )
}

/// The language, script and region of a BCP 47 language tag, with the script
/// and region in the case the tag gives them.
struct Subtags<'a> {
    language: Language,
    script: Option<&'a str>,
    region: Option<&'a str>,
}

fn is_alpha(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_alphabetic())
}

/// Parses the language, script and region of a BCP 47 language tag. An extended
/// language subtag replaces the language, so `zh-yue` is `yue`. Variants,
/// extensions and private use subtags are ignored.
fn parse(tag: &str) -> Option<Subtags<'_>> {
    let mut parts = tag.split(['-', '_']).peekable();
    let mut language = parts.next()?;
    if !is_alpha(language, 2) && !is_alpha(language, 3) {
        return None;
    }
    if let Some(extlang) = parts.next_if(|p| is_alpha(p, 3)) {
        language = extlang;
    }
    let script = parts.next_if(|p| is_alpha(p, 4));
    let region = parts.next_if(|p| is_alpha(p, 2) || (p.len() == 3 && p.bytes().all(|b| b.is_ascii_digit())));
    Some(Subtags {
        language: Language::find(language)?,
        script,
        region,
    })
}

pub mod autonym {
    use crate::tables;

    /// A row of the autonym table.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Looks up a BCP 47 language tag. Without a row for the exact tag, this
    /// falls back by dropping the region, then the script. The language's default
    /// script counts as no script, so `sr-Cyrl` finds the `sr` row.
    pub fn get(tag: &str) -> Option<Record> {
        let crate::Subtags {
            language,
            script,
            region,
        } = crate::parse(tag)?;
        // A script or region no row has is `None`, and finds no row.
        let script = crate::script_index(script).map(|s| match language.default_script() {
            Some(default) if default == s => 0,
            _ => s,
        });
        let region = crate::region_index(region);

        [(script, region), (script, Some(0)), (Some(0), Some(0))]
            .into_iter()
            .find_map(|(s, r)| row(language, s?, r?))
    }

    fn row(language: crate::Language, script: u8, region: u8) -> Option<Record> {
        let key = crate::key(language, script, region);
        let i = match (script, region) {
            (0, 0) => usize::from(language.0),
            _ => crate::Language::COUNT + tables::AUTONYM_VARIANT_KEYS.binary_search(&key).ok()?,
        };
        let (start, end) = (tables::NAME_OFFSETS[i] as usize, tables::NAME_OFFSETS[i + 1] as usize);
        let split = start + usize::from(tables::COMMON_NAME_LENGTHS[i]);
        let (_, script, region) = crate::unkey(key);
        Some(Record {
            tag3: language.tag3(),
            tag1: language.tag1(),
            script,
            region,
            common_name: &tables::NAMES[start..split],
            autonym: Some(&tables::NAMES[split..end]).filter(|a| !a.is_empty()),
            source: tables::SOURCES[usize::from(tables::AUTONYM_SOURCES[i])],
        })
    }
}

pub mod script {
    use crate::tables;

    /// A row of the default script table.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Record {
        pub tag3: &'static str,
        pub tag1: Option<&'static str>,
        pub script: &'static str,
        pub source: &'static str,
    }

    /// Looks up the default script of the language of a BCP 47 language tag.
    pub fn get(tag: &str) -> Option<Record> {
        let language = crate::parse(tag)?.language;
        let i = tables::DEFAULT_SCRIPT_LANGUAGES.binary_search(&language.0).ok()?;
        let [script, source] = tables::DEFAULT_SCRIPTS[i];
        Some(Record {
            tag3: language.tag3(),
            tag1: language.tag1(),
            script: crate::value_at(&tables::SCRIPTS, script)?,
            source: tables::SOURCES[usize::from(source)],
        })
    }
}

pub mod lcid {
    use crate::tables;

    /// A row of the LCID table.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Record {
        pub tag3: &'static str,
        pub tag1: Option<&'static str>,
        pub script: Option<&'static str>,
        pub region: Option<&'static str>,
        pub lcid: u32,
    }

    fn row(i: usize) -> Record {
        let (language, script, region) = crate::unkey(tables::LCID_KEYS[i]);
        Record {
            tag3: language.tag3(),
            tag1: language.tag1(),
            script,
            region,
            lcid: tables::LCIDS[i],
        }
    }

    /// Looks up the LCID of a BCP 47 language tag. A `script` or `region` given
    /// here replaces the one in the tag.
    pub fn get(tag: &str, script: Option<&str>, region: Option<&str>) -> Option<Record> {
        let subtags = crate::parse(tag)?;
        let key = crate::key(
            subtags.language,
            crate::script_index(script.or(subtags.script))?,
            crate::region_index(region.or(subtags.region))?,
        );
        tables::LCID_KEYS.binary_search(&key).ok().map(row)
    }

    /// The row for `lcid`. Where rows share an LCID, this is the first in the
    /// order of their language, script and region.
    pub fn get_by_lcid(lcid: u32) -> Option<Record> {
        tables::LCIDS.iter().position(|&l| l == lcid).map(row)
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
    let tag = parse(tag).ok_or(Error::InvalidTag)?.language.tag3();
    let region = region.map(|x| x.to_lowercase());

    let tag_nr: u16 = base26_decode(tag).ok_or(Error::InvalidTag)?;
    let mut is_m49 = false;
    let region_nr: u16 = match region {
        Some(x) => {
            if x.len() == 3 && x.bytes().all(|b| b.is_ascii_digit()) {
                is_m49 = true;
                x.parse::<u16>().map_err(|_| Error::InvalidRegion)?
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
    let mut x = tag_nr;
    if is_m49 {
        x |= 0b1000_0000_0000_0000
    }
    let mut y = region_nr << 4;
    y |= 0b1000_0000_0000_0000;

    let mut out = (y as u32) << 16;
    out |= x as u32;

    Ok(out)
}
