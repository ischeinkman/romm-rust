use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetadataResolveError<'a, 'b> {
    #[error("Error matching path: expected component {expected}, found {found:?}")]
    ComponentMismatch { expected: &'a str, found: &'b OsStr },
    #[error("Error matching component format: expected format {expected}, found {found:?}")]
    ComponentFormat { expected: &'a str, found: String },
}

pub fn path_matches(format: &str, path: &Path) -> bool {
    resolve_metadata_variables(format, path).is_ok()
}

/// Extracts the `$`-delimited values from a file path using the given format string.
///
/// # Examples
/// ```rust
/// let format = "/long/prefix/$EMULATOR/$ROM.$EXT";
/// let path = "/long/prefix/mgba/testrom.sav";
/// let resolved = resolve_metadata_variables(format, Path::new(path)).unwrap();
/// assert_eq!(resolved.len(), 3);
/// assert_eq!(resolved.get("$EMULATOR").unwrap(), "mgba");
/// assert_eq!(resolved.get("$ROM").unwrap(), "testrom");
/// assert_eq!(resolved.get("$EXT").unwrap(), "save");
/// ```
pub fn resolve_metadata_variables<'a, 'b>(
    format: &'a str,
    file: &'b Path,
) -> Result<HashMap<String, String>, MetadataResolveError<'a, 'b>> {
    let mut retvl = HashMap::new();
    let format_components = format.rsplit('/').filter(|s| !s.is_empty());
    let mut file_components = file.components().rev();
    if format.ends_with('/') {
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

pub fn format_root(raw: &str) -> &str {
    split_variable_portions(raw).next().unwrap_or("/")
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
        let format = "/long/prefix/$EMULATOR/$ROM.$EXT";
        let path = "/root/subdir/long/prefix/mgba/testrom.sav";
        let resolved = resolve_metadata_variables(format, Path::new(path)).unwrap();
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
