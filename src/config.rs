// Declarative configuration system using TOML.
// Loads from ~/.config/msplayer/config.toml (or platform equivalent),
// falling back to built-in defaults.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub engine: EngineConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub keybindings: Keybindings,
    #[serde(default)]
    pub features: Features,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    #[serde(default = "default_buffer_mb")]
    pub buffer_size_mb: u32,
    #[serde(default = "default_volume")]
    pub default_volume: u8,
    #[serde(default = "default_true")]
    pub auto_advance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub show_lyrics: bool,
    #[serde(default = "default_lang")]
    pub language: String,
}

/// Keybinding definitions. Each key supports modifiers joined by '+'.
/// Example: `"Space"`, `"Ctrl+T"`, `"Shift+D"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    #[serde(default = "default_kb_play")]
    pub play_selected: String,
    #[serde(default = "default_kb_pause")]
    pub toggle_pause: String,
    #[serde(default = "default_kb_next_song")]
    pub next_song: String,
    #[serde(default = "default_kb_prev_song")]
    pub prev_song: String,
    #[serde(default = "default_kb_next_album")]
    pub next_album: String,
    #[serde(default = "default_kb_prev_album")]
    pub prev_album: String,
    #[serde(default = "default_kb_cycle_mode")]
    pub cycle_mode: String,
    #[serde(default = "default_kb_love")]
    pub toggle_love: String,
    #[serde(default = "default_kb_lyrics")]
    pub toggle_lyrics: String,
    #[serde(default = "default_kb_search")]
    pub search: String,
    #[serde(default = "default_kb_vol_up")]
    pub volume_up: String,
    #[serde(default = "default_kb_vol_down")]
    pub volume_down: String,
    #[serde(default = "default_kb_seek_fwd")]
    pub seek_forward: String,
    #[serde(default = "default_kb_seek_back")]
    pub seek_backward: String,
    #[serde(default = "default_kb_play_next")]
    pub play_next: String,
    #[serde(default = "default_kb_play_prev")]
    pub play_prev: String,
    #[serde(default = "default_kb_settings")]
    pub settings: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Features {
    #[serde(default = "default_true")]
    pub enable_favorites: bool,
    #[serde(default = "default_true")]
    pub enable_global_playlist: bool,
    #[serde(default = "default_true")]
    pub enable_streaming: bool,
    #[serde(default = "default_true")]
    pub enable_lyrics: bool,
}

// --- Default value helpers ------------------------------------------------

fn default_buffer_mb() -> u32 { 8 }
fn default_volume() -> u8 { 80 }
fn default_theme() -> String { "tokyonight".into() }
fn default_lang() -> String { "en".into() }
const fn default_true() -> bool { true }

macro_rules! kb_default {
    ($name:ident, $val:literal) => {
        fn $name() -> String { $val.into() }
    };
}

kb_default!(default_kb_play, "Space");
kb_default!(default_kb_pause, "x");
kb_default!(default_kb_next_song, "j");
kb_default!(default_kb_prev_song, "k");
kb_default!(default_kb_next_album, "l");
kb_default!(default_kb_prev_album, "h");
kb_default!(default_kb_cycle_mode, "e");
kb_default!(default_kb_love, "s");
kb_default!(default_kb_lyrics, "v");
kb_default!(default_kb_search, "/");
kb_default!(default_kb_vol_up, "p");
kb_default!(default_kb_vol_down, "o");
kb_default!(default_kb_seek_fwd, "d");
kb_default!(default_kb_seek_back, "a");
kb_default!(default_kb_play_next, "Shift+D");
kb_default!(default_kb_play_prev, "Shift+A");
kb_default!(default_kb_settings, "Ctrl+T");

// --- Implementation -------------------------------------------------------

impl Default for Config {
    fn default() -> Self {
        Self {
            engine: EngineConfig::default(),
            ui: UiConfig::default(),
            keybindings: Keybindings::default(),
            features: Features::default(),
        }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            buffer_size_mb: default_buffer_mb(),
            default_volume: default_volume(),
            auto_advance: default_true(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            show_lyrics: default_true(),
            language: default_lang(),
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            play_selected: "Space".into(),
            toggle_pause: "x".into(),
            next_song: "j".into(),
            prev_song: "k".into(),
            next_album: "l".into(),
            prev_album: "h".into(),
            cycle_mode: "e".into(),
            toggle_love: "s".into(),
            toggle_lyrics: "v".into(),
            search: "/".into(),
            volume_up: "p".into(),
            volume_down: "o".into(),
            seek_forward: "d".into(),
            seek_backward: "a".into(),
            play_next: "Shift+D".into(),
            play_prev: "Shift+A".into(),
            settings: "Ctrl+T".into(),
        }
    }
}

impl Default for Features {
    fn default() -> Self {
        Self {
            enable_favorites: true,
            enable_global_playlist: true,
            enable_streaming: true,
            enable_lyrics: true,
        }
    }
}

impl Config {
    /// Configuration directory: `~/.config/msplayer/` on Linux,
    /// `C:\Users\<user>\AppData\Roaming\msplayer\` on Windows.
    pub fn config_dir() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "msplayer", "msplayer")
            .map(|d| d.config_dir().to_path_buf())
    }

    /// Load from `config_dir()/config.toml`, falling back to defaults.
    pub fn load() -> Self {
        let path = Self::config_dir().map(|mut p| {
            p.push("config.toml");
            p
        });
        match path {
            Some(ref p) if p.exists() => {
                match std::fs::read_to_string(p) {
                    Ok(content) => toml::from_str(&content).unwrap_or_default(),
                    Err(_) => Config::default(),
                }
            }
            _ => Config::default(),
        }
    }

    /// Save the current configuration to disk.
    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir().ok_or("cannot determine config directory")?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut path = dir;
        path.push("config.toml");
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Generate a default config file at the standard location.
    pub fn generate_default_config() -> Result<PathBuf, String> {
        let dir = Self::config_dir().ok_or("cannot determine config directory")?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut path = dir;
        path.push("config.toml");
        let default = Config::default();
        let content = toml::to_string_pretty(&default).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(path)
    }
}
