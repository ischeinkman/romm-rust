use futures::{Stream, TryStreamExt};
use md5::{Digest, Md5};
use std::fmt::{self};
use std::io::{self, Read};
use std::str::FromStr;
use thiserror::Error;

/// Represents a save file's MD5 hash for checking if 2 saves are the same.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Md5Hash([u8; Self::LENGTH]);

impl Md5Hash {
    pub const LENGTH: usize = 16;
    pub const fn from_raw(raw: [u8; Self::LENGTH]) -> Self {
        Self(raw)
    }
    pub const fn as_bytes(&self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl fmt::LowerHex for Md5Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            let high = byte >> 4;
            let low = byte & 0xF;
            fmt::LowerHex::fmt(&high, f)?;
            fmt::LowerHex::fmt(&low, f)?;
        }
        Ok(())
    }
}
impl fmt::UpperHex for Md5Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            let high = byte >> 4;
            let low = byte & 0xF;
            fmt::UpperHex::fmt(&high, f)?;
            fmt::UpperHex::fmt(&low, f)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Md5Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Md5Hash({:?})", self.0)
    }
}

impl fmt::Display for Md5Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self, f)
    }
}

fn parse_delimitted_nibble(raw: char) -> Result<Option<u8>, Md5ParseError> {
    match raw {
        '-' | '_' => Ok(None),
        n if n.is_whitespace() => Ok(None),
        c @ '0'..='9' => Ok(Some((c as u8) - b'0')),
        c @ 'a'..='f' => Ok(Some((c as u8) - b'a' + 10)),
        c @ 'A'..='F' => Ok(Some((c as u8) - b'A' + 10)),
        c => Err(Md5ParseError::InvalidChar(c)),
    }
}

impl FromStr for Md5Hash {
    type Err = Md5ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut idx = 0;
        let mut buff = [0; Md5Hash::LENGTH];
        let mut itr = s.chars();
        loop {
            let high = loop {
                let nxt_char = itr.next().ok_or(Md5ParseError::TooFewCharacters)?;
                if let Some(n) = parse_delimitted_nibble(nxt_char)? {
                    break n;
                }
            };
            let low = loop {
                let nxt_char = itr.next().ok_or(Md5ParseError::TooFewCharacters)?;
                if let Some(n) = parse_delimitted_nibble(nxt_char)? {
                    break n;
                }
            };
            let nxt = (high << 4) | low;
            buff[idx] = nxt;
            idx += 1;
            if idx == buff.len() {
                break;
            }
        }
        loop {
            match itr.next() {
                None => break,
                Some('-') | Some('_') => {}
                Some(c) if c.is_whitespace() => {}
                Some(_) => {
                    return Err(Md5ParseError::TooManyCharacters);
                }
            }
        }
        Ok(Self(buff))
    }
}

#[derive(Debug, Error)]
pub enum Md5ParseError {
    #[error("Invalid character in md5 hash: {0}")]
    InvalidChar(char),
    #[error("Too many characters in md5 hash. Expected 32.")]
    TooManyCharacters,
    #[error("Too few characters in md5 hash. Expected 32.")]
    TooFewCharacters,
}

/// Asynchronously calculates the MD5 hash of a byte stream.
pub async fn md5_stream<S, B, E>(stream: S) -> Result<Md5Hash, E>
where
    S: Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
{
    let mut hasher = Md5::new();
    futures::pin_mut!(stream);
    while let Some(nxt) = stream.try_next().await? {
        hasher.update(nxt.as_ref());
    }
    let hash: [u8; Md5Hash::LENGTH] = hasher.finalize().into();
    Ok(Md5Hash(hash))
}

/// Synchronously parsing the MD5 hash of an [`io::Read`] instance.
pub fn md5(mut rdr: impl Read) -> io::Result<Md5Hash> {
    let mut hasher = Md5::new();
    let mut buffer = [0u8; 4 * 1024];
    loop {
        let count = match rdr.read(&mut buffer) {
            Ok(0) => {
                break;
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Ok(n) => n,
            Err(e) => {
                return Err(e);
            }
        };
        let subslice = &buffer[..count];
        hasher.update(subslice);
    }
    let hash: [u8; Md5Hash::LENGTH] = hasher.finalize().into();
    Ok(Md5Hash(hash))
}

#[cfg(test)]
mod tests {
    use std::{
        future::Future,
        io::Cursor,
        task::{Context, Poll},
    };

    use super::*;

    #[test]
    fn test_parsing_stream() {
        const DATA: &[u8] = b"hello world, this is some test data";
        const EXPECTED: Md5Hash = Md5Hash::from_raw([
            0xa9, 0x6f, 0xf3, 0x7e, 0x9d, 0x45, 0x7f, 0x37, 0x7e, 0xd0, 0xed, 0x16, 0x92, 0x24,
            0xd5, 0x77,
        ]);
        for buffer_size in [1, 2, 4, DATA.len() / 2, DATA.len()] {
            let mut idx = 0;
            let itr = std::iter::from_fn(move || {
                if idx >= DATA.len() {
                    return None;
                }
                let buffer = &DATA[idx..(idx + buffer_size).min(DATA.len())];
                idx += buffer_size;
                Some(buffer)
            })
            .map(Result::<_, anyhow::Error>::Ok);

            let mut fut = Box::pin(md5_stream(futures::stream::iter(itr)));
            let res = loop {
                let fut = fut.as_mut();
                match fut.poll(&mut Context::from_waker(futures::task::noop_waker_ref())) {
                    Poll::Pending => {
                        continue;
                    }
                    Poll::Ready(res) => {
                        break res;
                    }
                }
            };
            let expected = res.unwrap();
            assert_eq!(expected, EXPECTED);

            let lower_actual = Md5Hash::from_str(&format!("{expected:x}")).unwrap();
            assert_eq!(expected, lower_actual);
            let upper_actual = Md5Hash::from_str(&format!("{expected:X}")).unwrap();
            assert_eq!(expected, upper_actual);
        }
    }

    #[test]
    fn test_parsing_reader() {
        const DATA: &[u8] = b"hello world, this is some test data";
        const EXPECTED: Md5Hash = Md5Hash::from_raw([
            0xa9, 0x6f, 0xf3, 0x7e, 0x9d, 0x45, 0x7f, 0x37, 0x7e, 0xd0, 0xed, 0x16, 0x92, 0x24,
            0xd5, 0x77,
        ]);
        let cursor = Cursor::new(DATA);
        let expected = md5(cursor).unwrap();
        assert_eq!(expected, EXPECTED);

        let lower_actual = Md5Hash::from_str(&format!("{expected:x}")).unwrap();
        assert_eq!(expected, lower_actual);
        let upper_actual = Md5Hash::from_str(&format!("{expected:X}")).unwrap();
        assert_eq!(expected, upper_actual);
    }
}
