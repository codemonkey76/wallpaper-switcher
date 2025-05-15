use crate::cache::{WallpaperMeta, get_mtime};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub fn classify_wallpapers(
    dir: &str,
    re: &Regex,
    metadata: &mut HashMap<String, WallpaperMeta>,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut portrait = Vec::new();
    let mut landscape = Vec::new();

    let mut current_files = vec![];

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path().to_path_buf();
        let path_str = path.to_string_lossy().to_string();

        if !re.is_match(&path_str) {
            continue;
        }

        current_files.push(path_str.clone());

        let mtime = match get_mtime(&path) {
            Some(m) => m,
            None => continue,
        };

        let orientation = if let Some(meta) = metadata.get(&path_str) {
            if meta.mtime == mtime {
                meta.orientation.clone()
            } else {
                detect_orientation(&path).unwrap_or("unknown").to_string()
            }
        } else {
            detect_orientation(&path).unwrap_or("unknown").to_string()
        };

        if orientation != "unknown" {
            metadata.insert(
                path_str.clone(),
                WallpaperMeta {
                    orientation: orientation.clone(),
                    mtime,
                },
            );
        }

        match orientation.as_str() {
            "portrait" => portrait.push(path),
            "landscape" => landscape.push(path),
            _ => (),
        }
    }

    metadata.retain(|k, _| current_files.contains(k));
    (portrait, landscape)
}

fn detect_orientation(path: &PathBuf) -> Option<&'static str> {
    let output = Command::new("identify")
        .arg("-format")
        .arg("%w %h")
        .arg(path)
        .output()
        .ok()?;

    let out_str = String::from_utf8_lossy(&output.stdout);
    let mut parts = out_str.split_whitespace();
    let width: u32 = parts.next()?.parse().ok()?;
    let height: u32 = parts.next()?.parse().ok()?;

    Some(if width >= height {
        "landscape"
    } else {
        "portrait"
    })
}
