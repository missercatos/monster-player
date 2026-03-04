use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct BrailleImage {
    pub width: usize,
    pub height: usize,
    pub art: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AboutInfo {
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub links: BTreeMap<String, String>,
    #[serde(default)]
    pub braille_images: Vec<BrailleImage>,
}

pub fn about_info() -> &'static AboutInfo {
    static INFO: OnceLock<AboutInfo> = OnceLock::new();
    INFO.get_or_init(|| {
        let raw = include_str!("../../about/about.toml");
        toml::from_str(raw).unwrap_or_else(|_| AboutInfo {
            description: String::new(),
            version: String::new(),
            links: BTreeMap::new(),
            braille_images: Vec::new(),
        })
    })
}
