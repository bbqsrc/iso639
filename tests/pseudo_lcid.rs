use iso639::{Error, make_pseudo_lcid, parse_pseudo_lcid};

// Pseudo-LCIDs are meant to be constant, so these values must never change.
const KNOWN: &[(&str, Option<&str>, u32)] = &[
    ("sje", None, 2147496054),
    ("fkv", Some("NO"), 2517634637),
    ("rmn", Some("SE"), 2643471913),
    ("sjd", Some("RU"), 2632986741),
    ("sje", Some("150"), 2304815222),
];

#[test]
fn make_known_values() {
    for &(tag, region, lcid) in KNOWN {
        assert_eq!(
            make_pseudo_lcid(tag, region).unwrap(),
            lcid,
            "{tag} {region:?}"
        );
    }
}

#[test]
fn parse_round_trips() {
    for &(tag, region, lcid) in KNOWN {
        let (parsed_tag, parsed_region) = parse_pseudo_lcid(lcid).unwrap();
        assert_eq!(parsed_tag, tag);
        assert_eq!(parsed_region.as_deref(), region);
    }
}

#[test]
fn real_lcids_are_not_pseudo() {
    assert!(parse_pseudo_lcid(1083).is_err());
}

#[test]
fn regions_that_are_not_regions() {
    for region in ["²1", "1²", "", "N", "NOR", "1234", "١٢"] {
        assert!(
            matches!(make_pseudo_lcid("sme", Some(region)), Err(Error::InvalidRegion)),
            "{region:?}"
        );
    }
    assert!(matches!(make_pseudo_lcid("zz", None), Err(Error::InvalidTag)));
}
