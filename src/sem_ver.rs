use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

/// Defines the possible components of a semantic version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemVerComponent {
    Major,
    Minor,
    Patch,
}

impl FromStr for SemVerComponent {
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

/// A set of semantic version components
/// implemented by bit flags.
pub struct SemVerComponentSet(u8);

impl SemVerComponentSet {
    /// Creates a new empty set.
    pub fn new() -> Self {
        Self(0)
    }

    /// Returns an iterator that yields the sem ver components
    /// that are missing from this set.
    ///
    /// Note that the iterator is not linked to the set.
    /// Its state captures the state of the set at the time
    /// of the method invocation. If the set changes
    /// while the iterator is used, changes will not be reflected
    /// to the iterator.
    pub fn missing(&self) -> impl Iterator<Item = SemVerComponent> {
        MissingSemVerIterator(self.0)
    }

    pub fn all() -> impl Iterator<Item = SemVerComponent> {
        Self::new().missing()
    }

    /// Maps the sem ver component to a bit flag.
    fn component_to_flag(component: SemVerComponent) -> u8 {
        match component {
            SemVerComponent::Major => 4,
            SemVerComponent::Minor => 2,
            SemVerComponent::Patch => 1,
        }
    }
}

impl std::ops::AddAssign<SemVerComponent> for SemVerComponentSet {
    fn add_assign(&mut self, rhs: SemVerComponent) {
        self.0 |= Self::component_to_flag(rhs);
    }
}

/// Implementation for the iterator at [SemVerComponentSet](SemVerComponentSet::missing).
struct MissingSemVerIterator(u8);

impl Iterator for MissingSemVerIterator {
    type Item = SemVerComponent;

    fn next(&mut self) -> Option<Self::Item> {
        for component in &[
            SemVerComponent::Major,
            SemVerComponent::Minor,
            SemVerComponent::Patch,
        ] {
            let flag = SemVerComponentSet::component_to_flag(*component);
            // if the flag is missing...
            if (self.0 & flag) == 0 {
                // ... then add it so we don't return it again
                self.0 |= flag;
                return Some(*component);
            }
        }
        None
    }
}

/// Represents a semantic version.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SemVer {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn bump(&self, step: SemVerComponent) -> Self {
        match step {
            SemVerComponent::Major => Self::new(self.major + 1, 0, 0),
            SemVerComponent::Minor => Self::new(self.major, self.minor + 1, 0),
            SemVerComponent::Patch => Self::new(self.major, self.minor, self.patch + 1),
        }
    }

    pub fn get_component(&self, component: SemVerComponent) -> u16 {
        match component {
            SemVerComponent::Major => self.major,
            SemVerComponent::Minor => self.minor,
            SemVerComponent::Patch => self.patch,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sem_ver_bump() {
        assert_eq!(
            SemVer::new(1, 0, 0).bump(SemVerComponent::Major),
            SemVer::new(2, 0, 0)
        );
        assert_eq!(
            SemVer::new(1, 2, 0).bump(SemVerComponent::Major),
            SemVer::new(2, 0, 0)
        );
        assert_eq!(
            SemVer::new(1, 2, 3).bump(SemVerComponent::Major),
            SemVer::new(2, 0, 0)
        );
        assert_eq!(
            SemVer::new(1, 0, 0).bump(SemVerComponent::Minor),
            SemVer::new(1, 1, 0)
        );
        assert_eq!(
            SemVer::new(1, 2, 0).bump(SemVerComponent::Minor),
            SemVer::new(1, 3, 0)
        );
        assert_eq!(
            SemVer::new(1, 2, 3).bump(SemVerComponent::Minor),
            SemVer::new(1, 3, 0)
        );
        assert_eq!(
            SemVer::new(1, 0, 0).bump(SemVerComponent::Patch),
            SemVer::new(1, 0, 1)
        );
        assert_eq!(
            SemVer::new(1, 2, 0).bump(SemVerComponent::Patch),
            SemVer::new(1, 2, 1)
        );
        assert_eq!(
            SemVer::new(1, 2, 3).bump(SemVerComponent::Patch),
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
}
