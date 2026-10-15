//! Binary KeyValues ("binary VDF") reader/writer.
//!
//! Non-Steam shortcuts are stored by Steam in `shortcuts.vdf` using the binary
//! KeyValues format. The file begins with the object marker `0x00`, the root
//! name `"shortcuts"`, the child objects, and is terminated by `0x08 0x08`.
//!
//! Reference framing (verified against a live Steam install):
//! `00 73 68 6F 72 74 63 75 74 73 00 08 08`.

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Bvdf {
    /// An ordered object; insertion order is preserved for faithful round-trips.
    Object(Vec<(String, Bvdf)>),
    String(String),
    Int(i32),
    UInt(u32),
    UInt64(u64),
    Float(f32),
}

impl Bvdf {
    pub fn as_object(&self) -> Option<&Vec<(String, Bvdf)>> {
        match self {
            Bvdf::Object(entries) => Some(entries),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Bvdf::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Bvdf> {
        self.as_object()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Bvdf::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn get_i32(&self, key: &str) -> Option<i32> {
        match self.get(key)? {
            Bvdf::Int(value) => Some(*value),
            Bvdf::UInt(value) => Some(*value as i32),
            _ => None,
        }
    }
}

/// Parse a binary KeyValues document.
pub fn parse(bytes: &[u8]) -> AppResult<Bvdf> {
    let mut parser = BinParser { bytes, pos: 0 };
    let mut entries = Vec::new();
    while let Some(entry) = parser.read_element()? {
        entries.push(entry);
    }
    Ok(Bvdf::Object(entries))
}

/// Serialize a binary KeyValues document, appending the trailing root terminator.
pub fn write(value: &Bvdf) -> Vec<u8> {
    let mut out = Vec::new();
    if let Bvdf::Object(entries) = value {
        for (key, child) in entries {
            write_element(key, child, &mut out);
        }
    }
    // Steam terminates the implicit root object with an extra 0x08.
    out.push(0x08);
    out
}

fn write_element(key: &str, value: &Bvdf, out: &mut Vec<u8>) {
    match value {
        Bvdf::Object(entries) => {
            out.push(0x00);
            write_cstr(key, out);
            for (k, v) in entries {
                write_element(k, v, out);
            }
            out.push(0x08);
        }
        Bvdf::String(text) => {
            out.push(0x01);
            write_cstr(key, out);
            write_cstr(text, out);
        }
        Bvdf::Int(value) => {
            out.push(0x02);
            write_cstr(key, out);
            out.extend_from_slice(&value.to_le_bytes());
        }
        Bvdf::Float(value) => {
            out.push(0x03);
            write_cstr(key, out);
            out.extend_from_slice(&value.to_le_bytes());
        }
        Bvdf::UInt(value) => {
            out.push(0x04);
            write_cstr(key, out);
            out.extend_from_slice(&value.to_le_bytes());
        }
        Bvdf::UInt64(value) => {
            out.push(0x07);
            write_cstr(key, out);
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
}

fn write_cstr(text: &str, out: &mut Vec<u8>) {
    out.extend_from_slice(text.as_bytes());
    out.push(0x00);
}

struct BinParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> BinParser<'a> {
    fn read_u8(&mut self) -> AppResult<u8> {
        let byte = *self
            .bytes
            .get(self.pos)
            .ok_or_else(|| AppError::BinVdf("unexpected end of input".into()))?;
        self.pos += 1;
        Ok(byte)
    }

    fn read_cstr(&mut self) -> AppResult<String> {
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != 0x00 {
            self.pos += 1;
        }
        if self.pos >= self.bytes.len() {
            return Err(AppError::BinVdf("unterminated string".into()));
        }
        let slice = &self.bytes[start..self.pos];
        self.pos += 1; // consume null
        Ok(String::from_utf8_lossy(slice).into_owned())
    }

    fn read_bytes<const N: usize>(&mut self) -> AppResult<[u8; N]> {
        if self.pos + N > self.bytes.len() {
            return Err(AppError::BinVdf("unexpected end of input".into()));
        }
        let mut buf = [0u8; N];
        buf.copy_from_slice(&self.bytes[self.pos..self.pos + N]);
        self.pos += N;
        Ok(buf)
    }

    /// Read one `(key, value)` element, or `None` at the object terminator/EOF.
    fn read_element(&mut self) -> AppResult<Option<(String, Bvdf)>> {
        if self.pos >= self.bytes.len() {
            return Ok(None);
        }
        let node_type = self.read_u8()?;
        match node_type {
            0x08 => Ok(None),
            0x00 => {
                let key = self.read_cstr()?;
                let mut entries = Vec::new();
                while let Some(child) = self.read_element()? {
                    entries.push(child);
                }
                Ok(Some((key, Bvdf::Object(entries))))
            }
            0x01 => {
                let key = self.read_cstr()?;
                let value = self.read_cstr()?;
                Ok(Some((key, Bvdf::String(value))))
            }
            0x02 => {
                let key = self.read_cstr()?;
                Ok(Some((
                    key,
                    Bvdf::Int(i32::from_le_bytes(self.read_bytes()?)),
                )))
            }
            0x03 => {
                let key = self.read_cstr()?;
                Ok(Some((
                    key,
                    Bvdf::Float(f32::from_le_bytes(self.read_bytes()?)),
                )))
            }
            0x04 => {
                let key = self.read_cstr()?;
                Ok(Some((
                    key,
                    Bvdf::UInt(u32::from_le_bytes(self.read_bytes()?)),
                )))
            }
            0x07 => {
                let key = self.read_cstr()?;
                Ok(Some((
                    key,
                    Bvdf::UInt64(u64::from_le_bytes(self.read_bytes()?)),
                )))
            }
            0x05 => {
                // Wide (UTF-16LE) string: read until a 0x0000 terminator.
                let key = self.read_cstr()?;
                let start = self.pos;
                while self.pos + 1 < self.bytes.len()
                    && !(self.bytes[self.pos] == 0 && self.bytes[self.pos + 1] == 0)
                {
                    self.pos += 2;
                }
                let slice = &self.bytes[start..self.pos];
                self.pos += 2;
                let text = String::from_utf16_lossy(
                    &slice
                        .chunks_exact(2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .collect::<Vec<_>>(),
                );
                Ok(Some((key, Bvdf::String(text))))
            }
            other => Err(AppError::BinVdf(format!("unknown node type 0x{other:02x}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_empty_shortcuts() {
        let bytes = [
            0x00, b's', b'h', b'o', b'r', b't', b'c', b'u', b't', b's', 0x00, 0x08, 0x08,
        ];
        let parsed = parse(&bytes).unwrap();
        let shortcuts = parsed.get("shortcuts").unwrap();
        assert!(shortcuts.as_object().unwrap().is_empty());
    }

    #[test]
    fn round_trips_shortcut() {
        let empty = [
            0x00, b's', b'h', b'o', b'r', b't', b'c', b'u', b't', b's', 0x00, 0x08, 0x08,
        ];
        let parsed = parse(&empty).unwrap();
        let written = write(&parsed);
        assert_eq!(written, empty);
    }
}
