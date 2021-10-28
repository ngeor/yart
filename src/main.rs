extern crate clap;

use std::str::FromStr;
use clap::{Arg, App};

enum SemVerBump {
    Major,
    Minor,
    Patch
}

impl FromStr for SemVerBump {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            _ => Err(())
        }
    }
}

struct CliOptions {
    version: SemVerBump,
    dir: String,
    message: String,
    dry_run: bool,
    no_push: bool,
    verbose: bool,
}

impl CliOptions {
    pub fn parse() -> Self {
        let matches = App::new("yart")
            .version("0.1.0")
            .author("Nikolaos Georgiou <nikolaos.georgiou@gmail.com>")
            .about("Yet another release tool")
            .arg(Arg::with_name("version")
                .short("v")
                .help("Specify the target SemVer version")
                .required(true)
                .takes_value(true)
                .possible_value("major")
                .possible_value("minor")
                .possible_value("patch")
            )
            .arg(
                Arg::with_name("dir")
                    .long("dir")
                    .help("The working directory of the git repository")
                    .required(false)
                    .default_value(".")
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("message")
                    .short("m")
                    .long("message")
                    .help("A custom message for the git commit")
                    .required(false)
                    .default_value("")
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("dry-run")
                    .long("dry-run")
                    .help("Do not actually modify anything")
                    .required(false)
            )
            .arg(
                Arg::with_name("no-push")
                    .long("no-push")
                    .help("Do not push changes to the remote repository")
                    .required(false)
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .help("Increase logging verbosity")
                    .required(false)
            )
            .get_matches();
        Self {
            version: SemVerBump::from_str(matches.value_of("version").unwrap()).unwrap(),
            dir: matches.value_of("dir").unwrap().to_string(),
            message: matches.value_of("message").unwrap().to_string(),
            dry_run: matches.is_present("dry-run"),
            no_push: matches.is_present("no-push"),
            verbose: matches.is_present("verbose")
        }
    }
}

fn main() {
    let args = CliOptions::parse();
}
