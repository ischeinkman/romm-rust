use std::collections::HashSet;
use std::ffi::OsStr;
use std::hash::Hash;
use std::path::Path;
use std::{borrow::Borrow, collections::HashMap};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A template for matching & extracting information from filesystem paths.
///
/// # Syntax
/// The syntax is very similar to shell variable substitutions. Variables within
/// the string are given by a `$` followed by an alpha-numeric name, and
/// constant portions are anything other than that. When resolving the variable
/// values from a string the algorithm attempts to match as much as possible.
///
/// # Examples
///
/// Given the format string `/root/subroot/$DIRNAME/$FILE-$TIMESTAMP.$EXT`:
/// * The [`FormatString::prefix`] value is `/root/subroot/`.
/// * The string `/root/subroot/my-dir/my-rom-file-ts.sav` will produce a
///   [`FormatString::resolve`] value of `{"$DIRNAME" : "my-dir", "$FILE" :
///   "my-rom-file", "$TIMESTAMP" : "ts", $"EXT" : "sav"}`.
///  
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormatString(String);

impl From<String> for FormatString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a str> for FormatString {
    fn from(value: &'a str) -> Self {
        value.to_owned().into()
    }
}

impl FormatString {
    pub fn build_with_vars<K, V>(&self, vars: &HashMap<K, V>) -> String
    where
        K: Borrow<str> + Eq + Hash,
        V: AsRef<str>,
    {
        let mut retvl = String::new();
        for portion in split_variable_portions(&self.0) {
            if portion.starts_with("$") {
                let nxt = vars.get(portion).map(|s| s.as_ref()).unwrap_or(portion);
                retvl.push_str(nxt);
            } else {
                retvl.push_str(portion);
            }
        }
        retvl
    }
    pub fn prefix(&self) -> &str {
        split_variable_portions(&self.0).next().unwrap_or("/")
    }
    pub fn variables(&self) -> HashSet<&str> {
        split_variable_portions(&self.0)
            .filter(|s| s.starts_with("$"))
            .collect()
    }

    /// Checks whether or not the given [`Path`] matches this [`FormatString`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// let format : FormatString = "/prefix/$EMULATOR/$ROM-$TIMESTAMP.$EXT".into();
    /// assert!(format.matches_path(Path::new("/prefix/my-emulator/my-rom-with-ts.sav")));
    /// assert!(!format.matches_path(Path::new("/prefix/my-emulator/my-rom-folder/my-rom-file.sav")));
    /// ```
    pub fn matches_path(&self, path: &Path) -> bool {
        self.resolve(path).is_ok()
    }
    pub fn matches(&self, s: &str) -> bool {
        self.matches_path(Path::new(s))
    }
    /// Extracts the `$`-delimited values from a file path using the given format string.
    ///
    /// # Examples
    /// ```rust
    /// let format : FormatString = "/long/prefix/$EMULATOR/$ROM.$EXT".into();
    /// let path = "/long/prefix/mgba/testrom.sav";
    /// let resolved = resolve_metadata_variables(format, Path::new(path)).unwrap();
    /// assert_eq!(resolved.len(), 3);
    /// assert_eq!(resolved.get("$EMULATOR").unwrap(), "mgba");
    /// assert_eq!(resolved.get("$ROM").unwrap(), "testrom");
    /// assert_eq!(resolved.get("$EXT").unwrap(), "save");
    /// ```
    pub fn resolve<'a, 'b>(
        &'a self,
        file: &'b Path,
    ) -> Result<HashMap<String, String>, MetadataResolveError<'a, 'b>> {
        let mut retvl = HashMap::new();
        let format_components = self.0.rsplit('/').filter(|s| !s.is_empty());
        let mut file_components = file.components().rev();
        if self.0.ends_with('/') {
            file_components.next();
        }
        for (fmt_comp, file_comp) in format_components.zip(file_components) {
            if !fmt_comp.contains('$') {
                if file_comp.as_os_str() != fmt_comp {
                    return Err(MetadataResolveError::ComponentMismatch {
                        expected: fmt_comp,
                        found: file_comp.as_os_str(),
                    });
                } else {
                    continue;
                }
            }

            let mapped_component = file_comp.as_os_str().to_string_lossy();
            let mut cur = &*mapped_component;
            let mut variable_itr = split_variable_portions(fmt_comp).peekable();
            while let Some(cur_portion) = variable_itr.next() {
                if cur_portion.starts_with('$') {
                    let (next_value, next_cur) =
                        consume_next_variable(cur, variable_itr.peek().copied()).ok_or(
                            MetadataResolveError::ComponentFormat {
                                expected: cur_portion,
                                found: cur.to_string(),
                            },
                        )?;
                    retvl.insert(cur_portion.to_owned(), next_value.to_owned());
                    cur = next_cur;
                } else {
                    match cur.split_once(cur_portion) {
                        Some(("", rest)) => {
                            cur = rest;
                        }
                        _ => {
                            return Err(MetadataResolveError::ComponentFormat {
                                expected: cur_portion,
                                found: cur.to_string(),
                            })
                        }
                    }
                }
            }
        }
        Ok(retvl)
    }
}

