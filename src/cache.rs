use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize, Deserialize)]
pub struct WallpaperMeta {
    pub orientation: String,
    pub mtime: u64,
}

pub fn load_metadata_cache(path: &str) -> HashMap<String, WallpaperMeta> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_metadata_cache(path: &str, cache: &HashMap<String, WallpaperMeta>) {
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = fs::write(path, json);
    }
}

pub fn get_mtime(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|mtime| mtime.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

pub fn load_index(path: &str) -> usize {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

pub fn save_index(path: &str, index: usize) {
    let _ = fs::write(path, index.to_string());
}
