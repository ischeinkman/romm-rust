use std::{hash::Hash, num::ParseIntError, ops::Deref, fmt, str::FromStr, time::Duration};

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Helper to serialize & deserialize a field that can either be a single value
/// or a list of values.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum FlattenedList<T> {
    #[default]
    None,
    Single(T),
    Multiple(Vec<T>),
}

impl<T> FlattenedList<T> {
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
    pub fn as_slice(&self) -> &[T] {
        match self {
            FlattenedList::None => &[],
            FlattenedList::Multiple(v) => v.as_slice(),
            FlattenedList::Single(itm) => std::slice::from_ref(itm),
        }
    }
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (FlattenedList::None, ret) | (ret, FlattenedList::None) => ret,
            (FlattenedList::Single(a), FlattenedList::Single(b)) => {
                FlattenedList::Multiple(vec![a, b])
            }
            (FlattenedList::Multiple(mut a), FlattenedList::Multiple(b)) => {
                a.extend(b);
                FlattenedList::Multiple(a)
            }
            (FlattenedList::Multiple(mut v), FlattenedList::Single(end)) => {
                v.push(end);
                FlattenedList::Multiple(v)
            }
            (FlattenedList::Single(start), FlattenedList::Multiple(mut v)) => {
                v.insert(0, start);
                FlattenedList::Multiple(v)
            }
        }
    }
}

impl<T> From<FlattenedList<T>> for Vec<T> {
    fn from(value: FlattenedList<T>) -> Self {
        match value {
            FlattenedList::None => Vec::new(),
            FlattenedList::Multiple(v) => v,
            FlattenedList::Single(itm) => vec![itm],
        }
    }
}

impl<T> Hash for FlattenedList<T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        T::hash_slice(self.as_slice(), state);
    }
}

impl<T> PartialEq for FlattenedList<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T> Eq for FlattenedList<T> where T: Eq {}

/// Helper for parsing a [`Duration`] from a number and a unit.
///
/// # Examples
/// ```ignore
/// # use syncer_model::config::loading::ParseableDuration;
/// let duration : ParsableDuration = serde_json::from_str("2h").unwrap();
/// assert_eq!(duration.as_nanos(), 60 * 60 * 1000 * 1000 * 1000);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, PartialOrd, Ord)]
pub struct ParseableDuration(Duration);

impl ParseableDuration {
    pub const fn new(inner : Duration) -> Self {
        Self(inner)
    }
}

const DURATION_SUFFIXES: &[(u64, &str)] = &[
    (0, "ns"),
    (1000, "us"),
    (1000 * 1000, "ms"),
    (1000 * 1000 * 1000, "s"),
    (60 * 1000 * 1000 * 1000, "m"),
    (60 * 60 * 1000 * 1000 * 1000, "h"),
    (24 * 60 * 60 * 1000 * 1000 * 1000, "d"),
];

const fn nanos_to_unitted(nanos: u128) -> (u64, &'static str) {
    let mut idx = 1;
    loop {
        if idx >= DURATION_SUFFIXES.len() {
            break;
        }
        let (cur_stop_point, _) = DURATION_SUFFIXES[idx];
        let cur_stop_point = cur_stop_point as u128;
        if nanos % cur_stop_point != 0 {
            idx -= 1;
            break;
        }
        idx += 1;
    }
    let (coeff, suffix) = DURATION_SUFFIXES[idx];
    let coeff = coeff as u128;
    if coeff == 0 {
        (nanos as _, suffix)
    } else {
        ((nanos / coeff) as _, suffix)
    }
}

impl AsRef<Duration> for ParseableDuration {
    fn as_ref(&self) -> &Duration {
        &self.0
    }
}

impl Deref for ParseableDuration {
    type Target = Duration;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum ParseDurationError {
    #[error(transparent)]
    InvalidInteger(#[from] ParseIntError),
    #[error("Invalid suffix: {0}")]
    InvalidSuffix(String),
}
impl FromStr for ParseableDuration {
    type Err = ParseDurationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
        let (n, suffix) = s.split_at(split);
        let coeff = DURATION_SUFFIXES
            .iter()
            .find(|(_, check)| check.eq_ignore_ascii_case(suffix))
            .map(|(k, _)| *k)
            .ok_or_else(|| ParseDurationError::InvalidSuffix(suffix.to_owned()))?;

        let n = n.parse::<u64>()?;
        Ok(ParseableDuration(Duration::from_nanos(n * coeff)))
    }
}

impl fmt::Display for ParseableDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (n, suffix) = nanos_to_unitted(self.0.as_nanos());
        write!(f, "{n}{suffix}")
    }
}

impl Serialize for ParseableDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ParseableDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ParseDurationVisitor)
    }
}

struct ParseDurationVisitor;

impl Visitor<'_> for ParseDurationVisitor {
    type Value = ParseableDuration;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a number with a unit suffix")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        ParseableDuration::from_str(v).map_err(E::custom)
    }
}

impl From<ParseableDuration> for Duration {
    fn from(value: ParseableDuration) -> Self {
        value.0
    }
}
impl From<Duration> for ParseableDuration {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_suffixes() {
        assert_eq!(nanos_to_unitted(500), (500, "ns"));
        assert_eq!(nanos_to_unitted(1000), (1, "us"));
        assert_eq!(nanos_to_unitted(1001), (1001, "ns"));
    }
}
