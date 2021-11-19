use crate::files::{ContentProcessor, DirUpdater, UpdateError};
use crate::sem_ver::SemVer;
use std::fs;
use std::path::PathBuf;

struct CargoTomlContentProcessor {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CargoTomlState {
    Initial,
    InPackageSection,
    Stop,
}

impl ContentProcessor for CargoTomlContentProcessor {
    type Err = UpdateError;

    fn process(&self, old_contents: &str, new_version: SemVer) -> Result<String, Self::Err> {
        let mut result = String::new();
        let mut state: CargoTomlState = CargoTomlState::Initial;
        for line in old_contents.lines() {
            let mut new_line: Option<String> = None;
            match state {
                CargoTomlState::Initial => {
                    if line == "[package]" {
                        state = CargoTomlState::InPackageSection;
                    }
                }
                CargoTomlState::InPackageSection => {
                    if line.starts_with('[') {
                        state = CargoTomlState::Stop;
                    } else if is_toml_key(line, "version") {
                        new_line = Some(format!("version = \"{}\"", new_version));
                    }
                }
                CargoTomlState::Stop => {}
            }
            if let Some(x) = new_line {
                result.push_str(x.as_str());
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        Ok(result)
    }
}

fn get_package_name_from_cargo_toml(contents: &str) -> Option<&str> {
    let mut state: CargoTomlState = CargoTomlState::Initial;
    for line in contents.lines() {
        match state {
            CargoTomlState::Initial => {
                if line == "[package]" {
                    state = CargoTomlState::InPackageSection;
                }
            }
            CargoTomlState::InPackageSection => {
                if line.starts_with('[') {
                    state = CargoTomlState::Stop;
                } else if let Some(x) = get_toml_key_value(line, "name") {
                    return Some(x);
                }
            }
            CargoTomlState::Stop => {
                return None;
            }
        }
    }
    None
}

struct CargoLockProcessor<'a> {
    name: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CargoLockState {
    Initial,
    InPackageSection,
    InName,
    Stop,
}

impl<'a> ContentProcessor for CargoLockProcessor<'a> {
    type Err = UpdateError;

    fn process(&self, old_contents: &str, new_version: SemVer) -> Result<String, Self::Err> {
        let mut result = String::new();
        let mut state: CargoLockState = CargoLockState::Initial;
        for line in old_contents.lines() {
            let mut new_line: Option<String> = None;
            match state {
                CargoLockState::Initial => {
                    if line == "[[package]]" {
                        state = CargoLockState::InPackageSection;
                    }
                }
                CargoLockState::InPackageSection => {
                    if get_toml_key_value(line, "name") == Some(self.name) {
                        state = CargoLockState::InName;
                    }
                }
                CargoLockState::InName => {
                    if is_toml_key(line, "version") {
                        new_line = Some(format!("version = \"{}\"", new_version));
                        state = CargoLockState::Stop;
                    }
                }
                CargoLockState::Stop => {}
            }
            if let Some(x) = new_line {
                result.push_str(x.as_str());
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        Ok(result)
    }
}

fn is_toml_key(line: &str, key: &str) -> bool {
    if line.is_empty() || key.is_empty() {
        false
    } else if line.starts_with(key) {
        let (_, second) = line.split_at(key.len());
        second.trim_start().starts_with('=')
    } else {
        false
    }
}

fn get_toml_key_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    if line.is_empty() || key.is_empty() {
        None
    } else if line.starts_with(key) {
        let (_, second) = line.split_at(key.len());
        let second = second.trim_start();
        if second.starts_with('=') {
            let (_, second) = second.split_at(1);
            Some(second.trim_start())
        } else {
            None
        }
    } else {
        None
    }
}

pub struct CargoDirUpdater {}

impl CargoDirUpdater {
    pub fn new() -> Self {
        Self {}
    }
}

impl DirUpdater for CargoDirUpdater {
    fn update(
        &self,
        dir: &str,
        new_version: SemVer,
    ) -> Result<Vec<(PathBuf, String)>, UpdateError> {
        let dir_path_buf = PathBuf::from(dir);
        let cargo_toml_path_buf = dir_path_buf.join("Cargo.toml");
        let mut result = Vec::<(PathBuf, String)>::new();
        if cargo_toml_path_buf.is_file() {
            let processor = CargoTomlContentProcessor {};
            let old_contents = fs::read_to_string(&cargo_toml_path_buf)?;
            let new_contents = processor.process(&old_contents, new_version)?;
            if old_contents != new_contents {
                result.push((cargo_toml_path_buf, new_contents));
            }

            // processing Cargo.lock even if Cargo.toml had no changes,
            // in case someone accidentally bumped the version only on the toml file

            let cargo_lock_path_buf = dir_path_buf.join("Cargo.lock");
            if cargo_lock_path_buf.is_file() {
                if let Some(name) = get_package_name_from_cargo_toml(&old_contents) {
                    let processor = CargoLockProcessor { name };
                    let old_contents = fs::read_to_string(&cargo_lock_path_buf)?;
                    let new_contents = processor.process(&old_contents, new_version)?;
                    if old_contents != new_contents {
                        result.push((cargo_lock_path_buf, new_contents));
                    }
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::files::ContentProcessor;
    use crate::rust::{
        get_package_name_from_cargo_toml, is_toml_key, CargoLockProcessor,
        CargoTomlContentProcessor,
    };
    use crate::SemVer;

    #[test]
    fn test_cargo_toml_content_processor() {
        let toml = r#"[package]
name = "yart"
version = "0.1.0"
authors = ["Nikolaos Georgiou <nikolaos.georgiou@gmail.com>"]
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
xml-rs = "~0.8"

[dependencies.clap]
version = "~2.27.0"
default-features = false
"#;
        let expected = r#"[package]
name = "yart"
version = "1.0.0"
authors = ["Nikolaos Georgiou <nikolaos.georgiou@gmail.com>"]
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
xml-rs = "~0.8"

[dependencies.clap]
version = "~2.27.0"
default-features = false
"#;
        let processor = CargoTomlContentProcessor {};
        let result = processor.process(toml, SemVer::new(1, 0, 0)).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_package_name_from_cargo_toml() {
        let toml = r#"[package]
name = "yart"
version = "0.1.0"
authors = ["Nikolaos Georgiou <nikolaos.georgiou@gmail.com>"]
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
xml-rs = "~0.8"

[dependencies.clap]
version = "~2.27.0"
default-features = false
"#;
        let result = get_package_name_from_cargo_toml(toml).unwrap();
        assert_eq!(result, "\"yart\"");
    }

    #[test]
    fn test_cargo_lock_processor() {
        let input = r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
[[package]]
name = "bitflags"
version = "0.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4efd02e230a02e18f92fc2735f44597385ed02ad8f831e7c1c1156ee5e1ab3a5"

[[package]]
name = "clap"
version = "2.27.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1b8c532887f1a292d17de05ae858a8fe50a301e196f9ef0ddb7ccd0d1d00f180"
dependencies = [
 "bitflags",
 "textwrap",
 "unicode-width",
]

[[package]]
name = "textwrap"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c0b59b6b4b44d867f1370ef1bd91bfb262bf07bf0ae65c202ea2fbc16153b693"
dependencies = [
 "unicode-width",
]

[[package]]
name = "unicode-width"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ed742d4ea2bd1176e236172c8429aaf54486e7ac098db29ffe6529e0ce50973"

[[package]]
name = "xml-rs"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d2d7d3948613f75c98fd9328cfdcc45acc4d360655289d0a7d4ec931392200a3"

[[package]]
name = "yart"
version = "0.1.0"
dependencies = [
 "clap",
 "xml-rs",
]
"#;
        let expected = r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
[[package]]
name = "bitflags"
version = "0.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4efd02e230a02e18f92fc2735f44597385ed02ad8f831e7c1c1156ee5e1ab3a5"

[[package]]
name = "clap"
version = "2.27.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1b8c532887f1a292d17de05ae858a8fe50a301e196f9ef0ddb7ccd0d1d00f180"
dependencies = [
 "bitflags",
 "textwrap",
 "unicode-width",
]

[[package]]
name = "textwrap"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c0b59b6b4b44d867f1370ef1bd91bfb262bf07bf0ae65c202ea2fbc16153b693"
dependencies = [
 "unicode-width",
]

[[package]]
name = "unicode-width"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ed742d4ea2bd1176e236172c8429aaf54486e7ac098db29ffe6529e0ce50973"

[[package]]
name = "xml-rs"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d2d7d3948613f75c98fd9328cfdcc45acc4d360655289d0a7d4ec931392200a3"

[[package]]
name = "yart"
version = "1.0.0"
dependencies = [
 "clap",
 "xml-rs",
]
"#;
        let processor = CargoLockProcessor { name: "\"yart\"" };
        let result = processor.process(input, SemVer::new(1, 0, 0)).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_is_toml_key() {
        assert!(is_toml_key("version = 1", "version"));
        assert!(is_toml_key("version=1", "version"));
        assert!(!is_toml_key("version", "version"));
        assert!(!is_toml_key("version = 1", "name"));
    }
}
