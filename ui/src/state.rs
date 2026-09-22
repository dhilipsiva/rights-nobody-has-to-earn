// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::data::{PREFIX, book, companion};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Preferences {
    pub theme: String,
    pub visited: Vec<String>,
    pub step: usize,
    pub last_read: Option<String>,
    pub positions: BTreeMap<String, f64>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            visited: vec![],
            step: 0,
            last_read: None,
            positions: BTreeMap::new(),
        }
    }
}
impl Preferences {
    pub fn sanitize(mut self) -> Self {
        if !["light", "dark"].contains(&self.theme.as_str()) {
            self.theme = "dark".into();
        }
        self.step = self.step.min(5);
        self.visited
            .retain(|id| companion().questions.iter().any(|q| q.id == *id));
        self.visited.sort();
        self.visited.dedup();
        self.positions.retain(|path, value| {
            book().pages.iter().any(|p| p.path == *path) && value.is_finite() && *value >= 0.0
        });
        if self
            .last_read
            .as_ref()
            .is_some_and(|path| !book().pages.iter().any(|p| p.path == *path))
        {
            self.last_read = None;
        }
        self
    }
    pub fn visit(&mut self, id: &str) {
        if !self.visited.iter().any(|v| v == id) {
            self.visited.push(id.into());
        }
    }
    pub fn resume(&self) -> String {
        self.last_read
            .clone()
            .unwrap_or_else(|| format!("{PREFIX}read/epigraph/"))
    }
}
#[cfg(feature = "web")]
const KEY: &str = "rights-book.preferences.v1";
#[cfg(feature = "web")]
pub fn load() -> Result<Preferences, String> {
    let storage = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .ok_or("Local storage is unavailable. Progress lasts for this visit.")?;
    match storage
        .get_item(KEY)
        .map_err(|_| "Cannot read local storage")?
    {
        None => Ok(Preferences::default()),
        Some(text) => Ok(serde_json::from_str::<Preferences>(&text)
            .unwrap_or_default()
            .sanitize()),
    }
}
#[cfg(feature = "web")]
pub fn save(state: &Preferences) -> Result<(), String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .ok_or("Local storage is unavailable. Progress lasts for this visit.")?
        .set_item(
            KEY,
            &serde_json::to_string(state).map_err(|e| e.to_string())?,
        )
        .map_err(|_| "Could not save reading progress on this device.".into())
}
#[cfg(feature = "desktop")]
fn path() -> Result<std::path::PathBuf, String> {
    if let Some(directory) = dioxus::prelude::try_consume_context::<crate::PreferencesDirectory>() {
        return Ok(directory.0.join("preferences.json"));
    }
    directories::ProjectDirs::from("dev", "dhilipsiva", "rights-book")
        .map(|d| d.data_local_dir().join("preferences.json"))
        .ok_or("Application-data directory is unavailable".into())
}
#[cfg(feature = "desktop")]
pub fn load() -> Result<Preferences, String> {
    match std::fs::read_to_string(path()?) {
        Ok(s) => Ok(serde_json::from_str::<Preferences>(&s)
            .unwrap_or_default()
            .sanitize()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Preferences::default()),
        Err(e) => Err(format!("Cannot read saved preferences: {e}")),
    }
}
#[cfg(feature = "desktop")]
pub fn save(state: &Preferences) -> Result<(), String> {
    let path = path()?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&path, serde_json::to_vec(state).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Could not save progress: {e}"))
}
#[cfg(not(any(feature = "web", feature = "desktop")))]
pub fn load() -> Result<Preferences, String> {
    Ok(Preferences::default())
}
#[cfg(not(any(feature = "web", feature = "desktop")))]
pub fn save(_: &Preferences) -> Result<(), String> {
    Ok(())
}
