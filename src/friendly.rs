//! Turns compiler messages into Pinch-style explanations using data/errors.toml.
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    pub code: Option<String>,
    pub pattern: Option<String>,
    pub title: String,
    pub explain: String,
    pub fix: String,
}

#[derive(Deserialize, Default)]
struct Catalog {
    #[serde(default)]
    error: Vec<Entry>,
}

pub struct Friendly {
    entries: Vec<Entry>,
}

impl Friendly {
    pub fn load(src: &str) -> Friendly {
        let mut entries = toml::from_str::<Catalog>(src).unwrap_or_default().error;
        // Longest pattern first, so "this file contains an unclosed delimiter"
        // wins over the shorter "unclosed delimiter".
        entries.sort_by_key(|e| std::cmp::Reverse(e.pattern.as_ref().map_or(0, |p| p.len())));
        Friendly { entries }
    }

    pub fn lookup(&self, code: Option<&str>, message: &str) -> Option<&Entry> {
        if let Some(code) = code
            && let Some(e) = self
                .entries
                .iter()
                .find(|e| e.code.as_deref() == Some(code))
        {
            return Some(e);
        }
        let msg = message.to_lowercase();
        self.entries
            .iter()
            .filter_map(|e| e.pattern.as_ref().map(|p| (p, e)))
            .find(|(p, _)| msg.contains(p.as_str()))
            .map(|(_, e)| e)
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_parses_and_finds_codes() {
        let f = Friendly::load(crate::lesson::ERROR_CATALOG);
        assert!(f.len() > 20, "error catalog should have many entries");
        assert!(f.lookup(Some("E0384"), "").is_some());
        assert!(f.lookup(None, "expected `;`, found `}`").is_some());
        let e = f
            .lookup(None, "this file contains an unclosed delimiter")
            .unwrap();
        assert_eq!(
            e.pattern.as_deref(),
            Some("this file contains an unclosed delimiter")
        );
    }
}