#[derive(Debug, Error)]
pub enum MetadataResolveError<'a, 'b> {
    #[error("Error matching path: expected component {expected}, found {found:?}")]
    ComponentMismatch { expected: &'a str, found: &'b OsStr },
    #[error("Error matching component format: expected format {expected}, found {found:?}")]
    ComponentFormat { expected: &'a str, found: String },
}

/// Splits a format string containing replacement variables into a stream of
/// constants to be matched and `$`-prefixed variable names to be extracted.
///
/// # Example
/// ```rust
/// let format = "/long/prefix/root/$EMULATOR/$ROM.$EXT";
/// let split = split_variable_portions(format).collect::<Vec<_>>();
/// let expected = &["/long/prefix/root/", "$EMULATOR", "/", "$ROM", ".", "$EXT"];
/// assert_eq!(expected, split.as_slice());
/// ```
fn split_variable_portions(raw: &str) -> impl Iterator<Item = &str> + '_ {
    let mut head = 0;
    let mut idx = 0;
    std::iter::from_fn(move || loop {
        if head >= raw.len() {
            return None;
        }
        if idx >= raw.len() {
            let retvl = &raw[head..];
            head = idx;
            return Some(retvl);
        }
        let in_variable = raw.as_bytes()[head] == b'$';
        let on_variable = raw.as_bytes()[idx] == b'$';
        if on_variable || (in_variable && !raw.as_bytes()[idx].is_ascii_alphanumeric()) {
            let retvl = &raw[head..idx];
            head = idx;
            idx += 1;
            return Some(retvl);
        }
        idx += 1;
    })
}

fn consume_next_variable<'a>(
    s: &'a str,
    next_component: Option<&str>,
) -> Option<(&'a str, &'a str)> {
    match next_component {
        None => Some((s, "")),
        Some(nxt) => {
            let (head, _) = s.split_once(nxt)?;
            let tail = &s[head.len()..];
            Some((head, tail))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve() {
        let format: FormatString = "/long/prefix/$EMULATOR/$ROM.$EXT".into();
        let path = "/root/subdir/long/prefix/mgba/testrom.sav";
        let resolved = format.resolve(Path::new(path)).unwrap();
        assert_eq!(resolved.len(), 3);
        assert_eq!(resolved.get("$EMULATOR").unwrap(), "mgba");
        assert_eq!(resolved.get("$ROM").unwrap(), "testrom");
        assert_eq!(resolved.get("$EXT").unwrap(), "sav");
    }

    #[test]
    fn test_split_variables() {
        let format = "/long/prefix/root/$EMULATOR/$ROM.$EXT";
        let split = split_variable_portions(format).collect::<Vec<_>>();
        let expected = &["/long/prefix/root/", "$EMULATOR", "/", "$ROM", ".", "$EXT"];
        assert_eq!(expected, split.as_slice());
    }

    #[test]
    fn test_consume() {
        let data = "a.b.c.d";
        let next_component = Some(".");
        assert_eq!(
            ("a", ".b.c.d"),
            consume_next_variable(data, next_component).unwrap()
        );
    }
}
