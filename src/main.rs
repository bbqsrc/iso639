use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Args {
    /// Prints the autonym for a given language tag
    Autonym(AutonymArgs),
    /// Prints a -1 or -3 variant of a provided tag, if valid
    Tag(TagArgs),
    /// Validates a provided tag as existing and valid
    Is(IsArgs),
    /// Checks if a provided tag in any format has a valid -1 variant
    #[structopt(name = "has-1")]
    Has1(Has1Args),
    /// Prints the default script for a provided tag, if available
    Script(ScriptArgs),
    /// Prints the LCID for a given tag, if available
    Lcid(LcidArgs),
}

#[derive(Debug, StructOpt)]
struct AutonymArgs {
    /// The ISO 639-1 or -3 tag
    tag: String,
}

#[derive(Debug, StructOpt)]
struct TagArgs {
    /// Convert given tag into -1 format, if possible
    #[structopt(short = "1")]
    to_tag1: bool,

    /// Convert given tag into -3 format
    #[structopt(short = "3")]
    to_tag3: bool,

    /// The ISO 639-1 or -3 tag
    tag: String,
}

#[derive(Debug, StructOpt)]
struct IsArgs {
    #[structopt(short = "1")]
    is_tag1: bool,

    #[structopt(short = "3")]
    is_tag3: bool,

    /// The ISO 639-1 or -3 tag
    tag: String,
}

#[derive(Debug, StructOpt)]
struct Has1Args {
    /// The ISO 639-1 or -3 tag
    tag: String,
}

#[derive(Debug, StructOpt)]
struct ScriptArgs {
    /// The ISO 639-1 or -3 tag
    tag: String,
}

#[derive(Debug, StructOpt)]
struct LcidArgs {
    /// The ISO 639-1 or -3 tag
    tag: String,
    #[structopt(short, long)]
    /// The ISO-15924 script, if required
    script: Option<String>,
    #[structopt(short, long)]
    /// The ISO 3166-1 alpha-2 code or UN M.49 region code, if required
    region: Option<String>,
}

fn main() {
    match Args::from_args() {
        Args::Lcid(x) => {
            let script = x.script.as_ref().map(|x| &**x);
            let region = x.region.as_ref().map(|x| &**x);
            let r = match iso639::lcid::get(&*x.tag, script, region) {
                Some(v) => v,
                None => std::process::exit(1),
            };
            println!("{}", r.lcid);
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
            println!("{}", r.autonym.unwrap_or_else(|| r.name));
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
