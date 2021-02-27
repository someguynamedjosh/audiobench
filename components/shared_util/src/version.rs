use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer,
};
use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Version {
    pub maj: u8,
    pub min: u8,
    pub patch: u8,
}

impl Version {
    pub fn new(maj: u8, min: u8, patch: u8) -> Self {
        assert!(maj < 16);
        assert!(min < 32);
        assert!(patch < 128);
        Self { maj, min, patch }
    }

    pub fn compatible_for(self, requirement: Version) -> bool {
        if self.maj == 0 && requirement.maj == 0 {
            self.min == requirement.min && self.patch >= requirement.patch
        } else if self.maj == requirement.maj {
            if self.min == requirement.min {
                self.patch >= requirement.patch
            } else {
                self.min > requirement.min
            }
        } else {
            false
        }
    }

    pub const unsafe fn new_unchecked(maj: u8, min: u8, patch: u8) -> Self {
        Self { maj, min, patch }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.maj, self.min, self.patch)
    }
}

impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(());
        }
        let maj = parts[0].parse().map_err(|_| ())?;
        let min = parts[1].parse().map_err(|_| ())?;
        let patch = parts[2].parse().map_err(|_| ())?;
        if maj >= 16 || min >= 32 || patch >= 128 {
            return Err(());
        }
        Ok(Self { maj, min, patch })
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(VersionVisitor)
    }
}

struct VersionVisitor;

impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a version number in the form maj.min.patch")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Version::from_str(v).map_err(|_| E::custom(format!("incorrectly formatted version")))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Self::cmp(self, other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // If major is less or greater than the other major version, it doesn't matter what the
        // other items are.
        let maj = self.maj.cmp(&other.maj);
        if maj != Ordering::Equal {
            return maj;
        }

        let min = self.min.cmp(&other.min);
        if min != Ordering::Equal {
            return min;
        }

        self.patch.cmp(&other.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmp2(a: &str, expected_result: Ordering, b: &str) {
        let a: Version = a.parse().unwrap();
        let b: Version = b.parse().unwrap();
        assert_eq!(a.cmp(&b), expected_result);
    }

    #[test]
    fn version_cmp() {
        use Ordering::*;
        cmp2("1.0.0", Greater, "0.9.8");
        cmp2("0.1.0", Greater, "0.0.9");
        cmp2("0.0.2", Equal, "0.0.2");
        cmp2("1.3.5", Equal, "1.3.5");
        cmp2("1.3.4", Less, "1.3.5");
        cmp2("1.1.9", Less, "1.3.5");
        cmp2("0.9.9", Less, "1.3.5");
    }
}
