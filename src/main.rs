use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Args {
    Autonym(AutonymArgs),
    Tag(TagArgs),
    Is(IsArgs),
    #[structopt(name = "has-1")]
    Has1(Has1Args),
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

fn main() {
    match Args::from_args() {
        Args::Autonym(x) => {
            let r = match iso639::get(&*x.tag) {
                Some(v) => v,
                None => std::process::exit(1),
            };
            println!("{}", r.autonym.unwrap_or_else(|| r.name));
        }
        Args::Tag(x) => {
            let r = match iso639::get(&*x.tag) {
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
            let r = match iso639::get(&*x.tag) {
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
            let r = match iso639::get(&*x.tag) {
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
