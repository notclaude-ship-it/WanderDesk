use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub speed: f32,
    pub fullscreen: bool,
    pub stereo: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            speed: 5.0,
            fullscreen: false,
            stereo: false,
        }
    }
}
impl Settings {
    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("wanderdesk/settings.toml"))
    }
    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn save(&self) -> io::Result<()> {
        let Some(path) = Self::path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(io::Error::other)?;
        let temp = path.with_extension("toml.tmp");
        fs::write(&temp, text)?;
        fs::rename(temp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_sane() {
        let s = Settings::default();
        assert_eq!(s.speed, 5.0);
        assert!(!s.fullscreen && !s.stereo);
    }
    #[test]
    fn settings_round_trip_through_toml() {
        let s = Settings {
            speed: 8.0,
            fullscreen: true,
            stereo: false,
        };
        let decoded: Settings = toml::from_str(&toml::to_string(&s).unwrap()).unwrap();
        assert_eq!(decoded.speed, 8.0);
        assert!(decoded.fullscreen);
        assert!(!decoded.stereo);
    }
}
