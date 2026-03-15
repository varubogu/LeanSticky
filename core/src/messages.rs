use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model::ResolvedLocale;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedMessage {
    #[serde(default)]
    pub ja: Option<String>,
    #[serde(default)]
    pub en: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageCatalog {
    #[serde(flatten)]
    pub entries: BTreeMap<String, LocalizedMessage>,
}

impl MessageCatalog {
    pub fn parse(input: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(input)?)
    }

    pub fn text(&self, locale: ResolvedLocale, key: &str) -> String {
        let Some(message) = self.entries.get(key) else {
            return key.to_owned();
        };

        match locale {
            ResolvedLocale::Ja => message
                .ja
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| message.en.as_str())
                .to_owned(),
            ResolvedLocale::En => {
                if message.en.trim().is_empty() {
                    key.to_owned()
                } else {
                    message.en.clone()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MessageCatalog;
    use crate::ResolvedLocale;

    #[test]
    fn falls_back_to_english_when_ja_is_missing() {
        let catalog = MessageCatalog::parse(
            r#"
sample:
  en: English
"#,
        )
        .expect("catalog should parse");

        assert_eq!(catalog.text(ResolvedLocale::Ja, "sample"), "English");
    }
}
