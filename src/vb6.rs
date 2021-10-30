//! Handles Visual Basic 6.0 VBG and VBP files

use crate::files::{has_extension, ContentProcessor, FileFinder};
use crate::SemVer;
use std::fs;
use std::path::PathBuf;

pub struct VB6Updater {}

/// Find vbp files in the current directory.
/// vbp files are detected in two ways:
/// 1. Directly at the root directory
/// 2. Referenced via vbg files at the root directory.
impl FileFinder for VB6Updater {
    fn find(&self, dir: &str) -> std::io::Result<Vec<PathBuf>> {
        let mut result = Vec::<PathBuf>::new();
        for res_entry in fs::read_dir(dir)? {
            let entry = res_entry?;
            let path = entry.path();
            if path.is_file() {
                if has_extension(&path, "vbg") {
                    let mut projects_in_vbg_file = vbg_parser::process_vbg_file(path)?;
                    result.append(&mut projects_in_vbg_file);
                } else if has_extension(&path, "vbp") {
                    result.push(path);
                }
            }
        }
        Ok(result)
    }
}

impl ContentProcessor for VB6Updater {
    type Err = std::io::Error; // does not really throw error
    fn process(&self, old_contents: &str, version: SemVer) -> Result<String, Self::Err> {
        Ok(vbp_parser::set_vbp_version(old_contents, version))
    }
}

mod vbg_parser {
    use std::fs;
    use std::path::PathBuf;

    pub fn process_vbg_file(path: PathBuf) -> std::io::Result<Vec<PathBuf>> {
        let contents = fs::read_to_string(&path)?;
        let projects = process_vbg_file_contents(path, contents.as_str());
        Ok(projects)
    }

    fn process_vbg_file_contents(path: PathBuf, contents: &str) -> Vec<PathBuf> {
        contents
            .lines()
            .map(str::trim)
            .filter(|s| is_project_line(*s))
            .map(extract_project)
            .map(|s| s.replace("\\", "/"))
            .map(|s| {
                s.split("/")
                    .fold(path.parent().unwrap().to_path_buf(), |l, r| l.join(r))
            })
            .collect()
    }

    fn is_project_line(s: &str) -> bool {
        let upper = s.to_ascii_uppercase();
        upper.starts_with("PROJECT=") || upper.starts_with("STARTUPPROJECT=")
    }

    fn extract_project(s: &str) -> &str {
        s.split("=").skip(1).next().unwrap()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_vbg_file() {
            let input = r"
        VBGROUP 5.0
        StartupProject=Server\RTFChatServer.vbp
        Project=Client\RTFChat.vbp
        Project=Shared\RTFChatShared.vbp
        ";
            let vbg_path = PathBuf::from("test/test.vbg");
            let actual = process_vbg_file_contents(vbg_path, input);
            let expected = vec![
                PathBuf::from("test/Server/RTFChatServer.vbp"),
                PathBuf::from("test/Client/RTFChat.vbp"),
                PathBuf::from("test/Shared/RTFChatShared.vbp"),
            ];
            assert_eq!(expected, actual);
        }
    }
}

mod vbp_parser {
    use crate::SemVer;

    pub fn set_vbp_version(contents: &str, version: SemVer) -> String {
        let mut result = String::new();
        for line in contents.lines() {
            result.push_str(map_line(line, version).as_str());
            result.push('\r');
            result.push('\n');
        }
        result
    }

    fn map_line(line: &str, version: SemVer) -> String {
        match line.find("=") {
            Some(idx) => {
                if idx > 0 {
                    let (property, _) = line.split_at(idx);
                    if property.eq_ignore_ascii_case("MajorVer") {
                        format!("{}={}", property, version.major)
                    } else if property.eq_ignore_ascii_case("MinorVer") {
                        format!("{}={}", property, version.minor)
                    } else if property.eq_ignore_ascii_case("RevisionVer") {
                        format!("{}={}", property, version.patch)
                    } else {
                        line.to_owned()
                    }
                } else {
                    line.to_owned()
                }
            }
            _ => line.to_owned(),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_set_ver() {
            let input = r"
Type=Exe
MajorVer=1
MinorVer=0
RevisionVer=0

NoAliasing=0
";
            let expected = r"
Type=Exe
MajorVer=2
MinorVer=3
RevisionVer=4

NoAliasing=0
"
            .replace("\n", "\r\n");
            let actual = set_vbp_version(input, SemVer::new(2, 3, 4));
            assert_eq!(expected, actual);
        }
    }
}
