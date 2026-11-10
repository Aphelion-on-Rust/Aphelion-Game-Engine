use std::cmp::Ordering;
use std::fmt;

/// A version like 0.7.0 or 1.0.0-beta.2. A leading "v" (like in a git tag) is accepted.
/// Versions are stored as plain text in the YAML files and parsed when they need comparing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    /// The part after the dash, like "beta.2". A version with it is a pre-release.
    pub pre: Option<String>,
}

impl Version {
    pub fn parse(text: &str) -> Option<Version> {
        let text = text.trim();
        let text = text.strip_prefix('v').unwrap_or(text);

        let (numbers, pre) = match text.split_once('-') {
            Some((numbers, pre)) => (numbers, Some(pre)),
            None => (text, None),
        };

        if pre == Some("") {
            return None;
        }

        let mut parts = numbers.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;

        if parts.next().is_some() {
            return None;
        }

        Some(Version { major, minor, patch, pre: pre.map(str::to_string) })
    }

    /// For sorting text versions. Anything unreadable counts as 0.0.0, the lowest.
    pub fn parse_or_zero(text: &str) -> Version {
        Version::parse(text).unwrap_or(Version { major: 0, minor: 0, patch: 0, pre: None })
    }

    pub fn is_prerelease(&self) -> bool {
        self.pre.is_some()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        match &self.pre {
            Some(pre) => write!(f, "-{pre}"),
            None => Ok(()),
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            .then_with(|| compare_pre(&self.pre, &other.pre))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 1.0.0-beta is older than 1.0.0.
fn compare_pre(a: &Option<String>, b: &Option<String>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a), Some(b)) => compare_identifiers(a, b),
    }
}

/// "beta.2" vs "beta.11": parts are compared one by one, numbers as numbers.
fn compare_identifiers(a: &str, b: &str) -> Ordering {
    let mut left = a.split('.');
    let mut right = b.split('.');

    loop {
        match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let order = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(m), Ok(n)) => m.cmp(&n),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => x.cmp(y),
                };

                if order != Ordering::Equal {
                    return order;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).unwrap()
    }

    #[test]
    fn versions_parse_with_or_without_a_v() {
        assert_eq!(v("0.7.0"), v("v0.7.0"));
        assert_eq!(v("1.2.3-beta.1").pre.as_deref(), Some("beta.1"));
        assert_eq!(v("1.2.3-beta.1").to_string(), "1.2.3-beta.1");
    }

    #[test]
    fn bad_versions_are_refused() {
        for text in ["", "1.2", "a.b.c", "1.2.3.4", "1.2.3-", "nightly"] {
            assert!(Version::parse(text).is_none(), "{text} should not parse");
        }
    }

    #[test]
    fn numbers_compare_as_numbers() {
        assert!(v("0.10.0") > v("0.9.0"));
        assert!(v("1.0.0") > v("0.99.99"));
    }

    #[test]
    fn a_pre_release_is_older_than_its_release() {
        assert!(v("1.0.0-beta.1") < v("1.0.0"));
        assert!(v("1.0.0-beta.2") < v("1.0.0-beta.11"));
        assert!(v("1.0.0-alpha") < v("1.0.0-beta"));
    }
}
