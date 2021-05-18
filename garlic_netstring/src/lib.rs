//! Implementation of the [netstring] wire format.
//!
//! [netstring]: https://cr.yp.to/proto/netstrings.txt

use std::io::{
    self,
    Read,
};

/// Decode a netstring.
///
/// First, the length specified in the netstring is read.
/// If the specified length passes the given validation predicate `f`,
/// then the netstring payload is read into the specified buffer `buf`.
/// The reader is left precisely after the comma that terminates the netstring.
///
/// The length of the netstring is returned if it was read successfully.
/// In case the netstring could not be read, an error is returned.
pub fn decode<R, F>(r: &mut R, f: F, buf: &mut Vec<u8>) -> Result<u64, Error>
    where R: Read, F: FnOnce(u64) -> bool
{
    // Read the length specified.
    let len = decode_len(r)?;
    if !f(len) {
        return Err(Error::Length(len));
    }

    // Read the payload.
    let nread = r.take(len).read_to_end(buf)?;
    if nread as u64 != len {
        return Err(Error::Incomplete);
    }

    // Read the terminating comma.
    let mut commabuf = [0];
    r.read_exact(&mut commabuf)?;
    if commabuf[0] != b',' {
        return Err(Error::Syntax);
    }

    Ok(len)
}

fn decode_len<R>(r: &mut R) -> Result<u64, Error>
    where R: Read
{
    let mut len = 0u64;
    let mut buf = [0];
    loop {
        r.read_exact(&mut buf)?;
        match buf[0] {
            b'0' ..= b'9' => {
                let digit = buf[0] - b'0';
                len = len.checked_mul(10).ok_or(Error::Overflow)?;
                len = len.checked_add(digit as u64).ok_or(Error::Overflow)?;
            },
            b':' => return Ok(len),
            _ => return Err(Error::Syntax),
        }
    }
}

/// Error related to decoding a netstring.
pub enum Error
{
    /// The `Read` impl returned an error.
    Io(io::Error),

    /// The netstring was shorter than was specified.
    Incomplete,

    /// The length specified in the netstring
    /// did not pass the validation function.
    /// The specified length is given.
    Length(u64),

    /// The length specified in the netstring
    /// would overflow a `u64`.
    Overflow,

    /// The netstring could not be parsed
    /// because the semicolon or comma was missing.
    Syntax,
}

impl From<io::Error> for Error
{
    fn from(other: io::Error) -> Self
    {
        Error::Io(other)
    }
}

#[cfg(test)]
mod tests
{
    use {
        super::*,
        std::io::Cursor,
    };

    #[test]
    fn test_examples()
    {
        let examples: &[(&[u8], Option<(&[u8], u64)>)] = &[

            // Erroneous examples.
            (b"", None),
            (b"1:A", None),
            (b"1:AB,", None),
            (b"1:,", None),
            (b"A:1,", None),

            // Valid examples.
            (b"0:,", Some((b"", 3))),
            (b"1:A,", Some((b"A", 4))),
            (b"1:A,B", Some((b"A", 4))),
            (b"2:AB,", Some((b"AB", 5))),
            (b"2:AB,C", Some((b"AB", 5))),
            (b"13:Hello, world!,X", Some((b"Hello, world!", 17))),

        ];

        for &(input, expected) in examples {
            let mut cursor = Cursor::new(input);
            let mut actual = Vec::new();
            let result = decode(&mut cursor, |_| true, &mut actual);
            match expected {
                None => assert!(result.is_err()),
                Some((expected_payload, expected_position)) => {
                    assert_eq!(actual, expected_payload);
                    assert_eq!(cursor.position(), expected_position);
                },
            }
        }
    }
}
