mod vbg;

extern crate clap;

use clap::{App, Arg};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemVerBump {
    Major,
    Minor,
    Patch,
}

impl FromStr for SemVerBump {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            _ => Err(()),
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
            .arg(
                Arg::with_name("version")
                    .short("v")
                    .help("Specify the target SemVer version")
                    .required(true)
                    .takes_value(true)
                    .possible_value("major")
                    .possible_value("minor")
                    .possible_value("patch"),
            )
            .arg(
                Arg::with_name("dir")
                    .long("dir")
                    .help("The working directory of the git repository")
                    .required(false)
                    .default_value(".")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("message")
                    .short("m")
                    .long("message")
                    .help("A custom message for the git commit")
                    .required(false)
                    .default_value("")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("dry-run")
                    .long("dry-run")
                    .help("Do not actually modify anything")
                    .required(false),
            )
            .arg(
                Arg::with_name("no-push")
                    .long("no-push")
                    .help("Do not push changes to the remote repository")
                    .required(false),
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .help("Increase logging verbosity")
                    .required(false),
            )
            .get_matches();
        Self {
            version: SemVerBump::from_str(matches.value_of("version").unwrap()).unwrap(),
            dir: matches.value_of("dir").unwrap().to_string(),
            message: matches.value_of("message").unwrap().to_string(),
            dry_run: matches.is_present("dry-run"),
            no_push: matches.is_present("no-push"),
            verbose: matches.is_present("verbose"),
        }
    }
}

fn main() -> Result<(), &'static str> {
    let args = CliOptions::parse();
    let output = Command::new("git")
        .arg("tag")
        .arg("--list")
        .current_dir(args.dir.as_str())
        .output()
        .unwrap();
    match find_biggest_tag(String::from_utf8(output.stdout).unwrap().as_str()) {
        Some(biggest_tag) => {
            let next_version = biggest_tag.bump(args.version);
            println!("Current version: {}, next version: {}", biggest_tag, next_version);
            let writer = create_writer(args.dry_run);
            let changed_files = update_files(args.dir.as_str(), biggest_tag, next_version, writer).unwrap();
            if args.dry_run {
                println!("Would have committed modified files, created tag, pushed to remote");
            } else {
                //create_tag();
                //push_tag();
            }
            Ok(())
        }
        _ => Err("Could not find a tag in vMajor.Minor.Patch format"),
    }
}

fn create_writer(dry_run: bool) -> Box<dyn FileWriter> {
    if dry_run {
        Box::new(DryFileWriter {})
    } else {
        Box::new(WetFileWriter {})
    }
}

fn find_biggest_tag(tag_lines: &str) -> Option<SemVer> {
    let mut tags: Vec<SemVer> = tag_lines
        .lines()
        .map(str::trim)
        .map(remove_v_prefix)
        .filter(Option::is_some)
        .map(Option::unwrap)
        .map(SemVer::from_str)
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .collect();
    tags.sort();
    tags.pop()
}

