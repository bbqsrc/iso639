use clap::Parser;

#[derive(Debug, Parser)]
enum Args {
    /// Prints the autonym for a given language tag
    Autonym(AutonymArgs),
    /// Prints a -1 or -3 variant of a provided tag, if valid
    Tag(TagArgs),
    /// Validates a provided tag as existing and valid
    Is(IsArgs),
    /// Checks if a provided tag in any format has a valid -1 variant
    #[clap(name = "has-1")]
    Has1(Has1Args),
    /// Prints the default script for a provided tag, if available
    Script(ScriptArgs),
    /// Prints the LCID for a given tag, if available
    Lcid(LcidArgs),
    // PseudoLcid(LcidArgs),
    FromLcid(FromLcidArgs),
}

#[derive(Debug, Parser)]
struct FromLcidArgs {
    /// The ISO 639-1 or -3 tag
    lcid: u32,

    #[clap(short = 'p', long = "pseudo")]
    allow_pseudo: bool,
}

#[derive(Debug, Parser)]
struct AutonymArgs {
    /// The BCP 47 language tag, such as se, sr-Latn or rmn-SE
    tag: String,
}

#[derive(Debug, Parser)]
struct TagArgs {
    /// Convert given tag into -1 format, if possible
    #[clap(short = '1')]
    to_tag1: bool,

    /// Convert given tag into -3 format
    #[clap(short = '3')]
    to_tag3: bool,

    /// The ISO 639-1 or -3 tag
    tag: String,
}

#[derive(Debug, Parser)]
struct IsArgs {
    #[clap(short = '1')]
    is_tag1: bool,

    #[clap(short = '3')]
    is_tag3: bool,

    /// The ISO 639-1 or -3 tag
    tag: String,
}

#[derive(Debug, Parser)]
struct Has1Args {
    /// The ISO 639-1 or -3 tag
    tag: String,
}

#[derive(Debug, Parser)]
struct ScriptArgs {
    /// The BCP 47 language tag
    tag: String,
}

#[derive(Debug, Parser)]
struct LcidArgs {
    /// The BCP 47 language tag
    tag: String,

    /// The ISO-15924 script, if required
    #[clap(short, long)]
    script: Option<String>,

    /// The ISO 3166-1 alpha-2 code or UN M.49 region code, if required
    #[clap(short, long)]
    region: Option<String>,

    /// Generate a constant pseudo-LCID (that is technically non-compliant with [MS-LCID])
    /// from a language, region (optional), and/or script (optional) to provide
    /// interoperability with some broken piece of software.
    #[clap(short = 'p', long = "pseudo")]
    allow_pseudo: bool,

    /// Printed as a signed integer
    #[clap(short = 'i', long = "signed")]
    as_signed: bool,

    /// Print with the hex representation
    #[clap(short = 'x', long = "hex")]
    as_hex: bool,
}

fn main() {
    match Args::parse() {
        Args::FromLcid(x) => {
            match iso639::lcid::get_by_lcid(x.lcid) {
                Some(v) => {
                    println!("{:?}", v);
                    return;
                }
                None => {}
            };

            if x.allow_pseudo {
                match iso639::parse_pseudo_lcid(x.lcid) {
                    Ok((tag, region)) => match region {
                        Some(region) => println!("{}-{}", tag, region),
                        None => println!("{}", tag),
                    },
                    Err(e) => {
                        eprintln!("{:?}", e);
                        std::process::exit(1)
                    }
                }
            } else {
                std::process::exit(1);
            }
        }
        Args::Lcid(x) => {
            let script = x.script.as_ref().map(|x| &**x);
            let region = x.region.as_ref().map(|x| &**x);
            let lcid = match iso639::lcid::get(&*x.tag, script, region) {
                Some(v) => {
                    v.lcid
                }
                None => {
                    if x.allow_pseudo {
                        let result =
                            iso639::make_pseudo_lcid(&x.tag, x.region.as_ref().map(|x| &**x));
                        match result {
                            Ok(v) => {
                                println!("WARNING: Pseudo-LCID.");
                                v
                            }
                            Err(e) => {
                                eprintln!("{:?}", e);
                                std::process::exit(1)
                            }
                        }
                    } else {
                        std::process::exit(1)
                    }
                }
            };

            match (x.as_signed, x.as_hex) {
                (true, true) => println!("{:x}", lcid as i32),
                (true, false) => println!("{}", lcid as i32),
                (false, true) => println!("{:x}", lcid),
                (false, false) => println!("{}", lcid),
            }
        }
        Args::Script(x) => {
            let r = match iso639::script::get(&*x.tag) {
                Some(v) => v,
                None => std::process::exit(1),
            };
            println!("{}", r.script);
        }
        Args::Autonym(x) => {
            let r = match iso639::autonym::get(&*x.tag) {
                Some(v) => v,
                None => std::process::exit(1),
            };
            println!("{}", r.display_name());
        }
        Args::Tag(x) => {
            let r = match iso639::autonym::get(&*x.tag) {
                Some(v) => v,
                None => std::process::exit(1),
            };

            if x.to_tag1 {
                if let Some(v) = r.tag1 {
                    println!("{}", v);
                } else {
                    std::process::exit(1)
                }
            } else if x.to_tag3 {
                println!("{}", r.tag3);
            }
        }
        Args::Is(x) => {
            let r = match iso639::autonym::get(&*x.tag) {
                Some(v) => v,
                None => std::process::exit(1),
            };

            if x.is_tag1 {
                if let Some(v) = r.tag1 {
                    if v == x.tag {
                        // Success!
                        return;
                    } else {
                        std::process::exit(1)
                    }
                } else {
                    std::process::exit(1)
                }
            } else if x.is_tag3 {
                if r.tag3 == x.tag {
                    // Success!
                    return;
                } else {
                    std::process::exit(1)
                }
            }
        }
        Args::Has1(x) => {
            let r = match iso639::autonym::get(&*x.tag) {
                Some(v) => v,
                None => std::process::exit(1),
            };

            if r.tag1.is_some() {
                return;
            }

            std::process::exit(1);
        }
    }
}
