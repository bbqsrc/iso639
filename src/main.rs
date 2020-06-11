use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Args {
    Autonym(AutonymArgs),
    Tag(TagArgs),
    Is(IsArgs),
    #[structopt(name = "has-1")]
    Has1(Has1Args),
    Script(ScriptArgs),
    Lcid(LcidArgs),
}

#[derive(Debug, StructOpt)]
struct AutonymArgs {
    tag: String,
}

#[derive(Debug, StructOpt)]
struct TagArgs {
    #[structopt(short = "1")]
    to_tag1: bool,

    #[structopt(short = "3")]
    to_tag3: bool,

    tag: String,
}

#[derive(Debug, StructOpt)]
struct IsArgs {
    #[structopt(short = "1")]
    is_tag1: bool,

    #[structopt(short = "3")]
    is_tag3: bool,

    tag: String,
}

#[derive(Debug, StructOpt)]
struct Has1Args {
    tag: String,
}

#[derive(Debug, StructOpt)]
struct ScriptArgs {
    tag: String,
}

#[derive(Debug, StructOpt)]
struct LcidArgs {
    tag: String,
    #[structopt(short, long)]
    script: Option<String>,
    #[structopt(short, long)]
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