fn remove_v_prefix(tag: &str) -> Option<&str> {
    if tag.starts_with("v") {
        let (_, tag_without_v_prefix) = tag.split_at(1);
        if tag_without_v_prefix.is_empty() {
            None
        } else {
            Some(tag_without_v_prefix)
        }
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SemVer {
    major: u16,
    minor: u16,
    patch: u16,
}

impl SemVer {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn bump(&self, step: SemVerBump) -> Self {
        match step {
            SemVerBump::Major => Self::new(self.major + 1, 0, 0),
            SemVerBump::Minor => Self::new(self.major, self.minor + 1, 0),
            SemVerBump::Patch => Self::new(self.major, self.minor, self.patch + 1),
        }
    }
}

impl Display for SemVer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.major < other.major {
            Ordering::Less
        } else if self.major > other.major {
            Ordering::Greater
        } else if self.minor < other.minor {
            Ordering::Less
        } else if self.minor > other.minor {
            Ordering::Greater
        } else if self.patch < other.patch {
            Ordering::Less
        } else if self.patch > other.patch {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

#[derive(Debug)]
pub enum SemVerParseError {
    ParseIntError(ParseIntError),
    IllegalComponentCount(usize),
}

impl FromStr for SemVer {
    type Err = SemVerParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts_result: Result<Vec<u16>, ParseIntError> =
            s.split(".").map(u16::from_str).collect();
        match parts_result {
            Ok(parts) => {
                if parts.len() == 3 {
                    Ok(Self::new(parts[0], parts[1], parts[2]))
                } else {
                    Err(SemVerParseError::IllegalComponentCount(parts.len()))
                }
            }
            Err(err) => Err(SemVerParseError::ParseIntError(err)),
        }
    }
}

pub trait FileWriter {
    fn write(&self, path: &PathBuf, contents: &str) -> std::io::Result<()>;
}

struct DryFileWriter {}

impl FileWriter for DryFileWriter {
    fn write(&self, path: &PathBuf, _contents: &str) -> std::io::Result<()> {
        println!("Would have written {:?}", path);
        Ok(())
    }
}

struct WetFileWriter {}

impl FileWriter for WetFileWriter {
    fn write(&self, path: &PathBuf, contents: &str) -> std::io::Result<()> {
        std::fs::write(path, contents)
    }
}

fn update_files(dir: &str, old_version: SemVer, new_version: SemVer, writer: Box<dyn FileWriter>) -> std::io::Result<Vec<PathBuf>> {
    let mut changed_files = Vec::<PathBuf>::new();
    let mut vbp_files = vbg::handle(dir, new_version, writer)?;
    changed_files.append(&mut vbp_files);
    Ok(changed_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sem_ver_bump() {
        assert_eq!(
            SemVer::new(1, 0, 0).bump(SemVerBump::Major),
            SemVer::new(2, 0, 0)
        );
        assert_eq!(
            SemVer::new(1, 2, 0).bump(SemVerBump::Major),
            SemVer::new(2, 0, 0)
        );
        assert_eq!(
            SemVer::new(1, 2, 3).bump(SemVerBump::Major),
            SemVer::new(2, 0, 0)
        );
        assert_eq!(
            SemVer::new(1, 0, 0).bump(SemVerBump::Minor),
            SemVer::new(1, 1, 0)
        );
        assert_eq!(
            SemVer::new(1, 2, 0).bump(SemVerBump::Minor),
            SemVer::new(1, 3, 0)
        );
        assert_eq!(
            SemVer::new(1, 2, 3).bump(SemVerBump::Minor),
            SemVer::new(1, 3, 0)
        );
        assert_eq!(
            SemVer::new(1, 0, 0).bump(SemVerBump::Patch),
            SemVer::new(1, 0, 1)
        );
        assert_eq!(
            SemVer::new(1, 2, 0).bump(SemVerBump::Patch),
            SemVer::new(1, 2, 1)
        );
        assert_eq!(
            SemVer::new(1, 2, 3).bump(SemVerBump::Patch),
            SemVer::new(1, 2, 4)
        );
    }

    #[test]
    fn test_sem_ver_display() {
        assert_eq!("1.2.3", SemVer::new(1, 2, 3).to_string());
    }

    #[test]
    fn test_sem_ver_comparison() {
        assert!(SemVer::new(1, 2, 3) < SemVer::new(1, 2, 4));
        assert!(SemVer::new(1, 2, 3) < SemVer::new(2, 0, 0));
        assert!(SemVer::new(3, 0, 0) > SemVer::new(2, 0, 0));
        assert!(SemVer::new(3, 1, 0) > SemVer::new(3, 0, 0));
        assert!(SemVer::new(3, 1, 1) > SemVer::new(3, 1, 0));
        assert_eq!(SemVer::new(3, 1, 1), SemVer::new(3, 1, 1));
    }

    #[test]
    fn test_sem_ver_parse() {
        assert_eq!(SemVer::new(1, 2, 3), SemVer::from_str("1.2.3").unwrap());
        assert!(matches!(
            SemVer::from_str(""),
            Err(SemVerParseError::ParseIntError(_))
        ));
        assert!(matches!(
            SemVer::from_str("v1.2.3"),
            Err(SemVerParseError::ParseIntError(_))
        ));
        assert!(matches!(
            SemVer::from_str("2.3"),
            Err(SemVerParseError::IllegalComponentCount(2))
        ));
    }

    #[test]
    fn test_find_biggest_tag() {
        let input = r"
        v0.3.0
        v0.4.0
        v0.2.0
        0.6.0
        ";
        let expected = SemVer::new(0, 4, 0);
        let actual = find_biggest_tag(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_find_biggest_tag_no_tags() {
        let input = r"
        not-a-valid-tag
        ";
        assert!(find_biggest_tag(input).is_none());
    }
}
