//! Valve KeyValues (text "VDF") parser and serializer.
//!
//! Steam stores most of its local state as KeyValues text files
//! (`libraryfolders.vdf`, `appmanifest_*.acf`, `localconfig.vdf`,
//! `loginusers.vdf`). This module implements a tolerant reader and a
//! deterministic writer so we can round-trip those files without losing data.

use std::collections::BTreeMap;

use crate::error::{AppError, AppResult};

/// A parsed KeyValues node: either an object (map of children) or a string leaf.
#[derive(Debug, Clone, PartialEq)]
pub enum Vdf {
    Object(BTreeMap<String, Vdf>),
    String(String),
}

impl Vdf {
    pub fn object() -> Self {
        Vdf::Object(BTreeMap::new())
    }

    pub fn string(value: impl Into<String>) -> Self {
        Vdf::String(value.into())
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, Vdf>> {
        match self {
            Vdf::Object(map) => Some(map),
            Vdf::String(_) => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, Vdf>> {
        match self {
            Vdf::Object(map) => Some(map),
            Vdf::String(_) => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Vdf::String(value) => Some(value),
            Vdf::Object(_) => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vdf> {
        self.as_object()?.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Vdf> {
        self.as_object_mut()?.get_mut(key)
    }

    /// Follow a slash-separated path of keys, e.g. `get_path("Software/Valve/Steam")`.
    pub fn get_path(&self, path: &str) -> Option<&Vdf> {
        let mut current = self;
        for segment in path.split('/').filter(|s| !s.is_empty()) {
            current = current.get(segment)?;
        }
        Some(current)
    }

    pub fn get_path_mut(&mut self, path: &str) -> Option<&mut Vdf> {
        let mut current = self;
        for segment in path.split('/').filter(|s| !s.is_empty()) {
            current = current.get_mut(segment)?;
        }
        Some(current)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Vdf::as_str)
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get_str(key).and_then(|s| s.trim().parse().ok())
    }

    pub fn get_u32(&self, key: &str) -> Option<u32> {
        self.get_str(key).and_then(|s| s.trim().parse().ok())
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get_str(key).and_then(|s| s.trim().parse().ok())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_str(key)
            .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
    }

    pub fn insert(&mut self, key: impl Into<String>, value: Vdf) {
        if let Vdf::Object(map) = self {
            map.insert(key.into(), value);
        }
    }

    pub fn insert_str(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.insert(key, Vdf::String(value.into()));
    }

    pub fn remove(&mut self, key: &str) -> Option<Vdf> {
        self.as_object_mut()?.remove(key)
    }

    /// Get an existing child object or create an empty one in place.
    pub fn entry(&mut self, key: &str) -> &mut Vdf {
        if self.get(key).is_none() {
            self.insert(key, Vdf::object());
        }
        self.get_mut(key).expect("entry just inserted")
    }

    /// Iterate child objects as `(key, value)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vdf)> {
        self.as_object().into_iter().flatten()
    }
}

/// Parse a KeyValues document. A UTF-8 BOM and `//` comments are tolerated.
pub fn parse(input: &str) -> AppResult<Vdf> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut parser = Parser {
        chars: input.chars().collect(),
        pos: 0,
    };
    parser.skip_ws();
    let root = if parser.peek() == Some('{') {
        parser.next();
        Vdf::Object(parser.parse_pairs()?)
    } else {
        Vdf::Object(parser.parse_pairs()?)
    };
    Ok(root)
}

/// Serialize a KeyValues node back to Steam-compatible text.
pub fn to_string(value: &Vdf) -> String {
    let mut out = String::new();
    if let Vdf::Object(map) = value {
        write_object(map, 0, &mut out);
    }
    out
}

fn write_object(map: &BTreeMap<String, Vdf>, indent: usize, out: &mut String) {
    let pad = "\t".repeat(indent);
    for (key, value) in map {
        out.push_str(&pad);
        out.push('"');
        out.push_str(&escape(key));
        out.push('"');
        match value {
            Vdf::String(text) => {
                out.push_str("\t\t\"");
                out.push_str(&escape(text));
                out.push_str("\"\n");
            }
            Vdf::Object(child) => {
                out.push('\n');
                out.push_str(&pad);
                out.push_str("{\n");
                write_object(child, indent + 1, out);
                out.push_str(&pad);
                out.push_str("}\n");
            }
        }
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_ws(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.next();
                }
                Some('/') if self.chars.get(self.pos + 1) == Some(&'/') => {
                    while let Some(c) = self.next() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn parse_quoted(&mut self) -> AppResult<String> {
        if self.next() != Some('"') {
            return Err(AppError::Vdf("expected opening quote".into()));
        }
        let mut out = String::new();
        while let Some(ch) = self.next() {
            match ch {
                '"' => return Ok(out),
                '\\' => match self.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('\\') => out.push('\\'),
                    Some('"') => out.push('"'),
                    Some(other) => {
                        out.push('\\');
                        out.push(other);
                    }
                    None => break,
                },
                _ => out.push(ch),
            }
        }
        Err(AppError::Vdf("unterminated quoted string".into()))
    }

    fn parse_pairs(&mut self) -> AppResult<BTreeMap<String, Vdf>> {
        let mut map = BTreeMap::new();
        loop {
            self.skip_ws();
            match self.peek() {
                None => break,
                Some('}') => {
                    self.next();
                    break;
                }
                Some('"') => {
                    let key = self.parse_quoted()?;
                    self.skip_ws();
                    match self.peek() {
                        Some('{') => {
                            self.next();
                            let child = self.parse_pairs()?;
                            map.insert(key, Vdf::Object(child));
                        }
                        Some('"') => {
                            let value = self.parse_quoted()?;
                            map.insert(key, Vdf::String(value));
                        }
                        other => {
                            return Err(AppError::Vdf(format!(
                                "expected value for key '{key}', found {other:?}"
                            )))
                        }
                    }
                }
                Some(other) => {
                    return Err(AppError::Vdf(format!("unexpected character '{other}'")))
                }
            }
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_objects() {
        let input = r#"
        "libraryfolders"
        {
            "0"
            {
                "path"    "C:\\Program Files (x86)\\Steam"
                "apps"
                {
                    "228980"   "563027845"
                }
            }
        }
        "#;
        let vdf = parse(input).unwrap();
        let root = vdf.get("libraryfolders").unwrap();
        let lib0 = root.get("0").unwrap();
        assert_eq!(
            lib0.get_str("path").unwrap(),
            r"C:\Program Files (x86)\Steam"
        );
        assert_eq!(
            lib0.get_path("apps/228980").unwrap().as_str(),
            Some("563027845")
        );
    }

    #[test]
    fn round_trips() {
        let vdf = parse("\"a\"\n{\n\t\"b\"\t\t\"c\"\n}\n").unwrap();
        let text = to_string(&vdf);
        let reparsed = parse(&text).unwrap();
        assert_eq!(vdf, reparsed);
    }
}
