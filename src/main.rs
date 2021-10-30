mod cli_options;
mod delphi;
mod files;
mod git;
mod sem_ver;
mod updater;
mod vb6;
mod writers;
mod xml_util;

use crate::sem_ver::SemVer;
use std::str::FromStr;

fn main() -> Result<(), &'static str> {
    let args = cli_options::CliOptions::parse();
    let git_tags_output = git::tags(&args.dir).unwrap();
    match find_biggest_tag(&git_tags_output) {
        Some(biggest_tag) => {
            let next_version = biggest_tag.bump(args.version);
            println!(
                "Current version: {}, next version: {}",
                biggest_tag, next_version
            );
            let changed_files =
                updater::update_files(args.dir.as_str(), next_version, args.dry_run).unwrap();
            if args.dry_run {
                println!("Would have committed modified files, created tag, pushed to remote");
            } else {
                let msg_prefix = if args.message.is_empty() {
                    "Releasing version".to_string()
                } else {
                    args.message
                };
                let msg = format!("{} {}", msg_prefix, next_version);

                if !changed_files.is_empty() {
                    git::commit(&args.dir, &msg).unwrap();
                }
                git::tag(&args.dir, &msg, format!("v{}", next_version)).unwrap();
                if args.no_push {
                    println!("Tagged, but not pushing because --no-push was specified");
                } else {
                    git::push(&args.dir).unwrap();
                }
            }
            Ok(())
        }
        _ => Err("Could not find a tag in vMajor.Minor.Patch format"),
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

#[cfg(test)]
mod tests {
    use super::*;

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
