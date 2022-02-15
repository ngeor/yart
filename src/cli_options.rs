use crate::sem_ver::SemVerComponent;
use std::str::FromStr;
extern crate clap;
use clap::{App, Arg};

pub struct CliOptions {
    pub version: SemVerComponent,
    pub dir: String,
    pub message: String,
    pub dry_run: bool,
    pub no_push: bool,
    pub verbose: bool,
}

impl CliOptions {
    pub fn parse() -> Self {
        let matches = App::new("yart")
            .version("0.1.0")
            .author("Nikolaos Georgiou <nikolaos.georgiou@gmail.com>")
            .about("Yet another release tool")
            .arg(
                Arg::new("version")
                    .short('v')
                    .help("Specify the target SemVer version")
                    .required(true)
                    .takes_value(true)
                    .possible_value("major")
                    .possible_value("minor")
                    .possible_value("patch"),
            )
            .arg(
                Arg::new("dir")
                    .long("dir")
                    .help("The working directory of the git repository")
                    .required(false)
                    .default_value(".")
                    .takes_value(true),
            )
            .arg(
                Arg::new("message")
                    .short('m')
                    .long("message")
                    .help("A custom message for the git commit")
                    .required(false)
                    .default_value("")
                    .takes_value(true),
            )
            .arg(
                Arg::new("dry-run")
                    .long("dry-run")
                    .help("Do not actually modify anything")
                    .required(false),
            )
            .arg(
                Arg::new("no-push")
                    .long("no-push")
                    .help("Do not push changes to the remote repository")
                    .required(false),
            )
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .help("Increase logging verbosity")
                    .required(false),
            )
            .get_matches();
        Self {
            version: SemVerComponent::from_str(matches.value_of("version").unwrap()).unwrap(),
            dir: matches.value_of("dir").unwrap().to_string(),
            message: matches.value_of("message").unwrap().to_string(),
            dry_run: matches.is_present("dry-run"),
            no_push: matches.is_present("no-push"),
            verbose: matches.is_present("verbose"),
        }
    }
}
